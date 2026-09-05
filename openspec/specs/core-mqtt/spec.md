## Purpose

MQTT broker interoperability, connection and status behavior, graceful shutdown, Home Assistant auto-discovery, watering-state synchronization, and reconciliation.

## Requirements

### Requirement: MQTT can be disabled via environment flag

The application SHALL allow operators to disable MQTT by setting `FLOWL_MQTT_DISABLED=true`.

#### Scenario: MQTT disabled by configuration

- **WHEN** the application starts with `FLOWL_MQTT_DISABLED=true`
- **THEN** the HTTP server starts normally without attempting an MQTT connection or publish
- **AND** runtime status reports `disabled`
- **AND** a log message notes that MQTT is disabled

### Requirement: MQTT Connection and Status

The application SHALL connect to the broker specified by `FLOWL_MQTT_HOST` (default `localhost`) and `FLOWL_MQTT_PORT` (default `1883`) on startup when `FLOWL_MQTT_DISABLED` is not true, and SHALL maintain an accurate runtime connection status.

#### Scenario: Successful connection

- **WHEN** the application starts and the MQTT broker is reachable
- **AND** `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** an MQTT connection is established
- **AND** runtime status reports `connected`
- **AND** a log message confirms the connection

#### Scenario: Broker unreachable at startup

- **WHEN** the application starts, the MQTT broker is not reachable, and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** the HTTP server starts normally
- **AND** runtime status reports `disconnected`
- **AND** the application automatically retries the connection
- **AND** a warning is logged

#### Scenario: MQTT disabled via flag

- **WHEN** the application starts with `FLOWL_MQTT_DISABLED=true`
- **THEN** no MQTT connection attempt is made
- **AND** runtime status reports `disabled`
- **AND** a log message notes that MQTT is disabled

### Requirement: MQTT Reconnection

The application SHALL automatically retry a failed connection and reconnect when an established connection to the broker is lost, provided MQTT is enabled (`FLOWL_MQTT_DISABLED` is not true). Runtime connection status SHALL reflect connection state transitions.

#### Scenario: Connection lost and recovered

- **WHEN** the MQTT connection drops and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** runtime status changes to `disconnected`
- **AND** the application automatically attempts to reconnect
- **AND** a warning is logged on disconnect
- **AND** runtime status changes to `connected` on successful reconnect
- **AND** an info message is logged on successful reconnect

### Requirement: MQTT Configuration

The MQTT integration SHALL use `FLOWL_MQTT_TOPIC_PREFIX` (default `flowl`) as the base prefix for all topics when MQTT is enabled.

#### Scenario: Default topic prefix

- **WHEN** the application starts without `FLOWL_MQTT_TOPIC_PREFIX` set and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** MQTT topics use `flowl` as their prefix

#### Scenario: Custom topic prefix

- **WHEN** the application starts with `FLOWL_MQTT_TOPIC_PREFIX=myplants` and MQTT is enabled
- **THEN** MQTT topics use `myplants` as their prefix

### Requirement: MQTT Graceful Disconnect

The application SHALL disconnect cleanly when the application shuts down, unless MQTT is disabled.

#### Scenario: Application shutdown

- **WHEN** the application receives a shutdown signal and `FLOWL_MQTT_DISABLED` is not `true`
- **THEN** an MQTT disconnect packet is sent to the broker

### Requirement: Home Assistant MQTT Auto-Discovery

The application SHALL publish retained MQTT auto-discovery configurations for each plant, registering them as Home Assistant sensor entities with a `json_attributes_topic`, whenever MQTT is enabled.

#### Scenario: Discovery config published

- **GIVEN** a plant with id 1 and name "Monstera"
- **AND** the MQTT topic prefix is `flowl`
- **AND** `FLOWL_MQTT_DISABLED` is not `true`
- **WHEN** a discovery config is synchronized
- **THEN** a retained JSON message is published to `homeassistant/sensor/flowl_plant_1/config`
- **AND** the payload contains `name`, `unique_id`, `state_topic`, `json_attributes_topic`, `icon`, and `device` fields
- **AND** `state_topic` is `flowl/plant/1/state`
- **AND** `json_attributes_topic` is `flowl/plant/1/attributes`

#### Scenario: Discovery config removed

- **GIVEN** a plant with id 1 is deleted
- **WHEN** deletion triggers MQTT cleanup and MQTT is enabled
- **THEN** an empty retained payload is published to `homeassistant/sensor/flowl_plant_1/config`
- **AND** an empty retained payload is published to `flowl/plant/1/state`
- **AND** an empty retained payload is published to `flowl/plant/1/attributes`

### Requirement: MQTT State Publishing

The application SHALL publish watering state to retained MQTT topics whenever MQTT is enabled.

#### Scenario: State published

- **GIVEN** a plant with id 1 and watering status `due`
- **AND** the MQTT topic prefix is `flowl`
- **AND** `FLOWL_MQTT_DISABLED` is not `true`
- **WHEN** watering state is synchronized
- **THEN** the string `due` is published as a retained message to `flowl/plant/1/state`

#### Scenario: State values

- **WHEN** watering state is synchronized and MQTT is enabled
- **THEN** the payload is one of: `ok`, `due`, `overdue`

### Requirement: MQTT Attributes Publishing

The application SHALL publish plant watering attributes as a retained JSON object to a dedicated attributes topic whenever MQTT is enabled.

#### Scenario: Attributes published

- **GIVEN** a plant with id 1, `last_watered` = `2026-02-13T14:30:00`, `watering_interval_days` = 7, `next_due` = `2026-02-20`
- **AND** the MQTT topic prefix is `flowl`
- **AND** MQTT is enabled
- **WHEN** attributes are synchronized
- **THEN** a retained JSON message is published to `flowl/plant/1/attributes`
- **AND** the payload contains `next_due`, `last_watered`, and `watering_interval_days`

#### Scenario: Attributes for never-watered plant

- **GIVEN** a plant with `last_watered` = NULL
- **AND** MQTT is enabled
- **WHEN** attributes are synchronized
- **THEN** the payload contains `next_due` = null, `last_watered` = null, and the `watering_interval_days` value

### Requirement: MQTT Publish Recovery

The application SHALL retry failed MQTT publishes for a finite period before deferring recovery. A final publish failure SHALL be warning-logged and SHALL NOT fail the domain operation that triggered synchronization. Failed state or attribute publication SHALL remain eligible for periodic reconciliation; failed discovery publication SHALL remain recoverable by a later full synchronization or broker repair; and failed retained-topic removal SHALL remain recoverable by broker repair.

#### Scenario: Transient publish failure

- **GIVEN** an MQTT publish fails
- **WHEN** publish recovery executes
- **THEN** retry attempts occur for a finite period without delaying the triggering operation indefinitely
- **AND** each failed publish attempt is logged as a warning identifying the attempt

#### Scenario: Final publish failure

- **GIVEN** publish recovery cannot synchronize a plant's state or attributes
- **WHEN** the final failure is reached
- **THEN** a warning is logged indicating fallback to later reconciliation
- **AND** no error is returned to the triggering domain caller
- **AND** the affected data remains recoverable through its applicable reconciliation or repair path
- **AND** partial success of state or attributes does not suppress later reconciliation until both have succeeded

### Requirement: Periodic MQTT Reconciliation

The application SHALL reconcile plant watering state with MQTT every 60 minutes whenever MQTT is enabled. Confirmed initial connection and confirmed reconnection SHALL immediately trigger a full synchronization of discovery, state, and attributes for all existing plants.

#### Scenario: State transition detected

- **GIVEN** a plant was previously `ok`
- **AND** enough time has passed that it is now `due`
- **AND** MQTT is enabled
- **WHEN** periodic reconciliation runs
- **THEN** the changed watering state `due` is published to the plant's MQTT state topic
- **AND** updated attributes are published to the plant's MQTT attributes topic

#### Scenario: No state change or pending synchronization

- **GIVEN** a plant's watering status has not changed since the prior successful reconciliation
- **AND** no state or attribute publication remains pending
- **AND** MQTT is enabled
- **WHEN** periodic reconciliation runs
- **THEN** no MQTT message is published for that plant

#### Scenario: Partial publication remains pending

- **GIVEN** a plant's state or attributes did not publish successfully during an earlier synchronization
- **WHEN** periodic reconciliation runs
- **THEN** both the current state and attributes remain eligible for publication until both have succeeded

#### Scenario: Reconciliation interval

- **WHEN** the application is running and MQTT is enabled
- **THEN** periodic reconciliation runs every 60 minutes

#### Scenario: Full synchronization on initial connection

- **WHEN** the initial MQTT connection is confirmed
- **THEN** discovery configurations, current watering state, and attributes are immediately synchronized for all existing plants

#### Scenario: Full synchronization on reconnect

- **GIVEN** the MQTT connection was lost and subsequently recovered
- **WHEN** reconnection is confirmed
- **THEN** discovery configurations, current watering state, and attributes are immediately synchronized for all existing plants
