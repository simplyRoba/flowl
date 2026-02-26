# TODO

## Refactor layout to enable iOS Safari chrome collapse on scroll

iOS Safari never collapses its address bar and bottom toolbar when scrolling,
because the app uses an "app shell" layout where scrolling happens inside an
inner `overflow-y: auto` container (`.content`), not on the document body.
Safari only collapses its chrome on body/document scroll.

### Root cause in `ui/src/routes/+layout.svelte`

- `html, body` has `overflow: hidden` and `height: 100%`
- `.app` is `display: flex; height: 100dvh` — a fixed viewport shell
- `.content` has `flex: 1; overflow-y: auto` — scrolling happens here, not on body

### Required changes

1. Remove `overflow: hidden` from `html, body`
2. Remove fixed `height: 100dvh` from `.app`
3. Remove `overflow-y: auto` from `.content` — let the body scroll naturally
4. Make the sidebar/nav use `position: fixed` instead of being a flex sibling:
   - **Desktop:** sidebar is 64px wide (200px at >=1280px), fixed to the left.
     `.content` gets a `margin-left` to compensate.
   - **Mobile (<=768px):** bottom nav is 56px tall, fixed to the bottom.
     `.content` gets `padding-bottom: 56px` to compensate.
5. Audit all `position: sticky` elements (e.g., `PageHeader` with `top: -24px`) —
   they currently stick relative to `.content`'s scroll. After refactor they'll
   stick relative to the viewport, so `top` values need adjustment.
6. Audit the chat drawer:
   - Desktop `.chat-drawer` uses `height: 100%` from the flex parent — will need
     rework (likely `position: fixed` or `height: 100dvh` with sticky).
   - The `detail.chat-open` flex layout (`height: 100%`) will need adjustment
     since the parent no longer has a fixed height.
7. Test that `100dvh` usages still work correctly after the refactor.

### Files affected

- `ui/src/routes/+layout.svelte` — major changes to `.app`, `.sidebar`, `.content`
- `ui/src/lib/components/PageHeader.svelte` — sticky `top` values may need updating
- `ui/src/routes/plants/[id]/+page.svelte` — chat-open flex layout needs adjustment
- All pages — verify scroll behavior works correctly with body scroll
