## MODIFIED Requirements

### Requirement: Migration Runner

The application SHALL run all pending sqlx migrations at startup before accepting HTTP requests.

#### Scenario: Plant and location tables created

- **WHEN** the application starts with the phase-2 migration pending
- **THEN** the `plants` table is created with columns: `id`, `name`, `species`, `icon`, `location_id`, `watering_interval_days`, `light_needs`, `notes`, `created_at`, `updated_at`
- **AND** the `locations` table is created with columns: `id`, `name`
- **AND** a foreign key from `plants.location_id` to `locations.id` is established
