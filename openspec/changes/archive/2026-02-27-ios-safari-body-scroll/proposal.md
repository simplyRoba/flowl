## Why

iOS Safari never collapses its address bar and bottom toolbar when scrolling in flowl, because the app uses an "app shell" layout where scroll happens inside an inner `overflow-y: auto` container (`.content`), not on the document body. Safari only collapses its chrome on body/document scroll. This wastes ~100px of vertical screen space on every iPhone, making the experience feel cramped — especially on plant detail and form pages. This needs to happen before Phase 7 (chat photos + save note) because the chat drawer layout is tightly coupled to the shell and must be reworked as part of this change.

## What Changes

- **Remove the app-shell scroll container**: Remove `overflow: hidden` from `html, body` and `overflow-y: auto` from `.content`. Let the document body scroll naturally so Safari can collapse its chrome.
- **Remove fixed viewport height**: Remove `height: 100dvh` from `.app`. The app wrapper becomes a normal flow container instead of a viewport-filling flex shell.
- **Make sidebar/nav fixed-position**:
  - **Desktop (>768px)**: Sidebar becomes `position: fixed`, pinned to the left. `.content` gets `margin-left` to compensate (64px, or 200px at >=1280px).
  - **Mobile (<=768px)**: Bottom nav becomes `position: fixed`, pinned to the bottom. `.content` gets `padding-bottom: 56px` to compensate.
- **Adjust sticky PageHeader**: `PageHeader`'s `position: sticky` currently works relative to `.content`'s scroll context. After refactor it sticks relative to the viewport — `top` values need recalculating.
- **Rework chat drawer for body-scroll context**: Desktop chat drawer currently uses `height: 100%` from the flex parent and the `.detail.chat-open` flex layout depends on the fixed-height shell. Both need reworking — likely `position: fixed` for the drawer with the detail content getting a `margin-right` offset. Mobile bottom sheet (dialog-based) should be unaffected.
- **Rework mobile action bar positioning**: `PageHeader`'s mobile `.action-bar` is `position: fixed; bottom: 56px` — this should continue to work but needs verification against body scroll.

## Capabilities

### New Capabilities

_(none — this is a pure refactor of existing layout behavior)_

### Modified Capabilities

- `ui/shell`: Scroll model changes from inner-container scroll to body scroll; sidebar/nav positioning changes from flex-sibling to `position: fixed`; `.app` no longer constrains viewport height.
- `ui/chat-drawer`: Desktop drawer positioning changes from flex-sibling with `height: 100%` to `position: fixed` with viewport height; `.detail.chat-open` layout reworked for body-scroll context.

## Impact

- **`ui/src/routes/+layout.svelte`** — Major changes: remove `overflow: hidden` from html/body, remove `height: 100dvh` from `.app`, remove `overflow-y: auto` from `.content`, make `.sidebar` `position: fixed`, add margin/padding offsets to `.content`.
- **`ui/src/lib/components/PageHeader.svelte`** — Sticky `top` values need recalculating (currently `-24px`/`-32px` to account for `.content` padding, will change to `0` for viewport-relative sticking). Negative margins for full-bleed also need adjustment.
- **`ui/src/routes/plants/[id]/+page.svelte`** — `.detail.chat-open` flex layout and `.detail-content` `overflow-y: auto` need rework for the new scroll model.
- **`ui/src/lib/components/ChatDrawer.svelte`** — Desktop `.chat-drawer` needs `position: fixed` instead of flex-sibling with `height: 100%`.
- **All pages** — Verify scroll behavior, sticky elements, and bottom padding work correctly with body scroll.
- **No backend changes** — This is purely a frontend CSS/layout refactor.
- **No new dependencies** — Uses existing CSS features only.
