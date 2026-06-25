## ADDED Requirements

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
- **AND** MQTT is enabled but the client is not connected
- **THEN** the server responds with HTTP 503 Service Unavailable

### Requirement: Orphan Discovery via Temporary Subscribe

The repair function SHALL discover orphaned plant topics on the broker by creating a temporary MQTT client, subscribing to wildcard topic patterns, and collecting retained messages.

#### Scenario: Temporary client connects and subscribes

- **WHEN** a repair is triggered
- **THEN** a temporary MQTT client connects to the same broker as the main client
- **AND** subscribes to `homeassistant/sensor/{prefix}_plant_+/config`
- **AND** subscribes to `{prefix}/plant/+/state`
- **AND** subscribes to `{prefix}/plant/+/attributes`
- **AND** collects retained messages until a silence timeout elapses
- **AND** the temporary client disconnects after collection is complete

#### Scenario: Orphaned plant detected

- **GIVEN** the broker has retained messages for plant ID 5
- **AND** plant ID 5 does not exist in the database
- **WHEN** the repair function collects retained messages and diffs against the database
- **THEN** plant ID 5 is identified as an orphan

#### Scenario: No orphans

- **GIVEN** the broker has retained messages only for plants that exist in the database
- **WHEN** the repair function collects retained messages and diffs against the database
- **THEN** no orphans are identified
- **AND** the `cleared` count in the response is 0

### Requirement: Orphan Cleanup

The repair function SHALL clear all three retained topics (discovery config, state, attributes) for each orphaned plant ID by publishing empty retained payloads via the main MQTT client.

#### Scenario: All orphan topics cleared

- **GIVEN** plant ID 5 is identified as an orphan
- **AND** the MQTT topic prefix is `flowl`
- **WHEN** orphan cleanup runs
- **THEN** an empty retained payload is published to `homeassistant/sensor/flowl_plant_5/config`
- **AND** an empty retained payload is published to `flowl/plant/5/state`
- **AND** an empty retained payload is published to `flowl/plant/5/attributes`

### Requirement: Full Republish of Current Plants

After orphan cleanup, the repair function SHALL republish discovery configs, current watering state, and attributes for all plants currently in the database.

#### Scenario: All current plants republished

- **GIVEN** the database contains plants with IDs 1, 2, and 3
- **AND** orphan cleanup has completed
- **WHEN** the republish phase runs
- **THEN** discovery config, state, and attributes are published for each of plants 1, 2, and 3
- **AND** all published messages are retained with QoS AtLeastOnce
