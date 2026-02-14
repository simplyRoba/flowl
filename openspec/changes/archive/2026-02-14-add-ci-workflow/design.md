## Context

The project is a new Rust service with no CI in place. The pixoo-bridge.rs project has a proven CI workflow that we can adapt directly, including multi-arch cross-compilation support.

## Goals / Non-Goals

**Goals:**
- Automated lint and test on every push/PR to main
- Cargo registry/target caching for faster builds
- Three-job structure: setup → lint + test (parallel)
- Multi-arch toolchain: `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu`

**Non-Goals:**
- (none currently)

## Decisions

### Decision 1: Three-job structure with shared cache

Use a `setup` job to install the toolchain and prime the Cargo cache, then run `lint` and `test` jobs in parallel depending on `setup`. This mirrors the pixoo-bridge.rs pattern and keeps CI runs fast.

### Decision 2: Multi-arch cross-compilation

Install both `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` targets with `gcc-aarch64-linux-gnu` cross-compiler, matching the pixoo-bridge.rs setup for future multi-arch Docker builds.

### Decision 3: Same action versions

Use `actions/checkout@v6` and `actions/cache@v5` matching the pixoo-bridge.rs workflow for consistency across projects.

### Decision 4: Release-please for automated releases

Use `googleapis/release-please-action@v4` with `release-type: rust`, identical to pixoo-bridge.rs. Runs as a separate workflow on pushes to `main`. Uses a `RELEASE_PLEASE_TOKEN` secret for PR creation permissions.

### Decision 5: Publish-release workflow on release published

Triggered by `release: [published]` events (created by release-please). Four-job pipeline matching pixoo-bridge.rs:
1. **setup** — install toolchain, cross-compilers, prime cache
2. **compile** — matrix build for `linux-amd64` and `linux-arm64`, upload artifacts
3. **upload-release-assets** — attach named binaries (`flowl-linux-amd64`, `flowl-linux-arm64`) to the GitHub release
4. **docker** — build multi-platform Docker image and push to `ghcr.io` with semver tags

### Decision 6: Dockerfile based on debian:bookworm-slim

Use `debian:bookworm-slim` as the base image (matching pixoo-bridge.rs) for `ca-certificates` and `curl` (needed for HTTPS and health checks). The binary is copied from `release-artifacts/linux-${TARGETARCH}/flowl` using Docker's `TARGETARCH` build arg for multi-platform support. Runs as non-root (UID 1000). Default port 8080, configurable via `FLOWL_PORT` env var.

### Decision 7: CODEOWNERS and Dependabot

Copy CODEOWNERS (`@simplyroba` for all files) and dependabot config (daily cargo + github-actions updates with conventional commit prefixes) directly from pixoo-bridge.rs for consistency.
