## Why

Phase 1 added the AI provider backend and a `GET /api/ai/status` endpoint, but there is no UI surface showing users whether AI is active, which provider they are connected to, or which model is in use. Adding an AI status section to Settings gives users immediate visibility into their AI configuration without needing to check environment variables or server logs.

## What Changes

- Add an "AI Assistant" section to the Settings page between MQTT and Data, following the existing read-only status pattern used by the MQTT section
- When AI is enabled: show a green "Enabled" indicator, the base URL (hostname only), and the configured model name
- When AI is disabled: show a "Disabled" indicator with guidance to set `FLOWL_AI_API_KEY`
- When the status fetch fails: hide the section entirely (same pattern as MQTT/About)
- Add i18n keys for the AI status section in all three locales (en, de, es)
- Add a frontend API helper to call `GET /api/ai/status`

## Capabilities

### New Capabilities

_(none — this change extends existing capabilities)_

### Modified Capabilities

- `ui/settings`: Add AI Assistant section requirement (section ordering, enabled/disabled states, display fields)

## Impact

- **Frontend `+page.svelte`**: New section block, new state variable, new `onMount` fetch
- **Frontend `api.ts`**: New `AiStatus` type and `fetchAiStatus()` function
- **Frontend i18n (`en.ts`, `de.ts`, `es.ts`)**: New keys under `settings.*`
- **Backend**: No changes — `GET /api/ai/status` already exists from Phase 1
- **Dependencies**: No new dependencies — uses existing `lucide-svelte` icon set
