## Why

Plants have care-relevant traits beyond light needs (difficulty, pet toxicity, growth speed, soil preference) that users want to record and see at a glance. These optional attributes extend the existing `light_needs` pattern to give a richer plant profile without adding complexity to the required fields.

## What Changes

- Add four nullable columns to the `plants` table: `difficulty`, `pet_safety`, `growth_speed`, `soil_type`
- Each column is an enum-style TEXT with defined allowed values, defaulting to NULL (unset)
- Extend plant CRUD API to accept and return the new fields
- Extend the PlantForm with a new "Care Info (optional)" section using toggle buttons that can be deselected
- Rename the detail page "Light" card to "Care Info" and display any set attributes as additional rows alongside the existing light needs row
- No MQTT changes — these are display-only metadata

## Capabilities

### New Capabilities

_None — all changes fit within existing capabilities._

### Modified Capabilities

- `data/plants`: Add four nullable columns (`difficulty`, `pet_safety`, `growth_speed`, `soil_type`) to schema, CRUD queries, and API response/request types
- `ui/plants`: Add "Care Info (optional)" toggle section to PlantForm; rename detail "Light" card to "Care Info" and show set attributes as rows

## Impact

- **Database**: One new migration adding 4 nullable TEXT columns to `plants`
- **Backend**: `PlantRow`, `Plant`, `CreatePlant`, `UpdatePlant` structs gain 4 optional fields; `PLANT_SELECT` query updated; no new routes
- **Frontend**: `PlantForm.svelte` gains a new form section; detail page `+page.svelte` renames Light card and adds conditional rows; `api.ts` types updated
- **No breaking changes** — all new fields are optional/nullable; existing API consumers are unaffected
