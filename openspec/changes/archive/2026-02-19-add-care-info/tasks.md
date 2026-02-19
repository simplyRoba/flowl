## 1. Database Migration

- [x] 1.1 Create migration adding `difficulty`, `pet_safety`, `growth_speed`, `soil_type` as nullable TEXT columns to `plants`

## 2. Backend — Rust Structs and Validation

- [x] 2.1 Add `difficulty`, `pet_safety`, `growth_speed`, `soil_type` fields to `PlantRow`, `Plant`, `CreatePlant`, `UpdatePlant` structs
- [x] 2.2 Add care info enum validation function that checks values against allowed lists and returns 422 on invalid input
- [x] 2.3 Update `PLANT_SELECT` query to include the four new columns
- [x] 2.4 Update `create_plant` handler to accept and insert the new fields (with validation)
- [x] 2.5 Update `update_plant` handler to accept and update the new fields (with validation, nullable clearing via `null`)

## 3. Backend — Tests

- [x] 3.1 Add unit tests for care info enum validation (valid values, invalid values, null allowed)
- [x] 3.2 Add integration tests for create plant with care info fields
- [x] 3.3 Add integration tests for update plant setting and clearing care info fields
- [x] 3.4 Add integration test for invalid care info value returning 422

## 4. Frontend — API Types and Client

- [x] 4.1 Add `difficulty`, `pet_safety`, `growth_speed`, `soil_type` to `Plant` and `CreatePlant` TypeScript interfaces in `api.ts`

## 5. Frontend — PlantForm Care Info Section

- [x] 5.1 Add state variables for the four care info fields in `PlantForm.svelte`
- [x] 5.2 Initialize state from `initial` prop for edit form pre-fill
- [x] 5.3 Add "Care Info (optional)" form section with four labeled toggle-button sub-groups using the light-selector visual style
- [x] 5.4 Implement deselect behavior: tapping an active button clears the value to null
- [x] 5.5 Include care info fields in `handleSubmit` form data

## 6. Frontend — Plant Detail Care Info Card

- [x] 6.1 Rename the "Light" card to "Care Info" on the plant detail page
- [x] 6.2 Add conditional rows for each non-null care info field below the light needs row, with human-readable labels
- [x] 6.3 Verify card displays only the light needs row when no optional attributes are set

## 7. Quality Checks

- [x] 7.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` and fix any issues
- [x] 7.2 Verify UI build completes without errors (`npm run build` in `ui/`)
