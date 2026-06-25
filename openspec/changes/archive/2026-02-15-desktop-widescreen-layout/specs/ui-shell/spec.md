## MODIFIED Requirements

### Requirement: Empty Shell Layout

The SvelteKit project SHALL include a root layout with the application name and navigation placeholder, ready for feature screens in later phases.

#### Scenario: Shell renders

- **WHEN** the SPA is loaded in a browser
- **THEN** the page displays the application name "flowl"
- **AND** a placeholder layout is visible

#### Scenario: Widescreen expanded sidebar

- **WHEN** the viewport width is >= 1280px
- **THEN** the sidebar SHALL expand to 200px width
- **AND** each navigation item SHALL display its icon alongside a text label ("Plants", "Log", "Settings")
- **AND** the logo area SHALL display the "flowl" brand name next to the sprout icon

#### Scenario: Below widescreen breakpoint

- **WHEN** the viewport width is < 1280px and > 768px
- **THEN** the sidebar SHALL remain at 64px width with icon-only navigation (unchanged behavior)

#### Scenario: Widescreen content padding

- **WHEN** the viewport width is >= 1280px
- **THEN** the main content area padding SHALL be 32px (increased from 24px)
