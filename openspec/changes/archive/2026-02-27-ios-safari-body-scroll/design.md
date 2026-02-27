## Context

flowl uses an "app shell" layout in `+layout.svelte`:

```
html, body  → overflow: hidden; height: 100%
.app        → display: flex; height: 100dvh
.sidebar    → flex-shrink: 0; width: 64px (200px at >=1280px)
.content    → flex: 1; overflow-y: auto
```

All scrolling happens inside `.content`. iOS Safari only collapses its chrome (address bar + bottom toolbar, ~100px combined) when the **document body** scrolls. The current layout prevents this entirely.

The chat drawer (`ChatDrawer.svelte`) and plant detail page (`plants/[id]/+page.svelte`) depend on the fixed-height shell: the desktop drawer uses `height: 100%` from the flex parent, and `.detail.chat-open` creates a side-by-side flex layout within the shell.

The `PageHeader` component uses `position: sticky; top: -24px` relative to `.content`'s scroll context.

## Goals / Non-Goals

**Goals:**

- Enable iOS Safari chrome collapse by moving scroll from `.content` to the document body
- Maintain identical visual appearance on all platforms (desktop Chrome/Firefox/Safari, mobile Android/iOS)
- Keep sidebar/nav visually fixed while the body scrolls
- Keep the chat drawer functional in both desktop (side panel) and mobile (bottom sheet) modes
- Keep `PageHeader` sticky behavior working

**Non-Goals:**

- Redesigning navigation structure or sidebar width/breakpoints
- Changing the chat drawer's mobile bottom sheet (it already uses `<dialog>` with `position: fixed`, unaffected by scroll model)
- Adding new features — this is a pure layout refactor
- Supporting browsers without `position: fixed` or `dvh` units

## Decisions

### 1. Body scroll instead of container scroll

**Decision:** Remove `overflow: hidden` from `html, body` and `overflow-y: auto` from `.content`. Let the document body be the scroll container.

**Why:** This is the only way to trigger Safari's chrome collapse. Safari explicitly checks for body/document scroll — no polyfill or workaround exists for inner-container scroll.

**Alternative considered:** Using JavaScript to programmatically trigger Safari chrome collapse — not possible, Safari doesn't expose this API.

### 2. CSS variable for bottom nav height

**Decision:** Add `--nav-bottom-height: 56px` to `:root`. This value is referenced across multiple components: the bottom nav itself (`+layout.svelte`), the mobile action bar offset (`PageHeader.svelte`), and the mobile content padding-bottom.

**Why:** 56px is a magic number repeated in 3+ files. A single variable prevents drift and makes the relationship between these values explicit. Sidebar widths (64px, 200px) stay as literals — they're only used within `+layout.svelte` where the sidebar and `.content` margin are co-located.

### 3. Fixed-position sidebar instead of flex sibling

**Decision:** Make `.sidebar` use `position: fixed` and offset `.content` with `margin-left`.

| Breakpoint | Sidebar width | `.content` `margin-left` |
|---|---|---|
| >768px, <1280px | 64px | 64px |
| >=1280px | 200px | 200px |
| <=768px (mobile) | bottom nav, `var(--nav-bottom-height)` tall | 0 (use `padding-bottom: var(--nav-bottom-height)` instead) |

**Why:** The sidebar must stay visually anchored while the body scrolls. `position: fixed` is the standard approach. Using `position: sticky` on the sidebar was considered but rejected — sticky requires a scroll container parent, and the whole point is to not have an intermediate scroll container.

**Alternative considered:** `position: sticky` on sidebar — doesn't work because we're removing the fixed-height `.app` container. The sidebar would scroll with the page.

### 4. Remove `.app` fixed height

**Decision:** Remove `height: 100vh` / `height: 100dvh` from `.app`. It becomes a plain `display: block` container (or is removed entirely if unnecessary).

**Why:** A fixed-height parent with `overflow: hidden` is what creates the inner scroll context. Removing it lets content flow naturally and the body becomes the scroll container.

### 5. Fixed-position chat drawer on desktop

**Decision:** Change the desktop `.chat-drawer` from a flex sibling (`height: 100%` from parent) to `position: fixed; top: 0; right: 0; bottom: 0; width: 400px`.

When the chat is open, `.detail-content` gets `margin-right: 400px` (only above 768px) to avoid content being hidden behind the drawer. Below ~900px viewport width, the drawer might overlap content — this is acceptable as the mobile bottom sheet takes over at <=768px.

**Why:** The drawer can no longer rely on a fixed-height flex parent. `position: fixed` is the simplest approach — the drawer stays viewport-anchored regardless of scroll position, which is the desired behavior (chat shouldn't scroll away).

**Alternative considered:** Making the drawer `position: sticky` — this would require it to be in the document flow and would scroll partially before sticking, creating a jarring UX.

### 6. PageHeader sticky recalculation

**Decision:** Change only `PageHeader`'s sticky `top` values. Keep negative margins and padding unchanged.

| Current | After refactor | Why |
|---|---|---|
| `top: -24px` (default) | `top: 0` | Sticks to viewport top instead of inner scroll container offset |
| `top: -32px` (>=1280px) | `top: 0` | Same |
| `margin: -24px -24px 16px` | _(unchanged)_ | Negative margins still needed to pull header into `.content` padding for a tight top edge |
| `padding: 12px 24px` | _(unchanged)_ | Internal padding still needed for header content alignment |

**Why:** The negative margins serve a visual purpose — pulling the header up into the `.content` padding area so there's no gap at the top of the page. Only the `top` value needed to change: from negative offsets (compensating for the old inner scroll container's padding) to `0` (viewport-relative with body scroll).

### 7. Plant detail chat-open layout rework

**Decision:** Remove all `.detail.chat-open` layout overrides. The chat drawer overlays content without shifting it.

**Current:**
```css
.detail.chat-open {
  display: flex;
  max-width: none;
  height: 100%;  /* relies on shell height */
  gap: 24px;
}
.detail.chat-open .detail-content {
  max-width: var(--content-width-default);
  overflow-y: auto;  /* inner scroll */
}
```

**After:** All `.chat-open` CSS rules removed. The detail layout stays unchanged when the chat opens. The fixed-position drawer overlays the right side of the viewport.

**Why:** The margin-right push approach caused two problems: (1) huge gap on widescreen between content and drawer, (2) content crushed on narrow desktops/tablets. Overlay is simpler, works at all widths, and matches how most chat panels behave (Intercom, GitHub Copilot, etc.).

**Alternative considered:** `margin-right: 424px` to push content — caused visual gaps on widescreen and content crushing on narrow screens. Breakpoint-based approach (overlay below 1200px, push above) was considered but added complexity for little benefit since the user's focus is on the chat.

### 8. Sidebar z-index layering

**Decision:** Set `z-index: 100` on the fixed sidebar to ensure it stays above page content during scroll. The chat drawer gets `z-index: 90` (below sidebar, above content). The PageHeader sticky gets `z-index: 10` (unchanged, below both).

Mobile bottom nav also gets `z-index: 100`.

## Risks / Trade-offs

**[Risk] Scroll position loss on navigation** → SvelteKit's client-side routing already handles scroll restoration via `scrollTo`. Since we're moving to body scroll, SvelteKit's default `window.scrollTo(0, 0)` on navigation should work correctly (it may have been a no-op before since body didn't scroll). Verify this works.

**[Risk] Desktop pages that intentionally fill the viewport** → The plant detail page with chat open currently fills the viewport height. After refactor, it flows naturally. If the content is shorter than the viewport, the page won't fill the screen. This is acceptable — most plant detail pages have enough content. The chat drawer is fixed-positioned so it always fills the viewport regardless.

**[Risk] `100dvh` usage elsewhere** → Grep for `100dvh` and `100vh` across all components. The only usage is in `.app` (being removed) and the mobile chat dialog (which uses `position: fixed` and is unaffected).

**[Risk] Third-party components or overlays** → `PhotoLightbox` and `ModalDialog` use `position: fixed` — unaffected by scroll model changes. No risk.

**[Risk] PageHeader mobile action bar** → Uses `position: fixed; bottom: var(--nav-bottom-height)` — continues to work correctly with body scroll since it's already viewport-relative.
