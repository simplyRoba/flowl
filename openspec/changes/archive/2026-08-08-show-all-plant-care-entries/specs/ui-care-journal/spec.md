## MODIFIED Requirements

### Requirement: Care Journal Timeline

The plant detail view SHALL display a care journal section showing a chronological timeline of all fetched care events.

#### Scenario: Care events displayed

- **WHEN** the plant detail view is rendered
- **AND** the plant has care events
- **THEN** a "Care Journal" section is shown below the watering card
- **AND** all fetched care events are displayed immediately
- **AND** care events are listed newest first
- **AND** each event shows an icon for the event type, the type label, the date, and notes (if present)

#### Scenario: No care events

- **WHEN** the plant detail view is rendered
- **AND** the plant has no care events
- **THEN** the care journal section shows an empty state message

#### Scenario: Event type icons

- **WHEN** a care event is displayed
- **THEN** the icon corresponds to the event type: droplet for `watered`, leaf for `fertilized`, shovel for `repotted`, scissors for `pruned`, pencil for `custom`, sparkles for `ai-consultation`

#### Scenario: Event limit

- **WHEN** the plant has more than 20 care events
- **THEN** all fetched care events are shown immediately
- **AND** no "Show more" control is displayed
