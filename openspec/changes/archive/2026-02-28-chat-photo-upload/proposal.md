## Why

The AI chat backend already accepts an optional base64-encoded image alongside text messages, but the frontend has no way to attach or send photos. Users need to share photos of plant problems (yellowing leaves, pests, rot) directly in chat for visual diagnostics — the core value proposition of a vision-model-powered plant care assistant.

## What Changes

- Add a photo attach button (file picker) to the chat input area
- Support drag-and-drop photo attachment on desktop (mirroring the PlantForm pattern)
- Show a removable thumbnail preview of the attached photo before sending
- Display attached photos inline in sent user message bubbles
- Pass the base64-encoded image to the existing `POST /api/ai/chat` endpoint via the `chatPlant()` API client function
- Track image data in chat history so re-sent context includes prior images

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `ui/chat-drawer`: Add photo attachment button, photo preview before send, photo display in message bubbles, and image parameter threading through send flow
- `ai/chat`: Update `chatPlant()` client function signature to accept an optional image, and include `image` field in the JSON request body

## Impact

- **Frontend components**: `ChatDrawer.svelte` — new photo button, preview strip, message bubble photo rendering
- **Frontend API client**: `api.ts` — `chatPlant()` gains an `image` parameter; `ChatMessage` interface gains an optional `image` field
- **Backend**: No changes needed — `ChatRequest.image`, provider trait `image: Option<&[u8]>`, and OpenAI provider image handling are all already implemented
- **i18n**: New keys for photo button tooltip, remove-photo label
