## Purpose

Application shell providing Flowl branding, navigation, and the shared visual structure for browser application routes.

## Requirements

### Requirement: Application shell

The browser application SHALL render a shared shell for normal application routes that displays the "flowl" brand and navigation.

#### Scenario: Shell renders

- **WHEN** a normal application route is loaded
- **THEN** the page displays the application name "flowl"
- **AND** application navigation is visible

#### Scenario: Widescreen expanded navigation

- **WHEN** the viewport width is >= 1280px
- **THEN** navigation SHALL use its expanded presentation
- **AND** each navigation item SHALL display its icon alongside a translated text label
- **AND** the brand area SHALL display the "flowl" name alongside its identifying mark

#### Scenario: Compact desktop navigation

- **WHEN** the viewport width is < 1280px and > 768px
- **THEN** navigation SHALL use its compact, icon-led presentation

#### Scenario: Document scrolling

- **WHEN** the SPA is loaded in a browser
- **THEN** the document SHALL provide the page's scrolling surface
- **AND** all page content, including content beyond the initial viewport, SHALL remain reachable through normal browser scrolling
- **AND** normal application routes SHALL NOT trap scrolling within a nested content region

#### Scenario: Widescreen content breathing room

- **WHEN** the viewport width is >= 1280px
- **THEN** the main content SHALL have more surrounding space than in compact desktop mode

### Requirement: Anchored navigation and unobscured content

The navigation SHALL remain visibly anchored while the document scrolls, and application content SHALL remain visible and usable alongside it.

#### Scenario: Desktop anchored navigation

- **WHEN** the viewport width is > 768px and < 1280px
- **THEN** compact navigation SHALL remain anchored along the left side while the document scrolls
- **AND** page content SHALL begin clear of the navigation

#### Scenario: Widescreen anchored navigation

- **WHEN** the viewport width is >= 1280px
- **THEN** expanded navigation SHALL remain anchored along the left side while the document scrolls
- **AND** page content SHALL begin clear of the navigation

#### Scenario: Mobile anchored navigation

- **WHEN** the viewport width is <= 768px
- **THEN** navigation SHALL remain available along the bottom edge while the document scrolls
- **AND** page content and controls, including those at the end of a page, SHALL not be obscured by navigation

#### Scenario: Navigation remains operable

- **WHEN** navigation is rendered over or alongside scrolling page content
- **THEN** it SHALL remain visible and operable without obscuring required content

### Requirement: Mobile navigation accessibility

Mobile navigation SHALL preserve space for itself without reducing the usability of page content.

#### Scenario: Accessible mobile navigation targets

- **WHEN** the viewport width is <= 768px
- **THEN** each mobile navigation target SHALL provide an accessible touch target of at least 44px
- **AND** users SHALL be able to reach page controls and content without navigation covering them

### Requirement: Apply persisted theme preference

The UI shell SHALL apply the stored theme preference across all screens.

#### Scenario: Stored light preference

- **GIVEN** the stored theme preference is `light`
- **WHEN** the UI shell loads
- **THEN** the UI renders in its light theme

#### Scenario: Stored dark preference

- **GIVEN** the stored theme preference is `dark`
- **WHEN** the UI shell loads
- **THEN** the UI renders in its dark theme

### Requirement: System theme preference

The UI shell SHALL follow the system color scheme when the theme preference is `system`.

#### Scenario: System preference is dark

- **GIVEN** the stored theme preference is `system`
- **AND** the system color scheme is dark
- **WHEN** the UI shell loads
- **THEN** the UI renders in its dark theme

#### Scenario: System preference is light

- **GIVEN** the stored theme preference is `system`
- **AND** the system color scheme is light
- **WHEN** the UI shell loads
- **THEN** the UI renders in its light theme

#### Scenario: System preference changes

- **GIVEN** the stored theme preference is `system`
- **WHEN** the system color scheme changes
- **THEN** the UI updates to the new theme without a reload

### Requirement: Minimum supported viewport width

The UI shell SHALL support application use at a viewport width of at least 320px without collapsing the layout.

#### Scenario: Viewport narrower than the supported minimum

- **WHEN** the viewport width is less than 320px
- **THEN** the application SHALL preserve the 320px minimum usable layout
- **AND** the browser MAY provide horizontal scrolling rather than collapsing content

### Requirement: Page content width hierarchy

Each page SHALL present its main content as a cohesive, centered region whose width suits the page's task while preserving readability.

#### Scenario: Wide dashboard content

- **WHEN** the dashboard page renders
- **THEN** it SHALL use the widest page region so multiple plant cards can make effective use of available horizontal space

#### Scenario: Readable default-width content

- **WHEN** the care journal, plant detail, or settings page renders
- **THEN** it SHALL use a page region narrower than the dashboard and bounded to keep its text and controls readable

#### Scenario: Focused form content

- **WHEN** the new plant or edit plant page renders
- **THEN** it SHALL use a page region narrower than the default-width pages so form fields remain focused and readable

#### Scenario: Widescreen page regions

- **WHEN** the viewport width is >= 1280px
- **THEN** each page region SHALL make more horizontal space available than in compact desktop mode
- **AND** the dashboard, default-width pages, and form pages SHALL retain their relative width hierarchy

#### Scenario: No competing page-width constraints

- **WHEN** a page uses its designated page width
- **THEN** its child content SHALL NOT impose a conflicting page-level width limit

### Requirement: Translated navigation labels

Sidebar navigation labels SHALL use the active language instead of hardcoded English text.

#### Scenario: Default English labels

- **GIVEN** the active language is English
- **WHEN** the sidebar renders
- **THEN** the navigation labels are "Plants", "Care Journal", and "Settings"

#### Scenario: German labels

- **GIVEN** the active language is German
- **WHEN** the sidebar renders
- **THEN** the navigation labels display the German translations

#### Scenario: Spanish labels

- **GIVEN** the active language is Spanish
- **WHEN** the sidebar renders
- **THEN** the navigation labels display the Spanish translations

#### Scenario: Widescreen expanded sidebar

- **WHEN** the viewport width is >= 1280px
- **THEN** the expanded sidebar displays translated text labels alongside icons

### Requirement: Offline connectivity indicator

The app shell SHALL display an offline dot badge on the Settings navigation item when the device has no network connectivity. No indicator SHALL be shown when the device is online.

#### Scenario: Dot badge visible when offline

- **WHEN** the device loses network connectivity
- **THEN** a small dot badge SHALL appear on the Settings navigation item

#### Scenario: Dot badge hidden when online

- **WHEN** the device has network connectivity
- **THEN** no dot badge SHALL be displayed on the Settings navigation item

#### Scenario: Initial state reflects connectivity

- **WHEN** the app shell loads
- **THEN** the dot badge SHALL reflect the current connectivity state at load time

#### Scenario: Dot badge updates on connectivity change

- **WHEN** the device transitions between online and offline
- **THEN** the dot badge SHALL appear or disappear without requiring a page reload

### Requirement: Public login shell isolation

The public `/login` route SHALL render outside Flowl's protected application shell. Existing canonical shell layout, navigation, widths, scrolling, theme, and pull-to-refresh behavior SHALL remain unchanged for normal application routes.

#### Scenario: Login route omits protected shell

- **WHEN** `/login` is loaded
- **THEN** the branded login content is visible without the application sidebar or bottom navigation
- **AND** the login route does not request `/api/settings` or other protected application data
- **AND** the login route does not initialize the network monitor, service-worker update UI, or pull-to-refresh behavior

#### Scenario: Normal routes retain canonical shell

- **WHEN** a normal authenticated application route is loaded
- **THEN** the existing canonical application shell and navigation behavior remains in effect

#### Scenario: Login route transition is reactively stable

- **WHEN** the SPA transitions to or from `/login`
- **THEN** the layout SHALL NOT enter a reactive update loop
- **AND** no effect SHALL both depend on and mutate the same authentication or pull-to-refresh state
