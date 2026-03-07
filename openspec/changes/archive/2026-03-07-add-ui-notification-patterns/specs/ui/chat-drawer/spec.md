## MODIFIED Requirements

### Requirement: Save note flow

The chat drawer SHALL provide a save-note flow that summarizes the conversation and saves it as a care journal entry.

#### Scenario: Save confirmed success feedback

- **WHEN** the user clicks "Save" on the summary editor and the care event is created successfully
- **THEN** the drawer SHALL close
- **AND** a global toast notification is displayed acknowledging that the note was saved
