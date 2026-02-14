## ADDED Requirements

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
