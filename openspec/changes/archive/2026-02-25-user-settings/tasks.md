## 1. Database

- [x] 1.1 Create migration `20260225000000_add_user_settings.sql` with single-row `user_settings` table (`id INTEGER PRIMARY KEY DEFAULT 1 CHECK (id = 1)`, `theme TEXT NOT NULL DEFAULT 'system'`, `locale TEXT NOT NULL DEFAULT 'en'`) and seed row
- [x] 1.2 Verify migration runs cleanly against a fresh database

## 2. Backend API

- [x] 2.1 Create `src/api/settings.rs` with `UserSettings` response struct (`#[derive(sqlx::FromRow, Serialize)]`) and `UpdateSettings` input struct (`#[derive(Deserialize)]` with `Option<String>` fields)
- [x] 2.2 Implement `get_settings` handler — `SELECT theme, locale FROM user_settings WHERE id = 1`
- [x] 2.3 Implement `update_settings` handler — validate theme/locale values against allowed lists, return 422 on invalid input, update via `COALESCE(?, column)`, return updated row
- [x] 2.4 Register routes in `src/api/mod.rs`: `.route("/settings", get(settings::get_settings).put(settings::update_settings))`

## 3. Backend Tests

- [x] 3.1 Test `GET /api/settings` returns defaults (`system`, `en`) on a fresh database
- [x] 3.2 Test `PUT /api/settings` with theme only — locale unchanged
- [x] 3.3 Test `PUT /api/settings` with locale only — theme unchanged
- [x] 3.4 Test `PUT /api/settings` with both fields
- [x] 3.5 Test `PUT /api/settings` with empty body — no changes, returns 200
- [x] 3.6 Test `PUT /api/settings` with invalid theme — returns 422
- [x] 3.7 Test `PUT /api/settings` with invalid locale — returns 422

## 4. Frontend — Settings API Client

- [x] 4.1 Add `getSettings` and `updateSettings` functions to the API client layer

## 5. Frontend — Store Integration

- [x] 5.1 Update `initTheme` to fetch settings from API first, fall back to `localStorage`
- [x] 5.2 Update `setThemePreference` to fire-and-forget `PUT /api/settings` alongside `localStorage` write
- [x] 5.3 Update `initLocale` to fetch settings from API first, fall back to `localStorage`
- [x] 5.4 Update `setLocale` to fire-and-forget `PUT /api/settings` alongside `localStorage` write
- [x] 5.5 Deduplicate the `GET /api/settings` call — fetch once in `+layout.svelte` `onMount` and seed both stores

## 6. Quality Gate

- [x] 6.1 Run `cd ui && npm run check` — no TypeScript errors
- [x] 6.2 Run `cargo fmt --check` — no formatting issues
- [x] 6.3 Run `cargo clippy -- -D warnings` — no warnings
- [x] 6.4 Run `cargo test` — all tests pass
