## Why

Care entries on a plant detail page can currently be created or deleted, but correcting an event type, note, date, or attachment requires deleting and recreating the record. Plant owners need a direct edit workflow while the global care journal remains a read-only overview.

## What Changes

- Add an API operation for updating an existing care event's editable fields while preserving its identity and creation timestamp.
- Keep watering-derived plant state and MQTT data correct when an edit changes whether an event is a watering or changes when watering occurred.
- Add an edit action to each individual care entry on the plant detail timeline, including entries revealed inside expanded watering groups.
- Reuse the care entry form in edit mode with existing values prefilled, including controls to retain, replace, or remove the existing photo.
- Refresh the plant and its care timeline after a successful edit and provide validation, failure feedback, cancellation, and offline-disabled behavior consistent with existing mutations.
- Keep `/care-journal` read-only: no edit controls, editing state, or mutation workflow will be added to the global care journal.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `data-care-events`: Define care-event update behavior, validation, ownership checks, response data, and watering-state side effects.
- `ui-care-journal`: Allow editing individual entries only in the plant detail timeline and add the typed frontend update operation while preserving the read-only global journal.
- `ui-plant-detail`: Integrate edit controls and edit-form lifecycle into the plant detail page, including grouped entries, refresh behavior, errors, and offline constraints.
- `ui-care-entry-form`: Extend the self-contained form with an edit mode that initializes existing values and manages existing, replacement, and removed photos.

## Impact

- Backend routing and handlers in `src/api/mod.rs` and `src/api/care_events.rs`, with care-event integration coverage in `tests/care_events.rs`.
- Frontend API types/functions in `ui/src/lib/api.ts` and tests.
- Plant detail timeline and route tests in `ui/src/routes/plants/[id]/+page.svelte` and `ui/src/tests/routes/plants/[id]/page.test.ts`.
- Reusable form behavior and tests in `ui/src/lib/components/CareEntryForm.svelte` and `CareEntryForm.test.ts`, plus translated edit actions and status/error labels.
- Existing photo upload/delete endpoints are reused; no database migration or new dependency is expected.
- The global care journal route and interaction model remain unchanged.
