## ADDED Requirements

### Requirement: Content width tokens

The UI shell SHALL define three content-width CSS custom properties as design tokens in `:root`, providing consistent width tiers for page content areas.

#### Scenario: Default viewport width tokens

- **WHEN** the viewport width is < 1280px and > 768px
- **THEN** `:root` SHALL define `--content-width-narrow` as `640px`
- **AND** `:root` SHALL define `--content-width-default` as `800px`
- **AND** `:root` SHALL define `--content-width-wide` as `1200px`

#### Scenario: Widescreen width tokens

- **WHEN** the viewport width is >= 1280px
- **THEN** `--content-width-narrow` SHALL be overridden to `720px`
- **AND** `--content-width-default` SHALL be overridden to `960px`
- **AND** `--content-width-wide` SHALL be overridden to `1400px`

### Requirement: Page content width tiers

Each page SHALL constrain its content width using one of the three content-width tokens and center itself with `margin: 0 auto`. Pages MUST NOT use hardcoded pixel values for content max-width.

#### Scenario: Wide-tier pages

- **WHEN** the dashboard page renders
- **THEN** its content wrapper SHALL use `max-width: var(--content-width-wide)`

#### Scenario: Default-tier pages

- **WHEN** the care journal, plant detail, or settings page renders
- **THEN** its content wrapper SHALL use `max-width: var(--content-width-default)`

#### Scenario: Narrow-tier pages

- **WHEN** the new plant or edit plant page renders
- **THEN** its content wrapper SHALL use `max-width: var(--content-width-narrow)`

#### Scenario: No nested width constraints

- **WHEN** a page uses a content-width token on its wrapper
- **THEN** child components within that page MUST NOT define their own `max-width` for layout purposes
