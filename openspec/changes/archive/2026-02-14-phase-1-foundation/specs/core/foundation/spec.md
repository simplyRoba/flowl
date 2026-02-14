## MODIFIED Requirements

### Requirement: Dockerfile

A Dockerfile SHALL provide a minimal multi-arch container image based on `debian:bookworm-slim` that runs the `flowl` binary as a non-root user with a health check.

#### Scenario: Container starts

- **WHEN** the container is started
- **THEN** the `flowl` binary runs on port 8080 by default

#### Scenario: Health check

- **WHEN** the container is running
- **THEN** the health check queries `http://localhost:${FLOWL_PORT:-8080}/health` every 30 seconds
- **AND** the `/health` endpoint returns HTTP 200 with `{"status": "ok"}`

#### Scenario: Non-root execution

- **WHEN** the container runs
- **THEN** the process runs as UID 1000:1000

#### Scenario: Database volume

- **WHEN** the container is started with a volume mounted at `/data`
- **THEN** the SQLite database is persisted at `/data/flowl.db`
