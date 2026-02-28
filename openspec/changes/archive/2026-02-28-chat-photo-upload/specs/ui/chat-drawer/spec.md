## ADDED Requirements

### Requirement: Photo attachment button

The chat drawer SHALL display a photo attach icon button to the left of the text input.

#### Scenario: Attach button rendered

- **WHEN** the chat input area is visible (not in summary editor mode)
- **THEN** a `Camera` (or `Image`) icon button SHALL be rendered to the left of the text input
- **AND** the button SHALL have a native tooltip (`title`) with the attach photo label
- **AND** clicking it SHALL open a file picker accepting `image/jpeg`, `image/png`, `image/webp`

#### Scenario: Attach button disabled during streaming

- **WHEN** an AI response is being streamed
- **THEN** the attach button SHALL be disabled

#### Scenario: Photo selected via file picker

- **WHEN** the user selects a file from the file picker
- **THEN** the file SHALL be staged as the attached photo
- **AND** a preview strip SHALL appear above the input area

#### Scenario: Only one photo at a time

- **WHEN** a photo is already staged and the user selects another
- **THEN** the previous photo SHALL be replaced by the new one
- **AND** the previous preview object URL SHALL be revoked

### Requirement: Photo drag-and-drop

The chat drawer SHALL accept drag-and-drop photo attachment on the message list area.

#### Scenario: Drag enter shows overlay

- **WHEN** the user drags a file over the chat message list area
- **THEN** a visual drag indicator (border or overlay) SHALL appear on the message list

#### Scenario: Drag leave hides overlay

- **WHEN** the user drags a file out of the chat message list area
- **THEN** the drag indicator SHALL be removed

#### Scenario: Drop stages photo

- **WHEN** the user drops an image file (`image/jpeg`, `image/png`, `image/webp`) on the message list area
- **THEN** the file SHALL be staged as the attached photo
- **AND** the preview strip SHALL appear above the input area
- **AND** the drag indicator SHALL be removed

#### Scenario: Non-image drop ignored

- **WHEN** the user drops a non-image file on the message list area
- **THEN** the drop SHALL be ignored and no photo SHALL be staged

### Requirement: Photo preview strip

The chat drawer SHALL display a preview strip above the input area when a photo is staged.

#### Scenario: Preview shown

- **WHEN** a photo is staged (via file picker or drag-and-drop)
- **THEN** a thumbnail (~48px) of the photo SHALL be displayed in a strip between the message list and the input row
- **AND** a remove button (X icon) SHALL be displayed on the thumbnail corner

#### Scenario: Remove staged photo

- **WHEN** the user clicks the remove button on the preview strip
- **THEN** the staged photo SHALL be cleared
- **AND** the preview strip SHALL be hidden
- **AND** the preview object URL SHALL be revoked

#### Scenario: Preview cleared after send

- **WHEN** the user sends a message with a staged photo
- **THEN** the preview strip SHALL be hidden
- **AND** the staged photo state SHALL be cleared

### Requirement: Photo in message bubbles

The chat drawer SHALL display attached photos inline in user message bubbles.

#### Scenario: User message with photo

- **WHEN** a user message has an associated image
- **THEN** the image SHALL be rendered as a rounded thumbnail (max-width ~200px) above the message text inside the user bubble

#### Scenario: User message without photo

- **WHEN** a user message has no associated image
- **THEN** the message bubble SHALL render text only (no change from existing behavior)

### Requirement: Photo memory cleanup

The chat drawer SHALL revoke object URLs for photo previews to prevent memory leaks.

#### Scenario: Cleanup on photo replace

- **WHEN** a new photo replaces a previously staged photo
- **THEN** `URL.revokeObjectURL()` SHALL be called on the previous preview URL

#### Scenario: Cleanup on component destroy

- **WHEN** the chat drawer component is destroyed while a photo is staged
- **THEN** `URL.revokeObjectURL()` SHALL be called on the staged preview URL

## MODIFIED Requirements

### Requirement: Chat text input

The chat drawer SHALL provide a photo attach button, text input, and send button at the bottom.

#### Scenario: Send message

- **WHEN** the user types text in the input and clicks the send button (or presses Enter)
- **THEN** the message SHALL be added to the message list
- **AND** if a photo is staged, it SHALL be converted to base64 and included in the request
- **AND** if a photo is staged, the image data URL SHALL be stored on the message for display
- **AND** a streaming request SHALL be initiated to `POST /api/ai/chat`
- **AND** the input SHALL be cleared
- **AND** the staged photo SHALL be cleared

#### Scenario: Empty input

- **WHEN** the input text is empty (regardless of whether a photo is staged)
- **THEN** the send button SHALL be visually disabled
- **AND** clicking it or pressing Enter SHALL do nothing

#### Scenario: Input disabled during streaming

- **WHEN** an AI response is being streamed
- **THEN** the input, send button, and attach button SHALL be disabled until the stream completes

### Requirement: Chat history

The chat drawer SHALL maintain conversation history for the current session.

#### Scenario: History sent with each request

- **WHEN** the user sends a new message
- **THEN** all previous messages (user + assistant, up to 20) SHALL be included in the `history` array of the request body
- **AND** the `history` array SHALL contain only `role` and `content` fields (no image data)

#### Scenario: History cap

- **WHEN** the conversation exceeds 20 messages
- **THEN** the oldest messages SHALL be dropped from the request history (FIFO)
- **AND** all messages SHALL still be visible in the UI

#### Scenario: History cleared on navigation

- **WHEN** the user navigates away from the Plant Detail page
- **THEN** the chat history SHALL be discarded

### Requirement: Chat API client function

The frontend API client SHALL provide a `chatPlant` async generator function.

#### Scenario: Streaming chat call

- **WHEN** `chatPlant(plantId, message, history, signal, image)` is called
- **THEN** a `POST` request SHALL be sent to `/api/ai/chat` with `{ plant_id, message, history, image }` as JSON
- **AND** the `image` field SHALL be omitted from the JSON when no image is provided
- **AND** the function SHALL yield string deltas as they arrive from the SSE stream
- **AND** the function SHALL return when a `{"done": true}` event is received

#### Scenario: Chat API error

- **WHEN** the API returns a non-200 status
- **THEN** the function SHALL throw an error with the message from the response
