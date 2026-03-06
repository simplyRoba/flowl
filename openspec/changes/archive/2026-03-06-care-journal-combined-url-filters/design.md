## Context

The care journal page (`/care-journal`) currently supports filtering by a single event type at a time via chip buttons. The filter selection is held in component state (`activeFilter: string`) and is lost on page reload. The backend `GET /api/care` endpoint accepts a single `type` query param.

## Goals / Non-Goals

**Goals:**
- Allow users to select multiple event type filters simultaneously
- Persist filter selection in URL query parameters so it survives reload and is shareable

**Non-Goals:**
- Adding new filter dimensions (date range, plant name) — future work
- Server-side rendering / `+page.ts` load function — the page uses client-side fetching today and this change doesn't alter that pattern

## Decisions

### 1. Query parameter format: repeated params with `axum-extra`

**Decision**: Use standard repeated query parameters `?type=watered&type=fertilized`. Add `axum-extra` with the `query` feature, which provides `axum_extra::extract::Query` backed by `serde_qs` — this deserializes repeated keys into a `Vec<String>` natively.

**Why**: Repeated params is the standard HTTP convention (what HTML forms produce). `axum-extra` is maintained by the same team as axum and is the idiomatic way to handle this.

**Alternative considered**: Comma-separated `?type=watered,fertilized` with manual string splitting. Avoids a dependency but deviates from HTTP conventions.

### 2. Backend: change `event_type` to `Vec<String>`, build `IN` clause

**Decision**: Change `GlobalCareQuery` to use `axum_extra::extract::Query` and deserialize `type` into `Vec<String>`. Validate each type, then build a `WHERE ce.event_type IN (?, ?, ...)` clause dynamically.

**Why**: Clean struct-level representation, validation loops over the vec, and the `IN` clause construction is straightforward since the set of valid types is small and bounded (6 values max). An empty vec means no type filter (return all).

### 3. Frontend: `Set<string>` state synced with URL via `goto()`

**Decision**: Replace `activeFilter: string` with `activeTypes: Set<string>` derived from `$page.url.searchParams`. On chip toggle, update the URL using SvelteKit's `goto()` with `replaceState: true`. The `$page.url` reactivity triggers re-fetch automatically.

**Why**: SvelteKit's `$page.url` is reactive and the canonical source of truth. Using `goto(..., { replaceState: true })` avoids polluting browser history with every filter toggle. Reading from URL on mount also means the filter state is restored on reload for free.

**Flow**:
1. On mount / URL change: read `$page.url.searchParams.getAll('type')` → `Set<string>`
2. On chip click: toggle type in set → build new URL with repeated `?type=a&type=b` params → `goto(newUrl, { replaceState: true })`
3. The reactive derivation triggers `loadPage(true)` with the new filter set

### 4. "All" chip behavior

**Decision**: "All" acts as a toggle with context-dependent behavior:

- **No filters active** (showing everything) → click "All" → selects all 6 types explicitly, enabling "all but X" workflows by toggling individual types off
- **All 6 types selected** → click "All" → clears back to no filter (removes `type` param)
- **Some types selected** → click "All" → clears to no filter

Toggling off the last remaining type also returns to the unfiltered state (no `type` param).

**Why**: Lets users quickly reach an "everything except X" selection without clicking 5 chips individually.

### 5. API client signature change

**Decision**: Change `fetchAllCareEvents(limit?, before?, type?: string)` to `fetchAllCareEvents(limit?, before?, types?: string[])`. The function appends a `type` param per entry.

**Why**: Matches the repeated-param convention on the backend. An empty array or undefined omits the param entirely.

## Risks / Trade-offs

- **New dependency** → `axum-extra` adds a crate to the build. It's maintained alongside axum and already common in axum projects, so maintenance risk is low.
- **URL length** → Selecting all 6 types produces `?type=watered&type=fertilized&type=repotted&type=pruned&type=custom&type=ai-consultation` (~90 chars). Well within URL limits.
- **No `+page.ts` SSR** → Filters are applied client-side after mount. This matches the existing pattern and avoids introducing a load function.
