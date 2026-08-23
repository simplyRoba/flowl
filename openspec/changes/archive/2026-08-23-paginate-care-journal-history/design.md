## Context

See `proposal.md` for motivation and the delta specs for required behavior. The global journal currently requests `limit=10000`, the backend clamps that value to 100, and the route discards `has_more`. Grouping runs in the browser after retrieval, so one raw page can collapse into only a few rendered rows.

The API sorts by `occurred_at DESC, id DESC`, but its current cursor predicate uses only `id < before`. Care events may be backdated, so ID order cannot represent the timeline order. Valid stored timestamps can use UTC, explicit offsets, or legacy SQLite-compatible separators, so raw text comparison also cannot represent chronological order reliably. The UI is embedded in the Rust binary, allowing the API and journal behavior to be deployed atomically without compatibility negotiation between independently released frontend and backend versions.

## Goals / Non-Goals

**Goals:**

- Bound initial and continuation payloads while keeping every historical event reachable.
- Preserve the existing public `before=<event-id>` API shape while making it chronologically correct.
- Prevent watering compaction from either hiding continuation or automatically draining all pages.
- Keep loaded data visible and recoverable when a continuation request fails.
- Preserve exact grouping on plant detail pages and clearly identify incomplete groups on the paginated global journal.

**Non-Goals:**

- Server-side grouping or a mixed event/group API response.
- Virtualizing expanded group members.
- Paginating the per-plant care endpoint.
- Persisting journal pages, scroll position, or expanded groups across navigation or reload.
- Changing which event types qualify for watering grouping.

## Decisions

### D1: Fetch 500 raw events per global-journal page

The journal will use a shared `JOURNAL_PAGE_SIZE` of 500 for both initial and continuation requests. The API default remains 20 for callers that omit `limit`, while its maximum accepted page size increases from 100 to 500. It will continue fetching one extra row internally to calculate `has_more` without a separate count query.

Five hundred event records remain a bounded payload and normally amount to a modest JSON response because image bytes are not included, only URLs. It also avoids excessive round trips for a personal journal where watering rows may compact heavily.

Alternatives considered:

- **Load all events:** provides exact client-side groups immediately but has unbounded latency and memory growth.
- **Keep 100 events:** bounded but produces too many continuation steps and can render as almost no timeline after compaction.
- **Adapt page size to rendered groups:** can repeatedly fetch the complete history when one watering streak spans years, recreating load-all unpredictably.

### D2: Keep the numeric `before` cursor but resolve its full ordering position

For compatibility, clients will continue sending the ID of the last event in the loaded page as `before`. When present, the handler first resolves that event's `occurred_at` and then applies this keyset boundary to the paginated query:

```sql
COALESCE(julianday(ce.occurred_at), -1.0)
  < COALESCE(julianday(:cursor_occurred_at), -1.0)
OR (
  COALESCE(julianday(ce.occurred_at), -1.0)
    = COALESCE(julianday(:cursor_occurred_at), -1.0)
  AND ce.id < :cursor_id
)
```

The query uses `ORDER BY COALESCE(julianday(ce.occurred_at), -1.0) DESC, ce.id DESC`, which compares supported UTC, offset, and legacy SQLite timestamp representations by their actual instant while preserving their stored/API text. New explicit occurrence timestamps are validated before insertion. Existing or imported malformed timestamps receive a deterministic fallback value, sort after valid timestamps, and remain reachable by the ID tie-breaker. An unknown cursor ID is a validation error rather than silently returning an empty page. Filter predicates apply alongside the cursor boundary, while the cursor event itself is resolved independently because it identifies a timeline position.

An append-only expression index on `care_events(COALESCE(julianday(occurred_at), -1.0) DESC, id DESC)` supports the same ordering so later pages do not require sorting the full table. No schema, stored-value rewrite, or response-shape change is needed.

Alternatives considered:

- **Continue using `id < before`:** fails for backdated events and can skip or duplicate rows.
- **Expose timestamp and ID as separate query parameters:** correct but expands and complicates the public API.
- **Introduce an opaque encoded cursor:** a good future option, but unnecessary while an event ID can be resolved to the same composite position and existing clients can remain compatible.

### D3: Use one guarded page loader with separate initial and continuation state

The route will replace `loadAllEvents` with a page loader that tracks:

- the loaded raw events;
- `hasMore`;
- initial loading versus continuation loading;
- a continuation error;
- the active filter request generation.

A reset load clears events, cursor state, continuation errors, and expanded groups, then requests the newest page. A continuation load snapshots the current filters and uses the last loaded event ID as `before`. Successful continuation responses append in API order and defensively ignore duplicate IDs.

Only one request may mutate a generation at a time. Every filter change advances the generation; responses from an older generation are discarded. This prevents a slow response for old filters from replacing or appending to the current timeline. The existing initial-load offline/error behavior remains in place.

Alternatives considered:

- **AbortController as the only stale-response defense:** cancellation is useful but does not guarantee that an already completed response cannot race with state changes. A generation check is deterministic and small.
- **Store pagination globally:** unnecessary because journal state is route-local and is not required to survive navigation.

### D4: The continuation button is also the infinite-scroll sentinel

Whenever `hasMore` is true, the route renders a real, keyboard-accessible “Load older entries” button. It is always a manual fallback. The route determines overflow from the document scrolling element after rendering and whenever loaded content, group expansion, or viewport size changes.

- If `scrollHeight` does not exceed `clientHeight`, automatic observation is disabled. The visible button waits for explicit user input.
- If the document overflows, an `IntersectionObserver` watches the button and invokes the same guarded loader as it approaches the viewport bottom.
- After an automatic page, the observer is disarmed until the sentinel first leaves and then re-enters the viewport. If compaction leaves it continuously visible, another page requires the manual button instead of draining all history.
- If observation is unavailable, the button remains fully functional.
- During a request the button exposes a busy state and cannot launch a second request.

A built-in resize observer or window resize listener plus a post-render overflow check is sufficient; no dependency is added. Observer and listener cleanup occurs when the route is destroyed or reset.

Alternatives considered:

- **Observer without an overflow gate:** compacted content can leave the sentinel continuously visible and automatically fetch every page.
- **Button only:** reliable but needlessly removes convenient infinite scrolling from long timelines.
- **Hide the button when scrollable:** removes the accessibility and failure fallback if the observer does not fire.

### D5: Represent page-boundary watering streaks as partial groups

`groupCareEvents` will receive whether older history remains. A `WateringGroup` gains a stable key anchored to its newest event ID and a partial flag.

After scanning the loaded events, any still-active per-plant streak is unresolved when `has_more` is true because an older same-plant event may extend or break it. Those streaks are emitted as partial groups, including a currently one-member streak. Their summaries use the loaded count with a `+` marker and explicitly state that older entries may continue the streak.

When another page is appended, the utility recomputes all groups over the combined raw list. The newest event remains unchanged, so the stable key preserves expansion state even though the count and earliest loaded date may change. Once a same-plant breaker or the end of history is observed, the group becomes exact; a completed one-member streak returns to an individual row.

Plant detail pages pass `has_more = false` because their existing endpoint returns the complete fetched history, preserving their current exact grouping behavior.

Alternatives considered:

- **Show an exact count for loaded members:** misleading when a streak crosses the page boundary.
- **Group each page independently:** creates duplicate summaries for one logical streak.
- **Server-side grouping:** produces stable summaries but significantly complicates mixed-item ordering, cursors, and expansion; it is not justified for the expected low-thousands scale.

### D6: Continuation errors require explicit retry

An initial-page failure keeps the existing full-page error/offline treatment. A continuation failure leaves all loaded events and the cursor unchanged, displays a translated retry state on the continuation control, and suspends observer-triggered loading. The user can retry explicitly; a successful retry clears the error and re-enables automatic observation when overflow still exists. If the API reports `CARE_EVENT_NOT_FOUND` because the cursor event was deleted, the control instead offers a full journal refresh from the newest page because retrying the same cursor cannot succeed.

There is deliberately no automatic retry or backoff. Automatic retries while a sentinel remains near the viewport can form an uncontrolled loop, while a user-triggered retry provides clear load and failure semantics. The existing network status UI remains the offline fallback.

## Risks / Trade-offs

- **[Groups change as older history is loaded]** → Mark unresolved groups with `+`, explain that they continue into older entries, and preserve their identity and expansion state.
- **[Five hundred events may still be large when notes are unusually long]** → Keep the request bounded, preserve the server maximum, and load subsequent pages only on scroll or explicit action.
- **[Document overflow can change after rendering or resizing]** → Recalculate after page append, expansion changes, and viewport resize; retain the button regardless of observer state.
- **[Care events can change between page requests]** → Keyset pagination avoids offset drift, but it is not a database snapshot. Refreshing or changing filters restarts from the newest page.
- **[Cursor event is deleted between requests]** → Return a validation error and offer retry after a journal refresh rather than returning an ambiguous page.
- **[Malformed historical timestamps have no chronological instant]** → Sort them after valid timestamps with a deterministic fallback value and descending ID so they remain reachable without rewriting preserved import data.
- **[Additional index consumes storage]** → The expression index is small relative to event history and directly supports the dominant global-journal ordering.

## Migration Plan

1. Add the chronological care-event index in a new append-only migration.
2. Update and test the backend page limit and composite cursor predicate while preserving request and response field names.
3. Update grouping behavior, translations, and frontend pagination tests.
4. Replace the load-all route logic with the guarded hybrid loader and deploy it in the same embedded-UI binary as the backend changes.

No data rewrite or feature flag is required. Rolling back to an older binary remains database-compatible because the added index is harmless, although the older journal's existing 100-event ceiling would return until the fixed binary is restored.
