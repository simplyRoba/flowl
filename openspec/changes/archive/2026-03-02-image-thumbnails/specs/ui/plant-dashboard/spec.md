## MODIFIED Requirements

### Requirement: Photo Display on Dashboard

The dashboard plant cards SHALL display the plant's photo when available, falling back to the emoji icon. Photo displays SHALL use thumbnail variants sized for the display context.

#### Scenario: Plant has photo on grid card

- **WHEN** a plant has a `photo_url`
- **THEN** the dashboard card shows the 600px thumbnail (derived via `thumbUrl(photo_url, 600)`) instead of the emoji icon

#### Scenario: Plant has photo on attention card

- **WHEN** a plant has a `photo_url` and appears in the "Needs Attention" section
- **THEN** the attention card shows the 200px thumbnail (derived via `thumbUrl(photo_url, 200)`)

#### Scenario: Plant has no photo

- **WHEN** a plant has no `photo_url`
- **THEN** the dashboard card shows the Noto emoji icon as before
