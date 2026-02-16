## Purpose

SvelteKit project scaffold with build pipeline, embedded in the Rust binary and served as the default route.

## Requirements

### Requirement: SvelteKit Project Structure

A SvelteKit project SHALL exist at `ui/` with `@sveltejs/adapter-static` producing a fully static build output.

#### Scenario: Static build output

- **WHEN** `npm run build` is executed in the `ui/` directory
- **THEN** a static build is produced at `ui/build/` containing `index.html` and all assets

### Requirement: Build Integration

The Rust build process SHALL compile the SvelteKit project before embedding its output via `rust-embed`.

#### Scenario: Frontend built during cargo build

- **WHEN** `cargo build` is executed
- **THEN** `build.rs` runs `npm run build` in the `ui/` directory
- **AND** the build output at `ui/build/` is embedded into the binary

#### Scenario: Frontend build failure

- **WHEN** `npm run build` fails during `cargo build`
- **THEN** the Rust compilation fails with an error referencing the frontend build

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

### Requirement: Apply persisted theme preference

The UI shell SHALL apply the stored theme preference across all screens.

#### Scenario: Stored light preference

- **GIVEN** the stored theme preference is `light`
- **WHEN** the UI shell loads
- **THEN** the UI renders with light theme tokens

#### Scenario: Stored dark preference

- **GIVEN** the stored theme preference is `dark`
- **WHEN** the UI shell loads
- **THEN** the UI renders with dark theme tokens

### Requirement: System theme preference

The UI shell SHALL follow the system color scheme when the theme preference is `system`.

#### Scenario: System preference is dark

- **GIVEN** the stored theme preference is `system`
- **AND** the system color scheme is dark
- **WHEN** the UI shell loads
- **THEN** the UI renders with dark theme tokens

#### Scenario: System preference is light

- **GIVEN** the stored theme preference is `system`
- **AND** the system color scheme is light
- **WHEN** the UI shell loads
- **THEN** the UI renders with light theme tokens

#### Scenario: System preference changes

- **GIVEN** the stored theme preference is `system`
- **WHEN** the system color scheme changes
- **THEN** the UI updates to the new theme tokens without a reload
