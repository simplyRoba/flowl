## Context

The backend (Phase 9a) already supports care event photos: `photo_path` column, upload/delete endpoints, `photo_url` in API responses. The frontend has no awareness of this field — the TypeScript `CareEvent` interface lacks `photo_url`, there are no API client functions for photo upload/delete, and neither timeline view renders photos. The chat drawer's save-note flow creates `ai-consultation` events with text only, ignoring any photos shared during the conversation.

The existing plant photo upload follows a two-step pattern: create the entity first, then upload the photo as a separate multipart request. Care event photos will follow the same pattern.

## Goals / Non-Goals

**Goals:**
- Let users attach a photo when logging a care event (fertilized, repotted, pruned, custom)
- Display care event photos in both timeline views (plant detail + global journal)
- Side-by-side photo layout on desktop, stacked on mobile
- Auto-attach the last user-sent chat photo when saving an AI consultation note
- Reuse existing PhotoLightbox for full-size viewing

**Non-Goals:**
- Multiple photos per care event (single photo only, matching plant photo pattern)
- Photo editing/cropping
- Photo management after event creation (no replace/delete UI — can be added later)
- Photos on the "Water now" quick action (it bypasses the log form entirely)

## Decisions

### 1. Two-step create-then-upload flow

Follow the established plant photo pattern: `createCareEvent()` returns the event with an `id`, then `uploadCareEventPhoto(plantId, eventId, file)` uploads the photo. This keeps the create endpoint as JSON and the photo upload as multipart, consistent with the rest of the API.

**Alternative considered:** Multipart create endpoint that accepts both JSON fields and a file. Rejected because it would require a new backend endpoint and diverges from the existing pattern.

### 2. Compact photo attachment in log form

Add a small camera-icon label (hidden file input) below the notes textarea, inline with the form. When a photo is selected, show a ~64px thumbnail preview with a remove button. This matches the ChatDrawer's attachment pattern — lightweight and optional.

The photo `File` object is held in component state (`logPhoto`) alongside `logNotes` and `logEventType`. On submit, `handleLogSubmit()` first creates the event, then uploads the photo if one is staged.

### 3. Side-by-side thumbnail layout

**Plant detail timeline:** The thumbnail uses `float: right` within the text content area, so text flows naturally beside the photo on wider viewports and wraps beneath it on narrow ones. This avoids extra flex containers while achieving the side-by-side visual.

**Global care journal:** Same `float: right` approach within the content column. Time stays pinned top-right in the entry's top row, unaffected by the float.

Thumbnail sizes: 72px in plant detail timeline, 80px in global journal (larger than originally planned — better visual weight at both densities). Both use `border-radius: 8px` and `object-fit: cover`.

### 4. PhotoLightbox reuse

The existing `PhotoLightbox` component takes `open`, `src`, `alt`, and `onclose` props. The plant detail page already has one instance for the hero photo. For care event photos, use the same instance — clicking a care event thumbnail sets `lightboxSrc` to the event's `photo_url` and opens the lightbox.

The global care journal page doesn't have a PhotoLightbox yet — add one instance there with the same pattern.

### 5. ChatDrawer save-note photo attachment

Track the last user-sent photo across the conversation in a `lastUserPhoto: File | null` state variable. When the user sends a message with a photo, store the `File` object (not just the base64 — we need the original file for the upload endpoint). When `handleConfirmSave()` runs:

1. Create the care event (existing flow)
2. If `lastUserPhoto` exists, call `uploadCareEventPhoto(plant.id, event.id, lastUserPhoto)`
3. Clear the photo state

The summary editor UI shows a small preview of the attached photo (if any) with a remove button, so the user can opt out of attaching it.

### 6. API client additions

Add to `api.ts`:
- `photo_url: string | null` field on `CareEvent` interface
- `uploadCareEventPhoto(plantId: number, eventId: number, file: File): Promise<CareEvent>` — `POST /api/plants/{plantId}/care/{eventId}/photo` with FormData
- `deleteCareEventPhoto(plantId: number, eventId: number): Promise<void>` — `DELETE /api/plants/{plantId}/care/{eventId}/photo`

The store's `addCareEvent` doesn't need to change — the photo upload is called separately after event creation at the component level (same as plant photo upload).

## Risks / Trade-offs

**Two requests for one action** — Creating an event with a photo requires two sequential API calls (create + upload). If the upload fails, the event exists without a photo. → Acceptable trade-off: the event is still valid without a photo, and this matches the plant photo pattern users already experience.

**No photo replace/delete UI** — Users can't change or remove a care event photo after creation. → Keeps scope focused. The delete endpoint exists in the backend, so a future UI addition is straightforward.

**Chat photo memory** — Holding `lastUserPhoto` as a `File` reference means the file stays in memory for the duration of the chat session. → Negligible impact since it's a single file reference and chat sessions are short-lived.

**Lightbox instance sharing** — Plant detail page shares one PhotoLightbox between hero photo and care event photos. Only one can be open at a time, which is the expected behavior. → No risk.
