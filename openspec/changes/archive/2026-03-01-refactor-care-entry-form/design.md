## Context

The plant detail page (`ui/src/routes/plants/[id]/+page.svelte`) is ~990 lines and contains the care entry form inline — 8 state variables, 5 handlers, ~80 lines of template, and ~100 lines of scoped CSS. The form layout vertically stacks photo upload and backdate controls between the textarea and action buttons, causing overflow on mobile and an incoherent visual hierarchy.

A mockup exploration resulted in the **toolbar-absorbed** design: photo and backdate controls live as compact compound buttons in a toolbar row beneath the textarea, alongside cancel/save. The mockup is at `mockups/care-entry-form.html`.

## Goals / Non-Goals

**Goals:**
- Extract form into `CareEntryForm.svelte` with a clean props/events interface
- Implement toolbar-absorbed layout with compound photo/date buttons
- Ensure grouped flex wrapping (left group: photo+date tools, right group: cancel+save)
- Preserve all existing functionality: event type chips, notes, photo attach, backdate, submit

**Non-Goals:**
- Inline editing of existing entries (future work — the component interface supports it but we don't implement edit mode now)
- Changes to the care event API or data model
- Restyling the event type chips or textarea

## Decisions

### 1. Component interface

```svelte
<CareEntryForm
  plantId={plant.id}
  onsubmit={() => { /* reload events, hide form */ }}
  oncancel={() => { /* hide form */ }}
/>
```

The component owns all form state internally (event type, notes, photo, backdate, submitting). The parent only controls visibility (`showLogForm`) and reacts to submit/cancel events.

**Why**: The parent page doesn't need to touch any form state. Encapsulating everything inside the component removes 8 state variables and 5 handlers from the 990-line page. The `plantId` prop is needed for the API call. Submit/cancel callbacks let the parent reload care events and toggle form visibility.

**Alternative considered**: Passing form state down as props — rejected because it splits ownership and keeps the parent bloated.

### 2. Component handles its own API call

The component imports `addCareEvent`, `uploadCareEventPhoto`, and `loadCareEvents` from the stores/api directly, performs the submission, and calls `onsubmit` on success.

**Why**: Keeps the parent simple — it just shows/hides the form and reloads data on the callback. The current code already uses these store functions directly.

### 3. Toolbar-absorbed layout

```
┌──────────────────────────────────────────────┐
│  [fertilized] [repotted] [pruned] [custom]   │   ← type chips (unchanged)
├──────────────────────────────────────────────┤
│  ┌──────────────────────────────────────┐    │
│  │ Notes (optional)                     │    │   ← textarea (unchanged)
│  └──────────────────────────────────────┘    │
│                                              │
│  ┌─toolbar──────────────────────────────┐    │
│  │ [left group]         [right group]   │    │
│  │  📷  📅              Cancel  Save    │    │   ← inactive state
│  │  [img|✕] [dt-input|✕] Cancel Save    │    │   ← active state
│  └──────────────────────────────────────┘    │
└──────────────────────────────────────────────┘
```

The toolbar is a single flex row with `flex-wrap: wrap`. Two inner divs (`.toolbar-left`, `.toolbar-right`) act as non-breaking groups. `.toolbar-right` uses `margin-left: auto` to push to the right. On narrow viewports the groups wrap as units — cancel and save never separate.

**Why**: Solves the mobile overflow by allowing the toolbar to wrap into two rows while keeping related buttons together. The compound buttons (photo/date) absorb their expanded state into the toolbar instead of creating new vertical sections.

### 4. Compound button morphing

- **Photo inactive**: ghost icon button with `Camera` icon
- **Photo active**: compound group `[thumbnail | ✕]` — no camera icon, the thumbnail replaces it
- **Date inactive**: ghost icon button with `CalendarClock` icon
- **Date active**: compound group `[datetime-local input | ✕]` — no calendar icon, the input replaces it

The compound groups use `display: inline-flex` with shared border and `border-radius: var(--radius-btn)`. The dismiss button uses `XIcon` at size 12, matching the existing codebase pattern.

**Why**: User feedback explicitly requested no redundant icons when active. The morphing approach keeps the toolbar compact while still showing the attached content inline.

### 5. Photo handling stays internal

The hidden file input, `stageLogPhoto`, `clearLogPhoto`, and preview URL lifecycle (create/revoke `ObjectURL`) all live inside the component. The photo upload happens after care event creation, same as today.

### 6. Styles scoped to component

All new CSS (toolbar, compound groups) lives in the component's `<style>` block. No changes to global stylesheets (`buttons.css`, `chips.css`, etc.). The component uses existing design tokens and CSS classes (`.btn`, `.chip`, `.input`).

## Risks / Trade-offs

**[Risk] Hidden file input inside compound button** → The `<label>` wrapping the camera icon button contains the hidden `<input type="file">`. This pattern already works in the current codebase. The compound group replaces the label entirely when a photo is selected, so no UX confusion.

**[Trade-off] Component does API call directly** → Couples the component to the store layer. Acceptable because this is a UI-internal component, not a generic library component. If we later need a generic form, we can lift the API call out via a callback prop.

**[Trade-off] No edit mode yet** → The component interface (`plantId`, `onsubmit`, `oncancel`) can accommodate future edit mode by adding optional `initialData` props. We don't build that now to keep scope focused.
