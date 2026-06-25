## Purpose

Client-side i18n system providing translations for English, German, and Spanish.

## Requirements

### Requirement: Locale store

A `locale` writable store SHALL hold the active locale (`'en' | 'de' | 'es'`), defaulting to `'en'`.

#### Scenario: Default locale

- **WHEN** the application loads with no stored locale preference
- **THEN** the locale store is set to `'en'`

#### Scenario: Persisted locale restored

- **WHEN** the application loads
- **AND** `localStorage` key `flowl.locale` contains a valid locale (`'en'`, `'de'`, or `'es'`)
- **THEN** the locale store is set to the stored value

#### Scenario: Invalid stored locale

- **WHEN** the application loads
- **AND** `localStorage` key `flowl.locale` contains an invalid or missing value
- **THEN** the locale store falls back to `'en'`

#### Scenario: Locale change persisted

- **WHEN** the locale is changed
- **THEN** the new value is written to `localStorage` key `flowl.locale`

### Requirement: Translation dictionaries

Each supported locale SHALL have a TypeScript translation object with identical keys.

#### Scenario: Dictionary structure

- **GIVEN** the English, German, and Spanish translation dictionaries
- **THEN** all three dictionaries SHALL have identical key structures
- **AND** keys are organized in shallow nested groups (e.g., `nav`, `dashboard`, `plant`, `status`, `settings`, `care`, `form`, `dialog`)

#### Scenario: English as canonical type

- **GIVEN** the translation dictionaries
- **THEN** the English dictionary SHALL serve as the canonical TypeScript type definition
- **AND** the German and Spanish dictionaries SHALL satisfy the same type

### Requirement: Plural helper

A `plural(forms: {one: string, other: string}, n: number)` function SHALL return the correct plural form with count substitution.

#### Scenario: Singular form

- **WHEN** `plural({one: '{n} plant', other: '{n} plants'}, 1)` is called
- **THEN** the result is `'1 plant'`

#### Scenario: Plural form

- **WHEN** `plural({one: '{n} plant', other: '{n} plants'}, 5)` is called
- **THEN** the result is `'5 plants'`

#### Scenario: Zero uses plural form

- **WHEN** `plural({one: '{n} plant', other: '{n} plants'}, 0)` is called
- **THEN** the result is `'0 plants'`

### Requirement: Reactive translations

A derived store `translations` SHALL resolve to the translation object for the current locale.

#### Scenario: Translations follow locale

- **GIVEN** the locale store is set to `'de'`
- **THEN** the `translations` store resolves to the German translation dictionary

#### Scenario: Locale change updates translations

- **WHEN** the locale store changes from `'en'` to `'es'`
- **THEN** the `translations` store reactively updates to the Spanish translation dictionary

#### Scenario: Component access pattern

- **GIVEN** a Svelte component subscribes to the `translations` store
- **THEN** translated strings are accessed via `$translations.group.key`
