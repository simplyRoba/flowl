# flowl

![Under Heavy Development](https://img.shields.io/badge/under%20heavy%20development-orange)
[![Conventional Commits](https://img.shields.io/badge/Conventional%20Commits-1.0.0-yellow.svg)](https://conventionalcommits.org)
![GitHub License](https://img.shields.io/github/license/simplyRoba/flowl?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Fblob%2Fmain%2FLICENSE)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/simplyRoba/flowl/ci.yml?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Factions%2Fworkflows%2Fci.yml%3Fquery%3Dbranch%253Amain)
[![GitHub release](https://img.shields.io/github/v/release/simplyRoba/flowl?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Freleases)](https://github.com/simplyRoba/flowl/releases)
[![GitHub issues](https://img.shields.io/github/issues/simplyRoba/flowl?link=https%3A%2F%2Fgithub.com%2FsimplyRoba%2Fflowl%2Fissues)](https://github.com/simplyRoba/flowl/issues)
![GitHub Repo stars](https://img.shields.io/github/stars/simplyRoba/flowl)

> **flowl** — short for **fl**ower **owl** — /flaʊl/ like "fowl" but with an *l*

A small Rust service that exposes plant care data (watering schedules, care needs, etc.) for integration with Home Assistant and other automation platforms.

## Configuration
### Project variables

| Variable | Default | Description |
| --- | --- | --- |
| `FLOWL_PORT` | `4100` | HTTP server listen port. |
| `FLOWL_DB_PATH` | `/data/flowl.db` | Filesystem path to the SQLite database. |
| `FLOWL_LOG_LEVEL` | `info` | `tracing` level filter for logs. |
| `FLOWL_MQTT_HOST` | `localhost` | MQTT broker hostname. |
| `FLOWL_MQTT_PORT` | `1883` | MQTT broker port. |
| `FLOWL_MQTT_TOPIC_PREFIX` | `flowl` | Topic prefix used for auto-discovery and plant topics. |
| `FLOWL_MQTT_DISABLED` | `false` | Skip MQTT client, state checker, and publishes when set to `true`. |

---

# Development
## Architecture

The Rust backend serves a SvelteKit SPA. The UI is built as static files (`ui/build/`) and embedded into the Rust binary at compile time via `rust-embed`. The result is a single self-contained binary with no external file dependencies. However for development, it is nice to have hot reloading so the UI can be updated independently for a faster feedback loop. Here is how to set that up:

### Dev server with hot reloading

Requires Node.js (LTS) and Rust (stable). A devcontainer config is included.

Run two terminals:

```bash
# Terminal 1: UI with hot module reload
cd ui && npm run dev

# Terminal 2: Rust backend with auto-restart on code changes
FLOWL_DB_PATH=/tmp/flowl.db FLOWL_MQTT_DISABLED=true SKIP_UI_BUILD=1 cargo watch -x run
```

Open `http://localhost:5173`. Vite proxies `/api`, `/uploads`, and `/health` to the Rust backend on port 4100.

`SKIP_UI_BUILD=1` tells `build.rs` to skip the SvelteKit build so Rust recompiles fast. `cargo-watch` is installed in the devcontainer automatically.

### Testing

Run the full test suite (Rust + UI):

```bash
cargo test
```

The `ui_tests` integration test in `tests/ui.rs` shells out to `npm run test` in `ui/`, so vitest runs as part of `cargo test`. Node.js must be installed and `ui/node_modules` present (the test runs `npm install` automatically if missing).

Run only Rust backend tests (faster, no Node.js needed):

```bash
cargo test -- --skip ui_tests
```

Run only UI tests:

```bash
cargo test --test ui
```

Or run vitest directly for watch mode during UI development:

```bash
cd ui && npx vitest
```

### Build with embedded UI

To compile a binary with the UI baked in (like production), run `cargo build` without `SKIP_UI_BUILD`. This triggers `build.rs` to build the SvelteKit frontend and embed it via `rust-embed`.

## Design

Warm, organic, calm. The UI should feel like a plant journal — not a corporate dashboard. Rounded shapes, soft colors, generous whitespace.

### Color Palette

#### Light Mode (default)

| Role        | Name         | Hex       | Usage                              |
|-------------|--------------|-----------|-------------------------------------|
| Background  | Linen        | `#FAF6F1` | Page background                     |
| Surface     | White        | `#FFFFFF` | Cards, modals, inputs               |
| Primary     | Sage         | `#6B8F71` | Buttons, active states, links       |
| Primary Dark| Forest       | `#4A6B4F` | Hover, pressed states               |
| Secondary   | Terracotta   | `#C4775B` | Accents, highlights, overdue badges |
| Water       | Stream       | `#5B9BC4` | Watering indicators, water actions  |
| Text        | Bark         | `#2C2418` | Primary text                        |
| Text Muted  | Driftwood    | `#8C7E6E` | Secondary text, captions            |
| Border      | Sand         | `#E5DDD3` | Dividers, card borders              |
| Success     | Sprout       | `#7AB87A` | Healthy, watered, ok states         |
| Warning     | Amber        | `#D4A843` | Due soon                            |
| Danger      | Dry          | `#C45B5B` | Overdue, errors                     |

#### Dark Mode

| Role        | Name         | Hex       | Usage                              |
|-------------|--------------|-----------|-------------------------------------|
| Background  | Soil         | `#1A1612` | Page background                     |
| Surface     | Loam         | `#252019` | Cards, modals, inputs               |
| Primary     | Sage         | `#8BB592` | Buttons, active states, links       |
| Primary Dark| Mint         | `#A3CDA9` | Hover, pressed states               |
| Secondary   | Clay         | `#D49478` | Accents, highlights                 |
| Water       | Sky          | `#78B4D8` | Watering indicators                 |
| Text        | Parchment    | `#EDE6DB` | Primary text                        |
| Text Muted  | Sandstone    | `#9C8E7E` | Secondary text                      |
| Border      | Root         | `#3A3228` | Dividers, card borders              |
| Success     | Leaf         | `#8BC48B` | Healthy states                      |
| Warning     | Honey        | `#D4B054` | Due soon                            |
| Danger      | Wilt         | `#D47878` | Overdue, errors                     |

### Typography

| Element     | Font           | Size   | Weight  |
|-------------|----------------|--------|---------|
| H1          | System sans    | 28px   | 700     |
| H2          | System sans    | 22px   | 600     |
| H3          | System sans    | 18px   | 600     |
| Body        | System sans    | 15px   | 400     |
| Caption     | System sans    | 13px   | 400     |
| Button      | System sans    | 14px   | 500     |
| Badge       | System sans    | 12px   | 600     |

System font stack: `-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif`

### Spacing & Layout

- Base unit: `4px`
- Standard spacing: `8px`, `12px`, `16px`, `24px`, `32px`
- Card padding: `16px`
- Card border-radius: `12px`
- Button border-radius: `8px`
- Max content width: `1200px`
- Card grid gap: `16px`
- Mobile breakpoint: `768px`
- Widescreen breakpoint: `1280px`

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
