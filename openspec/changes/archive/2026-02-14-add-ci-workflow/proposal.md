## Why

The project has no continuous integration. Every push and pull request to `main` should automatically verify that the code compiles, passes linting, and all tests pass so regressions are caught before merging.

## What Changes

- Add a GitHub Actions CI workflow (`.github/workflows/ci.yml`) that runs on pushes and pull requests to `main`.
- The workflow includes three stages: setup (toolchain + cache + cross-compilers), lint (`cargo fmt --check` + `cargo clippy`), and test (`cargo test`).
- Multi-arch support: installs both `x86_64-unknown-linux-gnu` and `aarch64-unknown-linux-gnu` targets with the `gcc-aarch64-linux-gnu` cross-compiler, matching the pixoo-bridge.rs setup.

## Capabilities

### New Capabilities
- `core/foundation`: GitHub Actions CI workflow (build, lint, test), release-please for automated version management, and publish-release for multi-arch binary compilation, release asset upload, and Docker image publishing.

## Impact

- `.github/workflows/ci.yml`: New file containing the CI workflow definition.
- `.github/workflows/release-please.yml`: New file containing the release-please workflow.
- `.github/workflows/publish-release.yml`: New file containing the publish-release workflow (compile, upload assets, Docker build+push).
- `.github/CODEOWNERS`: New file assigning `@simplyroba` as default code owner.
- `.github/dependabot.yml`: New file configuring daily dependency updates for cargo and github-actions.
- `Dockerfile`: Multi-arch container image based on `debian:bookworm-slim` with health check.
