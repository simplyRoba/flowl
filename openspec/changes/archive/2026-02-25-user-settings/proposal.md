## Why

Theme and language preferences are currently stored only in the browser's `localStorage`. This means preferences are lost when switching devices or clearing browser data, and — critically — the backend has no knowledge of the user's language when calling the AI provider. To generate plant names and summaries in the user's preferred language, the server needs access to these preferences.

## What Changes

- Add a `user_settings` key-value table to persist user preferences server-side
- Add REST endpoints (`GET /api/settings`, `PUT /api/settings`) to read and write settings
- Persist the theme selection (`light`, `dark`, `system`) to the backend instead of only `localStorage`
- Persist the language/locale selection (`en`, `de`, `es`) to the backend instead of only `localStorage`
- Frontend settings page writes preference changes to the API and reads them on load

## Capabilities

### New Capabilities
- `core/user-settings`: Backend key-value settings store with REST API for persisting user preferences (theme, language)

### Modified Capabilities
- `ui/settings`: Settings page will read/write preferences via the backend API instead of relying solely on `localStorage`
- `ui/i18n`: Locale store will initialise from the backend settings API and persist changes to it

## Impact

- **New migration**: `user_settings` table (SQLite)
- **New module**: `src/api/settings.rs` — REST handlers for settings CRUD
- **Modified**: `src/api/mod.rs` — register new routes
- **Modified**: Frontend settings page — call API on preference change
- **Modified**: Frontend i18n store — initialise from and sync to API
- **Future enablement**: Once settings are persisted server-side, a follow-up change can inject the language into the AI identify prompt
