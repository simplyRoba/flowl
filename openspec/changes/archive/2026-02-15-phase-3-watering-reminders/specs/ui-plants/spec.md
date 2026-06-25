## MODIFIED Requirements

### Requirement: Dashboard Watering Status Indicators

The dashboard plant cards SHALL display a visual indicator for plants that are due or overdue for watering.

#### Scenario: Plant overdue

- **WHEN** a plant card is rendered
- **AND** the plant's `watering_status` is `overdue`
- **THEN** a red status badge with "Overdue" is displayed on the card

#### Scenario: Plant due

- **WHEN** a plant card is rendered
- **AND** the plant's `watering_status` is `due`
- **THEN** an amber status badge with "Due" is displayed on the card

#### Scenario: Plant ok

- **WHEN** a plant card is rendered
- **AND** the plant's `watering_status` is `ok`
- **THEN** no watering status indicator is shown

### Requirement: Plant Detail Watering Section

The plant detail view SHALL display watering status and a "Water now" action.

#### Scenario: Watering status displayed

- **WHEN** the plant detail view is rendered
- **THEN** the watering info card shows the current watering status (`ok`, `due`, or `overdue`) with a colored indicator
- **AND** the last watered date is shown (or "Never" if null)
- **AND** the next due date is shown (or "N/A" if never watered)
- **AND** the watering interval is shown

#### Scenario: Water now action

- **WHEN** the user clicks the "Water now" button on the plant detail view
- **THEN** a `POST /api/plants/:id/water` request is sent
- **AND** the view refreshes to show updated watering status
- **AND** the `last_watered` field updates to the current time

#### Scenario: Water now when already ok

- **WHEN** the user clicks "Water now" on a plant that is already `ok`
- **THEN** the watering is recorded (last_watered updates)
- **AND** the status remains `ok` with a new next_due date

### Requirement: API Client — Water Plant

The frontend API client SHALL provide a `waterPlant` function.

#### Scenario: Water plant call

- **WHEN** `waterPlant(1)` is called
- **THEN** a `POST` request is made to `/api/plants/1/water`
- **AND** the updated `Plant` object is returned

### Requirement: Plant Store — Water Plant

The plant store SHALL provide a `waterPlant` function that calls the API and updates the store.

#### Scenario: Store updated after watering

- **WHEN** `waterPlant(1)` is called on the store
- **THEN** the plant list and current plant are updated with the new watering data
