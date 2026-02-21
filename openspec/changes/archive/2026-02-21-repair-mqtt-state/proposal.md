## Why

When the MQTT broker restarts, loses retained messages, or the flowl process reconnects after a network outage, the broker's retained state can become stale or incomplete. Worse, if plants were deleted while MQTT was disconnected, their retained discovery configs, states, and attributes linger on the broker as orphaned entities in Home Assistant. There is currently no way to clean up stale retained messages or force a full re-publish — the only partial recovery is restarting the flowl process (which triggers the checker's `first_run` logic but never clears orphans). Operators need an on-demand way to wipe and rebuild MQTT state without service restart.

## What Changes

- Add a new API endpoint that performs a clean-slate MQTT repair: first clear all retained messages under the topic prefix (discovery configs, states, attributes for all known plant IDs), then re-publish fresh state only for plants that currently exist in the database
- The clear step removes orphaned retained messages for deleted plants that would otherwise persist on the broker indefinitely
- The endpoint provides a single action an operator (or Home Assistant automation) can call to repair broker state
- Add a "Repair" button to the MQTT section on the settings page that calls the repair endpoint and shows feedback
- The background state checker re-publishes all state after an MQTT reconnect, not just on process startup

## Capabilities

### New Capabilities

- `core/mqtt-repair`: API endpoint and logic to clear all retained MQTT messages and re-publish fresh state for current plants

### Modified Capabilities

- `core/mqtt`: Add automatic full re-publish on reconnect (when `AtomicBool` transitions from `false` to `true`) so broker state self-heals after connection recovery
- `ui/settings`: Add a "Repair" button in the MQTT section to trigger the repair endpoint from the UI

## Impact

- **Code**: `src/mqtt.rs` (reconnect-triggered re-publish, new repair function), `src/api/` (new endpoint), `src/main.rs` (wiring)
- **API**: New `POST /api/mqtt/repair` endpoint
- **UI**: `ui/src/routes/settings/+page.svelte` (repair button in MQTT section), `ui/src/lib/api.ts` (new API call)
- **Dependencies**: No new crates — uses existing `rumqttc` client and SQLite queries
- **Systems**: Home Assistant integrations benefit from automatic state recovery
