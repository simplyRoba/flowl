## 1. API Client Foundation

- [x] 1.1 Add `photo_url: string | null` to `CareEvent` interface in `api.ts`
- [x] 1.2 Add `uploadCareEventPhoto(plantId, eventId, file)` function in `api.ts` (POST multipart to `/api/plants/{id}/care/{eventId}/photo`)
- [x] 1.3 Add `deleteCareEventPhoto(plantId, eventId)` function in `api.ts` (DELETE `/api/plants/{id}/care/{eventId}/photo`)

## 2. Photo Upload in Log Care Form

- [x] 2.1 Add `logPhoto` state and compact camera-icon upload control below the notes textarea in plant detail page
- [x] 2.2 Add ~64px thumbnail preview with remove button when a photo is staged
- [x] 2.3 Update `handleLogSubmit()` to upload photo after event creation (two-step flow)
- [x] 2.4 Clear staged photo on form cancel and after successful submit
- [x] 2.5 Add i18n keys for photo upload labels (en/de/es)

## 3. Photo Display in Plant Detail Timeline

- [x] 3.1 Render 48px rounded thumbnail in timeline items when `photo_url` is present
- [x] 3.2 Side-by-side layout on desktop (>768px), stacked below notes on mobile
- [x] 3.3 Wire thumbnail click to open existing PhotoLightbox (share instance with hero photo)

## 4. Photo Display in Global Care Journal

- [x] 4.1 Render 56px rounded thumbnail in log entries when `photo_url` is present
- [x] 4.2 Side-by-side layout on desktop with time pinned top-right, stacked on mobile
- [x] 4.3 Add PhotoLightbox instance to global care journal page and wire thumbnail clicks

## 5. Chat Drawer Save-Note Photo Attachment

- [x] 5.1 Track `lastUserPhoto: File | null` — store the File when user sends a message with a photo
- [x] 5.2 Show thumbnail preview with remove button in summary editor when `lastUserPhoto` exists
- [x] 5.3 Update `handleConfirmSave()` to upload photo after care event creation when attached
- [x] 5.4 Add i18n keys for attach-photo labels in summary editor (en/de/es)

## 6. Verify

- [x] 6.1 Run `cd ui && npm run check` and fix any type/lint errors
- [x] 6.2 Run `cargo fmt`, `cargo clippy`, and `cargo test` and fix any issues
