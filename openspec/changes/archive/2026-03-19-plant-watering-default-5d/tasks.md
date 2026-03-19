## 1. Watering interval component

- [x] 1.1 Swap the 30-day preset entry in `ui/src/lib/components/WateringInterval.svelte` for a 5-day entry that uses new translation keys (`fiveDays`, `fiveDaysShort`, a fresh description key) while leaving the existing `value`/`onchange` flow untouched.
- [x] 1.2 Remove any references to the old `thirtyDays*` keys from the component/test surface and wire the new keys in so the build compiles cleanly.

## 2. Localization & tests

- [x] 2.1 Add the 5-day translation strings (`fiveDays`, `fiveDaysShort`, `frequent`) to `ui/src/lib/i18n/en.ts`, `es.ts`, and `de.ts`, retiring the unused 30-day keys so the presets stay localized.
- [x] 2.2 Update `ui/src/lib/components/WateringInterval.test.ts` to look for the new descriptor text (e.g., the `frequent` label) and assert that the 5-day preset button is rendered and becomes active when `value` is 5.

## 3. Verification

- [x] 3.1 Run `npm run check --prefix ui`, `npm run format --prefix ui` and `npm run lint:fix --prefix ui` to validate the UI build and Rust crates before archiving this change.
