## Why

The inline care entry form on the plant detail page has grown organically and suffers from layout problems: buttons overflow on mobile, the photo upload and backdate controls are awkwardly stacked vertically between the textarea and action buttons, and the form markup is embedded directly in the 990-line `+page.svelte` with ~8 state variables and ~5 handlers. Extracting it into a standalone component with a compact toolbar-absorbed layout improves mobile usability, code maintainability, and prepares for future inline editing of existing entries.

## What Changes

- Extract the care entry form into a new `CareEntryForm.svelte` component with clearly defined props and events
- Replace the current vertical stack layout with a toolbar-absorbed design where photo and backdate controls become compound buttons that expand in-place within the toolbar row
- Photo control: inactive = camera icon button; active = `[thumbnail | ✕]` compound (no icon when active)
- Backdate control: inactive = calendar icon button; active = `[datetime-input | ✕]` compound (no icon when active)
- Toolbar uses two flex groups (`.toolbar-left` for photo/date, `.toolbar-right` for cancel/save) that wrap as groups, not individual items
- Action buttons (cancel/save) stay together on wrap and align right via `margin-left: auto`
- All existing functionality preserved: event type chips, notes textarea, photo attach, backdate, submit, cancel

## Capabilities

### New Capabilities

- `ui/care-entry-form`: Standalone care entry form component with toolbar-absorbed layout, compound photo/date controls, grouped flex wrapping, and component interface (props: plantId, eventTypes; events: submit, cancel)

### Modified Capabilities

- `ui/plant-detail`: Add requirement for the plant detail page to render the care entry form via the new `CareEntryForm` component instead of inline markup

## Impact

- **Files changed**: `ui/src/routes/plants/[id]/+page.svelte` (remove ~180 lines of form markup, styles, and state), new `ui/src/lib/components/CareEntryForm.svelte`
- **No API changes**: same `POST /api/plants/{id}/care-logs` endpoint
- **No dependency changes**: uses existing lucide-svelte icons and design tokens
- **Design reference**: `mockups/care-entry-form.html` contains the approved toolbar-absorbed mockup with all states
