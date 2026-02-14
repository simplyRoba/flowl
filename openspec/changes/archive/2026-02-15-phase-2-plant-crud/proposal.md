## Why

The foundation is in place (server, database, MQTT, frontend shell) but there is no application data or UI. Before watering schedules, care logs, or MQTT publishing can work, plants need to exist. This change adds the full plant CRUD lifecycle: database schema, REST API, and frontend screens for listing, viewing, creating, and editing plants.

## What Changes

- Add a `plants` database table (id, name, species, icon, location, watering interval, light needs, notes, created_at, updated_at).
- Add REST API endpoints: `GET /api/plants`, `GET /api/plants/:id`, `POST /api/plants`, `PUT /api/plants/:id`, `DELETE /api/plants/:id`.
- Add a `locations` database table to store reusable room/location labels.
- Add REST API endpoints: `GET /api/locations`, `POST /api/locations`.
- Build the Plants dashboard screen: grid of plant cards with icon, name, location, and status placeholder.
- Build the Plant Detail screen: full plant info, edit and delete actions.
- Build the Add/Edit Plant form: name, species, icon picker (Noto emoji), location chips, watering interval presets + custom stepper, light needs selector, notes.
- Store shared application state (plant list, locations) via Svelte stores.

## Capabilities

### New Capabilities

- `core/api`: REST API layer with JSON request/response handling, error responses, and route mounting.
- `data/plants`: Plant entity — database schema, CRUD queries, and validation.
- `data/locations`: Location entity — database schema and queries for reusable room labels.
- `ui/plants`: Plant list dashboard, plant detail view, add/edit plant form with all interactive controls.

### Modified Capabilities

- `core/server`: Mount the `/api` router on the Axum server.
- `core/database`: Add plant and location migrations.

## Impact

- `migrations/`: New migration for `plants` and `locations` tables.
- `src/`: New modules for API routing, plant handlers, location handlers, models, and validation.
- `ui/src/routes/`: New SvelteKit pages for dashboard, plant detail, and add/edit form.
- `ui/src/lib/`: Svelte stores for plants and locations, API client utility, shared components (plant card, icon picker, location chips, etc.).
- `Cargo.toml`: May need `uuid` or `chrono` for ID generation and timestamps.
