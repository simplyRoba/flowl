## 1. API Client

- [x] 1.1 Add optional `image?: string` field to `ChatMessage` interface in `api.ts`
- [x] 1.2 Add `image` parameter to `chatPlant()` function signature and include it in the JSON body (omit when undefined)
- [x] 1.3 Strip image data from history entries before sending (only `role` and `content`)

## 2. Photo State & Handlers

- [x] 2.1 Add `attachedPhoto`, `attachedPreview`, and `isDragging` state variables to `ChatDrawer.svelte`
- [x] 2.2 Implement `stagePhoto(file)` helper — validates image type, creates object URL preview, revokes previous if replacing
- [x] 2.3 Implement `clearPhoto()` helper — revokes object URL, resets state
- [x] 2.4 Implement File-to-base64 conversion helper using `FileReader.readAsDataURL()` (returns raw base64 string with prefix stripped)
- [x] 2.5 Add `onDestroy` cleanup to revoke preview URL on component unmount

## 3. Photo Attach Button

- [x] 3.1 Add Camera/Image icon button to the left of the text input in the input area
- [x] 3.2 Add hidden file input (`accept="image/jpeg,image/png,image/webp"`) triggered by the button
- [x] 3.3 Disable attach button during streaming
- [x] 3.4 Wire `onchange` to `stagePhoto()`

## 4. Drag-and-Drop

- [x] 4.1 Add `ondragenter`, `ondragover`, `ondragleave`, `ondrop` handlers to the message list container
- [x] 4.2 Add `class:dragging` visual indicator on the message list when dragging
- [x] 4.3 Filter drops to image types only — ignore non-image files
- [x] 4.4 Wire drop handler to `stagePhoto()`

## 5. Preview Strip

- [x] 5.1 Add preview strip markup between message list and input area (48px thumbnail + X remove button)
- [x] 5.2 Show preview strip only when `attachedPreview` is set
- [x] 5.3 Wire remove button to `clearPhoto()`
- [x] 5.4 Style preview strip (thumbnail rounded, remove button positioned on corner)

## 6. Send Flow Integration

- [x] 6.1 Update `sendMessage()` to convert staged photo to base64 and pass to `chatPlant()`
- [x] 6.2 Store the image data URL on the user message object for bubble display
- [x] 6.3 Clear staged photo after sending
- [x] 6.4 Ensure send button remains gated on text input (photo alone does not enable send)

## 7. Message Bubble Photos

- [x] 7.1 Update user message bubble rendering to show image thumbnail above text when message has `image`
- [x] 7.2 Style bubble photo (max-width ~200px, rounded corners, margin below)

## 8. i18n

- [x] 8.1 Add i18n keys for photo button tooltip and remove-photo label (en, de, es)

## 9. Verification

- [x] 9.1 Run `cd ui && npm run check` to verify no type errors
- [x] 9.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`
