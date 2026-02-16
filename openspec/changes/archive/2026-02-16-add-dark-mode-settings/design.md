## Context

The UI currently uses light theme tokens only, while DESIGN.md and mockups define both light and dark palettes plus a tri-state theme selector. The frontend is a SvelteKit static app embedded in the Rust binary, so theme selection must be handled client-side without new backend APIs or dependencies.

## Goals / Non-Goals

**Goals:**
- Provide a Light/Dark/System theme selector in Settings that matches the mockups.
- Apply the selected theme consistently across all UI screens.
- Persist the user preference and honor `prefers-color-scheme` when set to System.

**Non-Goals:**
- No server-side persistence or API changes for theme preference.
- No redesign of existing layout or component structure beyond theme tokens.

## Decisions

- **Theme application via root class + CSS variables**: Use a single class on the root element (e.g., `data-theme` or `class="dark"`) to swap CSS variables defined for light and dark palettes. This keeps changes localized to the design token layer and aligns with the mockup CSS approach.
  - **Alternative**: Per-component theme variants. Rejected due to higher maintenance and risk of inconsistent visuals.

- **Preference storage in localStorage**: Store the tri-state selection (`light`, `dark`, `system`) in localStorage to persist across sessions without backend changes.
  - **Alternative**: Cookies or server config. Rejected to avoid new dependencies and API surface.

- **System preference handling**: When selection is `system`, derive the effective theme from `matchMedia('(prefers-color-scheme: dark)')` and subscribe to changes to update the UI live.
  - **Alternative**: Snapshot only on load. Rejected because it does not react to OS theme changes.

## Risks / Trade-offs

- **LocalStorage unavailable** (privacy mode or disabled) → Fallback to `system` and keep the app functional without persistence.
- **CSS variable coverage gaps** → Use a single source of truth for tokens and audit key surfaces (background, cards, text, borders) to ensure full coverage.
- **Hydration mismatch** when reading localStorage on load → Apply a minimal inline theme bootstrap before hydration to avoid flash-of-incorrect-theme.

## Migration Plan

- No data migrations. Deploy frontend changes with updated CSS tokens and settings UI.
- Rollback is a standard UI rollback; theme selection will simply revert to light mode.

## Open Questions

- Should we default to `system` or `light` for first-time visitors? (Recommend `system` to respect OS settings.)
