## Context

Flowl currently constructs the MQTT client, background state checker, and publishes discovery/state/attribute messages on every startup regardless of whether a broker is reachable. When operators want to run without MQTT (for local development or simplified deployments) they need to point at a dummy broker or patches the code because the service tries to initialize MQTT and the watcher even when there is no broker available.

The new `FLOWL_MQTT_DISABLED` flag should make MQTT entirely optional while keeping the HTTP API, database migrations, and upload handling unchanged. We still want to ship a single minimal binary, respect the existing dev workflow with `cargo watch`, and avoid adding new runtime dependencies.

## Goals / Non-Goals

**Goals:**
- Allow the service to start and run without MQTT when `FLOWL_MQTT_DISABLED=true` while retaining the existing defaults and behavior when the flag is absent or false.
- Prevent MQTT-related background tasks from running when disabled so there are no unnecessary goroutines/timers or failed connection attempts.
- Keep API handlers functional by reflecting the absence of an MQTT client in `AppState` and skipping publishes gracefully.

**Non-Goals:**
- Removing MQTT entirely; disabling it remains opt-in and reversible via the configuration flag.
- Replacing the existing MQTT requirements/specs other than ensuring they are conditional on the new flag.

## Decisions

- **Configuration surface**: Add `FLOWL_MQTT_DISABLED` to `Config::from_env`, defaulting to `false`, rather than relying on a sentinel path or missing host. This keeps flag parsing close to other env vars and avoids interpreting empty strings.
- **Client lifecycle**: Gate `mqtt::connect`, `mqtt::spawn_state_checker`, and AppState population with the flag so MQTT setup happens only when enabled. That means `mqtt::connect` returns `None` (or an appropriate handle) when disabled, and the rest of the system treats the client as optional as already described in the MQTT spec.
- **Background checker**: Skip spawning the state-checker task when MQTT is disabled, ensuring no timers or MQTT publish attempts run. When the flag transitions to `true`, the HTTP server should still respond (no deferred startup) but logs should highlight that MQTT is disabled.
- **API handling**: Keep `AppState.mqtt_client` as `Option` and rely on existing guard clauses so request handlers degrade gracefully; new logs clarify behavior but no additional branches are needed.
- **Documentation**: Update README to mention the new flag and remind operators to keep MQTT enabled in production unless intentionally bypassing it, ensuring the minimal Docker image keeps its constraints.

## Risks / Trade-offs

- [Risk] Users may accidentally disable MQTT in production and miss notifications/auto-discovery → Mitigation: document flag prominently, log at startup when MQTT is disabled, and keep defaults as enabled.
- [Risk] Disabling MQTT could leave some state checks un-run, making background state updates stale when re-enabling later → Mitigation: clearly state the behavior in docs and recommend restarting with MQTT enabled; optional future enhancement could re-run a full publish after re-enabling.
- [Risk] Introducing the flag might complicate startup code → Mitigation: keep the gating logic localized around `mqtt::connect`, `spawn_state_checker`, and AppState construction so the overall architecture remains simple.

## Migration Plan

1. Add the flag and gating logic, run `cargo test` and `cargo watch` locally with `FLOWL_MQTT_DISABLED=true` to verify the service starts and the HTTP API works.<br>
2. Update README and any docs/specs to mention the flag and its default behavior so operators know how to disable MQTT in dev or simple deployments.<br>
3. Deploy the change, noting that existing deployments ignore the flag (since it defaults to `false`) but now have the option to opt out for testing.

## Open Questions

- Should we extend the flag to skip the MQTT spec requirements entirely (e.g., change the capability to be conditional) or leave the spec as-is and document the opt-out? (Current plan documents the conditional behavior.)
