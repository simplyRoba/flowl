## Purpose

Official Linux release artifacts and container deployment guarantees.

## Requirements

### Requirement: Official Linux Binaries

Each published GitHub release SHALL provide Flowl binary assets built for the `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` targets.

#### Scenario: AMD64 release binary

- **WHEN** an official GitHub release is published
- **THEN** it includes a `flowl-linux-amd64` binary asset built for `x86_64-unknown-linux-gnu`

#### Scenario: ARM64 release binary

- **WHEN** an official GitHub release is published
- **THEN** it includes a `flowl-linux-arm64` binary asset built for `aarch64-unknown-linux-gnu`

### Requirement: Official Container Image

Each published GitHub release SHALL provide the official `ghcr.io/simplyroba/flowl` container image for `linux/amd64` and `linux/arm64`. Published images SHALL provide tags for `latest`, the full semantic version, the major and minor version, and the major version, with every tag supporting both platforms.

#### Scenario: Supported container platforms

- **WHEN** an official release container image is published
- **THEN** each published `ghcr.io/simplyroba/flowl` tag supports `linux/amd64` and `linux/arm64`

#### Scenario: Release image tags

- **WHEN** version 2.3.4 is published
- **THEN** the official image is available through tags for `latest`, `2.3.4`, `2.3`, and `2`

### Requirement: Container Runtime

The official container image SHALL run Flowl as UID 1000 and GID 1000, listen according to the `core-server` port contract, expose persistent application data through `/data`, and provide an image-level health check based on the `core-server` health endpoint.

#### Scenario: Health check follows the configured port

- **WHEN** the container starts on the port selected according to `core-server`
- **THEN** the image-level health check queries `/health` on that port

#### Scenario: Healthy container

- **WHEN** the container's `/health` endpoint reports success
- **THEN** the image-level health probe succeeds

#### Scenario: Unhealthy container

- **WHEN** the container's `/health` endpoint cannot be reached or returns a non-success response
- **THEN** the image-level health probe fails

#### Scenario: Non-root execution

- **WHEN** the container runs
- **THEN** the Flowl process runs as UID 1000 and GID 1000

#### Scenario: Persistent application data

- **WHEN** persistent storage is mounted at `/data`
- **THEN** the default `core-database` data file at `/data/flowl.db` is retained across container replacement
