## Why

Plant photos are a primary visual reference on the detail page, but the current fixed size limits inspection of leaf texture and health cues. Providing a zoomable view improves usability without changing data or workflows.

## What Changes

- Add a lightbox-style zoom interaction for the plant detail image (click/tap to open, close via ESC or click outside).
- Support zoom and pan within the overlay for larger inspection, with mobile-friendly gestures.
- Keep the existing layout intact; no API or data model changes.

## Capabilities

### New Capabilities
- None.

### Modified Capabilities
- `ui/plants`: Add requirements for a zoomable plant detail image via a lightbox overlay.

## Impact

- UI: `ui/src/routes/plants/[id]/+page.svelte` and related styling.
- Possible new shared UI component for lightbox/zoom behavior.
- UI tests to cover open/close and basic interaction states.
