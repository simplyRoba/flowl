## ADDED Requirements

### Requirement: AI Assistant Section

The settings page SHALL include an "AI Assistant" section displaying the AI configuration status fetched from `GET /api/ai/status`.

#### Scenario: AI enabled

- **WHEN** the settings page loads
- **AND** the AI status API returns `enabled: true`
- **THEN** the AI Assistant section shows a "Status" row with a green dot and "Enabled" text
- **AND** a "Provider" row displays the hostname extracted from `base_url`
- **AND** a "Model" row displays the `model` value

#### Scenario: AI disabled

- **WHEN** the settings page loads
- **AND** the AI status API returns `enabled: false`
- **THEN** the AI Assistant section shows a "Status" row with "Disabled" text and no indicator dot
- **AND** a hint "Set FLOWL_AI_API_KEY to enable." is displayed
- **AND** the "Provider" and "Model" rows are NOT displayed

#### Scenario: API fetch failure

- **WHEN** the settings page loads and the `/api/ai/status` request fails
- **THEN** the AI Assistant section is not rendered

#### Scenario: Section ordering

- **WHEN** the settings page loads
- **THEN** the AI Assistant section appears after "MQTT" and before "Data"
