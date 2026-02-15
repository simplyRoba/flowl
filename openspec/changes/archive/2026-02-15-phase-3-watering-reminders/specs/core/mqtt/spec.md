## MODIFIED Requirements

### Requirement: MQTT Client Shared via AppState

The MQTT `AsyncClient` SHALL be available in `AppState` as an optional field so API handlers can publish messages.

#### Scenario: MQTT client available

- **WHEN** the MQTT client connects successfully
- **THEN** the `AsyncClient` is stored in `AppState`
- **AND** API handlers can access it to publish messages

#### Scenario: MQTT client unavailable

- **WHEN** the MQTT client is not connected or not configured
- **THEN** `AppState` holds `None` for the MQTT client
- **AND** API handlers skip MQTT publishing without error

### Requirement: Home Assistant MQTT Auto-Discovery

The application SHALL publish retained MQTT auto-discovery configs for each plant, registering them as Home Assistant sensor entities with a `json_attributes_topic`.

#### Scenario: Discovery config published

- **GIVEN** a plant with id 1 and name "Monstera"
- **AND** the MQTT topic prefix is `flowl`
- **WHEN** a discovery config is published
- **THEN** a retained JSON message is published to `homeassistant/sensor/flowl_plant_1/config`
- **AND** the payload contains `name`, `unique_id`, `state_topic`, `json_attributes_topic`, `icon`, and `device` fields
- **AND** `state_topic` is `flowl/plant/1/state`
- **AND** `json_attributes_topic` is `flowl/plant/1/attributes`

#### Scenario: Discovery config removed

- **GIVEN** a plant with id 1 is deleted
- **WHEN** the deletion triggers MQTT cleanup
- **THEN** an empty retained payload is published to `homeassistant/sensor/flowl_plant_1/config`
- **AND** an empty retained payload is published to `flowl/plant/1/state`
- **AND** an empty retained payload is published to `flowl/plant/1/attributes`

### Requirement: MQTT State Publishing

The application SHALL publish watering state to retained MQTT topics.

#### Scenario: State published

- **GIVEN** a plant with id 1 and watering status `due`
- **AND** the MQTT topic prefix is `flowl`
- **WHEN** state is published
- **THEN** the string `due` is published as a retained message to `flowl/plant/1/state`

#### Scenario: State values

- **WHEN** watering state is published
- **THEN** the payload is one of: `ok`, `due`, `overdue`

### Requirement: MQTT Attributes Publishing

The application SHALL publish plant watering attributes as a retained JSON object to a dedicated attributes topic.

#### Scenario: Attributes published

- **GIVEN** a plant with id 1, `last_watered` = `2026-02-13T14:30:00`, `watering_interval_days` = 7, `next_due` = `2026-02-20`
- **AND** the MQTT topic prefix is `flowl`
- **WHEN** attributes are published
- **THEN** a retained JSON message is published to `flowl/plant/1/attributes`
- **AND** the payload contains `next_due`, `last_watered`, and `watering_interval_days`

#### Scenario: Attributes for never-watered plant

- **GIVEN** a plant with `last_watered` = NULL
- **WHEN** attributes are published
- **THEN** the payload contains `next_due` = null, `last_watered` = null, and the `watering_interval_days` value

### Requirement: Background State Checker

The application SHALL run a background task that periodically checks all plants for watering state transitions and publishes updates to MQTT.

#### Scenario: State transition detected

- **GIVEN** a plant was previously `ok`
- **AND** enough time has passed that it is now `due`
- **WHEN** the background checker runs
- **THEN** the new state `due` is published to the plant's MQTT state topic
- **AND** updated attributes are published to the plant's attributes topic

#### Scenario: No state change

- **GIVEN** a plant's watering status has not changed since last check
- **WHEN** the background checker runs
- **THEN** no MQTT message is published for that plant

#### Scenario: Checker interval

- **WHEN** the application is running
- **THEN** the background state checker runs every 60 seconds

#### Scenario: Full publish on startup

- **WHEN** the application starts
- **THEN** the background checker publishes discovery configs, current state, and attributes for all existing plants
