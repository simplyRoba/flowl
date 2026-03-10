## 1. Error infrastructure

- [x] 1.1 Refactor `ApiError` enum in `src/api/error.rs` to carry `&'static str` error codes instead of `String` messages
- [x] 1.2 Add error code constants and a `default_message(code) -> &'static str` lookup function
- [x] 1.3 Update `IntoResponse` to return `{"code": "...", "message": "..."}` with message derived from code
- [x] 1.4 Add a `db_error(sqlx::Error) -> ApiError` helper that logs the real error and returns `INTERNAL_ERROR`
- [x] 1.5 Update `JsonBody` extractor rejection to use `INVALID_REQUEST_BODY` code

## 2. Migrate API modules

- [x] 2.1 Update `src/api/plants.rs`: replace all `BadRequest(e.to_string())` with `db_error`, assign specific codes to validation/not-found errors
- [x] 2.2 Update `src/api/care_events.rs`: same migration
- [x] 2.3 Update `src/api/locations.rs`: same migration, use `LOCATION_ALREADY_EXISTS` for conflicts
- [x] 2.4 Update `src/api/photos.rs`: same migration, use `PHOTO_*` codes for validation errors
- [x] 2.5 Update `src/api/settings.rs`: same migration, use `SETTINGS_INVALID_THEME`/`SETTINGS_INVALID_LOCALE`
- [x] 2.6 Update `src/api/backup.rs` and `src/api/restore.rs`: same migration, use `IMPORT_*` codes
- [x] 2.7 Update `src/api/ai.rs`: same migration, use `AI_*` codes
- [x] 2.8 Update `src/api/mqtt.rs`: use `MQTT_DISABLED`/`MQTT_UNAVAILABLE` codes
- [x] 2.9 Update `src/api/stats.rs`: same migration

## 3. Tests

- [x] 3.1 Update integration tests to assert on `code` field in error responses
- [x] 3.2 Add unit tests for `default_message` mapping and `db_error` helper
- [x] 3.3 Verify no sqlx error strings leak in any error response

## 4. Verification

- [x] 4.1 Run `cargo fmt`, `cargo clippy`, and `SKIP_UI_BUILD=1 cargo test`
- [x] 4.2 Mark B1 as done in `REVIEW.md`
