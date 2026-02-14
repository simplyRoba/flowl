## Tasks

- [x] **Task 1: Add plants and locations migration** — Create `migrations/20260214100000_plants.sql` with `locations` table (`id` INTEGER PRIMARY KEY, `name` TEXT NOT NULL UNIQUE) and `plants` table (`id` INTEGER PRIMARY KEY, `name` TEXT NOT NULL, `species` TEXT, `icon` TEXT NOT NULL DEFAULT '🪴', `location_id` INTEGER REFERENCES locations(id), `watering_interval_days` INTEGER NOT NULL DEFAULT 7, `light_needs` TEXT NOT NULL DEFAULT 'indirect', `notes` TEXT, `created_at` TEXT NOT NULL DEFAULT (datetime('now')), `updated_at` TEXT NOT NULL DEFAULT (datetime('now'))). Validates: `data/plants` — Plant Database Schema, `data/locations` — Location Database Schema, `core/database` — Migration Runner.

- [x] **Task 2: Create API error type** — Create `src/api/error.rs` with an `ApiError` enum implementing Axum's `IntoResponse`. Variants: `NotFound(String)` → 404, `Validation(String)` → 422, `Conflict(String)` → 409, `BadRequest(String)` → 400. Each returns JSON `{"message": "..."}`. Validates: `core/api` — JSON Error Responses.

- [x] **Task 3: Create location API handlers** — Create `src/api/locations.rs` with handlers: `list_locations` (GET, ordered by name), `create_location` (POST, validate name, 409 on duplicate, return 201), `update_location` (PUT /:id, 404 if missing, 409 on duplicate name, return 200), `delete_location` (DELETE /:id, 404 if missing, nullify plants.location_id, return 204). Structs: `Location { id, name }`, `CreateLocation { name }`, `UpdateLocation { name }`. Validates: `data/locations` — all requirements.

- [x] **Task 4: Create plant API handlers** — Create `src/api/plants.rs` with handlers: `list_plants` (GET, join locations, ordered by name), `get_plant` (GET /:id, include location_name, 404), `create_plant` (POST, name required, defaults for icon/watering/light, return 201), `update_plant` (PUT /:id, partial update, refresh updated_at, 404), `delete_plant` (DELETE /:id, 404, return 204). Structs: `Plant`, `PlantRow`, `CreatePlant`, `UpdatePlant`. Validates: `data/plants` — all requirements.

- [x] **Task 5: Create API router and mount on server** — Create `src/api/mod.rs` with routes: `/plants` (list, create), `/plants/:id` (get, update, delete), `/locations` (list, create), `/locations/:id` (update, delete). Pass `SqlitePool` as state. Update `src/server.rs` to accept pool, nest API under `/api`, keep `/health` and static fallback. Update `src/main.rs` to pass pool. Update `src/lib.rs` to add `pub mod api`. Validates: `core/api` — API Router, `core/server` — API routes scenario.

- [x] **Task 6: Add API integration tests** — Create `tests/api_plants.rs` and `tests/api_locations.rs`. Locations: list empty, create, duplicate 409, missing name 422, list ordered, update, update 404, update duplicate 409, delete 204, delete 404, delete nullifies plant references. Plants: list empty, create with defaults, missing name 422, get with location_name, get 404, update with refreshed updated_at, update 404, delete 204, delete 404. Shared test helper with in-memory SQLite pool + migrations. Validates: all data specs, `core/api` — JSON Error Responses.

- [x] **Task 7: Create API client module** — Create `ui/src/lib/api.ts` with typed functions: `fetchPlants`, `fetchPlant`, `createPlant`, `updatePlant`, `deletePlant`, `fetchLocations`, `createLocation`, `updateLocation`, `deleteLocation`. Define TS types: `Plant`, `CreatePlant`, `UpdatePlant`, `Location`. Extract error `message` from responses and throw. Validates: `ui/plants` — API Client.

- [x] **Task 8: Create Svelte stores for plants and locations** — Create `ui/src/lib/stores/plants.ts` (writable store: `loadPlants`, `loadPlant`, `createPlant`, `updatePlant`, `deletePlant`). Create `ui/src/lib/stores/locations.ts` (writable store: `loadLocations`, `createLocation`, `updateLocation`, `deleteLocation`). Stores call API client and update reactive state. Validates: Design Decision 6.

- [x] **Task 9: Build Plants Dashboard page** — Replace `ui/src/routes/+page.svelte`: on mount load plants, render card grid (Noto emoji icon resolved from emoji char to `/emoji/emoji_u{codepoint}.svg`, name, location name), empty state with "Add Plant" linking to `/plants/new`, add plant button in header. Validates: `ui/plants` — Plants Dashboard.

- [x] **Task 10: Build Plant Detail page** — Create `ui/src/routes/plants/[id]/+page.svelte`: fetch plant by id, display icon/name/species/location/watering/light/notes, edit button → `/plants/{id}/edit`, delete button with confirm → `deletePlant` → navigate `/`, 404 state. Validates: `ui/plants` — Plant Detail View.

- [x] **Task 11: Build shared form components** — Create reusable components: `ui/src/lib/components/PlantForm.svelte` (shared form layout, emits save), `IconPicker.svelte` (Noto emoji grid), `LocationChips.svelte` (chip selector with "Add new"), `WateringInterval.svelte` (preset chips 3d/7d/14d/30d + custom stepper). Supports Tasks 12 and 13.

- [x] **Task 12: Build Add Plant form** — Create `ui/src/routes/plants/new/+page.svelte`: uses PlantForm component, load locations on mount, validate name required, on submit call `createPlant`, navigate to `/plants/{id}`. Validates: `ui/plants` — Add Plant Form.

- [x] **Task 13: Build Edit Plant form** — Create `ui/src/routes/plants/[id]/edit/+page.svelte`: fetch plant by id, pre-fill PlantForm, on submit call `updatePlant`, navigate to `/plants/{id}`. Validates: `ui/plants` — Edit Plant Form.

- [x] **Task 14: Verify build and run** — Run `cargo clippy` (no warnings), `cargo test` (all pass), build full app (`npm run build` + `cargo build`), manually verify dashboard/create/edit/delete flow. Validates: end-to-end verification.
