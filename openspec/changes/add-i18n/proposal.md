## Why

All UI strings are hardcoded in English. Users who prefer German or Spanish have no way to switch the interface language. Adding a simple DIY i18n system now establishes the translation infrastructure and lets users choose their preferred language from the settings page.

## What Changes

- Add a locale store following the same pattern as the existing theme store, persisted to localStorage.
- Add translation dictionaries (EN, DE, ES) with identical key structures covering all UI strings.
- Add a plural helper for count-dependent strings.
- Add a "Language" pill selector to the Settings page (same pattern as the Appearance theme selector).
- Replace hardcoded nav labels in the shell sidebar with translated strings.

## Capabilities

### New Capabilities
- `ui/i18n`: Locale store, translation dictionaries, plural helper, and reactive translations accessor.

### Modified Capabilities
- `ui/settings`: Add a Language section with English/Deutsch/Español pill toggle after Appearance.
- `ui/shell`: Nav labels use translated strings from the locale store.

## Impact

- Frontend UI (SvelteKit) only — new store, translation objects, settings section, and sidebar labels.
- No API or backend changes; locale preference stored client-side in localStorage.
