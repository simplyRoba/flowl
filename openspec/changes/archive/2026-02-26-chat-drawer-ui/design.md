## Context

The backend chat (`POST /api/ai/chat`) and summarize (`POST /api/ai/summarize`) endpoints shipped in Phase 5. The chat endpoint returns an SSE stream of `{"delta": "..."}` events terminated by `{"done": true}`. No frontend streaming consumer exists yet — the identify flow uses a blocking `fetch` + `resp.json()` pattern.

The Plant Detail page (`plants/[id]/+page.svelte`) has a hero section with a "Water now" button. On mobile, a fixed action bar (Back / Edit / Delete) sits above the bottom tab nav. The app uses Svelte 5 runes, Lucide icons, and the existing `--color-ai` CSS tokens.

Mockups are in `openspec/changes/chat-drawer-ui/mockups.html`.

## Goals / Non-Goals

**Goals:**
- Deliver a text-only chat UI on the Plant Detail page — desktop drawer + mobile bottom sheet
- Consume SSE streaming from `POST /api/ai/chat` with live token rendering
- Gate behind `fetchAiStatus()` — hide button when AI is disabled
- Context-aware quick-question chips that adapt to plant state (overdue, no species, etc.)

**Non-Goals:**
- Photo upload in chat (Phase 7)
- Markdown/rich-text rendering of AI responses (Phase 7)
- "Save to journal" flow (Phase 7)
- Persistent chat history across sessions — history is session-scoped

## Decisions

### 1. Single `ChatDrawer.svelte` component — not two separate components

Use one component with CSS media queries to render as a right-side drawer (desktop) or a bottom sheet (mobile). Both share identical chat logic, message list, input area, and quick chips.

**Why over two components:** Avoids duplicating chat state management, message rendering, and streaming logic. The only difference is the container shell (slide-from-right vs slide-from-bottom), which CSS handles cleanly.

### 2. Native `<dialog>` element — not a custom overlay

Follow the existing pattern from `ModalDialog.svelte` and `PhotoLightbox.svelte`. Use `<dialog>` with `showModal()` for the bottom sheet on mobile (gets free backdrop, focus trap, Escape-to-close). On desktop, use a non-modal panel (no `<dialog>`) that slides in as a sibling flex element alongside the main content area.

**Why:** Desktop drawer should NOT block interaction with the page behind it (user might want to scroll plant details for reference). Mobile bottom sheet SHOULD be modal (overlay + focus trap) because the small screen can't meaningfully show both.

**Alternative considered:** Using `<dialog>` for both — rejected because desktop drawer shouldn't trap focus or show a backdrop.

### 3. SSE consumption via `fetch` + `ReadableStream` — not `EventSource`

`EventSource` only supports GET requests. The chat endpoint is `POST` with a JSON body (plant_id, message, history). Use `fetch()` with `response.body.getReader()` and parse `data: {...}` lines manually.

```
async function* streamChat(plantId, message, history) {
  const resp = await fetch('/api/ai/chat', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ plant_id: plantId, message, history })
  });
  const reader = resp.body.getReader();
  const decoder = new TextDecoder();
  let buf = '';
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    buf += decoder.decode(value, { stream: true });
    const lines = buf.split('\n');
    buf = lines.pop();
    for (const line of lines) {
      if (line.startsWith('data: ')) {
        const data = JSON.parse(line.slice(6));
        if (data.done) return;
        if (data.error) throw new Error(data.error);
        if (data.delta) yield data.delta;
      }
    }
  }
}
```

**Why over EventSource:** POST support is mandatory. No polyfill or library needed.

### 4. Chat state as local component state — not a Svelte store

Chat messages, streaming state, and history live inside `ChatDrawer.svelte` using Svelte 5 `$state()` runes. No global store needed — chat is scoped to one plant detail page at a time.

**Why:** Simplest approach. The chat session is ephemeral (cleared on navigation). No other component needs to read chat state. If Phase 7 adds "save to journal," it can call the summarize API from within the component.

### 5. Quick chips logic — derive from plant data

Pass the plant object as a prop. Derive chips based on:
- `watering_status === 'overdue'` → prepend "Why is it overdue?" chip (danger-styled)
- `species === null` → show "Help identify" chip
- Default set: "Health check", "Watering advice", "When to repot?", "Light requirements"

Chips disappear after the first message is sent (replaced by the conversation).

### 6. Mobile action bar hiding — CSS class toggle

Add a `chatOpen` state variable in `+page.svelte`. When true, apply a `.chat-open` class to the `PageHeader` component (or its container) that sets `display: none` on the mobile action bar. The bottom sheet sits directly above the bottom nav (56px), maximizing chat space.

**Why over z-index layering:** Hiding is cleaner than stacking. The action bar serves no purpose during chat — the close button on the sheet header replaces "Back."

### 7. Desktop drawer width — fixed 400px

The drawer takes 400px on the right side. The main content area flexes to fill the remaining space. No resize handle (keep it simple).

**Why 400px:** Wide enough for readable message bubbles, narrow enough to leave useful plant detail visible on a 1280px+ screen. Matches common chat sidebar widths.

## Risks / Trade-offs

**[Risk] SSE parsing edge cases** → The manual `ReadableStream` parser could miss events split across chunks. Mitigation: buffer incomplete lines (the `buf = lines.pop()` pattern handles this). Add a timeout (30s) to detect stalled streams.

**[Risk] Mobile keyboard pushes content up** → When the chat input is focused on mobile, the virtual keyboard may resize the viewport. Mitigation: use `100dvh` (already used in the app shell) and `visualViewport` API if needed. Test on iOS Safari specifically.

**[Risk] Large conversation history grows request payload** → Each message is sent back in the `history` array. Mitigation: cap history at 20 messages (matching backend's care event context limit). Oldest messages are dropped from the front.

**[Risk] No abort on navigation** → If the user navigates away while streaming, the fetch continues in the background. Mitigation: use an `AbortController` tied to Svelte's `onDestroy` lifecycle to cancel in-flight requests.
