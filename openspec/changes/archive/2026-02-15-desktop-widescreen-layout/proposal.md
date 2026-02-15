## Why

On widescreen monitors (1280px+), the app layout looks sparse and underutilizes available space. The 64px icon-only sidebar feels disconnected, content is capped at narrow max-widths leaving large empty margins, and the plant grid has no upper column limit. There is also no widescreen mockup in `mockups/index.html` to guide desktop design decisions.

## What Changes

- Add a widescreen breakpoint (`>= 1280px`) to the layout
- Expand the sidebar to 200px with icon + text labels at the widescreen breakpoint
- Show the "flowl" brand name in the expanded sidebar
- Increase dashboard max-width from 1200px to 1400px on widescreen
- Increase plant card photo height from 120px to 140px on widescreen for better proportions
- Increase content area padding from 24px to 32px on widescreen
- Increase plant detail max-width from 800px to 960px on widescreen
- Add a "Widescreen" device frame (1100px) to the HTML mockups file alongside existing Desktop (720px) and Mobile (340px) frames for all screens

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `ui/shell`: Add widescreen breakpoint with expanded sidebar layout (icon + text labels, 200px width)
- `ui/plants`: Adjust dashboard grid and detail page layout for widescreen proportions

## Impact

- `ui/src/routes/+layout.svelte`: New `@media (min-width: 1280px)` rules for expanded sidebar
- `ui/src/routes/+page.svelte`: Widescreen grid and spacing adjustments
- `ui/src/routes/plants/[id]/+page.svelte`: Wider detail max-width
- `mockups/index.html`: New widescreen device frames for all 5 screens
- `DESIGN.md`: Document the new widescreen breakpoint
