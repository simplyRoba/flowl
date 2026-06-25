## ADDED Requirements

### Requirement: About Section

The settings page SHALL include an "About" section displaying application metadata fetched from `GET /api/info`.

#### Scenario: About section displays version

- **WHEN** the settings page loads
- **THEN** the About section shows a "Version" row with the app version from the API

#### Scenario: About section displays source link

- **WHEN** the settings page loads
- **THEN** the About section shows a "Source" row with the repository URL as a clickable link
- **AND** the link text displays the URL without the `https://` prefix

#### Scenario: About section displays license

- **WHEN** the settings page loads
- **THEN** the About section shows a "License" row with the license identifier from the API

#### Scenario: API fetch failure

- **WHEN** the settings page loads and the `/api/info` request fails
- **THEN** the About section is not rendered
