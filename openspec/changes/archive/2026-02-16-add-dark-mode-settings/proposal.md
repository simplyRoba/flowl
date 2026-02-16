## Why

The UI currently ships only a light theme, which does not match the design system or mockups and limits usability in low-light environments. Adding theme selection now aligns implementation with the existing DESIGN.md and settings mockup, unblocking consistent visual work across screens.

## What Changes

- Add a theme preference setting with Light/Dark/System options in the Settings screen.
- Apply the selected theme across the UI, honoring system preference when set to System.
- Persist the theme selection so it remains consistent across sessions.

## Capabilities

### New Capabilities
- `none`: No new capabilities; extend existing UI specs.

### Modified Capabilities
- `ui/settings`: Add requirements for the Appearance section and theme selector (Light/Dark/System).
- `ui/shell`: Add requirements for applying the selected theme and honoring system preference.

## Impact

- Frontend UI (SvelteKit) theme tokens, settings view, and app shell layout.
- No API changes; possible use of local storage or persisted user preference.
