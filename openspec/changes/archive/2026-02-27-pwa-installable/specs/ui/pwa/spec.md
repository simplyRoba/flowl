## ADDED Requirements

### Requirement: Web app manifest

The app SHALL provide a web app manifest at `/manifest.json` that makes it installable on mobile devices.

#### Scenario: Manifest is served

- **WHEN** a browser requests `/manifest.json`
- **THEN** a valid JSON manifest SHALL be returned
- **AND** it SHALL include `name`, `short_name`, `start_url`, `display`, `theme_color`, `background_color`, and `icons`

#### Scenario: Display mode

- **WHEN** the app is installed via "Add to Home Screen"
- **THEN** it SHALL launch in `standalone` mode without browser chrome

#### Scenario: Theme colors match app

- **WHEN** the manifest is loaded
- **THEN** `theme_color` SHALL be `#FAF6F1` (light background)
- **AND** `background_color` SHALL be `#FAF6F1`

### Requirement: PWA icons

The app SHALL provide PNG icons in the sizes required for installability.

#### Scenario: Required icon sizes

- **WHEN** the manifest `icons` array is read
- **THEN** it SHALL include at least a 192x192 icon with `purpose: "any"`
- **AND** a 512x512 icon with `purpose: "any"`

### Requirement: Manifest link tag

The HTML document SHALL reference the manifest.

#### Scenario: Link tag present

- **WHEN** the HTML document is loaded
- **THEN** a `<link rel="manifest" href="/manifest.json">` tag SHALL be present in the `<head>`
