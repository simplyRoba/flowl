## MODIFIED Requirements

### Requirement: Care event photo in global care journal

Care events with a `photo_url` SHALL display a clickable thumbnail in the global care journal page, using a thumbnail variant sized for the display context.

#### Scenario: Event with photo

- **WHEN** a care event in the global journal has a `photo_url`
- **THEN** a 80px rounded thumbnail (`object-fit: cover`) SHALL be displayed using the 200px thumbnail (derived via `thumbUrl(photo_url, 200)`)
- **AND** the thumbnail SHALL float to the right of the text content, with the time remaining pinned to the top-right of the entry
- **AND** clicking the thumbnail SHALL open a PhotoLightbox with the original `photo_url` at full resolution

#### Scenario: Event without photo

- **WHEN** a care event in the global journal has no `photo_url`
- **THEN** no thumbnail space SHALL be rendered
