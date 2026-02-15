# flowl — Project Plan

## Vision

A lightweight, self-hosted plant care manager. Track your plants, manage watering schedules, keep notes, and get reminders — all from a single Docker container on your local network.

## Tech Stack

| Layer      | Technology                          |
|------------|-------------------------------------|
| Backend    | Rust, Axum, Tokio                   |
| Database   | SQLite via sqlx (embedded, file-based) |
| Frontend   | SvelteKit (built and embedded in binary via `rust-embed`) |
| Reminders  | MQTT (HA auto-discovery via Mosquitto) |
| Container  | `debian:bookworm-slim`, single binary |
| AI (later) | Cloud vision/chat API, optional Ollama |

## Architecture

Single Rust binary serving the SvelteKit SPA as static assets. SQLite database stored on a mounted Docker volume. MQTT client publishes plant state and watering reminders to a Mosquitto broker for Home Assistant integration.

```
┌─────────────────────────────┐
│         Docker              │
│  ┌───────────────────────┐  │
│  │   flowl (Rust binary) │  │
│  │  ┌─────────┐ ┌──────┐ │  │
│  │  │  Axum   │ │ MQTT │─┼──┼──▶ Mosquitto ──▶ Home Assistant
│  │  │  HTTP   │ │client│ │  │
│  │  └────┬────┘ └──────┘ │  │
│  │       │               │  │
│  │  ┌────┴────┐          │  │
│  │  │ SQLite  │          │  │
│  │  └─────────┘          │  │
│  └───────────────────────┘  │
│       │                     │
│  volume: /data/flowl.db     │
└─────────────────────────────┘
```

## MQTT Integration

flowl connects to a Mosquitto broker and uses Home Assistant MQTT auto-discovery. For each plant with a watering schedule, flowl publishes:

- **Discovery topic**: `homeassistant/sensor/flowl/<plant-id>/config` — registers the plant as an HA sensor entity with name, device info, and state topic.
- **State topic**: `flowl/<plant-id>/state` — publishes watering state (`due`, `overdue`, `ok`) as retained messages.

HA picks up the entities automatically. HA automations handle notifications (mobile push, Telegram, etc.) when a plant's state changes to `due` or `overdue`.

**Rust crate**: `rumqttc` (async MQTT client, fits Tokio runtime).

**Configuration** (env vars):
- `FLOWL_MQTT_HOST` — broker hostname (default: `localhost`)
- `FLOWL_MQTT_PORT` — broker port (default: `1883`)
- `FLOWL_MQTT_TOPIC_PREFIX` — topic prefix (default: `flowl`)

## Data Model (initial)

- **Plant** — id, name, species, location/room, photo, notes, created_at
- **Watering Schedule** — plant_id, interval_days, last_watered, next_due
- **Care Log** — plant_id, type (watered, fertilized, repotted, note), timestamp, text

## Features by Phase

### Phase 1 — Foundation

- Axum HTTP server with health endpoint
- SQLite database with migrations (sqlx)
- SvelteKit frontend scaffold, embedded in binary
- MQTT client connecting to Mosquitto
- Docker image with volume mount for DB
- Basic project structure and error handling

### Phase 2 — Core Plant Management

- CRUD API for plants (name, species, location, photo, notes)
- Plant list and detail views in UI
- Photo upload and storage
- Room/location grouping

### Phase 3 — Watering & Reminders

- Watering schedule per plant (interval-based)
- "Water now" action that resets the timer
- MQTT auto-discovery: register plants as HA sensors
- Publish watering state (`ok` / `due` / `overdue`) to MQTT
- Due/overdue indicators in UI dashboard

### Phase 4 — Care Journal

- Log care events (watered, fertilized, repotted, pruned, notes)
- Timeline view per plant
- Photo attachments on log entries

### Phase 5 — Polish

- Seasonal watering adjustments (less frequent in winter)
- Care tags (low light, high humidity, toxic to pets)
- Import/export (JSON backup)
- Dashboard with "due today" / "overdue" overview

### Phase 6 — AI Integration (future)

- Plant identification from photo (vision API)
- Auto-populated care profile from species identification
- Care assistant: ask questions about a specific plant with its profile as context
- AI-suggested watering schedules based on species, season, placement
- Optional Ollama support for fully local inference

## Key Dependencies (Rust)

- `axum` — HTTP framework
- `sqlx` — async SQLite with compile-time queries
- `rumqttc` — async MQTT client
- `rust-embed` — embed SvelteKit build output in binary
- `serde` / `serde_json` — serialization
- `chrono` — date/time for schedules
- `tower-http` — static serving, compression
- `tokio` — async runtime

## Configuration

All via environment variables:

| Variable              | Default      | Description                  |
|-----------------------|--------------|------------------------------|
| `FLOWL_PORT`          | `4100`       | HTTP listen port             |
| `FLOWL_DB_PATH`       | `/data/flowl.db` | SQLite database path     |
| `FLOWL_MQTT_HOST`     | `localhost`  | MQTT broker host             |
| `FLOWL_MQTT_PORT`     | `1883`       | MQTT broker port             |
| `FLOWL_MQTT_TOPIC_PREFIX` | `flowl` | MQTT topic prefix            |
