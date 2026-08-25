## MODIFIED Requirements

### Requirement: Empty Shell Layout

The SvelteKit project SHALL include a root layout with the application name and navigation for normal application routes. The public `/login` route SHALL render a dedicated branded login layout without the protected sidebar, protected settings bootstrap, protected data requests, network monitor, update notification UI, or pull-to-refresh behavior.

#### Scenario: Shell renders

- **WHEN** a normal authenticated application route is loaded in a browser
- **THEN** the page displays the application name `flowl`
- **AND** the normal navigation layout is visible

#### Scenario: Public login shell renders

- **WHEN** `/login` is loaded
- **THEN** the branded login content is visible without the application sidebar or bottom navigation
- **AND** the root layout does not request `/api/settings` or other protected application data

#### Scenario: Widescreen expanded sidebar

- **WHEN** a normal application route is loaded at a viewport width >= 1280px
- **THEN** the sidebar SHALL expand to 200px width
- **AND** each navigation item SHALL display its icon alongside a translated text label
- **AND** the logo area SHALL display the `flowl` brand name next to the sprout icon

#### Scenario: Below widescreen breakpoint

- **WHEN** a normal application route is loaded at a viewport width < 1280px and > 768px
- **THEN** the sidebar SHALL remain at 64px width with icon-only navigation

#### Scenario: Body scroll model

- **WHEN** the SPA is loaded in a browser
- **THEN** `html` and `body` SHALL NOT have `overflow: hidden`
- **AND** the `.app` container SHALL NOT constrain height to `100vh` or `100dvh`
- **AND** the document body SHALL be the scroll container with no inner `overflow-y: auto` on `.content`

#### Scenario: Widescreen content padding

- **WHEN** a normal application route is loaded at a viewport width >= 1280px
- **THEN** the main content area padding SHALL be 32px
