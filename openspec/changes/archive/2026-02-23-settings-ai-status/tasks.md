## 1. Frontend API Layer

- [x] 1.1 Add `AiStatus` type and `fetchAiStatus()` function to `ui/src/lib/api.ts`

## 2. i18n

- [x] 2.1 Add AI status keys to `settings` namespace in `en.ts` (`ai`, `aiEnabled`, `aiDisabled`, `aiDisabledHint`, `provider`, `model`)
- [x] 2.2 Add matching keys in `de.ts` and `es.ts`

## 3. Settings Page

- [x] 3.1 Add AI Assistant section to `+page.svelte` between MQTT and Data (fetch on mount, hide on failure, enabled/disabled states, `Sparkles` icon, hostname extraction from base URL)

## 4. Verify

- [x] 4.1 Run `cd ui && npm run check`, `cargo fmt --check`, `cargo clippy`, and `cargo test`
