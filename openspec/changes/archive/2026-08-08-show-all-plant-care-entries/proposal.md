## Why

The plant detail page already fetches the complete care history, but hides entries after the first 20 behind an unnecessary client-side “Show more” action. Showing the complete fetched timeline immediately makes the history easier to review and removes UI state that provides no loading or performance benefit.

## What Changes

- Display every fetched care entry immediately on the plant detail page.
- Remove the 20-entry client-side display cap and “Show more” control.
- Keep the existing unpaginated per-plant care API, newest-first ordering, and care-event grouping behavior unchanged.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `ui-care-journal`: Change the plant detail timeline requirement from initially showing 20 entries to immediately showing all fetched entries.

## Impact

- Affects the plant detail Svelte page and its route-level UI tests.
- Updates the `ui-care-journal` behavioral specification.
- Removes now-unused localized “Show more events” strings.
- Does not change backend APIs, database queries, dependencies, or the global care journal pagination behavior.
