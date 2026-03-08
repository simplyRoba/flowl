## Why

In standalone PWA mode, the browser chrome and its reload affordances disappear, which makes it awkward to manually refresh data on phones and tablets. A focused pull-to-refresh interaction on the main browse routes gives mobile PWA users a familiar way to reload current data without exposing that gesture on form-heavy screens where accidental refreshes would be disruptive.

## What Changes

- Add pull-to-refresh support for installed standalone PWA sessions on touch devices.
- Limit the gesture to browse routes: `/`, `/care-journal`, `/settings`, and `/plants/[id]`.
- Exclude plant creation and editing flows, especially `PlantForm`-driven routes, from custom pull-to-refresh handling.
- Use a custom pull gesture that triggers a full page reload after the user releases beyond a threshold, so each route can reuse its existing load path without a cross-route refresh refactor.
- Provide visible gesture feedback while pulling and suppress the behavior outside standalone PWA mode.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ui/pwa`: standalone PWA sessions gain route-scoped pull-to-refresh behavior for touch devices.

## Impact

- Affected UI shell and routing code in `ui/src/routes/+layout.svelte` for gesture detection, route allowlisting, and standalone-mode gating.
- Affected browse pages in `ui/src/routes/+page.svelte`, `ui/src/routes/care-journal/+page.svelte`, `ui/src/routes/settings/+page.svelte`, and `ui/src/routes/plants/[id]/+page.svelte` because they become eligible refresh targets.
- Testing impact on mobile/touch interaction coverage, route gating, and standalone-mode behavior.
- No backend API changes and no new runtime dependencies expected.
