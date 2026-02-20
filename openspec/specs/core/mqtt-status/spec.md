## Purpose

API endpoint exposing MQTT runtime connection state and configuration (broker, prefix, enabled/disabled).

## Requirements

### Requirement: MQTT Status Endpoint

The API SHALL expose a `GET /api/mqtt/status` endpoint that returns the current MQTT connection state and configuration as JSON.

#### Scenario: MQTT connected

- **WHEN** a GET request is made to `/api/mqtt/status`
- **AND** MQTT is enabled and the client is connected
- **THEN** the server responds with HTTP 200
- **AND** the response body is `{ "status": "connected", "broker": "<host>:<port>", "topic_prefix": "<prefix>" }`

#### Scenario: MQTT disconnected

- **WHEN** a GET request is made to `/api/mqtt/status`
- **AND** MQTT is enabled but the client is not connected
- **THEN** the server responds with HTTP 200
- **AND** the response body is `{ "status": "disconnected", "broker": "<host>:<port>", "topic_prefix": "<prefix>" }`

#### Scenario: MQTT disabled

- **WHEN** a GET request is made to `/api/mqtt/status`
- **AND** MQTT is disabled via `FLOWL_MQTT_DISABLED=true`
- **THEN** the server responds with HTTP 200
- **AND** the response body is `{ "status": "disabled", "broker": null, "topic_prefix": null }`
