## Why

The global care journal currently requests all history, but the API caps the response at 100 events and the page ignores `has_more`, making older entries inaccessible. Removing the cap would make initial load time and memory usage grow without bound, while watering compaction can leave too little rendered content for scroll-only pagination to activate reliably.

## What Changes

- Replace the journal's single load-all request with bounded keyset pagination using pages of 500 raw care events.
- Make the API cursor follow normalized chronological `occurred_at` plus `id` ordering so backdated events and supported timestamp representations are neither skipped nor duplicated, reject malformed occurrence timestamps at creation, and keep malformed historical/imported rows reachable through deterministic fallback ordering.
- Provide hybrid continuation behavior: keep a manual “Load older entries” control whenever more history exists, and automatically activate it near the bottom only when the journal already overflows its scroll container.
- Append fetched pages and recompute client-side watering groups, while marking groups that may continue into unloaded history with an inexact count such as `500+`.
- Give watering groups stable identity while older pages are appended, and reset pagination cleanly when event-type filters change.
- Preserve bounded requests, loading guards, retryable errors, deleted-cursor refresh recovery, and a manual fallback rather than automatically draining all history when compacted content does not scroll.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-care-journal`: Replace load-all behavior with hybrid manual/infinite pagination and define partial, stable watering groups across loaded-page boundaries.
- `data-care-events`: Support the journal page size and make cursor pagination consistent with chronological ordering.

## Impact

- Frontend route and tests: `ui/src/routes/care-journal/+page.svelte`, `ui/src/tests/routes/care-journal/page.test.ts`.
- Grouping utility and tests: `ui/src/lib/careGrouping.ts`, `ui/src/lib/careGrouping.test.ts`.
- API client and translations under `ui/src/lib/api.ts` and `ui/src/lib/i18n/`.
- Backend pagination handler, an append-only ordering index migration, and integration tests: `src/api/care_events.rs`, `migrations/`, `tests/care_events.rs`.
- Existing `GET /api/care` consumers retain bounded pagination; no new runtime dependency is required.
