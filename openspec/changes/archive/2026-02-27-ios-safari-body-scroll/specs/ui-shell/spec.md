## ADDED Requirements

### Requirement: Bottom nav height design token

The UI shell SHALL define a `--nav-bottom-height` CSS custom property in `:root` with the value `56px`, used by all components that reference the mobile bottom nav height.

#### Scenario: Token defined in root

- **WHEN** the UI shell loads
- **THEN** `:root` SHALL define `--nav-bottom-height` as `56px`

#### Scenario: Token used for bottom nav height

- **WHEN** the viewport width is <= 768px
- **THEN** the bottom nav height SHALL use `var(--nav-bottom-height)`

### Requirement: Fixed-position sidebar

The sidebar/nav SHALL use `position: fixed` so it remains visually anchored while the document body scrolls. The main content area SHALL use margin/padding offsets to avoid overlapping the fixed nav.

#### Scenario: Desktop fixed sidebar

- **WHEN** the viewport width is > 768px and < 1280px
- **THEN** the sidebar SHALL be `position: fixed` with `width: 64px`, anchored to the left edge
- **AND** the main content area SHALL have `margin-left: 64px`

#### Scenario: Widescreen fixed sidebar

- **WHEN** the viewport width is >= 1280px
- **THEN** the sidebar SHALL be `position: fixed` with `width: 200px`, anchored to the left edge
- **AND** the main content area SHALL have `margin-left: 200px`

#### Scenario: Mobile fixed bottom nav

- **WHEN** the viewport width is <= 768px
- **THEN** the nav SHALL be `position: fixed` at the bottom of the viewport
- **AND** the main content area SHALL have `padding-bottom: var(--nav-bottom-height)`

#### Scenario: Sidebar z-index layering

- **WHEN** the sidebar/nav is rendered
- **THEN** it SHALL have `z-index: 100` to remain above scrolling page content

## MODIFIED Requirements

### Requirement: Empty Shell Layout

The SvelteKit project SHALL include a root layout with the application name and navigation placeholder, ready for feature screens in later phases.

#### Scenario: Shell renders

- **WHEN** the SPA is loaded in a browser
- **THEN** the page displays the application name "flowl"
- **AND** a placeholder layout is visible

#### Scenario: Body scroll model

- **WHEN** the SPA is loaded in a browser
- **THEN** `html` and `body` SHALL NOT have `overflow: hidden`
- **AND** the `.app` container SHALL NOT constrain height to `100vh` or `100dvh`
- **AND** the document body SHALL be the scroll container (no inner `overflow-y: auto` on `.content`)

#### Scenario: Widescreen expanded sidebar

- **WHEN** the viewport width is >= 1280px
- **THEN** the sidebar SHALL expand to 200px width
- **AND** each navigation item SHALL display its icon alongside a translated text label
- **AND** the logo area SHALL display the "flowl" brand name next to the sprout icon

#### Scenario: Below widescreen breakpoint

- **WHEN** the viewport width is < 1280px and > 768px
- **THEN** the sidebar SHALL remain at 64px width with icon-only navigation (unchanged behavior)

#### Scenario: Widescreen content padding

- **WHEN** the viewport width is >= 1280px
- **THEN** the main content area padding SHALL be 32px (increased from 24px)
