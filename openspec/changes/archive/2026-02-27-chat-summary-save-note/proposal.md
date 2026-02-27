## Why

The summarize backend endpoint (`POST /api/ai/summarize`) is fully implemented but has no frontend integration — users cannot save chat conversations as care journal entries. This is the key missing piece to close the loop between AI chat and the care journal.

## What Changes

- Add a new care event type `ai-consultation` with its own icon and color, accepted by the backend and displayed in both care journal views
- Add a "Save note" button to the chat drawer that triggers AI summarization
- Add a `summarizeChat` API client function connecting the frontend to the existing backend endpoint
- Add a save-note flow: AI generates summary → user reviews/edits in a text field → confirmed summary is saved as an `ai-consultation` care event
- Add i18n keys for the new event type and save-note UI (en/de/es)

## Capabilities

### New Capabilities

_None — all changes extend existing capabilities._

### Modified Capabilities

- `ui/chat-drawer`: Add "Save note" button, summarize flow with editable summary field
- `data/care-events`: Add `ai-consultation` to the set of valid event types
- `ui/care-journal`: Add icon, color, label, and filter chip for the `ai-consultation` event type

## Impact

- **Backend**: Add `ai-consultation` to `VALID_EVENT_TYPES` array in `src/api/care_events.rs`
- **Frontend API client**: Add `summarizeChat()` function in `ui/src/lib/api.ts`
- **Chat drawer component**: Add save-note button, summary editing UI
- **Care journal components**: Add icon/color/label mappings for the new event type in both plant detail timeline and global care journal page
- **i18n files**: New keys in en/de/es for event type label, save-note UI strings
