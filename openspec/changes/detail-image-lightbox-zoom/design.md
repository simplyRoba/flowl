## Context

The plant detail page presents a fixed-size image in the hero section. Users want to inspect image details (leaf texture, damage, pests) without leaving the page. The change is UI-only; no API or data model adjustments are required. The UI is Svelte-based and should remain lightweight with minimal dependencies.

## Goals / Non-Goals

**Goals:**
- Provide a lightbox-style overlay for the detail image with zoom and pan controls.
- Support keyboard (ESC) and pointer/touch interactions for open/close and zooming.
- Keep the existing layout intact and avoid backend changes.

**Non-Goals:**
- Implement a gallery or multi-image carousel.
- Add zoom for form preview or list thumbnails.
- Introduce third-party zoom libraries.

## Decisions

- Implement a custom lightbox component in the UI layer (Svelte) to avoid adding dependencies and keep the Docker image small.
  - **Alternative:** Use an off-the-shelf lightbox/zoom library. **Rejected** due to additional bundle size and dependency maintenance.
- Use a full-viewport overlay with a centered image and CSS transforms for scale and translation.
  - **Alternative:** Inline expansion within the page. **Rejected** because it causes reflow and pushes content, especially on mobile.
- Provide input handling for:
  - Click/tap on the detail image to open the lightbox.
  - ESC and backdrop click to close.
  - Wheel/pinch to adjust scale, drag to pan when zoomed.

## Risks / Trade-offs

- Complex gesture handling across devices → Mitigation: start with minimal, predictable gestures (wheel zoom + drag pan) and add touch pinch only if needed after baseline works.
- Large images can be memory-heavy → Mitigation: cap maximum zoom (e.g., 3x) and constrain translation to the image bounds.
- Scroll conflicts when overlay is open → Mitigation: lock body scroll while lightbox is active.

## Migration Plan

No migration required; UI-only change. Deploy with standard frontend build and smoke test the detail page.

## Open Questions

- Do we need a reusable lightbox component for future screens, or keep it local to the plant detail page for now?
