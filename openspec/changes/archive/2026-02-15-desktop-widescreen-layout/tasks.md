## 1. Layout — Widescreen Sidebar

- [x] 1.1 Add `@media (min-width: 1280px)` rules to `+layout.svelte`: expand sidebar to 200px, show text labels ("Plants", "Log", "Settings") next to icons, display "flowl" brand name next to logo icon
- [x] 1.2 Increase content area padding to 32px at the widescreen breakpoint

## 2. Dashboard — Widescreen Adjustments

- [x] 2.1 Increase dashboard `max-width` from 1200px to 1400px at `>= 1280px` in `+page.svelte`
- [x] 2.2 Increase plant card photo area height from 120px to 140px at `>= 1280px`

## 3. Plant Detail — Widescreen Adjustments

- [x] 3.1 Increase detail page `max-width` from 800px to 960px at `>= 1280px` in `plants/[id]/+page.svelte`
- [x] 3.2 Increase hero photo/icon size from 80px to 100px at `>= 1280px`

## 4. Mockups

- [x] 4.1 Add widescreen CSS styles to `mockups/index.html`: `.device-frame.widescreen` (1100px width), `.app-widescreen` layout with expanded 200px sidebar with icon + label nav items
- [x] 4.2 Add widescreen device frame for Dashboard screen
- [x] 4.3 Add widescreen device frame for Plant Detail screen
- [x] 4.4 Add widescreen device frame for Add Plant screen
- [x] 4.5 Add widescreen device frame for Care Log screen
- [x] 4.6 Add widescreen device frame for Settings screen

## 5. Documentation

- [x] 5.1 Update DESIGN.md to document the widescreen breakpoint (1280px) alongside the existing mobile breakpoint (768px)
