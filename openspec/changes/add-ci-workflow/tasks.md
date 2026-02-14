## 1. CI Workflow

- [x] 1.1 Create `.github/workflows/ci.yml` with setup, lint, and test jobs based on the pixoo-bridge.rs CI workflow, including multi-arch cross-compilation targets (amd64 + arm64).

## 2. Release Please Workflow

- [x] 2.1 Create `.github/workflows/release-please.yml` with the release-please action for automated version management, matching the pixoo-bridge.rs setup.

## 3. Publish Release Workflow

- [x] 3.1 Create `.github/workflows/publish-release.yml` with setup, compile (matrix: amd64/arm64), upload-release-assets, and docker jobs, adapted from pixoo-bridge.rs (binary name `flowl` instead of `pixoo-bridge`).

## 4. CODEOWNERS and Dependabot

- [x] 4.1 Create `.github/CODEOWNERS` assigning `@simplyroba` as default owner.
- [x] 4.2 Create `.github/dependabot.yml` with daily cargo and github-actions updates.

## 5. Dockerfile

- [x] 5.1 Create `Dockerfile` based on `debian:bookworm-slim` with multi-arch support, health check, and non-root execution.

## 6. Verify

- [x] 6.1 Confirm all workflow YAMLs, CODEOWNERS, dependabot config, and Dockerfile are valid and match the spec requirements.
