## ADDED Requirements

### Requirement: MQTT Client Connection

The application SHALL connect an MQTT client to the broker specified by `FLOWL_MQTT_HOST` (default `localhost`) and `FLOWL_MQTT_PORT` (default `1883`) on startup.

#### Scenario: Successful connection

- **WHEN** the application starts and the MQTT broker is reachable
- **THEN** the MQTT client connects successfully
- **AND** a log message confirms the connection

#### Scenario: Broker unreachable at startup

- **WHEN** the application starts and the MQTT broker is not reachable
- **THEN** the HTTP server starts normally
- **AND** the MQTT client retries connection in the background
- **AND** a warning is logged

### Requirement: MQTT Reconnection

The MQTT client SHALL automatically reconnect when the connection to the broker is lost.

#### Scenario: Connection lost and recovered

- **WHEN** the MQTT connection drops
- **THEN** the client automatically attempts to reconnect
- **AND** a warning is logged on disconnect
- **AND** an info message is logged on successful reconnect

### Requirement: MQTT Configuration

The MQTT client SHALL use `FLOWL_MQTT_TOPIC_PREFIX` (default `flowl`) as the base prefix for all topics.

#### Scenario: Default topic prefix

- **WHEN** the application starts without `FLOWL_MQTT_TOPIC_PREFIX` set
- **THEN** the MQTT client uses `flowl` as the topic prefix

#### Scenario: Custom topic prefix

- **WHEN** the application starts with `FLOWL_MQTT_TOPIC_PREFIX=myplants`
- **THEN** the MQTT client uses `myplants` as the topic prefix

### Requirement: MQTT Graceful Disconnect

The MQTT client SHALL disconnect cleanly when the application shuts down.

#### Scenario: Application shutdown

- **WHEN** the application receives a shutdown signal
- **THEN** the MQTT client sends a disconnect packet to the broker
