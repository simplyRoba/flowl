## Why

The backend already supports photo attachments on care events (upload, delete, photo_url in responses) but the frontend has no UI for it. Users should be able to attach photos when logging care actions (e.g. a fertilizer photo, repotting progress) and see those photos in the care journal timeline. Additionally, when saving an AI chat note, any photo shared during the conversation should carry over to the care entry.

## What Changes

- Add a compact photo upload control to the inline care log form on the plant detail page
- Display care event photo thumbnails in the plant detail timeline (side-by-side with text on desktop, stacked on mobile)
- Display care event photo thumbnails in the global care journal page (side-by-side with text on desktop, stacked on mobile)
- Add photo lightbox support for care event thumbnails (reuse existing PhotoLightbox)
- Update the `CareEvent` TypeScript interface to include `photo_url`
- Add API client functions for care event photo upload/delete
- Update the chat drawer's save-note flow to auto-attach the last user-sent photo to the ai-consultation care entry

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ui/care-journal`: Add photo upload to log form, photo thumbnail display in both timelines (plant detail + global), photo upload/delete API client functions, CareEvent type update
- `ui/chat-drawer`: Update save-note flow to attach the last chat photo (if any) to the ai-consultation care entry

## Impact

- **Frontend components**: `plants/[id]/+page.svelte` (log form + timeline), `care-journal/+page.svelte` (global timeline)
- **Frontend component**: `ChatDrawer.svelte` (save-note flow)
- **API client**: `api.ts` — add `CareEvent.photo_url`, `uploadCareEventPhoto()`, `deleteCareEventPhoto()` functions
- **Store**: `careEvents.ts` — wire photo upload after event creation
- **No backend changes** — all endpoints already exist (Phase 9a)
- **No new dependencies**
