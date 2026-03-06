## Why

The care journal filter chips currently allow only a single event type selection, and the active filter is lost on page reload or when sharing the URL. Users who want to see e.g. "watered + fertilized" events together cannot do so, and bookmarking a filtered view is impossible.

## What Changes

- Allow selecting multiple event type filter chips simultaneously (toggle on/off); "All" deselects all specific types
- Persist active filters as URL query parameters (`?type=watered&type=fertilized`) so the view survives reload and can be shared/bookmarked
- Add `axum-extra` dependency for proper repeated query param support
- Update the `GET /api/care` endpoint to accept multiple `type` query parameters instead of a single value
- Update the frontend API client `fetchAllCareEvents` to pass an array of types

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ui/care-journal`: The "Filter by event type" requirement changes from single-select to multi-select with URL persistence
- `data/care-events-api`: The global care events endpoint changes to accept multiple `type` values

## Impact

- **Backend**: `src/api/care_events.rs` — `GlobalCareQuery` struct, `list_all_care_events` handler, and SQL query need to handle a `Vec<String>` of types
- **Frontend API client**: `ui/src/lib/api.ts` — `fetchAllCareEvents` signature changes from `type?: string` to `types?: string[]`
- **Frontend page**: `ui/src/routes/care-journal/+page.svelte` — filter state becomes a `Set<string>`, synced bidirectionally with `$page.url.searchParams`
- **Tests**: API and component tests need updating for multi-type filtering
