## 1. Setup and planning

- [x] 1.1 Create a feature branch off `main` for the change
- [x] 1.2 Confirm theme default behavior (use `system` unless stored preference exists)
- [x] 1.3 Identify UI files where theme tokens and root class are applied

## 2. Theme infrastructure

- [x] 2.1 Add light/dark CSS variables per DESIGN.md and wire a root theme class
- [x] 2.2 Implement theme preference storage (`light`/`dark`/`system`) with localStorage fallback
- [x] 2.3 Add system preference listener to update theme when set to `system`
- [x] 2.4 Add a small theme bootstrap to avoid flash-of-incorrect-theme on load

## 3. Settings UI

- [x] 3.1 Add Appearance section with Light/Dark/System selector matching mockups
- [x] 3.2 Wire selector state to stored preference and reflect active selection

## 4. Tests

- [x] 4.1 Add unit tests for theme preference resolution and persistence
- [x] 4.2 Add unit tests for system preference change handling

## 5. Docs and verification

- [x] 5.1 Review README.md for any end-user updates (apply only if needed)
- [x] 5.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`
