## ADDED Requirements

### Requirement: Detail image lightbox
The plant detail view SHALL allow opening a lightbox overlay for the hero photo when a photo is available.

#### Scenario: Open lightbox from detail photo
- **WHEN** the user clicks or taps the hero photo on `/plants/[id]`
- **THEN** a lightbox overlay opens displaying the same photo at an enlarged size
- **AND** the background content is visually dimmed
- **AND** page scrolling is disabled while the lightbox is open

#### Scenario: Close lightbox
- **WHEN** the user presses the Escape key
- **OR** clicks the overlay backdrop outside the image
- **THEN** the lightbox closes
- **AND** page scrolling is restored

#### Scenario: No photo available
- **WHEN** the plant has no `photo_url`
- **THEN** the hero displays the emoji icon as before
- **AND** no lightbox opens when the hero is clicked

### Requirement: Lightbox zoom and pan
The lightbox SHALL support zooming and panning of the photo for detailed inspection.

#### Scenario: Zoom in and out
- **WHEN** the user uses the mouse wheel or a pinch gesture on the lightbox image
- **THEN** the image scale increases or decreases within defined limits

#### Scenario: Pan a zoomed image
- **WHEN** the image is zoomed in and the user drags the image
- **THEN** the image pans within its bounds without exposing empty background beyond the overlay
