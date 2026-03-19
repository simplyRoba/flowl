## Why

The plant form’s watering presets currently jump from 14 days straight to a 30-day option, which many users never choose. Swapping the unused 30d chip for a 5d option exposes a more practical cadence without changing the default value or backend fallbacks.

## What Changes

- Replace the existing 30-day preset chip with a 5-day option so users can pick a shorter cadence from the quick presets without impacting the default value or the custom stepper.
- Refresh the translation keys and tests that describe the preset labels so they talk about the new 5-day chip instead of 30 days.
- Ensure any references (docs, analytics, snapshots) to the preset list include the new option while keeping the rest of the watering interval component unchanged.

## Capabilities

### New Capabilities
- _None_ (no brand-new capability is being introduced).

### Modified Capabilities
- `ui/plant-form`: The watering interval preset list swaps the 30-day chip for a 5-day chip while retaining the other presets and leaving the default/custom logic untouched.

## Impact

- UI components (`ui/src/lib/components/PlantForm.svelte`, `WateringInterval.svelte`) will replace the 30-day preset chip with the 5-day option while keeping the default value unchanged and continuing to expose the custom stepper; update `WateringInterval.test.ts` accordingly.
- Localization bundles (`ui/src/lib/i18n/en.ts`, `ui/src/lib/i18n/es.ts`, etc.) must get the new strings for the 5d chip description and remove or repurpose any text tied to the removed 30d option.
- Specs (`openspec/specs/ui/plant-form/spec.md`) will receive a delta file describing the new preset list so downstream docs/tests know about the 5-day chip.
