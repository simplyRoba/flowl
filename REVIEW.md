# flowl — Codebase Review

**Date:** 2026-03-01
**Scope:** Full codebase — technical, UI, UX, testing, security, architecture

---

## Table of Contents

1. [Executive Summary](#executive-summary)
2. [What's Good](#whats-good)
3. [Issues — Fix Immediately](#issues--fix-immediately)
4. [Issues — Fix Soon](#issues--fix-soon)
5. [Issues — Should Be Addressed](#issues--should-be-addressed)
6. [Issues — Nice to Have / Someday](#issues--nice-to-have--someday)

---

## Executive Summary

flowl is a well-architected plant care tracker with a clean Rust+SvelteKit stack. The single-binary distribution model is elegant, the codebase is consistent, and the spec-driven development approach is thorough. However, the review uncovered one genuine SQL injection risk, several security gaps in the data restore path, missing error recovery in the frontend, and significant test coverage gaps in the most complex UI components.

---

## What's Good

### Architecture
- **Single-binary distribution** — `build.rs` compiles the SvelteKit SPA and embeds it via `rust-embed`. Elegant, zero-dependency deployment.
- **Clean separation** — Rust handles data/API/MQTT; Svelte handles UI. No leaky abstractions between layers.
- **SQLite with migrations** — Simple, embedded, auto-migrating. Perfect for a self-hosted app.
- **Computed `last_watered`** — Derived from `MAX(care_events.occurred_at)` via subquery rather than a denormalized column. Correct by construction.
- **`Option<Option<T>>` for nullable-vs-absent** — The `UpdatePlant` pattern properly distinguishes "clear this field" from "don't change this field". Sophisticated and correct.
- **AI provider trait** — `AiProvider` is a clean async trait, making the OpenAI client swappable for Ollama or any other provider.

### Frontend
- **Svelte 5 runes throughout** — Consistent use of `$state()`, `$derived`, `$effect()`, `$props()`. No legacy API mixing.
- **Native `<dialog>` elements** — Modals and lightboxes use the platform correctly, getting focus trapping and Escape handling for free.
- **Theme flash prevention** — Inline script in `app.html` applies `data-theme` before first paint. Well-implemented.
- **Custom CSS design tokens** — A cohesive, warm color palette with consistent token usage. `color-mix()` for tints is modern and clean.
- **Dual-mode ChatDrawer** — Desktop drawer + mobile bottom sheet, sharing a single `{#snippet}` to avoid DOM duplication.
- **Optimistic UI** — Dashboard water button shows immediate feedback via `wateringIds` set.
- **Touch gesture support** — PhotoLightbox has pinch-to-zoom with proper pan clamping. ChatDrawer has drag-to-dismiss.

### Backend
- **Parameterized SQL queries** — All queries (except one critical gap noted below) use sqlx bind parameters. No string concatenation for user input.
- **Graceful shutdown** — Signal handling, MQTT disconnect, task abort on shutdown.
- **Image cleanup** — Orphaned file cleanup on startup prevents disk bloat.
- **MQTT Home Assistant integration** — Auto-discovery config publishing, state checker for watering transitions.
- **`ApiError` enum** — Consistent JSON error responses with proper HTTP status codes.

### Quality
- **Spec-driven development (OpenSpec)** — 37 archived changes, all with specs using RFC 2119 + Given/When/Then scenarios.
- **CI pipeline** — Lint (clippy pedantic + svelte-check), test (cargo + vitest), release automation (release-please + multi-arch Docker).
- **Co-located tests** — Test files live next to source files. Easy to find, easy to maintain.
- **Dependabot** — Daily dependency update checks for Cargo, npm, and GitHub Actions.
- **Release optimization** — LTO, single codegen unit, stripped binaries.

### UX
- **Responsive design** — Three breakpoints (mobile/desktop/wide) with appropriate layout shifts.
- **PWA manifest** — Installable as a home screen app with proper icons.
- **Safe area support** — `env(safe-area-inset-bottom)` for iPhone notch/home indicator.
- **i18n** — Three languages (en/de/es), locale persisted to both localStorage and server.
- **Dark mode** — Full light/dark/system theme support with correct meta `theme-color` updates.

---

## Issues — Fix Immediately

These are bugs, security issues, or data-corruption risks that should be addressed before the next release.

### ~~1. SQL Injection in `list_all_care_events`~~ DONE

**File:** `src/api/care_events.rs:245`

`event_type` is string-interpolated into a SQL query:
```rust
conditions.push(format!("ce.event_type = '{event_type}'"));
```
While `event_type` is validated against `VALID_EVENT_TYPES` a few lines above, the validation and interpolation are separated — a refactor could easily break the guard. Use a bound parameter instead. The `before` and `LIMIT` interpolations (type-safe `i64`) should also be converted for consistency.

### ~~2. `handleDeleteConfirm` and `handleWater` freeze on error~~ DONE

**File:** `ui/src/routes/plants/[id]/+page.svelte:60-69, 83-89`

Both functions set a loading flag (`deleting = true`, `watering = true`) but have no `try/finally` block. If the API call throws, the flag stays `true` forever, permanently disabling the button. The user must reload the page.

**Fix:** Wrap in `try/finally` to always reset the flag.

### ~~3. Non-atomic data restore — DB commits before disk ops~~ DONE

**File:** `src/api/restore.rs:227-295`

The import flow commits the database transaction, then deletes old photos, then writes new photos. If the process crashes between these steps, the database references files that no longer exist. This is a data corruption risk.

**Fix:** Write new photos to a staging directory first, commit the DB transaction, then atomically swap directories.

### ~~4. No field validation on imported data~~ DONE

**File:** `src/api/restore.rs:182-225`

Imported plants and care events bypass all API-level validation. A crafted backup can insert empty plant names, invalid `event_type` values, or malformed dates. These break downstream assumptions (MQTT checker, watering status computation, frontend rendering).

**Fix:** Apply the same validation rules used by the normal create endpoints.

### ~~5. `PlantForm` `$effect` resets user edits on parent re-render~~ DONE

**File:** `ui/src/lib/components/PlantForm.svelte:80-95`

The `$effect` that initializes form fields from the `initial` prop re-runs whenever the prop reference changes. If the edit page's parent re-renders with a new object reference (same values), all user edits in progress are silently discarded.

**Fix:** Use `$effect` with a guard that only initializes once, or move initialization to `onMount`.

---

## Issues — Fix Soon

These are real problems that affect reliability, maintainability, or user experience but are not immediately dangerous.

### ~~6. Error detection by string comparison in ChatDrawer~~ DONE

**File:** `ui/src/lib/components/ChatDrawer.svelte:427`

```svelte
class:note-error={noteSavedMessage === $translations.chat.noteSaveFailed}
```

Whether a message is an error is determined by comparing its string value to a translation key. If the translation changes, error styling silently breaks. Use a separate `noteError: boolean` state variable.

### ~~7. `ApiError` class not exported from `api.ts`~~ SKIPPED

**File:** `ui/src/lib/api.ts:113`

Callers cannot do `e instanceof ApiError` for status-code-specific error handling. All error discrimination must go through string parsing. Export the class.

**Decision:** Skipped — no caller currently needs status-code-specific handling. Every catch block treats all errors the same (show message to user), and the backend's error messages are descriptive enough. The only HTTP client beyond AI integration is the frontend-to-backend `api.ts`, and for a self-hosted single-user app the realistic error scenarios don't warrant branching by status code. Can revisit if a use case arises.

### ~~8. Three copy-paste file upload handlers in `api.ts`~~ DONE

**File:** `ui/src/lib/api.ts:215-226, 246-261, 328-342`

`uploadPlantPhoto`, `uploadCareEventPhoto`, and `importData` duplicate the same fetch + error-handling logic. Extract a `requestFormData<T>()` helper.

### ~~9. No loading state on dashboard — empty-state flash~~ DONE

**File:** `ui/src/routes/+page.svelte:68-70`

`loadPlants()` is called in `onMount`. The template renders the empty state ("No plants yet") immediately before data arrives. Users see a flash of the empty state every time they open the app.

**Fix:** Add a `loading` flag and show a skeleton or spinner until the first fetch completes.

### 10. `handleWater` error is invisible on dashboard

**File:** `ui/src/routes/+page.svelte:49-53`

The dashboard attention section has a water button. If `waterPlant` fails, the error is written to `$plantsError`, but that error is only displayed far down in the main grid section. The user sees no feedback at their current scroll position.

**Fix:** Show an inline error near the attention card, or use a toast notification.

### ~~11. Button nested inside `<a>` on dashboard~~ DONE

**File:** `ui/src/routes/+page.svelte:105-114`

A `<button>` inside an `<a>` is invalid HTML. Screen readers handle this inconsistently. Restructure the attention card layout to avoid nesting interactive elements.

### ~~12. `div role="link"` in care journal~~ DONE

**File:** `ui/src/routes/care-journal/+page.svelte:179`

A `<div role="link">` does not get Space-bar activation, middle-click, or "open in new tab" context menu. Use a real `<a>` element.

### 13. `currentPlant` blanked on transient error

**File:** `ui/src/lib/stores/plants.ts:27-29`

If `loadPlant` fails (e.g., transient network error), `currentPlant` is set to `null`, blanking the entire detail view. The user was just looking at valid data.

**Fix:** Keep stale data on error and show an error overlay instead.

### ~~14. `watering_interval_days` not validated~~ DONE

**File:** `src/api/plants.rs:270`

A client can submit 0 or negative values. Zero makes every plant permanently "due". Negative values produce nonsensical next-due dates.

**Fix:** Add a minimum (1) and maximum (365 or similar) bound.

### ~~15. No timeout on AI HTTP requests~~ DONE

**File:** `src/ai/openai.rs` (client creation)

`reqwest::Client::new()` has no timeout. A slow AI provider holds the Tokio task indefinitely. For SSE streaming, this keeps the connection to the client open forever.

**Fix:** Set `timeout` and `connect_timeout` on the `ClientBuilder`.

### ~~16. SSE buffer grows unboundedly~~ DONE

**File:** `src/ai/openai.rs:272-293`

The SSE line accumulator `buf` has no size cap. A misbehaving upstream can cause unbounded memory growth.

**Fix:** Add a `MAX_LINE_LENGTH` check and error if exceeded.

### ~~17. Path traversal check in restore is weak~~ DONE

**File:** `src/api/restore.rs:88-95`

The blocklist approach (`contains ".."`, `starts_with "/"`) misses edge cases. Canonicalize the resolved path and assert it starts with the upload directory.

### ~~18. No per-file size limit during restore (zip bomb potential)~~ DONE

**File:** `src/api/restore.rs:131-148`

ZIP file contents are read into `Vec<u8>` without per-file size validation. A zip bomb can extract to far more than the 100 MB body limit.

**Fix:** Apply `MAX_FILE_SIZE` from `images.rs` to each extracted file.

### ~~19. Photo file orphaned if DB update fails~~ DONE

**File:** `src/api/care_events.rs:304-325`, `src/api/photos.rs:52-61`

During photo upload/replace: the new file is saved, the old file is deleted, then the DB is updated. If the DB update fails, the new file is orphaned and the old file is gone. The DB still references the old (now-deleted) filename.

**Fix:** Update the DB first (in a transaction), then do file operations. Or reverse the order: save new file, update DB, then delete old file.

### 20. `fetchAiStatus` called redundantly

**Files:** `plants/[id]/+page.svelte`, `PlantForm.svelte`, `ChatDrawer.svelte`

Three independent `fetchAiStatus()` calls with no shared cache. Creates redundant network requests.

**Fix:** Create an `aiStatus` store similar to `plants`/`locations`.

---

## Issues — Should Be Addressed

These are code quality, maintainability, and UX improvements that make the codebase healthier long-term.

### 21. ChatDrawer is 1021 lines with 6 responsibilities

**File:** `ui/src/lib/components/ChatDrawer.svelte`

Handles: mobile/desktop rendering, body scroll lock, drag-to-dismiss gestures, SSE streaming chat state, photo attachment + base64 encoding, and AI summary save flow. Each could be a separate component or composable.

### 22. Seven parallel label-mapping functions on plant detail

**File:** `ui/src/routes/plants/[id]/+page.svelte:71-171`

`lightLabel`, `difficultyLabel`, `petSafetyLabel`, etc. are all structurally identical `if/else if` chains. Replace with lookup objects (`Record<string, string>`).

### 23. Duplicated utility functions across files

| Function | Duplicated In |
|---|---|
| `eventTypeLabel` | `plants/[id]/+page.svelte`, `care-journal/+page.svelte` |
| `parseEventDate` | `plants/[id]/+page.svelte`, `care-journal/+page.svelte` |
| `formatDate/formatShortDate/formatTime` | Scattered across 3 files |

Extract into a shared `$lib/utils/date.ts` and `$lib/utils/care.ts`.

### 24. Photo rendering pattern duplicated

**File:** `ui/src/routes/+page.svelte:89-97, 149-156`

The "photo or emoji fallback" pattern is duplicated between attention cards and plant grid cards. Extract a `PlantPhoto` component.

### 25. SSE JSON parsing has no error boundary

**File:** `ui/src/lib/api.ts:407`

```ts
const data = JSON.parse(line.slice(6));
```

Malformed SSE lines cause `SyntaxError` that propagates to the caller. Catch parse errors per-line to be resilient against upstream glitches.

### 26. `resp.body!` non-null assertion in chatPlant

**File:** `ui/src/lib/api.ts:395`

`Response.body` can be `null`. Add a guard: `if (!resp.body) throw new Error(...)`.

### 27. PlantForm photo drop does not validate file type

**File:** `ui/src/lib/components/PlantForm.svelte:121-129`

Unlike ChatDrawer which checks `VALID_IMAGE_TYPES`, PlantForm accepts any dropped file. A user can drop a PDF and it will be staged as a photo.

### 28. `willFillChips` strips i18n strings with hardcoded substrings

**File:** `ui/src/lib/components/PlantForm.svelte:229`

```ts
label: t.form.speciesLabel.replace(' (optional)', '').replace(' (opcional)', '')
```

Breaks for German and future languages. Add a separate short-label translation key.

### 29. `cancelled` flag is shared module state in settings

**File:** `ui/src/routes/settings/+page.svelte:76-96`

A single `cancelled` boolean tracks edit cancellation. If two edits overlap (quick tabbing), the flag misattributes which edit was cancelled.

### 30. Settings reads error from store side-channel

**File:** `ui/src/routes/settings/+page.svelte:89-96`

`commitEdit` reads `locationsError` via `get()` after the store sets it on failure. The store function should return or throw the error directly.

### 31. Content-type trusted from header, no magic-byte check

**File:** `src/images.rs:50-55`

A client can upload any binary with `Content-Type: image/jpeg` and it gets stored/served. While XSS risk is limited (served with image MIME), it's unexpectedly permissive.

### 32. Access log only at debug level

**File:** `src/server.rs:37`

At the default `info` log level, no access log is emitted. Access logs should be at `info` level for production monitoring.

### 33. Silent fallback on invalid config env vars

**File:** `src/config.rs:36-41`

`FLOWL_PORT=abc` silently falls back to `4100`. Add a warning log for parse failures.

### 34. MQTT reconnect has no exponential backoff

**File:** `src/mqtt.rs:128-132`

Fixed 5-second reconnect interval. Against an unreachable broker, this hammers it indefinitely.

### 35. Per-plant care events endpoint has no pagination

**File:** `src/api/care_events.rs:120-133`

A plant with thousands of care events returns all in one response. The global endpoint has pagination; the per-plant one does not.

### 36. `previousValues` typed as `Record<string, unknown>`

**File:** `ui/src/lib/components/PlantForm.svelte:316-332`

All `as` casts are unsafe. Define a typed snapshot interface.

### 37. No `prefers-reduced-motion` support

**File:** `ui/src/routes/+layout.svelte` (global styles)

Animations (slide-in, hover lifts, typing indicator) are not conditionally disabled for users who prefer reduced motion. Add a `@media (prefers-reduced-motion: reduce)` query.

### 38. Dead code: `pick<T>` helper and `extraInput` bindings

- `ui/src/routes/+page.svelte:17-19` — `pick<T>` defined but never called
- `ui/src/lib/components/PlantForm.svelte:70-71` — `extraInput1`/`extraInput2` bound but never accessed

### 39. `:global(.btn-ai)` leaks from scoped component

**File:** `ui/src/routes/plants/[id]/+page.svelte:442-449`

`.btn-ai` styles defined with `:global()` inside a scoped component's `<style>` block, leaking globally. Move to a shared stylesheet.

### 40. Settings delete without confirmation for empty locations

**File:** `ui/src/routes/settings/+page.svelte:183-190`

Locations with no plants are deleted immediately — no confirmation. A misclick loses the location with no undo. Should be consistent: either always confirm or provide undo.

### 41. `onMount` fetches not cancelled on settings page destroy

**File:** `ui/src/routes/settings/+page.svelte:51-65`

Four independent fetch calls fire on mount. If the user navigates away quickly, all promises resolve and set state on a destroyed component. Add an AbortController.

### 42. IIFE in settings template

**File:** `ui/src/routes/settings/+page.svelte:379`

```svelte
{(() => { try { return new URL(aiStatus.base_url).hostname; } catch { return aiStatus.base_url; } })()}
```

Use a `$derived` or helper function instead.

---

## Issues — Nice to Have / Someday

These are improvements that would polish the product but are not urgent.

### 43. No ESLint or Prettier for frontend

The UI has `svelte-check` for types but no linting or formatting enforcement. Inconsistent formatting is already visible (e.g., `onsave` at column 0 in plant detail). Consider adding `eslint-plugin-svelte` and `prettier-plugin-svelte`.

### 44. All three i18n dictionaries are bundled

All translation files are imported statically. For 3 languages this is fine, but as languages are added, consider lazy-loading non-default locales.

### 45. English-centric pluralization

The `plural()` helper only supports `one/other` forms. Languages like Polish, Arabic, and Russian need additional plural categories. Not urgent for en/de/es but limits future language support.

### 46. No skeleton/shimmer loading states

Loading states are plain text ("Loading...") with no skeleton UI. Skeleton screens reduce perceived load time and prevent layout shifts.

### 47. Care journal: no "no results for this filter" message

When a filter returns zero results, the empty state says "No care events" with no mention of the active filter. Users might think their data is missing.

### 48. `IntersectionObserver` dependency trick

**File:** `ui/src/routes/care-journal/+page.svelte:119`

```ts
void events.length; // force dependency
```

The `void` expression forces Svelte to track `events.length` as a dependency. While functional, it's a code smell. Restructure to make the dependency explicit.

### 49. `chatPlant` focus uses magic `setTimeout(300)`

**File:** `ui/src/lib/components/ChatDrawer.svelte:322-325`

Focus after opening uses a 300ms delay to outlast the CSS animation. Use `animationend` event or `tick()` instead.

### 50. No toast/snackbar notification system

Error and success feedback is handled ad-hoc per page (inline text, error stores, or nothing). A global toast system would provide consistent feedback.

### 51. `abortController` as reactive `$state`

**File:** `ui/src/lib/components/ChatDrawer.svelte:22`

An `AbortController` doesn't need reactive tracking. Use a plain `let`.

### 52. Naming inconsistency: delete vs remove

Store functions mix `deletePhoto` and `removeCareEvent`. Pick one convention.

### 53. `care.test.ts` uses wrong `event_type` value

**File:** `ui/src/lib/stores/care.test.ts`

Mock data uses `event_type: 'watering'` but the backend uses `'watered'`. Tests pass because the store doesn't validate event types, but the mock data is unrealistic.

### 54. No tests for most complex components

| Component | Lines | Tests |
|---|---|---|
| `PlantForm.svelte` | ~1641 | None |
| `CareEntryForm.svelte` | 307 | None |
| `ChatDrawer.svelte` | 1021 | None (only indirect via page test) |
| `PhotoLightbox.svelte` | 253 | None |
| `IconPicker.svelte` | 54 | None |
| `PageHeader.svelte` | 133 | None |

### 55. No tests for care journal, plant new, plant edit pages

Three route-level pages have zero test coverage.

### 56. Backend: MQTT publish logic is completely untested

The MQTT state checker and publish functions (`publish_discovery`, `publish_state`, `publish_attributes`) are never exercised in tests. All MQTT tests only check the `/api/mqtt/status` endpoint with a disabled/disconnected client.

### 57. `update_plant` test uses `sleep(1s)`

**File:** `tests/plants.rs:134`

Real-time dependency makes this test slow and fragile. Use a deterministic approach (e.g., set timestamps explicitly).

### 58. Plant detail page is 744 lines

While functional, the plant detail page handles the hero section, watering info grid, care info grid, notes, care timeline, inline care form, photo lightbox, chat drawer, and two modal dialogs. Consider extracting sections into sub-components.

### 59. No rate limiting on AI endpoints

**File:** `src/server.rs`

AI endpoints make outbound paid API calls. No rate limiting exists to prevent abuse on the local network.

### 60. No authentication

All routes are unauthenticated. By design for a local-network app, but `/api/data/import` is destructive (wipes the database) and accessible to any process on the network. Consider optional auth or at least a confirmation token for destructive operations.

---

## Summary Statistics

| Category | Count |
|---|---|
| Fix Immediately | 5 |
| Fix Soon | 15 |
| Should Be Addressed | 22 |
| Nice to Have / Someday | 18 |
| **Total issues** | **60** |

| Severity | Examples |
|---|---|
| Security | SQL injection, path traversal, no import validation, no auth |
| Data integrity | Non-atomic restore, photo orphaning, interval_days=0 |
| UX | Empty-state flash, frozen buttons, invisible errors, no loading skeletons |
| Accessibility | Button in `<a>`, `div role="link"`, no reduced-motion, missing aria-labels |
| Maintainability | 1000+ line components, duplicated utilities, no frontend linting |
| Testing | Most complex components untested, MQTT untested, wrong mock data |
