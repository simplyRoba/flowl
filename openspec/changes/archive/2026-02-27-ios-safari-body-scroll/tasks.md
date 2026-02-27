## 1. Shell layout — body scroll migration

- [x] 1.1 Add `--nav-bottom-height: 56px` CSS custom property to `:root` in `+layout.svelte`
- [x] 1.2 Remove `overflow: hidden` and `height: 100%` from `html, body` global styles
- [x] 1.3 Remove `height: 100vh` / `height: 100dvh` from `.app` — make it a plain block container
- [x] 1.4 Remove `overflow-y: auto` from `.content`

## 2. Fixed-position sidebar

- [x] 2.1 Make `.sidebar` `position: fixed` with `top: 0; left: 0; bottom: 0; z-index: 100` on desktop (>768px)
- [x] 2.2 Add `margin-left: 64px` to `.content` (default), `margin-left: 200px` at >=1280px
- [x] 2.3 On mobile (<=768px), make `.sidebar` `position: fixed; bottom: 0; left: 0; right: 0` with `z-index: 100` and `height: var(--nav-bottom-height)`
- [x] 2.4 Add `padding-bottom: var(--nav-bottom-height)` to `.content` on mobile, remove `margin-left`

## 3. PageHeader sticky adjustment

- [x] 3.1 Change `.page-header-inline` sticky `top` from `-24px` to `0` (default) and from `-32px` to `0` (>=1280px)
- [x] 3.2 Keep negative margins for visual alignment, only change sticky `top` values
- [x] 3.3 Apply same `top: 0` at the >=1280px breakpoint (keep `-32px -32px` negative margins)
- [x] 3.4 Update `.action-bar` `bottom: 56px` to `bottom: var(--nav-bottom-height)`

## 4. Chat drawer — fixed positioning

- [x] 4.1 Change desktop `.chat-drawer` from flex sibling to `position: fixed; top: 0; right: 0; bottom: 0; width: 400px; z-index: 90`
- [x] 4.2 Remove `.chat-drawer` `height: 100%` (no longer needed with fixed positioning)

## 5. Plant detail — chat-open layout rework

- [x] 5.1 Remove all `.detail.chat-open` CSS overrides — drawer overlays content without shifting it
- [x] 5.2 Remove `.detail.chat-open :global(.page-header-inline)` and `.action-bar` hiding rules
- [x] 5.3 Add body scroll lock (`overflow: hidden`) when mobile chat dialog is open
- [x] 5.4 Add `overscroll-behavior: contain` on `.chat-messages` to prevent scroll chaining

## 6. Audit and verification

- [x] 6.1 Grep for remaining `100dvh` and `100vh` usages — verify none depend on the old shell height
- [x] 6.2 Grep for remaining hardcoded `56px` values — replace with `var(--nav-bottom-height)` where they reference the nav height
- [x] 6.3 Verify `PhotoLightbox` and `ModalDialog` still work correctly (both use `position: fixed` / `<dialog>`)
- [x] 6.4 Verify SvelteKit scroll restoration works with body scroll (navigate between pages, check scroll resets)
- [x] 6.5 Test all pages for correct scroll behavior: dashboard, plant list, plant detail, care journal, settings, plant form (new/edit)
- [x] 6.6 Test chat drawer open/close on desktop and mobile — verify layout transitions
- [x] 6.7 Run `npm run check` in `ui/`, `cargo fmt`, `cargo clippy`, and `cargo test`
