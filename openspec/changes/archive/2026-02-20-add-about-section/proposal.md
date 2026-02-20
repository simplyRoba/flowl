## Why

The settings page has no app identity information. Users cannot see which version they're running, where to find the source code, or under what license the app is distributed. The mockup already specifies an About section with Version and Source rows.

## What Changes

- Add a `GET /api/info` backend endpoint that returns app version, repository URL, and license using Rust compile-time macros (`env!("CARGO_PKG_VERSION")`, `env!("CARGO_PKG_REPOSITORY")`, `env!("CARGO_PKG_LICENSE")`)
- Add an "About" section to the settings page UI displaying Version, Source (as a link), and License fetched from `/api/info`

## Capabilities

### New Capabilities

- `app-info`: Backend endpoint exposing application metadata (version, repository, license) from Cargo.toml at compile time

### Modified Capabilities

- `ui/settings`: Add an About section displaying version, source link, and license fetched from the app-info API

## Impact

- **Backend**: New route `GET /api/info` added to the API router — no existing routes affected
- **Frontend**: Settings page (`ui/src/routes/settings/+page.svelte`) gains a new section — no existing sections changed
- **Dependencies**: None — uses only Rust built-in `env!()` macros and existing SvelteKit fetch
