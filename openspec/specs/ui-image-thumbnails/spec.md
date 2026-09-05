## Purpose

Responsive managed-image display and fallback behavior across the frontend.

## Requirements

### Requirement: Responsive rendition selection

For every stored-photo display, the frontend SHALL make all three canonical rendition URLs defined by `core-image-store` available for responsive selection. It SHALL enable selection according to the rendered container width and device pixel ratio while retaining the original `photo_url` as the fallback.

#### Scenario: Attention card thumbnail

- **WHEN** a plant attention card displays a photo in a compact container
- **THEN** all canonical renditions are available for selection according to that container's rendered width and device pixel ratio

#### Scenario: Plant grid card thumbnail

- **WHEN** a plant grid card displays a photo
- **THEN** all canonical renditions are available for selection according to the grid column's rendered width and device pixel ratio

#### Scenario: Plant detail hero photo

- **WHEN** the plant detail page displays a hero photo
- **THEN** all canonical renditions are available for selection according to the hero container's rendered width and device pixel ratio

#### Scenario: Timeline and journal photo thumbnails

- **WHEN** a care timeline or care journal entry displays a photo in a compact container
- **THEN** all canonical renditions are available for selection according to that container's rendered width and device pixel ratio

### Requirement: Rendition fallback

Stored-photo displays that use a canonical rendition SHALL gracefully fall back to the original `photo_url` when that rendition is unavailable.

#### Scenario: Rendition loads successfully

- **WHEN** a stored-photo display requests an available canonical rendition
- **THEN** the rendition is displayed

#### Scenario: Rendition is unavailable

- **WHEN** a stored-photo display requests a canonical rendition that is unavailable
- **THEN** the original `photo_url` is displayed as a graceful fallback
