## 1. Lightbox structure and styling

- [x] 1.1 Add lightbox overlay markup (detail page or shared component) with image container and backdrop close target
- [x] 1.2 Add lightbox styles for full-viewport overlay, centered image, dimmed backdrop, and scroll lock

## 2. Interaction behavior

- [x] 2.1 Wire lightbox open state to detail hero photo only when `photo_url` exists
- [x] 2.2 Implement close behavior (ESC key, backdrop click) and restore scroll on close
- [x] 2.3 Implement zoom and pan interactions with scale bounds and constrained translation

## 3. Tests

- [x] 3.1 Add UI tests for lightbox open/close behavior and no-photo fallback
- [x] 3.2 Add UI test coverage for zoom/pan state changes where feasible

## 4. Wrap-up

- [x] 4.1 Review README.md for any needed user-facing updates
- [ ] 4.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`
