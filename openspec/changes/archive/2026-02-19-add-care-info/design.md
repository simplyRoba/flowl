## Context

The `plants` table currently has a required `light_needs` TEXT column (default `indirect`) with three allowed values. The plant detail page renders this in a dedicated "Light" card, and the PlantForm uses toggle buttons where one is always active.

We are adding five more care attributes (`difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture`) that follow the same enum-style pattern but differ in that they are **optional** (nullable, no default). This affects form interaction (deselectable toggles) and detail display (conditional rows). The `soil_moisture` attribute is displayed on the Watering card rather than the Care Info card, since it relates to watering preferences.

## Goals / Non-Goals

**Goals:**
- Add five nullable enum columns to the plants table via migrations
- Expose them through the existing plant CRUD API (no new endpoints)
- Provide toggle-button form controls that support deselection (tap active = clear)
- Merge them into the existing Light card on the detail page, renamed to "Care Info"

**Non-Goals:**
- Filtering or searching plants by these attributes (future work)
- Validation of enum values at the database level (enforced in Rust handler only)
- Showing care info on dashboard grid cards (detail-level data only)
- Changing anything about `light_needs` behavior (stays required with default)

## Decisions

### 1. Nullable columns vs. separate table

**Decision**: Add columns directly to `plants` as nullable TEXT.

**Rationale**: These are fixed, single-value-per-plant attributes — not user-defined tags. A junction table would add complexity for no benefit. The `light_needs` column already establishes this pattern. Five nullable columns is simpler than five rows in a key-value table.

**Alternative considered**: A `plant_attributes` key-value table. Rejected because it adds a join, requires pivoting for API responses, and doesn't match the existing column-per-attribute pattern.

### 2. Migrations for new columns

**Decision**: Two migration files — one adding `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, and a second adding `soil_moisture` — via `ALTER TABLE`.

**Rationale**: The initial four columns were scoped together. `soil_moisture` was added in a follow-up migration when the feature scope expanded to include watering-related metadata. SQLite supports `ALTER TABLE ADD COLUMN` but only one column per statement.

### 3. Enum validation in Rust, not SQL

**Decision**: Validate allowed values in the Axum handler (create/update), not via SQL CHECK constraints.

**Rationale**: Consistent with how `light_needs` is handled today — there's no CHECK constraint on it either. Rust-side validation gives better error messages (HTTP 422 with specific field name). SQLite CHECK constraints would require a table rebuild for future value additions.

### 4. Form section: single card with sub-groups

**Decision**: One "Care Info (optional)" form section containing all five attributes as labeled toggle-button rows. Each row works independently: tap to select, tap again to deselect.

**Rationale**: Keeps the form compact. Five separate form sections would bloat the page. Grouping them signals they're related optional metadata. The "(optional)" suffix in the title differentiates from required sections.

### 5. Detail card: extend the Light card

**Decision**: Rename the existing "Light" card to "Care Info" and append rows for any set optional attributes below the light needs row.

**Rationale**: Avoids adding a new card to the grid. Light needs is always shown (it's required), so the card always has content. Optional attributes appear as additional rows only when set — no empty "N/A" rows.

## Risks / Trade-offs

- **Schema growth**: Adding columns to `plants` for each new attribute doesn't scale indefinitely. → Acceptable for 5 attributes; if we need 10+ in the future, revisit with a key-value approach.
- **No DB-level validation**: Invalid values could be inserted via direct DB access. → Acceptable for a self-hosted single-user app; Rust handlers are the only write path.
- **Form height increase**: The new section adds vertical space to the form. → Mitigated by keeping all five in one card and using compact toggle buttons.
