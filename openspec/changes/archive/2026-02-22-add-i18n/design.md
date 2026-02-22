## Context

The UI currently has all strings hardcoded in English. The frontend is a SvelteKit static app embedded in the Rust binary, so language selection must be handled client-side without new backend APIs or external i18n libraries. The existing theme store provides a proven pattern for a persisted user preference with a reactive Svelte store.

## Goals / Non-Goals

**Goals:**
- Provide an English/German/Spanish language selector in Settings using the same pill toggle pattern as the theme selector.
- Replace all hardcoded UI strings with translated lookups that update reactively on locale change.
- Persist the language preference in localStorage across sessions.
- Keep the solution simple — plain TypeScript objects, no i18n library.

**Non-Goals:**
- No server-side locale detection or API changes.
- No runtime loading of translation files; all dictionaries are bundled statically.
- No right-to-left layout support (none of the three languages require it).
- No date/number formatting changes in this iteration.

## Decisions

- **DIY store + translation objects**: Mirror the theme store pattern — a writable `locale` store, a derived `translations` store that resolves to the active dictionary. This avoids adding a library dependency for three languages.
  - **Alternative**: Use an i18n library (e.g., svelte-i18n, typesafe-i18n). Rejected to keep the dependency footprint minimal and give full control over translation strings.

- **Flat-ish nested dictionaries**: Keys organized in shallow groups (`nav.plants`, `settings.language`, etc.) typed against the English dictionary. This provides autocomplete and compile-time safety without deep nesting.
  - **Alternative**: Flat string keys with dot notation. Rejected because grouped objects give better TypeScript ergonomics.

- **Locale persistence in localStorage**: Store the locale string (`'en'`, `'de'`, `'es'`) under key `flowl.locale`, matching the `flowl.theme` pattern.
  - **Alternative**: Combine with theme in a single preferences object. Rejected to keep stores independent and simple.

- **Simple plural helper**: A `plural({one, other}, n)` function covers EN/DE/ES plural rules (which all use the one-vs-other distinction). The `{n}` placeholder is replaced with the count.
  - **Alternative**: CLDR-based plural rules. Rejected as overkill for three Western European languages with identical plural categories.

## Risks / Trade-offs

- **Missing translation keys** → TypeScript enforces identical dictionary shapes at compile time, so missing keys cause build errors.
- **localStorage unavailable** → Falls back to `'en'`, same resilience pattern as the theme store.
- **Translation drift** → Since translations are user-maintained TypeScript objects, keeping them in sync is a manual process. The shared type definition mitigates this.

## Migration Plan

- No data migrations. Deploy frontend changes with new store, dictionaries, and updated components.
- Existing users see no change (default locale is `'en'`).

## Open Questions

- None at this time.
