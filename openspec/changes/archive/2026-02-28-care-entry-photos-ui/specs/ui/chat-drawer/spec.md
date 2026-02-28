## ADDED Requirements

### Requirement: Save note photo attachment

The chat drawer's save-note flow SHALL auto-attach the last user-sent photo to the ai-consultation care entry.

#### Scenario: Summary editor shows chat photo

- **WHEN** the summary editor opens after a successful summarize request
- **AND** the user sent at least one photo during the conversation
- **THEN** a thumbnail preview of the last user-sent photo SHALL be displayed below the textarea
- **AND** a remove button SHALL allow opting out of attaching the photo

#### Scenario: Summary editor without chat photo

- **WHEN** the summary editor opens
- **AND** no photos were sent during the conversation
- **THEN** no photo preview section SHALL be displayed

#### Scenario: Save with photo attached

- **WHEN** the user confirms save with a photo attached
- **THEN** the care event SHALL be created first via `POST /api/plants/:id/care`
- **AND** the photo SHALL be uploaded via `POST /api/plants/:id/care/:event_id/photo`
- **AND** the `onsave` callback SHALL be invoked and the drawer SHALL close

#### Scenario: Save after removing photo

- **WHEN** the user removes the auto-attached photo and confirms save
- **THEN** the care event SHALL be created without a photo upload
