## Context

Theme and locale preferences are stored exclusively in `localStorage` today (`flowl.theme`, `flowl.locale`). The backend has no awareness of user preferences, which blocks passing language context to the AI provider. This is a single-user application (no auth), so a single-row settings table is sufficient.

The codebase uses inline `sqlx` queries in handler files, `State(pool): State<SqlitePool>` extraction, and `JsonBody<T>` for input validation. Frontend stores (`themePreference`, `locale`) are Svelte writables with dedicated `set*` functions that currently write to `localStorage`.

## Goals / Non-Goals

**Goals:**
- Persist theme and locale preferences in SQLite so they survive device/browser changes
- Expose `GET /api/settings` and `PUT /api/settings` endpoints
- Frontend syncs preferences to the API while keeping stores reactive
- Validate inputs server-side (only accept known theme/locale values)

**Non-Goals:**
- Multi-user support or authentication — single-row table with `CHECK (id = 1)`
- Injecting language into AI calls — that's a follow-up change
- Migrating other configuration (MQTT, AI keys) into this table — those remain env vars
- Removing `localStorage` fallback — keep it as offline/startup fallback

## Decisions

### Single-row table vs key-value store

**Chosen: Single-row with typed columns.**

A key-value table (`key TEXT PRIMARY KEY, value TEXT`) is flexible but loses type safety and requires multiple queries or pivoting. Since we have exactly two settings with known types and enums, a single row with `theme TEXT` and `locale TEXT` is simpler, matches the existing migration style, and lets `sqlx::query_as` map directly to a struct.

*Alternative considered:* Key-value table — rejected because it adds complexity for no real benefit given the small, fixed set of settings.

### Partial updates via COALESCE

`PUT /api/settings` accepts an object where both fields are optional. The query uses `COALESCE(?, column)` so clients can update one field without supplying the other. This avoids read-modify-write races and keeps the API ergonomic.

*Alternative considered:* Separate `PUT /api/settings/theme` and `PUT /api/settings/locale` endpoints — rejected as over-engineered for two fields.

### Frontend sync strategy

On `initTheme` / `initLocale` (called in `+layout.svelte` `onMount`), fetch `GET /api/settings` once and seed both stores. On preference change, fire-and-forget `PUT /api/settings` (the store updates optimistically). Keep `localStorage` writes as a fallback so the UI works instantly on next load even if the API is slow.

*Alternative considered:* Remove `localStorage` entirely — rejected because it would cause a flash of default theme on every page load while waiting for the API response.

### Input validation

Server-side validation rejects unknown values with `422 Unprocessable Entity`. Valid values:
- `theme`: `light`, `dark`, `system`
- `locale`: `en`, `de`, `es`

This uses the existing `ApiError::Validation` variant.

## Risks / Trade-offs

- **[Single-user assumption]** → The `CHECK (id = 1)` constraint enforces exactly one row. If multi-user is ever needed, a migration to add a `user_id` column would be required. Acceptable for current scope.
- **[Fire-and-forget writes]** → If `PUT /api/settings` fails silently, the preference reverts on next full load. Mitigation: `localStorage` acts as a write-ahead cache, and the UI stays consistent within the session.
- **[Extra API call on startup]** → One `GET /api/settings` on layout mount. Negligible latency for a local/LAN service.
