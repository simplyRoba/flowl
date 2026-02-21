## 1. Dependencies and structs

- [x] 1.1 Add `zip` crate with `deflate` feature to `Cargo.toml`
- [x] 1.2 Define export data structs (`ExportData`, `ExportPlant`, `ExportLocation`, `ExportCareEvent`) with serde serialization, covering all columns including `last_watered` and care info fields (`difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture`)

## 2. Backend export endpoint

- [x] 2.1 Implement `GET /api/data/export` handler that queries all data, builds a ZIP archive containing `data.json` and photo files from the uploads directory, returns with `Content-Disposition: attachment`
- [x] 2.2 Add integration tests for export (empty database, populated database with photos, response headers, ZIP structure)

## 3. Backend import endpoint

- [x] 3.1 Implement `POST /api/data/import` handler with 100 MB body size limit that accepts a ZIP upload, fully validates (ZIP structure, JSON parsing, version check, filename sanitization) before modifying any existing data
- [x] 3.2 Implement transactional database replace: delete all existing data, insert locations, plants, and care events preserving original timestamps
- [x] 3.3 Implement photo replace: clear uploads directory and extract `photos/` entries from ZIP with path traversal protection
- [x] 3.4 Add integration tests for import (valid import with photos, invalid ZIP, missing data.json, version mismatch, path traversal rejection, atomicity on failure)
- [x] 3.5 Add round-trip integration test: export → import into clean DB → export again → compare the two JSON manifests are identical (excluding `exported_at`)

## 4. Wire routes and MQTT

- [x] 4.1 Register export and import routes in `src/api/mod.rs`
- [x] 4.2 Call MQTT repair after successful import to clear orphaned retained topics from pre-import plants and republish fresh state for all imported plants

## 5. Settings UI

- [x] 5.1 Add export button to Data section that triggers a ZIP file download via `GET /api/data/export`
- [x] 5.2 Add import button with file picker (`.zip` only) and confirmation dialog warning about data and photo replacement
- [x] 5.3 Handle import success (reload stats) and error (show error message) states

## 6. Tests and wrap-up

- [x] 6.1 Add UI tests for export button, import flow, confirmation dialog, and error handling
- [x] 6.2 Run `ui/npm run check`, `cargo fmt`, `cargo clippy`, and `cargo test`
