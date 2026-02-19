## Why

The dashboard currently shows all plants as a flat card grid with per-card status badges, but there is no at-a-glance summary of what needs attention. Users have to visually scan every card to find overdue or due plants. A dedicated "Needs Attention" section surfaces actionable plants immediately — with an inline Water button — so users can act without navigating to the detail page.

## What Changes

- The greeting subtitle becomes dynamic: "N plants need water today" (count of due + overdue plants).
- A "Needs Attention" section appears between the greeting and the plant grid, showing cards for overdue and due plants with photo/icon, name, status badge, and an inline "Water" button.
- The section is hidden when no plants are due or overdue.
- On desktop the attention cards display in a 2-column grid; on mobile they stack in a single column.
- On mobile the Water button is icon-only; on desktop it shows the "Water" label.
- Watering from the attention section refreshes the dashboard so the plant drops out of the section when it becomes "ok".

## Capabilities

### New Capabilities

_(none — this change extends an existing capability)_

### Modified Capabilities
- `ui/plants`: The Plants Dashboard requirement gains a dynamic greeting subtitle, a "Needs Attention" section with attention cards and inline Water action.

## Impact

- **Frontend only** — no backend/API changes required. All data (`watering_status`, `photo_url`) is already returned by `GET /api/plants`. The `waterPlant` API call already exists.
- **Affected files**: `ui/src/routes/+page.svelte` (dashboard page), `ui/src/lib/stores/plants.ts` (water action already exists).
- **Tests**: Existing dashboard tests (`ui/src/tests/routes/dashboard.test.ts`) need updates; new tests for the attention section visibility, card rendering, and water action.
