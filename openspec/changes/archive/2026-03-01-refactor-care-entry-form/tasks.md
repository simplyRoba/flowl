## 1. Create CareEntryForm component

- [x] 1.1 Create `ui/src/lib/components/CareEntryForm.svelte` with props (`plantId: number`, `onsubmit`, `oncancel`) and internal state (eventType, notes, photo, photoPreview, occurredAt, showOccurredAt, submitting)
- [x] 1.2 Implement event type chips row (fertilized, repotted, pruned, custom) using existing `.chip.chip-solid` classes and lucide icons
- [x] 1.3 Implement notes textarea with translation placeholder
- [x] 1.4 Implement toolbar layout with `.toolbar-left` and `.toolbar-right` flex groups, `flex-wrap: wrap`, and `margin-left: auto` on the right group
- [x] 1.5 Implement photo tool button: inactive state (Camera icon ghost button with hidden file input), active compound group `[thumbnail | ✕]` with no icon, dismiss revokes object URL
- [x] 1.6 Implement backdate tool button: inactive state (CalendarClock icon ghost button), active compound group `[datetime-local input | ✕]` with no icon, defaults to current time, max constrained to now
- [x] 1.7 Implement action buttons in toolbar-right: save (primary, disabled until type selected, shows saving text during submit) and cancel (outline, resets state and calls oncancel)
- [x] 1.8 Implement submit handler: call `addCareEvent`, upload photo if attached via `uploadCareEventPhoto`, reload events via `loadCareEvents`, call `onsubmit`

## 2. Integrate into plant detail page

- [x] 2.1 Replace inline form markup in `+page.svelte` (lines ~438-520 template, ~847-949 styles) with `<CareEntryForm>` component
- [x] 2.2 Remove form state variables from `+page.svelte` (showLogOccurredAt, logEventType, logNotes, logOccurredAt, logSubmitting, logPhoto, logPhotoPreview) — keep only `showLogForm` for visibility toggle
- [x] 2.3 Remove form handler functions from `+page.svelte` (handleLogSubmit, handleLogCancel, handleLogPhotoSelect, stageLogPhoto, clearLogPhoto, nowLocalInputValue)
- [x] 2.4 Wire `onsubmit` to reload care events and hide form, `oncancel` to hide form

## 3. Verify

- [x] 3.1 Run `npm run check` from `ui/` to verify TypeScript and Svelte compilation
