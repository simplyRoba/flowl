## Context

The plant detail page requests the full per-plant care-event list from the existing unpaginated API, then slices that in-memory list to 20 entries for rendering. A local boolean reveals the remainder without any additional network request. Care-event day grouping and watering-event collapsing operate on the sliced list.

## Goals / Non-Goals

**Goals:**
- Pass the complete fetched care-event list into the existing grouping utility.
- Remove the client-side limit, reveal state, and “Show more” control.
- Verify more than 20 individual events render immediately.

**Non-Goals:**
- Changing the per-plant care API or its newest-first ordering.
- Changing global care-journal pagination.
- Changing watering-event grouping or other timeline presentation.

## Decisions

### Render the complete existing response

The timeline will derive its grouped items directly from the fetched `careEvents` array. This is simpler than automatically activating the existing reveal state and accurately reflects that all records are already in browser memory.

Alternative considered: paginate the per-plant endpoint. Rejected because the requested behavior is to show all entries directly, and pagination would add backend and frontend complexity while requiring multiple requests.

### Remove obsolete state and translations

The display limit constant, reveal boolean, derived “has more” flag, button markup, and localized button labels will be removed rather than retained unused. This keeps the page state aligned with observable behavior.

## Risks / Trade-offs

- **[Large histories produce more DOM nodes on initial render]** → The endpoint already transfers all records, and the application is intended for personal plant histories; retain the existing watering grouping to reduce repetitive visual rows.
- **[Grouped watering events mean the number of visible rows can be lower than the number of fetched records]** → Test with alternating event types so it specifically verifies removal of the 20-entry cap without changing grouping semantics.
