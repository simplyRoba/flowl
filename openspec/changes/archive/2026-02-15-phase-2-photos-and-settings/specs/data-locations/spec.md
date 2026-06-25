## Purpose

Location entity — MODIFIED to include `plant_count` in the list response.

## Requirements

### Requirement: Plant Count in List Response

The `GET /api/locations` response SHALL include a `plant_count` field for each location indicating how many plants reference that location.

#### Scenario: Location with plants

- **WHEN** a location has 3 plants assigned to it
- **THEN** the list response includes `plant_count: 3` for that location

#### Scenario: Location with no plants

- **WHEN** a location has no plants assigned to it
- **THEN** the list response includes `plant_count: 0` for that location

#### Scenario: Newly created location

- **WHEN** a location is created via `POST /api/locations`
- **THEN** the response includes `plant_count: 0`
