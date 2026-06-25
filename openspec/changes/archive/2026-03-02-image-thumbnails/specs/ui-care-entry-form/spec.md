## MODIFIED Requirements

### Requirement: Photo tool button

The toolbar-left group SHALL contain a photo tool button that morphs between inactive and active states.

#### Scenario: Photo button inactive

- **WHEN** no photo is attached
- **THEN** the toolbar SHALL show a ghost-style icon button with the Camera icon
- **AND** clicking it SHALL open the file picker (via hidden `<input type="file">`)
- **AND** accepted types SHALL be `image/jpeg, image/png, image/webp`

#### Scenario: Photo button active (compound group)

- **WHEN** a photo is attached
- **THEN** the camera icon button SHALL be replaced by a compound group: `[thumbnail | x]`
- **AND** the compound group SHALL have a shared border with `border-radius: var(--radius-btn)`
- **AND** the thumbnail SHALL show the selected image as `object-fit: cover` using a local object URL (not a server thumbnail, since the photo has not been uploaded yet)
- **AND** the dismiss button SHALL use XIcon at size 12
- **AND** the compound group SHALL NOT show a camera icon (no redundant icon)

#### Scenario: Dismissing a photo

- **WHEN** the user clicks the dismiss button on the photo compound group
- **THEN** the photo SHALL be cleared
- **AND** the compound group SHALL revert to the inactive camera icon button
- **AND** the preview object URL SHALL be revoked
