## ADDED Requirements

### Requirement: User settings table

The system SHALL maintain a `user_settings` table with a single row containing `theme` and `locale` columns. The table SHALL be seeded with default values (`theme = 'system'`, `locale = 'en'`) on creation.

#### Scenario: Table exists after migration

- **WHEN** the application starts
- **THEN** the `user_settings` table contains exactly one row
- **AND** the row has `theme = 'system'` and `locale = 'en'` if no prior updates were made

#### Scenario: Single-row constraint

- **WHEN** an INSERT is attempted with `id != 1`
- **THEN** the database rejects it with a CHECK constraint violation

### Requirement: Get settings endpoint

The system SHALL expose `GET /api/settings` returning the current user settings as JSON.

#### Scenario: Fetch settings

- **WHEN** a GET request is made to `/api/settings`
- **THEN** the response status is 200
- **AND** the body contains `{"theme": "<value>", "locale": "<value>"}`

### Requirement: Update settings endpoint

The system SHALL expose `PUT /api/settings` accepting a JSON body with optional `theme` and `locale` fields. Only provided fields SHALL be updated; omitted fields SHALL retain their current values.

#### Scenario: Update theme only

- **WHEN** a PUT request is made to `/api/settings` with `{"theme": "dark"}`
- **THEN** the response status is 200
- **AND** the `theme` is updated to `"dark"`
- **AND** the `locale` remains unchanged

#### Scenario: Update locale only

- **WHEN** a PUT request is made to `/api/settings` with `{"locale": "de"}`
- **THEN** the response status is 200
- **AND** the `locale` is updated to `"de"`
- **AND** the `theme` remains unchanged

#### Scenario: Update both fields

- **WHEN** a PUT request is made to `/api/settings` with `{"theme": "light", "locale": "es"}`
- **THEN** the response status is 200
- **AND** both fields are updated
- **AND** the response body reflects the new values

#### Scenario: Empty body

- **WHEN** a PUT request is made to `/api/settings` with `{}`
- **THEN** the response status is 200
- **AND** no fields are changed

### Requirement: Settings input validation

The update endpoint SHALL reject invalid values with HTTP 422.

#### Scenario: Invalid theme value

- **WHEN** a PUT request is made to `/api/settings` with `{"theme": "blue"}`
- **THEN** the response status is 422
- **AND** the body contains `{"message": "..."}`

#### Scenario: Invalid locale value

- **WHEN** a PUT request is made to `/api/settings` with `{"locale": "fr"}`
- **THEN** the response status is 422
- **AND** the body contains `{"message": "..."}`

#### Scenario: Valid theme values

- **WHEN** a PUT request is made with `theme` set to `"light"`, `"dark"`, or `"system"`
- **THEN** the value is accepted

#### Scenario: Valid locale values

- **WHEN** a PUT request is made with `locale` set to `"en"`, `"de"`, or `"es"`
- **THEN** the value is accepted
