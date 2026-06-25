## MODIFIED Requirements

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
