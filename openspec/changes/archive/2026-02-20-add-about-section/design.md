## Context

The settings page currently has Appearance and Locations sections. The mockup includes an About section showing Version and Source. We agreed to also show License (AGPL requires visibility), all sourced from `Cargo.toml` compile-time macros. No build commit hash.

The backend already has a `GET /health` endpoint returning `{"status": "ok"}`. The frontend is a SvelteKit SPA fetching data from `/api/*`.

## Goals / Non-Goals

**Goals:**
- Expose app metadata (version, repository, license) via a dedicated API endpoint
- Display this metadata in an About section on the settings page

**Non-Goals:**
- Build commit hash or build date (explicitly excluded)
- Runtime system info (uptime, memory, OS)
- Extending the existing `/health` endpoint (keep health checks simple)

## Decisions

### 1. Separate `/api/info` endpoint vs extending `/health`

**Decision**: New `GET /api/info` endpoint under the API router.

**Rationale**: The `/health` endpoint is used by container orchestrators and should stay minimal (`{"status": "ok"}`). Mixing metadata into it couples health checks with app identity. A separate endpoint keeps both focused.

**Alternative considered**: Extending `/health` with optional fields. Rejected because health probes should be stable and fast.

### 2. Compile-time macros vs config file vs env vars

**Decision**: Use `env!("CARGO_PKG_VERSION")`, `env!("CARGO_PKG_REPOSITORY")`, `env!("CARGO_PKG_LICENSE")`.

**Rationale**: These are baked in at compile time directly from `Cargo.toml`, always in sync, zero runtime cost, no extra files or env vars to manage. The values are `&'static str` — no allocation needed.

### 3. Source link display

**Decision**: Show the repository URL as a clickable link in the UI, displayed as the short form (without `https://`).

**Rationale**: Matches the mockup pattern (`github.com/simplyRoba/flowl`) and keeps the row compact.

## Risks / Trade-offs

- **[Minimal risk]** The compile-time macros make the version static per binary build. This is correct behavior — the version should reflect the compiled artifact, not a mutable config. → No mitigation needed.
