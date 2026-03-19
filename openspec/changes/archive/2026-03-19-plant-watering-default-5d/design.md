## Context

The plant add/edit form exposes preset chips for watering intervals (3d, 7d, 14d, 30d) alongside a custom stepper. The 30-day chip is almost never selected and creates a large jump in the preset choices, so users still drop into the stepper to pick a practical cadence. This change keeps the default behavior untouched but replaces 30d with a 5d chip so the quick presets cover shorter intervals more naturally.

## Goals / Non-Goals

**Goals:**
- Swap the preset list inside `WateringInterval.svelte` so it now reads 3d, 5d, 7d, 14d and expose the same translations/labels (long name, short abbreviation, description) for the new option.
- Keep the rest of the component behavior unchanged (the `value` binding, custom stepper, keyboard support, etc.).
- Update translations and UI tests that enumerate the preset labels so they reference the new 5d chip instead of 30d.

**Non-Goals:**
- Touching backend defaults, API payloads, or database fixtures for watering intervals.
- Changing analytics, telemetry, or user defaults beyond the preset button list.

## Decisions

- **Presets array update** → Modify `PRESET_KEYS` in `WateringInterval.svelte` to include the new 5-day entry with fresh translation keys (`fiveDays`, `fiveDaysShort`, `frequent`). Remove the 30-day entry so it no longer renders. This keeps rendering logic and button structure the same.
- **Translations** → Add the missing translation strings for the 5d chip (long label, short label, description) in `ui/src/lib/i18n/en.ts`/`es.ts` and any other locales we support. Existing strings tied to 30d can be recycled or deleted if unused.
- **Tests & snapshots** → Adjust `WateringInterval.test.ts` expectations to look for `{$translations.form.frequent}` or whatever descriptor we choose for 5 days, ensuring the button still receives `.active` when `value` equals 5. No new integration tests are necessary because the logic still uses the same data-driven loop.

## Risks / Trade-offs

- [Risk] Removing the 30d preset could surprise some advanced users who relied on it → Mitigation: Document the change in release notes so people know the preset list changed but they can still use the stepper to reach longer intervals.
- [Risk] Translation bundles might retain stale 30d keys leading to unused strings → Mitigation: Clean up the unused keys or repurpose them for the new 5d option as we touch each locale.
- [Risk] Downstream specs/tests may have hardcoded expectations for a 30d label → Mitigation: Update the relevant spec delta and test files after landing this change.

## Migration Plan

- No migration required; the change is purely a UI preset swap.

## Open Questions

- None.
