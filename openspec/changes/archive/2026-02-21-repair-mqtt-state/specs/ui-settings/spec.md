## MODIFIED Requirements

### Requirement: MQTT Section

The settings page SHALL include an "MQTT" section displaying the MQTT connection status, configuration (fetched from `GET /api/mqtt/status`), and a repair action.

#### Scenario: MQTT connected

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "connected"`
- **THEN** the MQTT section shows a "Status" row with a green dot and "Connected" text
- **AND** a "Broker" row displays the broker address
- **AND** a "Topic prefix" row displays the configured prefix
- **AND** a "Repair" button is displayed and enabled

#### Scenario: MQTT disconnected

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "disconnected"`
- **THEN** the MQTT section shows a "Status" row with a muted dot and "Disconnected" text
- **AND** a "Broker" row displays the broker address
- **AND** a "Topic prefix" row displays the configured prefix
- **AND** a "Repair" button is displayed but disabled

#### Scenario: MQTT disabled

- **WHEN** the settings page loads
- **AND** the MQTT status API returns `status: "disabled"`
- **THEN** the MQTT section shows a "Status" row with "Disabled" text and no indicator dot
- **AND** the "Broker" and "Topic prefix" rows are NOT displayed
- **AND** the "Repair" button is NOT displayed

#### Scenario: API fetch failure

- **WHEN** the settings page loads and the `/api/mqtt/status` request fails
- **THEN** the MQTT section is not rendered

#### Scenario: Section ordering

- **WHEN** the settings page loads
- **THEN** the MQTT section appears after "Locations" and before "Data"

#### Scenario: Repair button triggers repair

- **GIVEN** the MQTT section is visible and status is "connected"
- **WHEN** the user clicks the "Repair" button
- **THEN** the button shows a loading state
- **AND** a POST request is sent to `/api/mqtt/repair`

#### Scenario: Repair success feedback

- **GIVEN** the user clicked the "Repair" button
- **WHEN** the API responds with HTTP 200 and `{ "cleared": N, "published": M }`
- **THEN** the button returns to its default state
- **AND** an inline message shows the cleared and published counts

#### Scenario: Repair error feedback

- **GIVEN** the user clicked the "Repair" button
- **WHEN** the API responds with an error (409 or 503)
- **THEN** the button returns to its default state
- **AND** an inline error message is displayed
