## MODIFIED Requirements

### Requirement: Photo Display on Detail View

The plant detail hero section SHALL display the plant's photo when available, using a thumbnail variant sized for the display context.

#### Scenario: Plant has photo

- **WHEN** a plant has a `photo_url`
- **THEN** the detail hero shows the 600px thumbnail (derived via `thumbUrl(photo_url, 600)`) instead of the original

#### Scenario: Plant has no photo

- **WHEN** a plant has no `photo_url`
- **THEN** the detail hero shows the Noto emoji icon as before

### Requirement: Detail image lightbox

The plant detail view SHALL allow opening a lightbox overlay for the hero photo when a photo is available. The lightbox SHALL display the full-resolution original image.

#### Scenario: Open lightbox from detail photo

- **WHEN** the user clicks or taps the hero photo on `/plants/[id]`
- **THEN** a lightbox overlay opens displaying the original `photo_url` at full resolution
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

### Requirement: Care event photo in plant detail timeline

Care events with a `photo_url` SHALL display a clickable thumbnail in the plant detail timeline, using a thumbnail variant sized for the display context.

#### Scenario: Event with photo

- **WHEN** a care event in the plant detail timeline has a `photo_url`
- **THEN** a 72px rounded thumbnail (`object-fit: cover`) SHALL be displayed using the 200px thumbnail (derived via `thumbUrl(photo_url, 200)`)
- **AND** the thumbnail SHALL float to the right of the text content, with text wrapping beside it on wider viewports
- **AND** clicking the thumbnail SHALL open the PhotoLightbox with the original `photo_url` at full resolution

#### Scenario: Event without photo

- **WHEN** a care event has no `photo_url`
- **THEN** no thumbnail space SHALL be rendered
