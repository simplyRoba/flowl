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

### Build with embedded UI

To compile a binary with the UI baked in (like production), run `cargo build` without `SKIP_UI_BUILD`. This triggers `build.rs` to build the SvelteKit frontend and embed it via `rust-embed`.

---

**This project is developed spec-driven with AI assistance, reviewed by a critical human.**
