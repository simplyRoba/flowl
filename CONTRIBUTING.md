# Contributing

## Architecture

The Rust backend serves a SvelteKit SPA. The UI is built as static files (`ui/build/`) and embedded into the Rust binary at compile time via `rust-embed`. The result is a single self-contained binary with no external file dependencies. However for development, it is nice to have hot reloading so the UI can be updated independently for a faster feedback loop. Here is how to set that up:

### Dev server with hot reloading

Requires Node.js (LTS) and Rust (stable). A devcontainer config is included.

Install UI dependencies first:

```bash
npm ci --prefix ui
```

Run two terminals:

```bash
# Terminal 1: UI with hot module reload (--host exposes on network for phone testing)
npm run dev --prefix ui -- --host

# Terminal 2: Rust backend with auto-restart on code changes
FLOWL_DB_PATH=/tmp/flowl.db FLOWL_AI_API_KEY=ENABLE-AI-UI FLOWL_MQTT_DISABLED=true SKIP_UI_BUILD=1 cargo watch -x run
```

Open `http://localhost:5173` (or the network URL printed by Vite for phone testing). Vite proxies `/api`, `/uploads`, and `/health` to the Rust backend on port 4100.

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
npm --prefix ui exec vitest --watch
```

### Linting and formatting

Run UI formatting check:

```bash
npm run format:check --prefix ui
```

Apply UI formatting:

```bash
npm run format --prefix ui
```

Run UI linting:

```bash
npm run lint --prefix ui
```

Auto-fix UI lint issues where possible:

```bash
npm run lint:fix --prefix ui
```

Run Svelte/TypeScript checks:

```bash
npm run check --prefix ui
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
| AI          | Iris         | `#9B7ED8` | AI actions, assistant states        |
| Text        | Bark         | `#2C2418` | Primary text                        |
| Text Muted  | Driftwood    | `#736556` | Secondary text, captions            |
| Border      | Sand         | `#E5DDD3` | Dividers, card borders              |
| Success     | Sprout       | `#7AB87A` | Healthy, watered, ok states         |
| Success Text| Clover       | `#3D7A3D` | Status badge text (ok)              |
| Warning     | Amber        | `#D4A843` | Due soon                            |
| Warning Text| Tawny        | `#8A6D1B` | Status badge text (due)             |
| Danger      | Dry          | `#C45B5B` | Overdue, errors                     |
| Danger Text | Rust         | `#9E3A3A` | Status badge text (overdue)         |

#### Dark Mode

| Role        | Name         | Hex       | Usage                              |
|-------------|--------------|-----------|-------------------------------------|
| Background  | Soil         | `#1A1612` | Page background                     |
| Surface     | Loam         | `#252019` | Cards, modals, inputs               |
| Primary     | Sage         | `#8BB592` | Buttons, active states, links       |
| Primary Dark| Mint         | `#A3CDA9` | Hover, pressed states               |
| Secondary   | Clay         | `#D49478` | Accents, highlights                 |
| Water       | Sky          | `#78B4D8` | Watering indicators                 |
| AI          | Lilac        | `#B89EE8` | AI actions, assistant states        |
| Text        | Parchment    | `#EDE6DB` | Primary text                        |
| Text Muted  | Sandstone    | `#B5A899` | Secondary text                      |
| Border      | Root         | `#3A3228` | Dividers, card borders              |
| Success     | Leaf         | `#8BC48B` | Healthy states                      |
| Success Text| Fern         | `#A8DCA8` | Status badge text (ok)              |
| Warning     | Honey        | `#D4B054` | Due soon                            |
| Warning Text| Wheat        | `#E0C46E` | Status badge text (due)             |
| Danger      | Wilt         | `#D47878` | Overdue, errors                     |
| Danger Text | Coral        | `#E89A9A` | Status badge text (overdue)         |

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
