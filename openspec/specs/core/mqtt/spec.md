## Purpose

MQTT client connection to Mosquitto, graceful connect/disconnect, configuration via environment variables, Home Assistant auto-discovery, watering state publishing, and background state checking.

## Requirements

### Requirement: MQTT can be disabled via environment flag

The application SHALL allow operators to skip MQTT setup by setting `FLOWL_MQTT_DISABLED=true`.

#### Scenario: MQTT disabled by configuration

- **WHEN** the application starts with `FLOWL_MQTT_DISABLED=true`
- **THEN** the MQTT client is never created
- **AND** the background state checker is not spawned
- **AND** the HTTP server starts normally without attempting MQTT publishes

### Requirement: MQTT Client Connection

The application SHALL connect an MQTT client to the broker specified by `FLOWL_MQTT_HOST` (default `localhost`) and `FLOWL_MQTT_PORT` (default `1883`) on startup when `FLOWL_MQTT_DISABLED` is not true. The connection function SHALL accept a shared `Arc<AtomicBool>` and update it to reflect the live connection state.

#### Scenario: Successful connection

- **WHEN** the application starts and the MQTT broker is reachable
- **AND** `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** the MQTT client connects successfully
- **AND** the shared `AtomicBool` is set to `true`
- **AND** a log message confirms the connection

#### Scenario: Broker unreachable at startup

- **WHEN** the application starts, the MQTT broker is not reachable, and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** the HTTP server starts normally
- **AND** the shared `AtomicBool` remains `false`
- **AND** the MQTT client retries connection in the background
- **AND** a warning is logged

#### Scenario: MQTT disabled via flag

- **WHEN** the application starts with `FLOWL_MQTT_DISABLED=true`
- **THEN** no MQTT connection attempt is made
- **AND** no `AtomicBool` is created
- **AND** a log message notes that MQTT is disabled

### Requirement: MQTT Reconnection

The MQTT client SHALL automatically reconnect when the connection to the broker is lost, provided MQTT is enabled (`FLOWL_MQTT_DISABLED` is not true). The shared `AtomicBool` SHALL reflect connection state transitions.

#### Scenario: Connection lost and recovered

- **WHEN** the MQTT connection drops and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** the shared `AtomicBool` is set to `false`
- **AND** the client automatically attempts to reconnect
- **AND** a warning is logged on disconnect
- **AND** the shared `AtomicBool` is set to `true` on successful reconnect
- **AND** an info message is logged on successful reconnect

### Requirement: MQTT Configuration

The MQTT client SHALL use `FLOWL_MQTT_TOPIC_PREFIX` (default `flowl`) as the base prefix for all topics when MQTT is enabled.

#### Scenario: Default topic prefix

- **WHEN** the application starts without `FLOWL_MQTT_TOPIC_PREFIX` set and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** the MQTT client uses `flowl` as the topic prefix

#### Scenario: Custom topic prefix

- **WHEN** the application starts with `FLOWL_MQTT_TOPIC_PREFIX=myplants` and MQTT is enabled
- **THEN** the MQTT client uses `myplants` as the topic prefix

### Requirement: MQTT Graceful Disconnect

The MQTT client SHALL disconnect cleanly when the application shuts down, unless MQTT is disabled.

#### Scenario: Application shutdown

- **WHEN** the application receives a shutdown signal and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** the MQTT client sends a disconnect packet to the broker

### Requirement: MQTT Client Shared via AppState

The MQTT `AsyncClient` SHALL be available in `AppState` as an optional field so API handlers can publish messages whenever MQTT is enabled.

#### Scenario: MQTT client available

- **WHEN** the MQTT client connects successfully and MQTT is enabled
- **THEN** the `AsyncClient` is stored in `AppState`
- **AND** API handlers can access it to publish messages

#### Scenario: MQTT client unavailable

- **WHEN** the MQTT client is not connected or MQTT is disabled
- **THEN** `AppState` holds `None` for the MQTT client
- **AND** API handlers skip MQTT publishing without error

### Requirement: MQTT Configuration in AppState

`AppState` SHALL include MQTT configuration fields (`mqtt_host`, `mqtt_port`, `mqtt_disabled`) and the shared connection status (`mqtt_connected: Option<Arc<AtomicBool>>`) so the status endpoint can report runtime state.

#### Scenario: MQTT enabled

- **WHEN** MQTT is enabled
- **THEN** `AppState.mqtt_connected` holds `Some(Arc<AtomicBool>)`
- **AND** `AppState.mqtt_host`, `mqtt_port`, and `mqtt_prefix` reflect the configured values

#### Scenario: MQTT disabled

- **WHEN** MQTT is disabled
- **THEN** `AppState.mqtt_connected` holds `None`
- **AND** `AppState.mqtt_disabled` is `true`

### Requirement: Home Assistant MQTT Auto-Discovery

The application SHALL publish retained MQTT auto-discovery configs for each plant, registering them as Home Assistant sensor entities with a `json_attributes_topic` whenever MQTT is enabled.

#### Scenario: Discovery config published

- **GIVEN** a plant with id 1 and name "Monstera"
- **AND** the MQTT topic prefix is `flowl`
- **AND** `FLOWL_MQTT_DISABLED` is not `true`
- **WHEN** a discovery config is published
- **THEN** a retained JSON message is published to `homeassistant/sensor/flowl_plant_1/config`
- **AND** the payload contains `name`, `unique_id`, `state_topic`, `json_attributes_topic`, `icon`, and `device` fields
- **AND** `state_topic` is `flowl/plant/1/state`
- **AND** `json_attributes_topic` is `flowl/plant/1/attributes`

#### Scenario: Discovery config removed

- **GIVEN** a plant with id 1 is deleted
- **WHEN** the deletion triggers MQTT cleanup and MQTT is enabled
- **THEN** an empty retained payload is published to `homeassistant/sensor/flowl_plant_1/config`
- **AND** an empty retained payload is published to `flowl/plant/1/state`
- **AND** an empty retained payload is published to `flowl/plant/1/attributes`

### Requirement: MQTT State Publishing

The application SHALL publish watering state to retained MQTT topics whenever MQTT is enabled.

#### Scenario: State published

- **GIVEN** a plant with id 1 and watering status `due`
- **AND** the MQTT topic prefix is `flowl`
- **AND** `FLOWL_MQTT_DISABLED` is not `true`
- **WHEN** state is published
- **THEN** the string `due` is published as a retained message to `flowl/plant/1/state`

#### Scenario: State values

- **WHEN** watering state is published and MQTT is enabled
- **THEN** the payload is one of: `ok`, `due`, `overdue`

### Requirement: MQTT Attributes Publishing

The application SHALL publish plant watering attributes as a retained JSON object to a dedicated attributes topic whenever MQTT is enabled.

#### Scenario: Attributes published

- **GIVEN** a plant with id 1, `last_watered` = `2026-02-13T14:30:00`, `watering_interval_days` = 7, `next_due` = `2026-02-20`
- **AND** the MQTT topic prefix is `flowl`
- **AND** MQTT is enabled
- **WHEN** attributes are published
- **THEN** a retained JSON message is published to `flowl/plant/1/attributes`
- **AND** the payload contains `next_due`, `last_watered`, and `watering_interval_days`

#### Scenario: Attributes for never-watered plant

- **GIVEN** a plant with `last_watered` = NULL
- **AND** MQTT is enabled
- **WHEN** attributes are published
- **THEN** the payload contains `next_due` = null, `last_watered` = null, and the `watering_interval_days` value

### Requirement: Background State Checker

The application SHALL run a background task that periodically checks all plants for watering state transitions and publishes updates to MQTT whenever MQTT is enabled. The checker SHALL also detect MQTT reconnection and trigger a full republish of all plant state.

#### Scenario: State transition detected

- **GIVEN** a plant was previously `ok`
- **AND** enough time has passed that it is now `due`
- **AND** MQTT is enabled
- **WHEN** the background checker runs
- **THEN** the new state `due` is published to the plant's MQTT state topic
- **AND** updated attributes are published to the plant's attributes topic

#### Scenario: No state change

- **GIVEN** a plant's watering status has not changed since last check
- **AND** MQTT is enabled
- **WHEN** the background checker runs
- **THEN** no MQTT message is published for that plant

#### Scenario: Checker interval

- **WHEN** the application is running and MQTT is enabled
- **THEN** the background state checker runs every 60 seconds

#### Scenario: Full publish on startup

- **WHEN** the application starts and MQTT is enabled
- **THEN** the background checker publishes discovery configs, current state, and attributes for all existing plants

#### Scenario: Full republish on reconnect

- **GIVEN** the MQTT connection was lost (AtomicBool is `false`)
- **AND** the connection is recovered (AtomicBool transitions to `true`)
- **WHEN** the background checker runs its next tick
- **THEN** the checker SHALL republish discovery configs, current state, and attributes for all existing plants
