## ADDED Requirements

### Requirement: CI Trigger

The CI pipeline SHALL execute on every push to `main` and on every pull request targeting `main`.

#### Scenario: Push to main triggers CI

- **WHEN** a commit is pushed to the `main` branch
- **THEN** the CI workflow runs all jobs (setup, lint, test)

#### Scenario: PR to main triggers CI

- **WHEN** a pull request targeting `main` is opened or updated
- **THEN** the CI workflow runs all jobs (setup, lint, test)

### Requirement: Toolchain Setup

The CI pipeline SHALL install the stable Rust toolchain with `rustfmt` and `clippy` components, add `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` targets, install `gcc-aarch64-linux-gnu` cross-compiler, and cache Cargo artifacts.

#### Scenario: Cache hit

- **WHEN** the `Cargo.lock` hash matches a previously cached run
- **THEN** the cached Cargo registry, git, and target directories are restored

#### Scenario: Cross-compilation targets available

- **WHEN** the setup job completes
- **THEN** both `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` targets are installed
- **AND** the `gcc-aarch64-linux-gnu` cross-compiler is available

### Requirement: Lint Checks

The CI pipeline SHALL verify formatting with `cargo fmt -- --check` and run `cargo clippy -- -D warnings`.

#### Scenario: Formatting violation

- **WHEN** source code does not conform to `rustfmt` defaults
- **THEN** the lint job fails

#### Scenario: Clippy warning

- **WHEN** `cargo clippy` emits a warning
- **THEN** the lint job fails (warnings treated as errors)

### Requirement: Test Execution

The CI pipeline SHALL run `cargo test` and fail if any test does not pass.

#### Scenario: All tests pass

- **WHEN** all tests pass
- **THEN** the test job succeeds

#### Scenario: Test failure

- **WHEN** any test fails
- **THEN** the test job fails

### Requirement: Release Please

A release-please workflow SHALL run on every push to `main`, automatically creating or updating a release PR based on conventional commits.

#### Scenario: Conventional commit pushed to main

- **WHEN** a conventional commit (e.g., `feat:`, `fix:`) is pushed to `main`
- **THEN** release-please creates or updates a release PR with the appropriate version bump and changelog

#### Scenario: Release PR merged

- **WHEN** the release-please PR is merged
- **THEN** a GitHub release is created with the new version tag

### Requirement: Publish Release

A publish-release workflow SHALL trigger when a GitHub release is published and compile multi-arch binaries, upload them as release assets, and build+push a Docker image.

#### Scenario: Release published triggers compilation

- **WHEN** a GitHub release is published
- **THEN** the workflow compiles release binaries for `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`

#### Scenario: Binaries uploaded to release

- **WHEN** compilation succeeds for both architectures
- **THEN** `flowl-linux-amd64` and `flowl-linux-arm64` binaries are attached to the GitHub release

#### Scenario: Docker image published

- **WHEN** release assets are uploaded
- **THEN** a multi-platform Docker image (`linux/amd64`, `linux/arm64`) is built and pushed to `ghcr.io`
- **AND** the image is tagged with `latest`, semver version, major.minor, and major tags

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

### Requirement: Code Ownership

A CODEOWNERS file SHALL assign `@simplyroba` as the default reviewer for all files.

#### Scenario: PR opened

- **WHEN** a pull request is opened
- **THEN** `@simplyroba` is automatically requested as a reviewer

### Requirement: Dependency Updates

Dependabot SHALL be configured to check for daily updates to cargo dependencies and GitHub Actions, using conventional commit prefixes.

#### Scenario: Cargo dependency update available

- **WHEN** a new version of a cargo dependency is available
- **THEN** dependabot creates a PR with commit prefix `fix(deps): `

#### Scenario: GitHub Actions update available

- **WHEN** a new version of a GitHub Action is available
- **THEN** dependabot creates a PR with commit prefix `fix(ci): `
