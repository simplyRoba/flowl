## Why

Users can add and delete locations but cannot rename them. The backend `PUT /api/locations/{id}` endpoint and the `updateLocation` store function already exist but no UI exposes them. Adding inline rename in settings closes this gap without new backend work.

## What Changes

- Add an inline-editable location name in the settings page so users can click a location name, edit it in-place, and save on blur/Enter
- Wire the existing `updateLocation` store function into the settings UI
- Show validation feedback for empty or duplicate names (backend returns 409)

## Capabilities

### New Capabilities

### Modified Capabilities

- `ui/settings`: Add inline rename to the Location Management requirement

## Impact

- `ui/src/routes/settings/+page.svelte` — add edit UI and import `updateLocation` from the locations store
- No backend changes required (`PUT /api/locations/{id}` already handles rename with conflict detection)
- No database migration needed
