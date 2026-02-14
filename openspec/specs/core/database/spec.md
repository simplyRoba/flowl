## Purpose

SQLite connection pool via sqlx, migration runner, and database file configuration.

## Requirements

### Requirement: SQLite Connection Pool

The application SHALL create a SQLite connection pool via sqlx at startup, using the path specified by `FLOWL_DB_PATH` (default `/data/flowl.db`).

#### Scenario: Database created on first startup

- **WHEN** the application starts and no database file exists at `FLOWL_DB_PATH`
- **THEN** a new SQLite database file is created at that path

#### Scenario: Existing database reused

- **WHEN** the application starts and a database file exists at `FLOWL_DB_PATH`
- **THEN** the existing database is opened without data loss

#### Scenario: Custom database path

- **WHEN** the application starts with `FLOWL_DB_PATH=/custom/path/flowl.db`
- **THEN** the database is created or opened at `/custom/path/flowl.db`

### Requirement: Migration Runner

The application SHALL run all pending sqlx migrations at startup before accepting HTTP requests.

#### Scenario: Migrations applied on startup

- **WHEN** the application starts with pending migrations
- **THEN** all pending migrations are applied in order

#### Scenario: No pending migrations

- **WHEN** the application starts with all migrations already applied
- **THEN** startup proceeds without errors

#### Scenario: Migration failure

- **WHEN** a migration fails to apply
- **THEN** the application exits with a non-zero exit code
- **AND** an error message is logged describing the failure

#### Scenario: Plant and location tables created

- **WHEN** the application starts with the phase-2 migration pending
- **THEN** the `plants` table is created with columns: `id`, `name`, `species`, `icon`, `location_id`, `watering_interval_days`, `light_needs`, `notes`, `created_at`, `updated_at`
- **AND** the `locations` table is created with columns: `id`, `name`
- **AND** a foreign key from `plants.location_id` to `locations.id` is established
