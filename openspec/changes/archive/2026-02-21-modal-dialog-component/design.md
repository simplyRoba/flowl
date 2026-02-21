## Context

The app currently uses `window.confirm()` in three places (delete plant, delete location, import data) and has no confirmation on the MQTT repair action. Native confirm dialogs cannot be styled, look different across browsers, and block the main thread. Error feedback is shown via inline text spans that are easy to miss. The app already has a `PhotoLightbox` component that demonstrates the overlay pattern (fixed positioning, backdrop, Escape key handling) but it's purpose-built and not reusable as a dialog.

## Goals / Non-Goals

**Goals:**
- Provide a single `ModalDialog` component that handles both confirmation prompts and alert/error messages.
- Use the native HTML `<dialog>` element for built-in accessibility (focus trapping, aria roles, Escape key).
- Match the app's existing design language (CSS variables, border radius, button styles).
- Replace all `confirm()` usage and add confirmation to MQTT repair.

**Non-Goals:**
- Form dialogs or dialogs with custom content/inputs.
- Stacking multiple dialogs.
- Animation or transition effects.

## Decisions

- Use the HTML `<dialog>` element with `showModal()` / `close()` rather than a custom overlay div.
  - **Alternative:** Custom div with `position: fixed` and manual focus trap (like PhotoLightbox). **Rejected** — `<dialog>` provides focus trapping, Escape-to-close, `::backdrop` styling, and `aria-modal` for free.

- Two modes controlled by a `mode` prop:
  - `confirm` — shows a cancel button and a confirm button. Fires `onconfirm` or `oncancel` callbacks.
  - `alert` — shows a single "OK" button. Fires `onclose` callback.

- Two variants controlled by a `variant` prop:
  - `danger` — confirm/OK button uses `btn-danger` styling (red). For destructive actions.
  - `warning` — confirm/OK button uses default `btn-primary` styling. For caution prompts.

- Props: `open` (boolean), `title` (string), `message` (string), `mode` (`"confirm" | "alert"`, default `"confirm"`), `variant` (`"danger" | "warning"`, default `"warning"`), `confirmLabel` (string, default `"Confirm"`), `onconfirm`, `oncancel`, `onclose`.

- The dialog reacts to the `open` prop via a Svelte `$effect` that calls `showModal()` / `close()`. The parent owns the open state.

- Escape key is handled natively by `<dialog>`. The component listens for the `cancel` event to notify the parent.

- Clicking the backdrop (`::backdrop`) closes the dialog in confirm mode (treated as cancel) but not in alert mode (must acknowledge).

## Risks / Trade-offs

- `<dialog>` has full browser support in all modern browsers. No polyfill needed.
- Replacing `confirm()` changes the flow from synchronous to callback-based. Each call site needs minor refactoring to use state variables for dialog open/close instead of inline `if (!confirm(...)) return`.
