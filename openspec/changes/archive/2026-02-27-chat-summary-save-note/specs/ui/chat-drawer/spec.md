## ADDED Requirements

### Requirement: Save note button

The chat drawer SHALL display a "Save note" icon button in the header that allows saving the conversation as a care journal entry.

#### Scenario: Button visible after assistant response

- **WHEN** the chat contains at least one assistant message
- **AND** streaming is not in progress
- **AND** the summary editor is not open
- **THEN** a "Save note" icon button (`BookOpen`) SHALL be visible in the chat header, left of the close button
- **AND** the button SHALL have a native tooltip (`title`) with the save note label

#### Scenario: Button hidden when no assistant messages

- **WHEN** the chat contains no messages or only user messages
- **THEN** the "Save note" button SHALL NOT be visible

#### Scenario: Button hidden during streaming

- **WHEN** an AI response is being streamed
- **THEN** the "Save note" button SHALL NOT be visible

### Requirement: Summarize API client function

The frontend API client SHALL provide a `summarizeChat` function.

#### Scenario: Summarize call

- **WHEN** `summarizeChat(plantId, history)` is called
- **THEN** a `POST` request SHALL be sent to `/api/ai/summarize` with `{ plant_id, history }` as JSON
- **AND** the function SHALL return the `summary` string from the response

#### Scenario: Summarize API error

- **WHEN** the API returns a non-200 status
- **THEN** the function SHALL throw an error with the message from the response

### Requirement: Save note flow

The chat drawer SHALL provide a save-note flow that summarizes the conversation and saves it as a care journal entry.

#### Scenario: Summarize initiated

- **WHEN** the user clicks the "Save note" icon button
- **THEN** the icon SHALL change to a spinner to indicate loading
- **AND** a `POST /api/ai/summarize` request SHALL be sent with the current plant ID and chat history

#### Scenario: Summary editing

- **WHEN** the summarize request succeeds
- **THEN** the input area SHALL be replaced with an editable textarea pre-filled with the AI-generated summary
- **AND** a "Save" button and a "Cancel" button SHALL be displayed

#### Scenario: Save confirmed

- **WHEN** the user clicks "Save" on the summary editor
- **THEN** a `POST /api/plants/:id/care` request SHALL be sent with `event_type: "ai-consultation"` and the textarea content as `notes`
- **AND** a success message SHALL be shown inside the chat messages area
- **AND** the input area SHALL return to its normal state

#### Scenario: Save cancelled

- **WHEN** the user clicks "Cancel" on the summary editor
- **THEN** the summary editor SHALL be dismissed
- **AND** the input area SHALL return to its normal state
- **AND** no care event SHALL be created

#### Scenario: Summarize error

- **WHEN** the summarize request fails
- **THEN** an error message SHALL be displayed inside the chat messages area
- **AND** the button SHALL return to its normal state

#### Scenario: Save error

- **WHEN** the save request fails
- **THEN** an error message SHALL be displayed inside the chat messages area
- **AND** the summary editor SHALL remain open

#### Scenario: Status message cleared on new chat message

- **WHEN** a success or error status message is displayed
- **AND** the user sends a new chat message
- **THEN** the status message SHALL be cleared
