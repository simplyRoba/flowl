## Context

The settings page (`ui/src/routes/settings/+page.svelte`) displays locations in a list with a delete button per row. The backend already exposes `PUT /api/locations/{id}` for renaming (with 409 conflict detection for duplicates), and the store already exports `updateLocation`. The only missing piece is UI.

## Goals / Non-Goals

**Goals:**
- Let users rename a location by clicking its name in the settings list
- Provide immediate inline editing (no modal/dialog)
- Handle validation errors (empty name, duplicate name) inline

**Non-Goals:**
- No new backend endpoints or database changes
- No bulk rename or drag-to-reorder
- No rename UI outside the settings page (e.g., LocationChips component stays as-is)

## Decisions

**Edit button triggers inline edit mode**
Add a pencil icon button next to the existing trash button. Clicking it replaces the static name text with a focused `<input>` and a confirm button (check icon). Commit on Enter, blur, or confirm button click; cancel on Escape. The edit and delete buttons are replaced with the confirm button while in edit mode. This keeps the list clean by default while making editability discoverable.

**Revert on empty input, inline error on conflict**
If the name is trimmed to empty, revert the input to the original name silently. If the API returns an error (e.g., 409 duplicate), keep the input in edit mode and display the error message inline below the input so the user can correct and retry without re-entering edit mode.

## Risks / Trade-offs

- [Button discoverability] → The pencil icon is a well-understood affordance for "edit". Placing it next to the trash button keeps the action area consistent.
- [Mobile usability] → On touch devices, tapping the edit button enters edit mode. The input gets auto-focused. Standard mobile keyboard behavior handles Enter/blur. No special handling needed.
