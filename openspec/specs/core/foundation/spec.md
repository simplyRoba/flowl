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

The CI pipeline SHALL verify Rust formatting with `cargo fmt -- --check`, run `cargo clippy -- -D warnings`, verify UI formatting with `npm run format:check --prefix ui`, run UI linting with `npm run lint --prefix ui`, and run UI type/framework checks with `npm run check --prefix ui`.

#### Scenario: Formatting violation

- **WHEN** source code does not conform to `rustfmt` defaults
- **THEN** the lint job fails

#### Scenario: Clippy warning

- **WHEN** `cargo clippy` emits a warning
- **THEN** the lint job fails (warnings treated as errors)

#### Scenario: UI formatting violation

- **WHEN** UI source code does not conform to the configured Prettier rules
- **THEN** the UI lint job fails

#### Scenario: UI lint violation

- **WHEN** `npm run lint --prefix ui` reports an ESLint error
- **THEN** the UI lint job fails

#### Scenario: UI type or Svelte check failure

- **WHEN** `npm run check --prefix ui` reports a Svelte or TypeScript error
- **THEN** the UI lint job fails

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
