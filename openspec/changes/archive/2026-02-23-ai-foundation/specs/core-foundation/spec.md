## MODIFIED Requirements

### Requirement: Dockerfile

A Dockerfile SHALL provide a minimal multi-arch container image based on `debian:bookworm-slim` that runs the `flowl` binary as a non-root user with a health check.

#### Scenario: Container starts

- **WHEN** the container is started
- **THEN** the `flowl` binary runs on port 4100 by default

#### Scenario: Health check

- **WHEN** the container is running
- **THEN** the health check queries `http://localhost:${FLOWL_PORT:-4100}/health` every 30 seconds
- **AND** the `/health` endpoint returns HTTP 200 with `{"status": "ok"}`

#### Scenario: Non-root execution

- **WHEN** the container runs
- **THEN** the process runs as UID 1000:1000

#### Scenario: Database volume

- **WHEN** the container is started with a volume mounted at `/data`
- **THEN** the SQLite database is persisted at `/data/flowl.db`

#### Scenario: AI environment variables

- **WHEN** the container is started with `FLOWL_AI_API_KEY` set
- **THEN** the AI provider is initialized and `GET /api/ai/status` returns `enabled: true`
- **AND** `FLOWL_AI_BASE_URL` and `FLOWL_AI_MODEL` are optional with defaults
