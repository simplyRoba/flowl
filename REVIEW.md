# Flowl Project Review

A Rust + SvelteKit plant care service with MQTT Home Assistant integration, AI features, and a polished PWA. Overall this is a well-built, well-organized project for its scope.

Importance: 1 = critical, 5 = nice-to-have
Difficulty: 1 = trivial, 5 = very hard

---

## Backend (Rust + Axum + SQLite)

### Strengths

- **Clean architecture** — Clear module separation: `api/`, `ai/`, `mqtt.rs`, `images.rs`, `config.rs`, `db.rs`, `state.rs`. Each API resource has its own file.
- **Clippy pedantic enabled** — `pedantic = { level = "deny" }` in `Cargo.toml` is excellent discipline.
- **Excellent test coverage** — ~3,900 lines of integration tests covering all API endpoints, plus thorough unit tests in `images.rs`, `mqtt.rs`, `config.rs`, `ai/openai.rs`, and `ai/prompts.rs`.
- **Security-conscious restore** — `validate_filename()` and `validate_dest_path()` with canonical path verification prevent zip-slip attacks. Size limits on imports/photos.
- **Image handling** — Magic-byte content type detection (not trusting client headers), EXIF orientation handling, multi-size thumbnail generation on `spawn_blocking`, orphan cleanup on startup.
- **MQTT design** — Reconnection with exponential backoff, state caching to only publish changes, discovery/removal cleanup, repair mechanism.
- **Graceful shutdown** — SIGTERM/SIGINT handling with task abortion and MQTT disconnect.
- **Release profile** — LTO, single codegen unit, strip — good for a small self-hosted binary.

### Issues

| # | Issue | Imp. | Diff. | Done | Details |
|---|-------|------|-------|------|---------|
| B1 | **All DB errors mapped to `BadRequest`** | 2 | 2 | ✅ | Every `sqlx` error becomes `ApiError::BadRequest(e.to_string())`. Internal DB failures (connection issues, constraint violations) should be `InternalError(500)`, not `400`. Raw sqlx error messages are leaked to clients. |
| B2 | **No request validation for `light_needs`** | 2 | 1 | ✅ | `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture` are validated against allowed values. `light_needs` is not — any string is accepted. |
| B3 | **MQTT publish functions silently swallow failures** | 3 | 3 | ✅ | All `publish_*` functions only `warn!()` on failure. If MQTT publishing is critical to HA integration, there's no retry or user feedback mechanism. The state checker only runs hourly after initial publish (`sleep(3600)`). |
| B4 | ~~**`compute_watering_status` has a dead branch**~~ | 3 | 1 | ❌ | ~~Not a bug. When `today == next_due`, the `>` check is false so execution correctly falls to `>=` which returns `"due"`.~~ |
| B5 | ~~**SQL strings built via `format!`**~~ | 3 | 3 | ❌ | ~~Not a real issue. `PLANT_SELECT` is a `const &str` and every call site only appends static string literals (`" WHERE p.id = ?"`, `" ORDER BY p.name"`). No user input enters the format string; all dynamic values use `?` bind parameters. Defining separate constants per variant would add indirection without any safety benefit.~~ |
| B6 | **Stats counts photos from plants only** | 4 | 1 | ✅ | `photo_count` query is `SELECT COUNT(*) FROM plants WHERE photo_path IS NOT NULL` — care event photos aren't counted. |
| B7 | **No WAL mode for SQLite** | 4 | 2 | ✅ | SQLite WAL mode isn't explicitly enabled. Under concurrent writes (API + background state checker), the default journal mode may cause `SQLITE_BUSY`. |
| B8 | **`Option<Option<T>>` complexity in UpdatePlant** | 4 | 2 | | While functionally correct for distinguishing "not sent" vs "sent as null", having 8 fields with `#[allow(clippy::option_option)]` is complex. A PATCH semantics crate or explicit "unset" sentinel could be cleaner. |
| B9 | **No rate limiting on AI endpoints** | 3 | 3 | ✅ | AI endpoints forward requests to OpenAI without any rate limiting or cost protection. A malicious or buggy client could burn through API credits. |
| B10 | **Duplicate `last_watered` subquery** | 4 | 2 | ✅ | The same `(SELECT MAX(occurred_at) FROM care_events WHERE plant_id = ... AND event_type = 'watered')` subquery is repeated in `PLANT_SELECT`, `republish_all`, `spawn_state_checker`, `export_data`, and `build_plant_context`. Should be a shared SQL fragment or view. |
| B11 | **Import clears photos before DB transaction** | 3 | 3 | ✅ | `import_data` runs Phase 2 (clear uploads + write new photos) before Phase 3 (replace DB in transaction). If the DB transaction fails and rolls back, the original photos are already deleted from disk. Could lose data on partial failure. |

---

## UI (SvelteKit 5 + TypeScript)

### Strengths

- **Modern stack** — Svelte 5 with runes (`$state`, `$derived`, `$effect`, `$props`), TypeScript throughout, Vite 7, vitest for testing.
- **Comprehensive i18n** — Three locales (en, de, es) with a clean dictionary-based approach. Locale persisted to backend settings.
- **Good component library** — Reusable components: `ModalDialog`, `PhotoLightbox`, `PlantForm`, `IconPicker`, `LocationChips`, `StatusBadge`, `WateringInterval`, `CareEntryForm`, `ChatDrawer`, `PageHeader`, `ToastHost`.
- **Store pattern** — Clean separation with Svelte stores for `plants`, `care`, `locations`, `theme`, `locale`, `ai`, `notifications`. Each handles its own API calls and error state.
- **Strong typing** — Full TypeScript interfaces matching the backend API types. No `any` types found. `ApiError` class with proper status codes.
- **UI testing** — Tests for components (`*.test.ts`), stores, routes, and utilities using `@testing-library/svelte`.
- **Responsive design** — Three breakpoints (mobile <=768, tablet, desktop >=1280) with appropriate layouts (bottom nav on mobile, sidebar on desktop).
- **Design system** — CSS custom properties for colors, typography, radii, motion. Light and dark theme support via `[data-theme="dark"]`.
- **PWA features** — Pull-to-refresh with nice physics (damping, reverse buffer), standalone detection.
- **Thumbnail system** — `thumbUrl`/`thumbSrcset` utilities with proper `sizes` attributes and fallback on error.
- **AI integration** — Streaming chat via SSE (async generator), plant identification with structured results, conversation summarization.

### Issues

| # | Issue | Imp. | Diff. | Done | Details |
|---|-------|------|-------|------|---------|
| U1 | **Backend error strings shown directly in UI** | 2 | 4 | ✅ | Already noted in `TODO.md`. Store error handlers do `e instanceof Error ? e.message : t.error.xxx` — the first branch shows raw backend English strings (e.g., "Plant name is required") regardless of active locale. Needs error codes from the API mapped to i18n keys. |
| U2 | **No loading states for many operations** | 3 | 2 | | Settings page, locations management, import/export — many operations don't show loading indicators. The user gets no feedback during network calls. |
| U3 | **Plant detail page re-fetches everything on every action** | 3 | 2 | | `refreshPlantDetails` fetches both plant and care events after any action (water, delete event, add event). This works but is wasteful — watering only changes plant state, not care event photos. |
| U4 | **No offline support** | 3 | 5 | | `TODO.md` lists this. As a self-hosted service often used on mobile, losing connectivity means a blank screen. |
| U5 | **Large component files** | 3 | 3 | | `PlantForm.svelte` is 1400+ lines, `plants/[id]/+page.svelte` is 967 lines. These could be broken into smaller sub-components for maintainability. |
| U6 | **CSS duplication across components** | 4 | 3 | | CSS is all in `<style>` blocks within components. Shared CSS files exist (`buttons.css`, `chips.css`, etc.) but many page-level styles are duplicated (`.error`, `.loading`, card patterns). |
| U7 | **Hardcoded color values in component styles** | 4 | 2 | ✅ | Most colors use CSS vars, but some components have hardcoded `rgba(0,0,0,...)` and `#fff` values, especially in card overlays and shadows. |
| U8 | **Stringly-typed status fields** | 4 | 2 | ✅ | `Plant.watering_status` is `string` but only has 3 values. Should be a union type `"ok" \| "due" \| "overdue"`. Same for `event_type`, `light_needs`, `difficulty`, etc. |
| U9 | **Notification auto-dismiss missing** | 4 | 2 | ✅ | `ToastHost` shows notifications but they stack without automatic dismissal (only `MAX_VISIBLE=3` limit). No timeout-based auto-dismiss. |
| U10 | **Accessibility gaps** | 3 | 3 | | Some good patterns (aria-labels on icon buttons, `aria-live` on pull indicator), but no comprehensive a11y testing. Interactive cards use `<a>` with nested `<button>` children which can cause nested interactive element issues. |

---

## Cross-cutting (Both / Project-level)

### Strengths

- **Spec-driven development** — OpenSpec workflow with archived changes documenting design decisions. Impressive traceability.
- **CI pipeline** — Separate lint + test jobs for Rust and UI. Cargo cache, Node cache. Format, clippy, svelte-check, ESLint all enforced.
- **Release pipeline** — Multi-arch (amd64/arm64) cross-compilation, Docker multi-platform build, GitHub Container Registry, semver tagging. Very polished.
- **Docker setup** — Non-root user (`1000:1000`), healthcheck, volume mount, slim base image. Best practices followed.
- **UI-in-binary embedding** — `rust-embed` bundles the SvelteKit build into the Rust binary via `build.rs`. Single-binary deployment with SPA fallback.
- **Documentation** — Good README with config table, HA automation example, compatible AI models list.
- **Conventional commits** — Enforced commit style visible in the git log.
- **Dependabot** — Configured for automated dependency updates.

### Issues

| # | Issue | Imp. | Diff. | Done | Details |
|---|-------|------|-------|------|---------|
| X1 | **No authentication** | 2 | 4 | ✅ | The service binds to `0.0.0.0` with no authentication. Anyone on the network can access/modify all data, trigger AI calls (spending API credits), export/import. Typical for HA add-ons behind a reverse proxy, but should at minimum be documented as a requirement, or optionally support basic auth. |
| X2 | **No automated database backup** | 3 | 3 | | No automated backup mechanism. The export feature exists but requires manual action. A corrupted DB means data loss. |
| X3 | ~~**`build.rs` runs UI build on every compile**~~ | 3 | 2 | ❌ | ~~Not an issue. `rerun-if-changed` hints already prevent re-runs when UI files haven't changed. `SKIP_UI_BUILD` exists for fully skipping.~~ |
| X4 | **No E2E tests** | 3 | 4 | | Unit and integration tests are solid, but no Playwright/Cypress tests verify the full stack. The `tests/ui.rs` file is just 29 lines testing static file serving. |
| X5 | **Import version check is too strict** | 4 | 1 | ❌ | `check_version` requires matching major.minor. Won't fix — strict check is safe. Export filename now includes the version so users know which image version to deploy for import. |
| X6 | **No structured logging output** | 4 | 2 | | Uses `tracing_subscriber::fmt()` which outputs human-readable logs. For a Docker service, JSON structured logging would be better for log aggregation. |
| X7 | **Health check doesn't verify DB** | 4 | 1 | ✅ | `/health` returns `{"status": "ok"}` unconditionally without checking if the DB pool is healthy. |
| X8 | **Test upload directories not cleaned up** | 5 | 1 | ✅ | `test_app_with_uploads` creates temp dirs with UUIDs under `std::env::temp_dir()` but there's no cleanup. They accumulate over time on dev machines. |
| X9 | **`unsafe` env var manipulation in config tests** | 4 | 2 | | `env::set_var`/`env::remove_var` are `unsafe` in Rust 2024 edition and require `unsafe` blocks. The tests work but the pattern is fragile with parallel test execution (mitigated by `ENV_LOCK` mutex). |

---

## Priority Summary

### Top priorities (high importance, reasonable difficulty)

1. ~~**B1 + U1: Error handling overhaul** (imp 2, diff 3) — Backend error codes done. Frontend error mapping pending.~~
2. ~~**B4: `compute_watering_status` logic bug** (imp 3, diff 1) — Not a bug; the code is correct.~~
3. ~~**B2: Validate `light_needs`** (imp 2, diff 1) — One-liner to add the same validation pattern used for all other enum fields.~~
4. ~~**X7: Health check DB verification** (imp 4, diff 1) — Add a simple `SELECT 1` to the health endpoint.~~
5. ~~**X1: Auth documentation or implementation** (imp 2, diff 1-4) — Security note added to README. No built-in auth; use a reverse proxy.~~

### Quick wins (low difficulty, meaningful impact)

- ~~B2: Validate `light_needs` (diff 1)~~
- ~~B4: Fix watering status logic (diff 1) — not a bug~~
- ~~B6: Count care event photos in stats (diff 1)~~
- ~~X7: DB health check (diff 1)~~
- ~~X5: Relax import version check (diff 1) — won't fix, version added to export filename~~
- ~~U8: Add union types for status fields (diff 2)~~
- ~~U9: Add toast auto-dismiss timeout (diff 2) — already implemented~~

### Longer-term improvements

- U4: Offline support / service worker (diff 5)
- ~~X1: Authentication system (diff 4) — documented, use reverse proxy~~
- X4: E2E test suite (diff 4)
- ~~U1: Full error code system (diff 4)~~
- ~~B9: AI rate limiting (diff 3)~~
