## Context

The AI chat backend already supports images end-to-end: `ChatRequest.image: Option<String>` (base64), the `AiProvider::chat` trait method accepts `image: Option<&[u8]>`, and the OpenAI provider encodes it into the vision API format. The 30 MB body limit is already set on the route. The only gap is the frontend — `chatPlant()` doesn't pass an image, and `ChatDrawer.svelte` has no photo attachment UI.

The PlantForm component already implements drag-and-drop photo handling (`handlePhotoDrop`, `handleDragEnter/Leave`, `isDraggingPhoto`) and file picker selection — these patterns can be reused directly.

## Goals / Non-Goals

**Goals:**

- Let users attach a single photo to a chat message via file picker button or drag-and-drop
- Show a removable preview thumbnail before sending
- Display the photo inline in the sent message bubble
- Thread the base64-encoded image through the existing API

**Non-Goals:**

- Multiple photos per message (one is sufficient for chat; identify handles multi-photo)
- Persistent photo storage (chat photos are ephemeral, same as today's text messages)
- Photo-only messages (text is always required)
- Image compression or resizing (send as-is; the 30 MB limit is generous)
- Lightbox/full-size view of chat photos (can be added later)

## Decisions

### 1. Base64 in JSON body (not multipart)

Keep the existing `POST /api/ai/chat` JSON format with the `image` field as a base64 string. The backend already handles this. No need to switch to multipart — the 30 MB body limit accommodates typical phone photos (3-5 MB raw, ~4-7 MB base64).

**Alternative considered:** Multipart form-data (like `/api/ai/identify`). Rejected because the backend endpoint is already built and working with JSON+base64, and switching would require backend changes for no benefit.

### 2. File → base64 conversion via FileReader

Use `FileReader.readAsDataURL()` to convert the selected `File` to a base64 data URL, then strip the `data:image/...;base64,` prefix to get the raw base64 string for the API. The full data URL is kept for the `<img src>` preview.

### 3. Photo state: staged File + preview URL

Mirror the PlantForm pattern: track `attachedPhoto: File | null` and `attachedPreview: string | null` (object URL for the thumbnail). On send, convert to base64 and include in the request. Clear both after sending.

### 4. Drag-and-drop on the message list area

Apply drag-and-drop handlers to the chat message list container (the main content area), not just the input. This gives a large drop target on desktop. Show a visual overlay/border when dragging, matching the PlantForm's `class:dragging` pattern. Only accept single image files (`image/jpeg`, `image/png`, `image/webp`).

### 5. Photo button placement: left of input

Place the attach button to the left of the text input, as a small icon button matching the send button style. Layout: `[camera] [input] [send]`.

### 6. Preview strip: above the input area

When a photo is staged, show a small thumbnail (~48px) with an X remove button in a strip between the message list and the input row. This keeps the input area compact and the preview clearly associated with the next message to send.

### 7. Message bubble photo display

Store a `image` data URL on the local `ChatMessage` for display. Render it as a rounded thumbnail (max-width ~200px) above the text inside the user message bubble. The image is display-only — not sent back in history (history `image` field would be too large; only the current message's image is sent to the API per request).

### 8. History handling

The `ChatMessage` interface gains an optional `image?: string` field. This is used for **display only** in the UI — showing which messages had photos attached. The base64 image data is NOT included in the `history` array sent to the API (it would bloat every subsequent request). Only the current message's image is sent via the top-level `image` field.

## Risks / Trade-offs

**Large photos increase request size** → The 30 MB body limit is already set. Typical phone photos are well within this. No mitigation needed now; client-side resize could be added later if it becomes an issue.

**No image in history context** → The AI won't "remember" photos from earlier messages when responding to later ones. This is acceptable — each photo is analyzed in the message it's attached to, and the AI's text response captures the findings. Sending images in history would multiply request sizes quickly.

**Object URL memory leaks** → Must call `URL.revokeObjectURL()` when clearing the preview or when the component unmounts, matching the PlantForm cleanup pattern.
