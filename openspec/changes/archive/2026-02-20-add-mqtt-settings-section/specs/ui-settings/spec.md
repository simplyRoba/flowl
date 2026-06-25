## ADDED Requirements

### Requirement: MQTT Section

The settings page SHALL include an "MQTT" section displaying the MQTT connection status and configuration, fetched from `GET /api/mqtt/status`.

#### Scenario: MQTT connected

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "connected"`
- **THEN** the MQTT section shows a "Status" row with a green dot and "Connected" text
- **AND** a "Broker" row displays the broker address
- **AND** a "Topic prefix" row displays the configured prefix

#### Scenario: MQTT disconnected

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "disconnected"`
- **THEN** the MQTT section shows a "Status" row with a muted dot and "Disconnected" text
- **AND** a "Broker" row displays the broker address
- **AND** a "Topic prefix" row displays the configured prefix

#### Scenario: MQTT disabled

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "disabled"`
- **THEN** the MQTT section shows a "Status" row with "Disabled" text and no indicator dot
- **AND** the "Broker" and "Topic prefix" rows are NOT displayed

#### Scenario: API fetch failure

- **WHEN** the settings page loads and the `/api/mqtt/status` request fails
- **THEN** the MQTT section is not rendered

#### Scenario: Section ordering

- **WHEN** the settings page loads
- **THEN** the MQTT section appears after "Locations" and before "Data"
