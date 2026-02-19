## Context

The dashboard (`/`) fetches all plants via `GET /api/plants` and renders them as a card grid. Each card already includes a `StatusBadge` component showing `ok`, `due`, or `overdue`. However, there is no way to see at a glance which plants need attention, and no way to water a plant without navigating to its detail page.

The mockup (`mockups/index.html`) defines the target UI: a "Needs Attention" section between the greeting and the "All Plants" grid, containing attention cards for overdue/due plants with inline Water buttons. The greeting subtitle dynamically shows the count of plants needing water.

All required data (`watering_status`, `photo_url`, `next_due`) and the `waterPlant` API call already exist.

## Goals / Non-Goals

**Goals:**
- Show a "Needs Attention" section with cards for overdue and due plants, each with an inline Water button.
- Make the greeting subtitle dynamic: "N plants need water today".
- Allow watering directly from the dashboard without navigating to the detail page.

**Non-Goals:**
- New backend endpoints — all data is already available from `GET /api/plants`.
- Changing the "All Plants" grid below — it stays as-is with its existing status badges.
- Sorting or filtering the main plant grid.

## Decisions

### 1. Client-side filtering over a dedicated API endpoint

Filter plants with `watering_status !== 'ok'` from the `$plants` array to build the attention list. The plant list is bounded (typical home user has <100 plants), so this is negligible. No new backend endpoint needed.

*Alternative*: `GET /api/plants?status=due,overdue` — rejected; adds backend complexity for a purely presentational concern.

### 2. Inline Water action reuses existing store method

The `waterPlant` function in the plants store already calls `POST /api/plants/:id/water` and updates the store. The attention card's Water button calls this directly, and reactivity removes the plant from the attention section once its status becomes `ok`.

*Alternative*: A separate lightweight water action — rejected; duplicates existing logic.

### 3. Attention section as inline markup in +page.svelte

The attention section is ~40 lines of markup tightly coupled to the dashboard's plant data and the existing `waterPlant` store action. No reuse elsewhere. Keeping it inline avoids extra component indirection.

*Alternative*: `NeedsAttention.svelte` component — reconsider if the section is needed on other pages later.

### 4. Responsive layout follows the mockup

- Desktop/tablet: attention cards in a 2-column grid, Water button shows icon + "Water" label.
- Mobile: attention cards stack single-column, Water button is icon-only.

This matches the existing responsive patterns used elsewhere in the dashboard.

## Risks / Trade-offs

- **[Risk] Attention list stale after watering on detail page** → `loadPlants()` is called on dashboard mount, so returning always fetches fresh data.
- **[Trade-off] Attention section duplicates plant info from the grid below** → Acceptable; the attention section serves a different purpose (quick action) and matches the mockup design.
