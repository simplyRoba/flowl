## MODIFIED Requirements

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
