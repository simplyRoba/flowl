## 1. ModalDialog component

- [x] 1.1 Create `ModalDialog.svelte` with `open`, `title`, `message`, `mode` (`confirm`/`alert`), `variant` (`danger`/`warning`), `confirmLabel`, `onconfirm`, `oncancel`, `onclose` props using the HTML `<dialog>` element with `showModal()`/`close()`
- [x] 1.2 Add component tests covering: confirm mode (two buttons, confirm callback, cancel callback), alert mode (single OK button, close callback), danger/warning variant styling, Escape key behavior, backdrop click behavior (closes in confirm, ignored in alert), open prop reactivity

## 2. Replace confirm() in settings page

- [x] 2.1 Replace native `confirm()` for delete-location with `ModalDialog` (danger variant, message includes location name and plant count)
- [x] 2.2 Replace native `confirm()` for import-data with `ModalDialog` (danger variant, message includes file name and replacement warning)
- [x] 2.3 Add confirmation dialog to MQTT repair action (warning variant, message warns about topic clearing and republishing)
- [x] 2.4 Update settings page tests for delete-location, import, and repair confirmation dialogs

## 3. Replace confirm() in plant detail page

- [x] 3.1 Replace native `confirm()` for delete-plant with `ModalDialog` (danger variant, message includes plant name)
- [x] 3.2 Update plant detail page tests for delete confirmation dialog

## 4. Checks

- [x] 4.1 Run `ui/npm run check`, `cargo fmt`, `cargo clippy`, and `cargo test`
