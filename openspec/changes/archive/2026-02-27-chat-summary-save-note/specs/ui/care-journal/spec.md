## MODIFIED Requirements

### Requirement: Care Journal Timeline

The plant detail view SHALL display a care journal section showing a chronological timeline of care events.

#### Scenario: Care events displayed

- **WHEN** the plant detail view is rendered
- **AND** the plant has care events
- **THEN** a "Care Journal" section is shown below the watering card
- **AND** care events are grouped by day (e.g., "Today", "Yesterday", "Feb 10") and listed newest first within each group
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
- **THEN** only the 20 most recent events are shown initially
- **AND** a "Show more" link is displayed to load the rest

### Requirement: Global Care Log Page

The route `/care-journal` SHALL display a paginated feed of care events across all plants.

#### Scenario: Events displayed

- **WHEN** the user navigates to `/care-journal`
- **THEN** the page fetches care events from `GET /api/care`
- **AND** displays events grouped by day (e.g., "Today", "Yesterday", "Feb 11, 2026")
- **AND** each event shows the plant name, event type icon, type label, time, and notes (if present)

#### Scenario: Filter by event type

- **WHEN** the user clicks a filter chip (All, Watered, Fertilized, Repotted, Pruned, Custom, AI Consultation)
- **THEN** the event list reloads showing only events of the selected type
- **AND** the "All" chip shows all event types

#### Scenario: Infinite scroll

- **WHEN** the user scrolls near the bottom of the event list
- **AND** more events are available
- **THEN** the next page is fetched automatically using the cursor (`before` parameter)
- **AND** new events are appended to the list

#### Scenario: No events

- **WHEN** no care events exist across any plant (or for the selected filter)
- **THEN** the page displays an empty state message

#### Scenario: Navigate to plant

- **WHEN** the user clicks a plant name in the global log
- **THEN** the app navigates to that plant's detail view

## ADDED Requirements

### Requirement: AI consultation event styling

The `ai-consultation` event type SHALL have distinct visual treatment in both the plant detail timeline and global care journal.

#### Scenario: AI consultation icon

- **WHEN** an `ai-consultation` care event is displayed in any timeline
- **THEN** the event icon SHALL be `Sparkles` (from lucide-svelte)

#### Scenario: AI consultation color

- **WHEN** an `ai-consultation` care event is displayed in the global care journal
- **THEN** the icon background SHALL use `var(--color-ai)` as its accent color

#### Scenario: AI consultation label

- **WHEN** an `ai-consultation` care event is displayed
- **THEN** the event type label SHALL be "AI Consultation"
