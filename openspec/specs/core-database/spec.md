## Purpose

Durable application data configuration and safe startup data upgrades.

## Requirements

### Requirement: Durable Application Data

The application SHALL durably preserve application data in the data file specified by `FLOWL_DB_PATH`, which defaults to `/data/flowl.db`.

#### Scenario: Data created on first startup

- **WHEN** the application starts and no data file exists at `FLOWL_DB_PATH`
- **THEN** a durable application data file is initialized at that exact path

#### Scenario: Existing data reopened

- **WHEN** the application starts and a data file exists at `FLOWL_DB_PATH`
- **THEN** that exact data file is reopened without loss

#### Scenario: Custom data path

- **WHEN** the application starts with `FLOWL_DB_PATH=/custom/path/flowl.db`
- **THEN** the application data file is initialized or reopened at exactly `/custom/path/flowl.db`

### Requirement: Startup Data Upgrades

The application SHALL safely apply all required data upgrades in order during startup before accepting HTTP requests.

#### Scenario: Upgrades applied on startup

- **WHEN** the application starts with required data upgrades pending
- **THEN** all required upgrades are applied in order before HTTP requests are accepted

#### Scenario: No upgrades required

- **WHEN** the application starts with application data already current
- **THEN** startup proceeds without errors

#### Scenario: Upgrade failure

- **WHEN** a required data upgrade fails
- **THEN** an error describing the failure is logged
- **AND** the application exits with a non-zero exit code

### Requirement: Persisted Data Compatibility

The application SHALL preserve compatibility with application data persisted by existing Flowl installations.

#### Scenario: Existing installation data upgraded

- **WHEN** the application starts with data persisted by an earlier compatible Flowl version
- **THEN** any required data upgrades complete before HTTP requests are accepted
- **AND** existing logical records, identifiers, timestamps, optional values, relationships, and managed-media associations remain available without loss, except for deliberate transformations defined by an upgrade
