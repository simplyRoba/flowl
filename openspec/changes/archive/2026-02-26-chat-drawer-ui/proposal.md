## Why

The streaming chat and summarize backend endpoints are complete (Phase 5), but there is no UI for users to interact with the plant AI assistant. Users need a conversational interface on the Plant Detail page to ask care questions and receive streaming AI responses with full plant context.

## What Changes

- Add an "Ask AI" button to the Plant Detail hero section (gated behind AI enabled status)
- New chat drawer component that slides in from the right on desktop
- New bottom sheet component that slides up on mobile with drag-to-dismiss
- On mobile, the page action bar (Back / Edit / Delete) is hidden while the bottom sheet is open to maximize chat space
- Chat message list rendering user and AI messages
- Text input with send button
- SSE streaming response rendering with typing indicator
- Context-aware quick-question chips shown when chat is empty (e.g. "Health check", "Watering advice", "Why overdue?" when plant is overdue)
- Session-scoped chat history (preserved while drawer is open, cleared on page navigation)
- New i18n keys for chat UI across all three locales (en, de, es)

**Out of scope (Phase 7):**
- Inline photo upload in chat messages
- Markdown rendering for AI responses
- "Save note" flow (AI summary → care journal entry)
- New `ai-consultation` care event type

## Capabilities

### New Capabilities
- `ui/chat-drawer`: Chat drawer/bottom-sheet component with message list, text input, streaming response rendering, and quick-question chips

### Modified Capabilities
- `ui/plants`: Add "Ask AI" button to Plant Detail hero section, wire up chat drawer open/close
- `ui/i18n`: Add chat namespace with keys for the chat UI

## Impact

- **Frontend only** — no backend changes needed (chat + summarize endpoints already exist)
- **New component:** `ChatDrawer.svelte` (or similar) — first streaming UI consumer in the codebase
- **New API wrapper:** `chatPlant()` function in `src/lib/api.ts` consuming SSE stream via `fetch` + `ReadableStream`
- **Modified page:** `plants/[id]/+page.svelte` — adds AI button, mounts chat drawer, hides mobile action bar when chat is open
- **CSS:** Uses existing `--color-ai` / `--color-ai-tint` / `--color-ai-soft` tokens; new styles for drawer, message bubbles, and input area
- **i18n:** New keys in `en.ts`, `de.ts`, `es.ts` under a `chat` namespace
- **Dependencies:** No new npm packages — uses native `fetch` streaming and CSS transitions
