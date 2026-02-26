## 1. i18n

- [x] 1.1 Add `chat` namespace keys to `en.ts` (`askAi`, `placeholder`, `send`, `thinking`, `errorMessage`, `close`, `emptyState`, `quickQuestions`, `healthCheck`, `wateringAdvice`, `whenToRepot`, `lightRequirements`, `whyOverdue`, `helpIdentify`)
- [x] 1.2 Add matching `chat` namespace keys to `de.ts` and `es.ts`

## 2. API Client

- [x] 2.1 Add `chatPlant(plantId, message, history, signal?)` async generator function to `api.ts` — consumes SSE stream via `fetch` + `ReadableStream`, yields string deltas, throws on error
- [x] 2.2 Add `ChatMessage` interface to `api.ts` (`{ role: string, content: string }`)

## 3. ChatDrawer Component

- [x] 3.1 Create `ChatDrawer.svelte` with props: `plant` (Plant), `open` (boolean), `onclose` callback
- [x] 3.2 Implement desktop layout — 400px right panel, flex sibling to main content, slide-in animation
- [x] 3.3 Implement mobile layout — bottom sheet above 56px nav bar, fixed overlay with drag handle, backdrop
- [x] 3.4 Implement chat header with sparkle icon, plant name, and close button
- [x] 3.5 Implement quick-question chips — context-aware based on `watering_status` and `species`, hidden after first message
- [x] 3.6 Implement empty state — centered sparkle icon + "Ask anything about your [name]'s care"
- [x] 3.7 Implement message list — user bubbles (right, `--color-ai`) and AI bubbles (left, `--color-ai-tint`), auto-scroll
- [x] 3.8 Implement text input + send button — pill input, circular send button, disabled when empty or streaming
- [x] 3.9 Implement streaming response rendering — typing indicator (animated dots), incremental token append, finalize on `done`
- [x] 3.10 Implement error handling — display error message in chat on stream failure, re-enable input
- [x] 3.11 Implement `AbortController` — cancel in-flight fetch on component destroy
- [x] 3.12 Implement history cap — send max 20 messages in request, keep all visible in UI
- [x] 3.13 Implement mobile drag-to-dismiss gesture

## 4. Plant Detail Integration

- [x] 4.1 Add `aiEnabled` check via `fetchAiStatus()` on mount in `+page.svelte`
- [x] 4.2 Add "Ask AI" button (sparkle icon, `--color-ai`) next to "Water now" in hero section, gated behind `aiEnabled`
- [x] 4.3 Add `chatOpen` state and mount `ChatDrawer` component
- [x] 4.4 Hide mobile action bar (`PageHeader`) when `chatOpen` is true via CSS class toggle

## 5. Testing

- [x] 5.1 Add test: "Ask AI" button visible when AI enabled, hidden when disabled
- [x] 5.2 Add test: chat drawer opens on button click, displays quick chips, closes on X
- [x] 5.3 Add test: AI status check failure hides button gracefully

## 6. Verification

- [x] 6.1 Run `npm run check` in `ui/`, `cargo fmt`, `cargo clippy` — all pass, 0 errors, 174 tests green
