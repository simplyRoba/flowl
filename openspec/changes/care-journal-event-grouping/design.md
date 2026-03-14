## Context

The care journal has two views: a global page (`/care-journal`) with infinite scroll (20 events per page), and a per-plant timeline on the plant detail page (loads all events, shows first 20 with "Show more"). Both render every care event as an individual row. Frequent watering creates visual noise.

The global page currently uses cursor-based pagination (`before` param) with an IntersectionObserver sentinel. The plant detail page loads all events via `fetchCareEvents(plantId)`.

## Goals / Non-Goals

**Goals:**
- Collapse consecutive note-less, photo-less watering events per plant into expandable summaries
- Show count and date range in the summary (e.g. "Watered 5 times, Feb 1 - Mar 14")
- Allow expand/collapse to see individual grouped entries
- Share grouping logic between both views
- Keep the implementation client-side only (no API changes)

**Non-Goals:**
- Grouping non-watering event types
- Server-side grouping or new API endpoints
- Virtual scrolling or other performance optimizations
- Editing or deleting events from within a collapsed group

## Decisions

### D1: Client-side grouping over server-side

Client-side grouping with a pure utility function. This avoids API changes, complex pagination cursors for mixed item types, and keeps the transform testable and simple.

Trade-off: requires loading all events at once (see D2). For a personal plant care app the total event count is manageable (a few hundred to low thousands).

Alternative considered: server-side grouping with union response types. Rejected because pagination becomes complex (a group of 5 = 1 slot, cursor can't be a simple event ID), and the API contract becomes significantly harder to evolve.

### D2: Global page switches from infinite scroll to load-all

The global care journal drops the sentinel/IntersectionObserver pagination and loads all events in a single API call (using a high limit or removing the limit). This is required for accurate client-side grouping — with paginated loading, group counts change as pages load, which feels broken.

The existing `fetchAllCareEvents` API already supports a `limit` param. We'll pass a large limit (e.g. 10000) or add support for no-limit. The type filters still work as before.

Alternative considered: keep infinite scroll and group only within loaded pages. Rejected because groups would grow/change as you scroll, which is confusing.

### D3: Grouping algorithm — per-plant streak detection

The grouping utility scans the event list (sorted newest-first) and tracks per-plant watering streaks. A streak is consecutive watering events for the same plant where none have notes or photos. When a non-watering event, a watering-with-notes/photo, or a different event type for that plant appears, the streak for that plant breaks.

Since the global timeline interleaves events from multiple plants, streaks are tracked independently per plant. Events from other plants between two waterings of the same plant do NOT break that plant's streak.

A streak of 1 renders as a normal individual event. A streak of 2+ renders as a group summary.

The function signature: `groupCareEvents(events: CareEvent[]): Array<CareEvent | WateringGroup>` where `WateringGroup` contains `plantId`, `plantName`, `count`, `firstAt`, `lastAt`, and the original `events` array for expansion.

### D4: Skeleton loading while fetching all events

Since load-all may take a moment (especially with many events or slow connections), the global care journal shows skeleton shimmer lines during the fetch. The shimmer styles are currently scoped inside `IdentifyPanel.svelte`. We extract them into a shared `ui/src/lib/styles/skeletons.css` (following the pattern of `buttons.css`, `chips.css`, etc.) and import it in `+layout.svelte`. The IdentifyPanel then drops its local shimmer styles in favor of the shared ones.

### D5: Expand/collapse is local UI state

Each group summary has a chevron toggle. Clicking expands to show the individual watering entries inline. This is purely component state — no URL persistence, no store involvement. The expanded events show their individual dates but no delete buttons (you'd need to go to the plant detail or use an unexpanded view to delete).

## Risks / Trade-offs

- [Large event counts] Loading all events at once could be slow for power users with thousands of events. Mitigation: a personal plant app rarely exceeds a few hundred events; if needed, a limit cap (e.g. last 1000 events) can be added later without changing the grouping logic.
- [Grouping hides gaps] A summary "Watered 5 times, Jan 1 - Mar 14" hides that there may have been a 6-week gap. Mitigation: expand/collapse reveals individual entries with dates, making gaps discoverable.
- [Delete from group] Users can't delete individual events from within a collapsed group on the global page. Mitigation: expanding the group shows entries; deletion can be added as a follow-up if needed. The plant detail page still has delete buttons on individual entries.
