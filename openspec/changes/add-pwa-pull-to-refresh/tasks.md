## 1. Gesture foundation

- [ ] 1.1 Add standalone-PWA and touch-capability detection in the shared UI shell.
- [ ] 1.2 Add route allowlist matching for `/`, `/care-journal`, `/settings`, and `/plants/[id]`, while excluding `/plants/new` and `/plants/[id]/edit`.
- [ ] 1.3 Implement top-of-page pull gesture tracking in the shared shell using the existing body-scroll model.

## 2. Refresh behavior and safety gates

- [ ] 2.1 Add a lightweight pull-to-refresh indicator with idle, pulling, release-to-refresh, and refreshing states.
- [ ] 2.2 Trigger a full route reload when the gesture is released past the refresh threshold.
- [ ] 2.3 Suppress gesture activation when the page is not at scroll top or when transient overlays are open on allowlisted routes.

## 3. Route integration and tests

- [ ] 3.1 Verify the gesture works on dashboard, care journal, settings, and plant detail without changing their existing data-loading paths.
- [ ] 3.2 Add UI tests for standalone-mode gating, route allowlisting, threshold behavior, and excluded form routes.
- [ ] 3.3 Add UI tests covering plant-detail suppression while modal/lightbox/chat/inline care entry states are open.

## 4. Final verification

- [ ] 4.1 Review `README.md` and update it only if the new PWA refresh behavior is important end-user guidance.
- [ ] 4.2 Run `npm run check --prefix ui`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test`.
