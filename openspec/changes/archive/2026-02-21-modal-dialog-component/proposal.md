## Why

The app uses the browser's native `confirm()` for destructive actions (delete plant, delete location, import data). These are ugly, inconsistent across browsers, block the main thread, and cannot be styled to match the app's theme. Error feedback is currently inline text that's easy to miss. A reusable `ModalDialog` component would provide a consistent, themed experience for confirmations, warnings, and error messages.

## What Changes

- Add a `ModalDialog` component using the HTML `<dialog>` element with backdrop, themed styling, and keyboard/focus management.
- Support modes:
  - **confirm**: Two-button dialog (cancel + confirm) for "are you sure?" prompts. Confirm button style varies by variant.
  - **alert**: Single-button dialog for error or informational messages the user must acknowledge.
- Support variants: `danger` (destructive actions, red confirm button) and `warning` (caution, default-styled confirm button).
- Replace all three existing `confirm()` calls with the new component:
  - Delete plant confirmation (`ui/src/routes/plants/[id]/+page.svelte`)
  - Delete location confirmation (`ui/src/routes/settings/+page.svelte`)
  - Import data confirmation (`ui/src/routes/settings/+page.svelte`)
- Add a confirmation dialog to the MQTT repair action (`ui/src/routes/settings/+page.svelte`), which currently fires immediately on click with no confirmation.

## Capabilities

### New Capabilities
- `ui/modal-dialog`: Reusable modal dialog component for confirmations, warnings, and error/info alerts

### Modified Capabilities
- `ui/settings`: Replace native `confirm()` calls with `ModalDialog` for delete-location, import-data, and MQTT-repair flows
- `ui/plants`: Replace native `confirm()` call with `ModalDialog` for delete-plant flow

## Impact

- New file: `ui/src/lib/components/ModalDialog.svelte`
- Modified: `ui/src/routes/settings/+page.svelte` (replace 2 confirm calls, add repair confirmation)
- Modified: `ui/src/routes/plants/[id]/+page.svelte` (replace 1 confirm call)
- Tests: component tests for ModalDialog, updated page tests for settings and plant detail
- No backend changes, no new dependencies
