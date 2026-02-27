## 1. Backend: ai-consultation event type

- [x] 1.1 Add `"ai-consultation"` to `VALID_EVENT_TYPES` in `src/api/care_events.rs`
- [x] 1.2 Add integration test: create care event with `event_type: "ai-consultation"` returns 201
- [x] 1.3 Add integration test: `ai-consultation` event does not trigger MQTT publish

## 2. Frontend API client: summarizeChat

- [x] 2.1 Add `summarizeChat(plantId: number, history: ChatMessage[]): Promise<string>` to `ui/src/lib/api.ts`

## 3. i18n keys

- [x] 3.1 Add i18n keys for `ai-consultation` event type label in en/de/es (en: "AI Consultation", de: "KI-Beratung", es: "Consulta IA")
- [x] 3.2 Add i18n keys for save-note UI strings in en/de/es: save note button, saving state, summary placeholder, save/cancel buttons, success message, error message

## 4. Care journal: ai-consultation event display

- [x] 4.1 Add `Sparkles` icon for `ai-consultation` in plant detail timeline (`plants/[id]/+page.svelte`)
- [x] 4.2 Add `ai-consultation` label to `eventTypeLabel()` in plant detail view
- [x] 4.3 Add `Sparkles` icon and `--color-ai` color mapping for `ai-consultation` in global care journal (`care-journal/+page.svelte`)
- [x] 4.4 Add `ai-consultation` to `FILTER_VALUES` and filter chip UI in global care journal
- [x] 4.5 Add `ai-consultation` label to `eventTypeLabel()` in global care journal

## 5. Chat drawer: save note flow

- [x] 5.1 Add "Save note" icon button (`BookOpen`) in the chat header with native tooltip, visible when assistant messages exist and not streaming
- [x] 5.2 Implement summarize call on button click with loading state
- [x] 5.3 Implement summary editor: replace input area with editable textarea pre-filled with AI summary, show Save/Cancel buttons
- [x] 5.4 Implement save: POST care event with `event_type: "ai-consultation"` and notes, show success indication, restore input area
- [x] 5.5 Implement cancel: dismiss editor, restore input area
- [x] 5.6 Implement error handling for summarize and save failures (displayed in chat messages area)
- [x] 5.7 Clear status messages when a new chat message is sent
- [x] 5.8 Add plain-text and informal tone instructions to chat system prompt

## 6. Tests

- [x] 6.1 Add unit test for `summarizeChat` API function
- [x] 6.2 Add component tests for save-note button visibility (hidden when no assistant messages, hidden during streaming, visible otherwise)
- [x] 6.3 Add component tests for save-note flow (summarize → edit → save, summarize → cancel)

## 7. Verification

- [x] 7.1 Run `cargo fmt`, `cargo clippy`, `cargo test`
- [x] 7.2 Run `cd ui && npm run check` and `npx vitest run`
