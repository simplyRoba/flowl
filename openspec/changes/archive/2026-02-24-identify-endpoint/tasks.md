## 1. Type and Error Changes

- [x] 1.1 Add `Serialize` derive to `IdentifyResult` and `CareProfile` in `src/ai/types.rs` (add `use serde::Serialize` and update derive macros)
- [x] 1.2 Add `InternalError(String)` variant to `ApiError` in `src/api/error.rs`, mapping to HTTP 500 with JSON `{"message": "..."}`
- [x] 1.3 Add unit test for `IdentifyResult` JSON serialization round-trip (serialize then verify output contains expected fields)

## 2. Identify Endpoint Handler

- [x] 2.1 Add `identify_plant` handler in `src/api/ai.rs`: extract multipart `photos` fields, validate content types (JPEG/PNG/WebP), validate at least one photo present, call `AiProvider::identify`, return `Json<IdentifyResult>`
- [x] 2.2 Add error handling in handler: return `ServiceUnavailable` when AI provider is `None`, `Validation` for missing photos or invalid content type, `InternalError` for AI provider failures
- [x] 2.3 Register route `POST /api/ai/identify` in `src/api/mod.rs` with `DefaultBodyLimit::max(30 * 1024 * 1024)`

## 3. Tests

- [x] 3.1 Add integration test: `POST /api/ai/identify` returns 503 when AI is not configured
- [x] 3.2 Add integration test: `POST /api/ai/identify` returns 422 when no photos are provided
- [x] 3.3 Add integration test: `POST /api/ai/identify` returns 422 for invalid content type

## 4. Verification

- [x] 4.1 Run `cargo fmt`, `cargo clippy`, and `cargo test` and fix any issues
