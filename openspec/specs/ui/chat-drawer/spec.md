## Purpose

Chat drawer UI component — conversational AI chat interface on the Plant Detail page with desktop drawer and mobile bottom sheet layouts, streaming response rendering, and context-aware quick-question chips.

## Requirements

### Requirement: Chat drawer component

A `ChatDrawer.svelte` component SHALL provide a conversational AI chat interface on the Plant Detail page. On desktop (>768px) it SHALL render as a 400px-wide right-side panel. On mobile (<=768px) it SHALL render as a bottom sheet with a drag handle.

#### Scenario: Desktop drawer open

- **WHEN** the chat drawer is opened on desktop (viewport > 768px)
- **THEN** a 400px panel SHALL slide in from the right
- **AND** the main content area SHALL flex to fill the remaining width
- **AND** the panel SHALL NOT block interaction with the page content behind it

#### Scenario: Mobile bottom sheet open

- **WHEN** the chat drawer is opened on mobile (viewport <= 768px)
- **THEN** a bottom sheet SHALL slide up from above the bottom nav bar (56px)
- **AND** a semi-transparent backdrop SHALL overlay the page content
- **AND** a drag handle bar SHALL be visible at the top of the sheet

#### Scenario: Close drawer

- **WHEN** the user clicks the close button (X) in the chat header
- **THEN** the drawer/sheet SHALL close with a slide-out animation
- **AND** the page layout SHALL return to normal

#### Scenario: Mobile drag to dismiss

- **WHEN** the user drags the bottom sheet downward past a threshold on mobile
- **THEN** the sheet SHALL dismiss

#### Scenario: Escape key closes on mobile

- **WHEN** the bottom sheet is open on mobile
- **AND** the user presses the Escape key
- **THEN** the sheet SHALL close

### Requirement: Chat message list

The chat drawer SHALL display a scrollable list of user and AI messages.

#### Scenario: User message rendered

- **WHEN** a user sends a message
- **THEN** the message SHALL appear right-aligned with the AI accent color background (`--color-ai`) and white text
- **AND** the message bubble SHALL have rounded corners with a flattened bottom-right corner

#### Scenario: AI message rendered

- **WHEN** the AI responds
- **THEN** the message SHALL appear left-aligned with the AI tint background (`--color-ai-tint`) and normal text color
- **AND** the message bubble SHALL have rounded corners with a flattened bottom-left corner

#### Scenario: Auto-scroll on new message

- **WHEN** a new message is added (user or AI)
- **THEN** the message list SHALL auto-scroll to the bottom

### Requirement: Chat text input

The chat drawer SHALL provide a text input and send button at the bottom.

#### Scenario: Send message

- **WHEN** the user types text in the input and clicks the send button (or presses Enter)
- **THEN** the message SHALL be added to the message list
- **AND** a streaming request SHALL be initiated to `POST /api/ai/chat`
- **AND** the input SHALL be cleared

#### Scenario: Empty input

- **WHEN** the input is empty
- **THEN** the send button SHALL be visually disabled
- **AND** clicking it or pressing Enter SHALL do nothing

#### Scenario: Input disabled during streaming

- **WHEN** an AI response is being streamed
- **THEN** the input and send button SHALL be disabled until the stream completes

### Requirement: Streaming response rendering

The chat drawer SHALL consume the SSE stream from `POST /api/ai/chat` and render tokens as they arrive.

#### Scenario: Typing indicator shown

- **WHEN** a chat request is sent and before the first token arrives
- **THEN** a typing indicator (three animated dots) SHALL be displayed in the AI message position

#### Scenario: Tokens rendered incrementally

- **WHEN** `{"delta": "..."}` events arrive from the SSE stream
- **THEN** the delta text SHALL be appended to the current AI message in real time
- **AND** the typing indicator SHALL be replaced by the accumulating text after the first token

#### Scenario: Stream completes

- **WHEN** a `{"done": true}` event is received
- **THEN** the AI message SHALL be finalized
- **AND** the input SHALL be re-enabled

#### Scenario: Stream error

- **WHEN** a `{"error": "..."}` event is received or the stream fails
- **THEN** an error message SHALL be displayed in the chat
- **AND** the input SHALL be re-enabled

#### Scenario: Abort on unmount

- **WHEN** the component is destroyed while a stream is in progress
- **THEN** the in-flight fetch SHALL be aborted via `AbortController`

### Requirement: Chat history

The chat drawer SHALL maintain conversation history for the current session.

#### Scenario: History sent with each request

- **WHEN** the user sends a new message
- **THEN** all previous messages (user + assistant, up to 20) SHALL be included in the `history` array of the request body

#### Scenario: History cap

- **WHEN** the conversation exceeds 20 messages
- **THEN** the oldest messages SHALL be dropped from the request history (FIFO)
- **AND** all messages SHALL still be visible in the UI

#### Scenario: History cleared on navigation

- **WHEN** the user navigates away from the Plant Detail page
- **THEN** the chat history SHALL be discarded

### Requirement: Quick-question chips

The chat drawer SHALL display context-aware quick-question chips when the chat is empty.

#### Scenario: Default chips

- **WHEN** the chat opens with no messages
- **AND** the plant has a species and is not overdue
- **THEN** the chips SHALL include: "Health check", "Watering advice", "When to repot?", "Light requirements"

#### Scenario: Overdue plant chip

- **WHEN** the plant's `watering_status` is `overdue`
- **THEN** a "Why is it overdue?" chip SHALL be prepended
- **AND** it SHALL use danger styling (`--color-danger` border and text)

#### Scenario: No species chip

- **WHEN** the plant's `species` is null
- **THEN** a "Help identify" chip SHALL replace "When to repot?"

#### Scenario: Chip click sends message

- **WHEN** the user clicks a quick-question chip
- **THEN** the chip text SHALL be sent as a user message
- **AND** the chips section SHALL be hidden (replaced by the conversation)

### Requirement: Empty state

The chat drawer SHALL display an empty state when no messages exist.

#### Scenario: Empty state content

- **WHEN** the chat drawer opens with no messages
- **THEN** a centered sparkle icon and text "Ask anything about your [plant name]'s care" SHALL be displayed below the quick chips

### Requirement: AI gating

The chat drawer SHALL only be available when the AI provider is enabled.

#### Scenario: AI disabled

- **WHEN** `GET /api/ai/status` returns `{ "enabled": false }`
- **THEN** the "Ask AI" button SHALL NOT be rendered on the Plant Detail page

#### Scenario: AI enabled

- **WHEN** `GET /api/ai/status` returns `{ "enabled": true }`
- **THEN** the "Ask AI" button SHALL be rendered in the Plant Detail hero section

### Requirement: Chat API client function

The frontend API client SHALL provide a `chatPlant` async generator function.

#### Scenario: Streaming chat call

- **WHEN** `chatPlant(plantId, message, history)` is called
- **THEN** a `POST` request SHALL be sent to `/api/ai/chat` with `{ plant_id, message, history }` as JSON
- **AND** the function SHALL yield string deltas as they arrive from the SSE stream
- **AND** the function SHALL return when a `{"done": true}` event is received

#### Scenario: Chat API error

- **WHEN** the API returns a non-200 status
- **THEN** the function SHALL throw an error with the message from the response
