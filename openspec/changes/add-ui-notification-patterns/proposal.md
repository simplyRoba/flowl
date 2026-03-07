## Why

Feedback in the UI is currently fragmented. Some actions show inline errors, some show row-local status text, some failures only land in route-level stores, and some success paths give no explicit acknowledgement at all. The review calls this out directly in `REVIEW.md` item 50, and item 10 exposes a concrete usability bug: watering from the dashboard attention cards can fail without visible feedback near the action.

Before implementing a toast/snackbar system, we should define the global concept clearly: when feedback belongs inline vs in a toast vs at page level, how notifications behave across breakpoints, and which existing screens should adopt which pattern.

## What Changes

- Add a new `ui/notifications` capability that defines a feedback taxonomy for field-inline, section-inline, page-level, modal, and toast notifications.
- Define a global toast/snackbar pattern with responsive placement, stack behavior, severity rules, dismissal policy, and accessibility expectations.
- Audit current UI surfaces and document a route-by-route recommendation matrix for where toast/snackbar usage is appropriate and where inline feedback should remain the primary pattern.
- Add an initial modified requirement for the dashboard attention-card watering flow so its success/error feedback is visible where the user acts.
- Include small ASCII mockups in the design so implementation and review share the same visual target.

## Capabilities

### New Capabilities
- `ui/notifications`: Global notification taxonomy and toast/snackbar behavior

### Modified Capabilities
- `ui/settings`: Use global toast feedback for import and MQTT repair outcomes instead of row-inline status text
- `ui/plant-dashboard`: Define visible feedback for attention-card watering actions

## Impact

- New spec: `openspec/specs/ui/notifications/spec.md`
- New design artifact: `openspec/changes/add-ui-notification-patterns/mockups/notifications.html`
- Likely UI touchpoints during implementation: `ui/src/routes/+layout.svelte`, `ui/src/routes/+page.svelte`, `ui/src/routes/settings/+page.svelte`, `ui/src/routes/plants/new/+page.svelte`, `ui/src/routes/plants/[id]/edit/+page.svelte`, `ui/src/routes/plants/[id]/+page.svelte`, `ui/src/lib/components/ChatDrawer.svelte`, `ui/src/lib/components/CareEntryForm.svelte`
- New shared UI pieces are expected during implementation (toast host, store, component, translation strings, tests)
- No backend changes
