## Context

The MQTT integration publishes retained messages to three topic patterns per plant: discovery config (`homeassistant/sensor/{prefix}_plant_{id}/config`), state (`{prefix}/plant/{id}/state`), and attributes (`{prefix}/plant/{id}/attributes`). When the broker or flowl restarts, or the connection drops, retained state can become stale or orphaned. The only current recovery is a process restart, which triggers `first_run` in the background checker — but even that doesn't clear orphans from deleted plants.

The `remove_plant` function already publishes empty retained payloads to clear a single plant's topics. The state checker already has a `first_run` path that republishes everything. The repair feature builds on both patterns.

## Goals / Non-Goals

**Goals:**
- Provide a `POST /api/mqtt/repair` endpoint that clears stale retained messages and republishes fresh state for all current plants
- Automatically trigger a full republish when the MQTT connection recovers (broker restart, network outage)
- No new crates or dependencies

**Non-Goals:**
- Tracking historical plant IDs in a separate table
- Retry/queue logic for failed publishes during repair

## Decisions

### 1. Subscribe-and-filter for targeted orphan cleanup

**Decision**: Use a temporary, dedicated MQTT client to discover which topics actually have retained messages on the broker, diff against the current database, and clear only the orphaned entries. The repair function:

1. Creates a temporary `AsyncClient` + event loop connecting to the same broker
2. Subscribes to three wildcard topic patterns to discover all retained plant topics:
   - `homeassistant/sensor/{prefix}_plant_+/config` (HA auto-discovery configs)
   - `{prefix}/plant/+/state` (watering state)
   - `{prefix}/plant/+/attributes` (watering attributes)
3. Polls the temporary event loop, collecting incoming retained `Publish` packets
4. After a timeout (no new messages within a threshold), extracts plant IDs from the collected topic names
5. Queries the database for current plant IDs
6. Diffs: any plant ID present on the broker but not in the DB is an orphan
7. Clears all three topics (discovery, state, attributes) for each orphaned plant ID by publishing empty retained payloads via the main MQTT client
8. Republishes fresh discovery, state, and attributes for all current plants via the main MQTT client
9. Disconnects and drops the temporary client

**Rationale**: This approach clears exactly the orphaned topics that exist — no guessing, no sweeping IDs that were never used. It keeps the main event loop untouched by using an independent temporary connection for the subscribe/collect phase. The temporary client is created on demand and disposed of after the repair completes.

**Topic patterns and what gets cleared**: Each orphaned plant has three retained topics on the broker. All three must be cleared (empty retained payload) to fully remove the plant from Home Assistant: the auto-discovery config (which registers the entity), the state topic, and the attributes topic. Clearing only the state/attributes but leaving the discovery config would leave a ghost entity in HA.

**Timeout strategy**: After subscribing, the broker delivers retained messages quickly (they're already stored). A short timeout (e.g., 2 seconds of silence after the last received message) is sufficient to determine all retained messages have been received.

**Alternatives considered**:
- *Sweep 1..=MAX(id)*: Publish empty retained payloads for all IDs from 1 to the highest current ID. Simple but wasteful — publishes many unnecessary clears for IDs that never existed, and misses orphans with IDs higher than the current max. Rejected for being imprecise.
- *Track deleted IDs*: Store deleted plant IDs in a separate table. Ongoing storage overhead for a rare operation. Rejected.

### 2. Shared `republish_all` function for both repair endpoint and reconnect

**Decision**: Extract a single `pub async fn republish_all(pool: &SqlitePool, client: &AsyncClient, prefix: &str)` function in `mqtt.rs` that performs the full sweep-and-republish. Both the API endpoint and the reconnect handler call this same function.

**Rationale**: Avoids duplicating the query-plants-and-publish logic. The API endpoint calls it directly. The reconnect handler in the state checker calls it when it detects a connection recovery.

### 3. Reconnect detection via `AtomicBool` transition in the state checker

**Decision**: Pass the `Arc<AtomicBool>` (connection status) to `spawn_state_checker`. The checker tracks the previous connection state each tick. When it detects a `false → true` transition, it calls `republish_all` instead of the normal incremental check.

**Rationale**: The event loop in `connect()` already sets the `AtomicBool` on `ConnAck`. Rather than adding channels or a `Notify`, the checker simply polls the flag it already implicitly depends on. This keeps the event loop unchanged and confines the new logic to the checker.

**Alternatives considered**:
- *Channel from event loop*: Have the event loop send a message on `ConnAck`. Would require plumbing a channel through `connect()` and restructuring the event loop. More coupling for no real benefit since the checker runs every 60 seconds anyway.
- *Separate reconnect task*: Spawn a dedicated task that watches the `AtomicBool`. Adds another long-lived task for a simple flag check.

### 4. Repair endpoint returns a count summary

**Decision**: `POST /api/mqtt/repair` returns `{ "cleared": N, "published": M }` where `cleared` is the number of IDs whose topics were wiped and `published` is the number of current plants republished.

**Rationale**: Gives the caller feedback that something happened, and helps debug (if `published` is 0, there are no plants; if `cleared` > `published`, orphans were cleaned up).

### 5. Repair button on settings page

**Decision**: Add a "Repair" button in the MQTT section of the settings page, visible only when MQTT status is `connected` or `disconnected` (not when `disabled`). The button calls `POST /api/mqtt/repair`, shows a brief loading state while the request is in-flight, and displays the result summary (cleared/published counts) or an error message inline.

**Rationale**: The settings page already shows MQTT status (broker, prefix, connection dot). Adding the repair action here keeps MQTT controls co-located. Showing results inline avoids modal dialogs and matches the page's existing information-dense style.

**UX details**:
- Button label: "Repair" — concise and matches the endpoint name
- Disabled with tooltip while MQTT is disconnected (repair requires a live connection)
- On success: show "Cleared N, published M" as a brief inline confirmation
- On error: show the error message inline

### 6. Repair endpoint guards

**Decision**: Return `409 Conflict` if MQTT is disabled. Return `503 Service Unavailable` if MQTT is enabled but currently disconnected (the `AtomicBool` is `false`).

**Rationale**: Attempting to publish while disconnected would silently queue messages in `rumqttc`'s internal buffer with no guarantee of delivery. Better to fail fast and let the caller retry when connected.

## Risks / Trade-offs

- **Timeout-based message collection** → There is no MQTT signal for "all retained messages delivered." The repair function uses a silence timeout to decide when collection is complete. If the broker is slow or the network has high latency, a message could arrive after the timeout. Mitigation: use a conservative timeout (2 seconds) and document that running repair a second time will catch anything missed.
- **Temporary connection overhead** → Each repair creates a second TCP connection to the broker. Negligible for an infrequent manual operation. The connection is short-lived and cleaned up immediately.
- **60-second detection latency for reconnect** → The checker polls every 60 seconds, so reconnect-triggered republish can be delayed up to 60 seconds after `ConnAck`. Acceptable for a plant care app.
- **Concurrent repair and checker** → If the API repair runs while the checker tick is in progress, both may publish for the same plants simultaneously. Since all messages are retained and idempotent, the last writer wins — no data corruption, just redundant publishes.
- **Repair during partial connectivity** → Some publishes may fail if the connection drops mid-repair. The endpoint doesn't retry; the operator can call it again. The reconnect-triggered republish will also fire on the next recovery.
