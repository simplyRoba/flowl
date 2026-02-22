## 1. Setup and planning

- [x] 1.1 Create a feature branch off `main` for the change
- [x] 1.2 Identify all hardcoded UI strings across components and pages
- [x] 1.3 Define the translation key structure (groups and keys)

## 2. i18n infrastructure

- [x] 2.1 Create locale store with localStorage persistence (`flowl.locale`) following theme store pattern
- [x] 2.2 Create English translation dictionary as the canonical type definition
- [x] 2.3 Create German translation dictionary satisfying the English type
- [x] 2.4 Create Spanish translation dictionary satisfying the English type
- [x] 2.5 Implement plural helper (`plural({one, other}, n)` with `{n}` substitution)
- [x] 2.6 Create derived `translations` store that resolves to the active locale dictionary

## 3. Settings UI

- [x] 3.1 Add Language section to settings page after Appearance with pill toggle (English / Deutsch / Español)
- [x] 3.2 Wire selector to locale store for immediate persistence and reactive update

## 4. Shell integration

- [x] 4.1 Replace hardcoded sidebar nav labels with `$translations.nav.*` lookups
- [x] 4.2 Verify widescreen expanded sidebar renders translated labels correctly

## 5. Component string replacement

- [ ] 5.1 Replace hardcoded strings in dashboard/plant list pages
- [ ] 5.2 Replace hardcoded strings in plant detail and care journal pages
- [ ] 5.3 Replace hardcoded strings in form and dialog components
- [ ] 5.4 Replace hardcoded strings in settings page sections

## 6. Tests

- [ ] 6.1 Add unit tests for locale store persistence and fallback behavior
- [ ] 6.2 Add unit tests for plural helper
- [ ] 6.3 Add unit tests for translations store reactivity

## 7. Verification

- [ ] 7.1 Manually verify all three languages render correctly across pages
- [ ] 7.2 Run `npm run check` and `npm run build` in `ui/`
- [ ] 7.3 Run `cargo fmt`, `cargo clippy`, and `cargo test`
