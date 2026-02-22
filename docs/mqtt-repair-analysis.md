# MQTT repair warning analysis

## Context
The warning observed during MQTT repair is:

```
MQTT repair subscribe error for homeassistant/sensor/flowl_plant_+/config: Failed to send mqtt requests to eventloop
```

This appears alongside successful repair completion logs:

```
MQTT republish_all complete: 4 plants
MQTT repair complete: cleared 0 orphans, published 4 plants
```

## What the warning means
The repair flow creates a temporary MQTT client to discover retained topics. During discovery, it connects, then subscribes to wildcard topics. The warning means the temporary client's subscribe request could not be sent to its event loop. In rumqttc, this error usually indicates the event loop is no longer accepting requests (for example, the connection dropped or the event loop stopped).

This warning does not stop the rest of the repair flow. Republish still uses the primary MQTT client and completed successfully in the provided logs.

## Impact
- Discovery may have been incomplete, so `cleared 0 orphans` may be inaccurate.
- Republish still occurs, so current plant state is re-published even if discovery fails.

## Likely causes
- Broker immediately disconnects or denies the temporary repair client (ACLs, connection limits).
- Broker denies wildcard subscription for discovery topics.
- Short-lived network hiccup right after connection, causing the event loop to shut down.

## Code locations
- Temporary repair client and subscriptions: `src/mqtt.rs`
  - `discover_broker_plant_ids` establishes a temporary client and subscribes to:
    - `homeassistant/sensor/{prefix}_plant_+/config`
    - `{prefix}/plant/+/state`
    - `{prefix}/plant/+/attributes`
- Repair flow and final log lines: `src/mqtt.rs`
  - `repair` calls `discover_broker_plant_ids`, then republish_all.

## Suggested follow-ups
If the warning is frequent and orphan clearing matters, consider one of:
- Retry the subscribe once or twice after a short delay.
- Treat discovery failure as non-fatal but report it explicitly to the caller.
- Lower the log level for this warning to reduce noise when republish still succeeds.
