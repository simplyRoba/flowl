## Context

Phase 1 established the runtime: Axum server, SQLite with sqlx migrations, MQTT client, and a SvelteKit shell embedded in the binary. The application has no data model or functional UI yet. Phase 2 adds the core plant entity with full CRUD and the primary UI screens (dashboard, detail, add/edit).

## Goals / Non-Goals

**Goals:**
- A `plants` table with all fields needed for plant management
- A `locations` table for reusable room/location labels
- REST API for plant and location CRUD
- Plant dashboard with card grid
- Plant detail view with all plant info
- Add/Edit plant form with rich controls (icon picker, location chips, watering presets, light selector)
- Svelte stores for client-side state management
- Proper API error handling with consistent JSON error responses

**Non-Goals:**
- No watering schedule logic or due/overdue tracking (phase 3)
- No care log entries (phase 4)
- No photo upload (deferred — use Noto emoji icons for now)
- No MQTT publishing (phase 3)
- No search or filtering

## Decisions

### Decision 1: Integer primary keys with rowid

Use SQLite's auto-incrementing `INTEGER PRIMARY KEY` for plant and location IDs. This maps naturally to sqlx, avoids UUID overhead, and keeps URLs simple (`/api/plants/1`). SQLite optimizes rowid access.

**Alternative considered:** UUIDs — unnecessary complexity for a single-user local app.

### Decision 2: Shared API router module

Create `src/api/mod.rs` that builds a nested Axum `Router` with all API routes mounted under `/api`. The main router nests this under the root. This keeps API routes separate from static file serving.

```
src/api/
├── mod.rs        # Router: /api/plants/*, /api/locations/*
├── plants.rs     # Plant handlers (list, get, create, update, delete)
├── locations.rs  # Location handlers (list, create)
└── error.rs      # API error type → JSON responses
```

**Alternative considered:** Flat file with all handlers — gets unwieldy as endpoints grow.

### Decision 3: sqlx query macros with runtime checking

Use `sqlx::query_as!` and `sqlx::query!` for type-safe queries at runtime. Skip compile-time checking (`sqlx prepare`) for now since the schema is evolving rapidly. Add offline mode in a later phase when the schema stabilizes.

**Alternative considered:** Compile-time checking with `DATABASE_URL` — adds build complexity and CI friction during rapid iteration.

### Decision 4: Serde structs for API request/response

Define separate Rust structs for API input (`CreatePlant`, `UpdatePlant`) and output (`Plant`). Use `serde::Deserialize` for request bodies and `serde::Serialize` for responses. Validate required fields at the struct level using `Option` for optional fields.

### Decision 5: SvelteKit client-side routing with fetch API

Use SvelteKit's file-based routing for pages (`/`, `/plants/[id]`, `/plants/new`, `/plants/[id]/edit`). Use the browser `fetch` API in Svelte stores to call the REST API. No SSR — all pages load client-side since the app runs as an embedded SPA.

### Decision 6: Svelte stores for state management

Use Svelte's built-in `writable` stores for plants and locations. The stores expose async functions (`loadPlants`, `createPlant`, etc.) that call the API and update the store. Components subscribe reactively.

### Decision 7: Icon stored as emoji codepoint

Store the plant icon as a string field containing the emoji character (e.g., `🪴`). The frontend resolves this to the corresponding Noto SVG path at render time. This keeps the database simple and the icon set extensible.

**Alternative considered:** Store SVG filename — couples the DB to the asset naming convention.

### Decision 8: Watering interval as integer days

Store `watering_interval_days` as an integer on the plant. This is a simple interval that phase 3 will use to compute next-due dates. No schedule table needed yet — just the interval preference.

### Decision 9: Light needs as enum string

Store `light_needs` as a TEXT column with values `direct`, `indirect`, or `low`. Validate on the API side. This is simpler than a separate enum table and directly maps to the UI selector.

## Risks / Trade-offs

- **No offline query checking**: Queries are only validated at runtime. A typo in a query won't be caught until the endpoint is called. → Mitigated by integration tests for all endpoints.
- **No pagination**: Plant list returns all plants. → Acceptable for a personal plant manager (unlikely to exceed 100 plants). Add pagination if needed.
- **No photo upload**: Plants use emoji icons instead of real photos. → Keeps phase 2 scoped. Photo upload can be added independently later.
- **Integer IDs in URLs**: Exposes sequential IDs in the URL. → Not a concern for a single-user local network app.
