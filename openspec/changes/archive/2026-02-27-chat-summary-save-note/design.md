## Context

The AI chat drawer (`ChatDrawer.svelte`) supports streaming conversation with plant context, but conversations are ephemeral — there is no way to persist insights. The backend already has a fully implemented `POST /api/ai/summarize` endpoint that condenses a chat history into a 1–3 sentence care journal note. The frontend has no client function for it and no save-note UI.

Care events support five types (`watered`, `fertilized`, `repotted`, `pruned`, `custom`) with hardcoded validation in Rust. Each type has an icon, color, and label mapped in two UI locations: the plant detail timeline and the global care journal page.

## Goals / Non-Goals

**Goals:**

- Let users save a chat conversation as a care journal entry with one tap
- Add `ai-consultation` as a new care event type with distinct icon and color
- Keep the save-note flow simple: summarize → review/edit → save

**Non-Goals:**

- Markdown rendering for AI chat messages (decided against — system prompt instructs plain text and informal tone)
- Persisting full chat history in the database (only the summary is saved)
- Photo attachments in the saved care entry (Phase 9)

## Decisions

### 1. Save-note flow: inline in chat drawer

The "Save note" action is an icon button (`BookOpen`) in the chat header, next to the close button. It appears once at least one assistant message exists and streaming is not active. A native browser tooltip (`title`) provides discoverability. Tapping it:

1. Calls `POST /api/ai/summarize` with the current chat history and plant ID
2. Shows a spinner icon replacing the `BookOpen` icon in the header
3. Replaces the input area with an editable textarea pre-filled with the AI summary
4. User can edit, then confirm (save) or cancel
5. On confirm: `POST /api/plants/:id/care` with `event_type: "ai-consultation"` and the summary as `notes`
6. Success/error feedback shown as a status message inside the chat messages area, then restore normal input area

**Why header placement:** Avoids consuming vertical space below the input — critical on mobile where the chat drawer height is limited. The header row already exists and has room for an additional icon button.

**Why inline editing:** Avoids a modal or separate view. The summary editing naturally replaces the input area since the user won't be chatting while reviewing the summary. Cancel restores the input.

### 2. `ai-consultation` event type: extend existing validation

Add `"ai-consultation"` to the `VALID_EVENT_TYPES` array in `src/api/care_events.rs`. No migration needed — `event_type` is a free-text column. The validation is purely in application code.

**Icon:** `Sparkles` (from lucide) — matches the "Ask AI" button and chat header, reinforcing the AI association.

**Color:** `var(--color-ai)` — the existing AI accent color already used by the chat drawer.

### 3. Frontend API: thin `summarizeChat` function

Add `summarizeChat(plantId: number, history: ChatMessage[]): Promise<string>` to `api.ts`. It calls `POST /api/ai/summarize` and returns the `summary` string. Reuses the existing `ChatMessage` interface.

### 4. "Save note" button placement

Icon button in the chat header, left of the close button. Only visible when there is at least one assistant message (don't show if only user messages exist — nothing to summarize yet). Hidden during streaming and while the summary editor is open. Uses a native `title` attribute for tooltip.

## Risks / Trade-offs

**Summarize latency** — The AI summarize call adds a round-trip (~1–3s). Mitigated by a clear loading state (spinner icon in the header button).

**Summary quality** — The AI might produce poor summaries for very short conversations. Mitigated by letting the user edit before saving. The editable textarea makes this a review step, not a blind save.

**Chat drawer height on mobile** — Mitigated by placing the save-note trigger in the header (zero extra vertical space) and swapping the input area with the edit area (not stacking them).
