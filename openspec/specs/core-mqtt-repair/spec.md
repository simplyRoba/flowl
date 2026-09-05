## Purpose

MQTT broker-state repair: orphan discovery, cleanup, and full republish of current plant state.

## Requirements

### Requirement: MQTT Repair Endpoint

The API SHALL expose a `POST /api/mqtt/repair` endpoint that clears orphaned retained MQTT messages from the broker and republishes fresh state for all current plants.

#### Scenario: Successful repair

- **WHEN** a POST request is made to `/api/mqtt/repair`
- **AND** MQTT is enabled and connected
- **THEN** the server responds with HTTP 200
- **AND** the response body is `{ "cleared": N, "published": M }` where `N` is the number of orphaned plant IDs cleared and `M` is the number of current plants republished

#### Scenario: MQTT disabled

- **WHEN** a POST request is made to `/api/mqtt/repair`
- **AND** MQTT is disabled via `FLOWL_MQTT_DISABLED=true`
- **THEN** the server responds with HTTP 409 Conflict

#### Scenario: MQTT disconnected

- **WHEN** a POST request is made to `/api/mqtt/repair`
- **AND** MQTT is enabled but not connected
- **THEN** the server responds with HTTP 503 Service Unavailable

### Requirement: Broker-Side Orphan Discovery

The repair operation SHALL determine the retained plant IDs represented in the MQTT namespaces defined by `core-mqtt` and identify IDs that are absent from the current plants. Discovery SHALL terminate after a finite collection window rather than waiting indefinitely.

#### Scenario: Orphaned plant detected

- **GIVEN** the broker has retained messages representing plant ID 5 in the MQTT namespaces defined by `core-mqtt`
- **AND** plant ID 5 is absent from the current plants
- **WHEN** the repair operation discovers broker-side retained plant IDs
- **THEN** plant ID 5 is identified as an orphan

#### Scenario: No orphans

- **GIVEN** the broker has retained messages only representing current plants
- **WHEN** the repair operation discovers broker-side retained plant IDs
- **THEN** no orphans are identified
- **AND** the `cleared` count in the response is 0

### Requirement: Orphan Cleanup

The repair operation SHALL clear every retained discovery, state, and attributes topic for each orphaned plant ID by publishing empty retained payloads. The exact topic paths and message contracts are defined by `core-mqtt`.

#### Scenario: All orphan topics cleared

- **GIVEN** plant ID 5 is identified as an orphan
- **AND** the MQTT topic prefix is `flowl`
- **WHEN** orphan cleanup runs
- **THEN** an empty retained payload is published to `homeassistant/sensor/flowl_plant_5/config`
- **AND** an empty retained payload is published to `flowl/plant/5/state`
- **AND** an empty retained payload is published to `flowl/plant/5/attributes`

### Requirement: Full Republish of Current Plants

After orphan cleanup, the repair operation SHALL republish discovery configurations, current watering state, and attributes for all current plants according to the message contracts defined by `core-mqtt`.

#### Scenario: All current plants republished

- **GIVEN** the current plants have IDs 1, 2, and 3
- **AND** orphan cleanup has completed
- **WHEN** the republish phase runs
- **THEN** discovery configuration, state, and attributes are published for each of plants 1, 2, and 3
- **AND** all published messages are retained with QoS AtLeastOnce
