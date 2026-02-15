## Context

The flowl UI currently has two layout modes: a 64px icon-only sidebar for desktop (>768px) and bottom tabs for mobile (<=768px). The "desktop" device frame in the mockups file is 720px wide, which is closer to a tablet viewport. On actual widescreen monitors (1280px+), the narrow sidebar wastes space, and content areas with max-width caps (1200px for dashboard, 800px for detail, 640px for forms) leave large empty margins.

The DESIGN.md already describes the sidebar as supporting both 64px (icon-only) and 220px (expanded) variants, but only the 64px version was implemented. This change realizes the expanded sidebar for widescreen viewports and adds matching mockups.

## Goals / Non-Goals

**Goals:**
- Add a widescreen breakpoint at `>= 1280px` with an expanded sidebar (200px, icons + labels)
- Adjust content max-widths, spacing, and card style for better widescreen proportions (full-bleed overlay cards with 240px image area, gradient overlay for name/location)
- Add widescreen device frames (1100px) to the mockups HTML for all 5 screens
- Update DESIGN.md to document the breakpoint

**Non-Goals:**
- Collapsible/toggleable sidebar (expansion is purely viewport-driven)
- New pages or features
- Dark mode changes (existing dark mode applies uniformly)
- Changing mobile or current desktop behavior

## Decisions

### 1. Breakpoint at 1280px
The widescreen breakpoint is set at 1280px. This is above the common 1024px tablet boundary and covers typical laptop displays (1366px, 1440px) and desktop monitors (1920px+). Below 1280px, the current 64px icon-only sidebar remains.

Alternative considered: 1440px — rejected because it would exclude many laptop displays that would benefit from the expanded layout.

### 2. Sidebar width of 200px
200px provides a compact feel that leaves more room for content while still fitting icon + label navigation comfortably. DESIGN.md has been updated to reflect 200px as the canonical expanded width.

### 3. CSS-only implementation
The expansion is implemented purely via CSS `@media` queries. No JavaScript state management or user preference for sidebar mode. This keeps the implementation simple and consistent with the existing mobile breakpoint approach.

### 4. Full-bleed overlay plant cards on widescreen
On widescreen, plant cards switch from the standard image-top / info-bottom layout to full-bleed image cards (240px tall) with the name and location floating over a bottom gradient overlay. This makes better use of the available space and gives the dashboard a photo-gallery feel instead of looking empty. The fallback emoji icon is scaled to 80px (from 56px) to fill the taller card area.

### 5. Mockup widescreen frame at 1100px
The widescreen device frame in mockups is 1100px wide (compared to 720px desktop and 340px mobile). This is large enough to demonstrate the expanded sidebar and wider content but not so large it breaks the mockup page layout.

## Risks / Trade-offs

- [Transition gap] The sidebar jumps from 64px to 200px at the breakpoint. There is no animated transition. This is acceptable since viewport resizing is not a frequent user action. → No mitigation needed.
- [Mockup page width] Adding 1100px widescreen frames makes the mockups page require wider viewports to see all devices side by side. → Mockup frames already wrap via flexbox, so they will stack on smaller screens.
