## Context

See `proposal.md` for motivation. Care-event writes currently comprise create/delete plus separate photo upload/delete endpoints. Plant detail owns its fetched timeline while `CareEntryForm` encapsulates create fields and submission. The global journal uses the same event response type but is intentionally read-only. Watering status is derived from care history and mirrored to MQTT, so changing an existing event can affect more than its displayed row.

## Goals / Non-Goals

**Goals:**

- Add an explicit, ownership-scoped update path without changing the database schema.
- Reuse one form for create and edit while keeping field and photo state encapsulated.
- Keep derived watering data correct after updates.
- Confine edit controls and state to the plant detail route.

**Non-Goals:**

- Editing from the global care journal.
- Batch editing, revision history, optimistic concurrency, or undo.
- Changing event identity, plant ownership, or creation timestamp.
- Adding dependencies or changing photo storage.

## Decisions

### Use a full PUT for editable event data

Add `PUT /api/plants/{id}/care/{event_id}` with a typed request containing `event_type`, nullable `notes`, and `occurred_at`. Requiring the complete editable shape avoids ambiguous absent-versus-null patch semantics and matches a form that always holds all editable values. The handler validates event type and datetime at the boundary, verifies the plant and event ownership, updates one row, and returns the standard event response.

A partial `PATCH` was considered, but it would require extra nullable/absent modeling without benefit for this UI. Delete-and-recreate was rejected because it changes identity, can lose the attachment, and introduces avoidable failure windows.

### Keep photos on the existing endpoints

The update request does not include photo data. Edit mode tracks photo intent as retain, remove, or replace. After the event PUT succeeds, it uses the existing photo delete/upload endpoint only when necessary; uploading a replacement relies on existing server behavior that removes the old file.

A combined multipart update was rejected because it duplicates established endpoints and complicates the JSON API. This creates a possible partial success: event fields may save while a subsequent photo mutation fails. The form reports the failure, retains the selected replacement where possible, and the page refreshes only after the complete edit workflow succeeds; retry repeats an idempotent field PUT before retrying the photo step. No automatic background retry is added.

### Recompute watering publication from old and new event state

Before updating, the handler reads the owned event's current type. After a successful update it republishes watering state whenever either the old or new type is `watered`. This covers removing a watering, adding a watering, and moving a watering in time. Updates between two non-watering types skip MQTT publication.

Database success remains authoritative if MQTT publication fails, matching existing graceful MQTT behavior; failure is logged rather than rolling back a valid care-history update.

### Reuse CareEntryForm with explicit edit input

`CareEntryForm` receives an optional existing `CareEvent`. Its absence selects create mode; its presence initializes edit mode. The component owns type, notes, occurrence time, photo intent, validation, and submit state in both modes. The route owns only whether add mode is open and which event is selected for editing, ensuring only one care form is visible.

A second edit-specific component was considered, but would duplicate type chips, date/photo controls, validation, translations, and responsive layout.

### Render edit affordances only in plant-detail event rows

The plant detail page adds edit controls to standalone events and individual events exposed by expanded watering groups. Group summaries remain aggregate controls and do not edit a synthetic group. The global journal component and route receive no mutation state or controls; they may naturally show updated server data on a later fetch without changing their interaction model.

### Refresh both plant and timeline after success

After the complete edit workflow succeeds, the route reloads the plant and care events instead of applying an optimistic local mutation. This keeps grouping and watering-derived fields consistent with the server. Edit/save controls follow existing online state and are disabled offline; there is no offline mutation queue or retry/backoff.

## Risks / Trade-offs

- **[Event fields save but photo mutation fails]** → Show a toast, retain form input/staged replacement, and allow retry; use existing replace semantics to avoid duplicate photos.
- **[An edit changes grouping while a group is expanded]** → Rebuild groups from the refreshed event list and allow transient expansion state to reset.
- **[MQTT publish fails after database update]** → Keep the database update, log through the existing MQTT degradation path, and allow later state publication to reconcile consumers.
- **[Concurrent edits overwrite one another]** → Accept last-write-wins for this local/self-hosted workflow; revision tokens are out of scope.
- **[More actions crowd narrow timeline rows]** → Use the existing compact icon-button pattern and verify responsive behavior in route tests.

## Migration Plan

1. Deploy the backend update endpoint and frontend together; no schema or data migration is required.
2. Existing clients remain compatible because all current endpoints and response fields are unchanged.
3. Roll back by reverting the UI and endpoint; care events edited while deployed remain valid under the prior schema.
