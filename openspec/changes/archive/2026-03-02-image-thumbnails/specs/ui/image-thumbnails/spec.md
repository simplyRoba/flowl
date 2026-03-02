## ADDED Requirements

### Requirement: Thumbnail URL utility

The frontend SHALL provide a shared `thumbUrl(photoUrl: string, size: number): string` utility that derives a thumbnail URL from a `photo_url` by inserting a size suffix before the file extension.

#### Scenario: Derive thumbnail URL

- **WHEN** `thumbUrl('/uploads/a1b2c3.jpg', 200)` is called
- **THEN** the return value SHALL be `'/uploads/a1b2c3_200.jpg'`

#### Scenario: Derive thumbnail URL for PNG original

- **WHEN** `thumbUrl('/uploads/d4e5f6.png', 600)` is called
- **THEN** the return value SHALL be `'/uploads/d4e5f6_600.jpg'`

### Requirement: Thumbnail image fallback

All `<img>` elements that use a thumbnail URL SHALL fall back to the original `photo_url` if the thumbnail fails to load.

#### Scenario: Thumbnail loads successfully

- **WHEN** an `<img>` element uses a thumbnail URL and the file exists on the server
- **THEN** the thumbnail SHALL be displayed normally

#### Scenario: Thumbnail fails to load

- **WHEN** an `<img>` element uses a thumbnail URL and the server returns a 404 (e.g., thumbnail generation failed for a corrupt image)
- **THEN** the `onerror` handler SHALL replace the `src` with the original `photo_url`
- **AND** the original image SHALL be displayed as a graceful fallback
