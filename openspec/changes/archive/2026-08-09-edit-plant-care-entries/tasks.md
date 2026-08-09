## 1. Change Setup and Backend Tests

- [x] 1.1 Create a short-lived feature branch from `main` for `edit-plant-care-entries` without merging or pushing to `main`.
- [x] 1.2 Add care-event integration tests for successful full updates, cleared notes, preserved identity/photo/creation data, invalid input, missing plant, and event ownership mismatch.
- [x] 1.3 Add backend tests for watering-derived behavior when an update adds, removes, or moves a watered event, plus the non-watering case.

## 2. Backend Care Event Update

- [x] 2.1 Add the typed care-event update request and `PUT /api/plants/{id}/care/{event_id}` route/handler with boundary validation and standard response formatting.
- [x] 2.2 Preserve immutable event fields and existing photo data, enforce plant ownership, and return user-safe 404/422 errors without modifying invalid records.
- [x] 2.3 Republish derived watering MQTT state when either the old or new event type is `watered`, using existing graceful MQTT failure handling.

## 3. Frontend API and Form Tests

- [x] 3.1 Add frontend API client tests for the typed `updateCareEvent` PUT request and response.
- [x] 3.2 Add `CareEntryForm` tests for create-mode regression and edit initialization of type, notes, occurrence time, and existing photo.
- [x] 3.3 Add form tests for successful field-only updates, photo retain/remove/replace sequencing, cancellation, validation, API/photo failures, retained input, and offline-disabled save.

## 4. Frontend API and Reusable Form

- [x] 4.1 Add the update payload type and `updateCareEvent` function to `ui/src/lib/api.ts`.
- [x] 4.2 Extend `CareEntryForm.svelte` with explicit edit mode while preserving its existing self-contained create mode and responsive toolbar behavior.
- [x] 4.3 Implement edit-mode datetime initialization/validation and photo intent handling that reuses the existing upload/delete APIs and retains form state on failure.
- [x] 4.4 Add translated edit, save, validation, accessibility, and error labels to all supported locale files.

## 5. Plant Detail Integration and Tests

- [x] 5.1 Add plant-detail route tests for opening, cancelling, and successfully saving an edit, including refresh of both plant data and care events.
- [x] 5.2 Add route tests for edit failure feedback, offline controls, connectivity loss during editing, mutually exclusive add/edit forms, and individual edits inside expanded watering groups.
- [x] 5.3 Add or update a global care-journal regression test confirming that no edit affordance or update workflow is exposed there.
- [x] 5.4 Add compact edit controls and edit-form lifecycle state only to `ui/src/routes/plants/[id]/+page.svelte`, including standalone and expanded grouped events.
- [x] 5.5 Reload plant and timeline data only after the complete event/photo edit workflow succeeds so grouping and watering-derived fields remain server-authoritative.

## 6. Documentation and Verification

- [x] 6.1 Revisit `README.md` and update only if plant-detail care-entry editing is important end-user documentation; do not add implementation details.
- [x] 6.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, and `npm run check --prefix ui`.
- [x] 6.3 Run `cargo test` (including the UI test bridge) and resolve all failures before requesting human review or archiving.
