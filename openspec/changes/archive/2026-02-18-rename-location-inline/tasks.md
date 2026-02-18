## 1. Inline Rename UI

- [x] 1.1 Import `updateLocation` from the locations store and `Pencil` from `lucide-svelte` in `+page.svelte`
- [x] 1.2 Add `editingId` state variable to track which location is in edit mode (null when none)
- [x] 1.3 Add a pencil icon edit button next to the existing trash delete button for each location row
- [x] 1.4 On edit button click, set `editingId` to that location's id, replace the name span with a focused `<input>` (text selected), and hide the edit/delete buttons for that row
- [x] 1.5 On Enter or blur, trim input value — if non-empty and different from original, call `updateLocation(id, name)` then exit edit mode; if empty or unchanged, revert and exit edit mode
- [x] 1.6 On Escape, revert to original name and exit edit mode (no API call)

## 2. Styling

- [x] 2.1 Style the edit button to match the existing delete button (same size, border, icon style)
- [x] 2.2 Style the inline edit input to match the existing `.location-name` font size/weight so the switch feels seamless

## 3. Verification

- [x] 3.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` to verify no regressions
- [x] 3.2 Manual test: edit button to rename, Enter to save, Escape to cancel, blur to save, empty name rejected, duplicate name shows error
