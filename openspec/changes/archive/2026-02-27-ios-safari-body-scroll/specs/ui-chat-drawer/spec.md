## MODIFIED Requirements

### Requirement: Chat drawer component

A `ChatDrawer.svelte` component SHALL provide a conversational AI chat interface on the Plant Detail page. On desktop (>768px) it SHALL render as a 400px-wide right-side panel using `position: fixed`. On mobile (<=768px) it SHALL render as a bottom sheet with a drag handle.

#### Scenario: Desktop drawer open

- **WHEN** the chat drawer is opened on desktop (viewport > 768px)
- **THEN** a 400px panel SHALL be `position: fixed` anchored to the right edge, spanning the full viewport height
- **AND** the panel SHALL have `z-index: 90`
- **AND** the panel SHALL overlay the page content without shifting it

#### Scenario: Mobile bottom sheet open

- **WHEN** the chat drawer is opened on mobile (viewport <= 768px)
- **THEN** a bottom sheet SHALL slide up covering the full viewport width, from `bottom: 0` to `top: 60px`
- **AND** the sheet SHALL overlay the bottom nav bar
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
