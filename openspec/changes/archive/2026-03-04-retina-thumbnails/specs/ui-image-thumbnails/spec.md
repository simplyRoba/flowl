## ADDED Requirements

### Requirement: Thumbnail srcset utility

The frontend SHALL provide a shared `thumbSrcset(photoUrl: string): string` utility that returns an `srcset` attribute value listing all three thumbnail sizes with `w` descriptors.

#### Scenario: Generate srcset string

- **WHEN** `thumbSrcset('/uploads/a1b2c3.jpg')` is called
- **THEN** the return value SHALL be `'/uploads/a1b2c3_200.jpg 200w, /uploads/a1b2c3_600.jpg 600w, /uploads/a1b2c3_1000.jpg 1000w'`

#### Scenario: Generate srcset string for PNG original

- **WHEN** `thumbSrcset('/uploads/d4e5f6.png')` is called
- **THEN** the return value SHALL be `'/uploads/d4e5f6_200.jpg 200w, /uploads/d4e5f6_600.jpg 600w, /uploads/d4e5f6_1000.jpg 1000w'`

### Requirement: Responsive thumbnail display

All `<img>` elements that display stored photo thumbnails SHALL use `srcset` with all three thumbnail sizes (200, 600, 1000) and a `sizes` attribute matching the element's CSS container width, allowing the browser to select the optimal image for the device pixel ratio.

#### Scenario: Attention card thumbnail

- **WHEN** a plant attention card displays a photo thumbnail in a 120px-wide container
- **THEN** the `<img>` SHALL include `srcset` with 200w, 600w, and 1000w variants
- **AND** the `sizes` attribute SHALL be `120px`

#### Scenario: Plant grid card thumbnail

- **WHEN** a plant grid card displays a photo thumbnail
- **THEN** the `<img>` SHALL include `srcset` with 200w, 600w, and 1000w variants
- **AND** the `sizes` attribute SHALL reflect the grid column width

#### Scenario: Plant detail hero photo

- **WHEN** the plant detail page displays the hero photo
- **THEN** the `<img>` SHALL include `srcset` with 200w, 600w, and 1000w variants
- **AND** the `sizes` attribute SHALL reflect the hero container width across breakpoints

#### Scenario: Timeline and journal photo thumbnails

- **WHEN** a care timeline or care journal entry displays a photo thumbnail in a small container (72–80px)
- **THEN** the `<img>` SHALL include `srcset` with 200w, 600w, and 1000w variants
- **AND** the `sizes` attribute SHALL match the container width

## MODIFIED Requirements

### Requirement: Thumbnail image fallback

All `<img>` elements that use a thumbnail URL SHALL fall back to the original `photo_url` if the thumbnail fails to load. The `src` attribute SHALL use the smallest thumbnail (200px) as the default/fallback source.

#### Scenario: Thumbnail loads successfully

- **WHEN** an `<img>` element uses a thumbnail URL and the file exists on the server
- **THEN** the thumbnail SHALL be displayed normally

#### Scenario: Thumbnail fails to load

- **WHEN** an `<img>` element uses a thumbnail URL and the server returns a 404 (e.g., thumbnail generation failed for a corrupt image)
- **THEN** the `onerror` handler SHALL replace the `src` with the original `photo_url`
- **AND** the original image SHALL be displayed as a graceful fallback
