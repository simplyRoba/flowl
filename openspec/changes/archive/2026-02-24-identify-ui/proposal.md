## Why

The `POST /api/ai/identify` endpoint is live and tested, but there is no UI to reach it. Users must use curl to identify plants. Adding the identify UI to PlantForm completes the identification flow end-to-end and is a prerequisite for the chat features (Phase 5+), since users will expect AI features to "just work" from the form.

## What Changes

- Add an "Identify Plant" section in `PlantForm` between the media section and the identity fields, visible only when a photo is present and AI is enabled
- Add optional extra photo upload slots (leaf close-up, stem/pot) for better identification accuracy — these are ephemeral, not stored on the plant
- Add a loading state (shimmer/spinner) while identification is in progress
- Add a suggestion card that shows the AI result (common name, scientific name, confidence, care profile preview) with "Apply to form" and "Dismiss" actions
- "Apply to form" auto-fills: species, name (if empty), notes (if empty), watering interval, light needs, difficulty, pet safety, growth speed, soil type, soil moisture
- Add an `identifyPlant` function to the frontend API client that sends multipart photos to `POST /api/ai/identify`
- Add i18n keys for all identify UI strings (en, de, es)

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ui/plants`: Add requirements for the identify section in PlantForm (button visibility, extra photo slots, loading state, suggestion card, form auto-fill)
- `ui/i18n`: Add translation keys for identify UI strings

## Impact

- **Code:** `ui/src/lib/components/PlantForm.svelte` (main changes), `ui/src/lib/api.ts` (new `identifyPlant` function), `ui/src/lib/i18n/en.ts`, `de.ts`, `es.ts` (new keys)
- **API:** Consumes existing `POST /api/ai/identify` — no backend changes
- **Dependencies:** None — uses existing Svelte/SvelteKit tooling and lucide-svelte icons
