## Context

The API currently has six `ApiError` variants (`NotFound`, `Validation`, `Conflict`, `BadRequest`, `ServiceUnavailable`, `InternalError`), each carrying a `String` message. The `IntoResponse` implementation returns `{"message": "..."}` with a corresponding HTTP status code.

The problem is twofold:

1. All `sqlx` errors are mapped to `BadRequest(e.to_string())`, so database failures (500-class) surface as 400s and leak internal details to clients.
2. The `message` is the only identifier. The frontend cannot distinguish between two different 422 errors without parsing English strings.

There are ~80 error sites across 10 API modules.

## Goals / Non-Goals

**Goals:**

- Every API error response includes a stable `code` field that uniquely identifies the error.
- Database/IO errors become `InternalError` with a generic code; real errors are logged server-side.
- The `message` field remains as a human-readable English fallback for API consumers and debugging.
- No new dependencies.

**Non-Goals:**

- Frontend i18n mapping (covered by `frontend-error-mapping` change).
- Structured/machine-readable error details beyond the code (e.g., field-level error arrays).
- Changing HTTP status codes for existing validation/not-found semantics.

## Decisions

### 1. Static error codes as `&'static str`

The `ApiError` enum carries `&'static str` codes instead of `String` messages. Codes are `SCREAMING_SNAKE_CASE` constants defined in `error.rs`.

**Why over String:** Compile-time guarantee that only known codes are used. No allocations. Impossible to accidentally leak dynamic content.

**Why over a typed enum per code:** A dedicated enum with 30+ variants would be large and require match arms everywhere. Static string constants are simpler, greppable, and easy to extend. The frontend treats them as opaque keys anyway.

### 2. Response shape: `{"code": "...", "message": "..."}`

Both fields always present. `code` is the stable contract. `message` is derived from code via a `fn default_message(code) -> &'static str` lookup.

**Why keep message:** Curl users, logs, and API consumers outside the frontend benefit from a readable string. Removing it would hurt debuggability.

**Why derive instead of passing separately:** Guarantees message and code stay in sync. One source of truth.

### 3. DB errors become `InternalError`

All `.map_err(|e| ApiError::BadRequest(e.to_string()))` on sqlx calls become:

```rust
.map_err(|e| {
    tracing::error!("DB error: {e}");
    ApiError::InternalError("INTERNAL_ERROR")
})
```

This is a bulk find-and-replace. The real error is logged; the client sees a generic 500.

**Exception:** The `JsonBody` extractor rejection stays `BadRequest("INVALID_REQUEST_BODY")` since that is genuinely a client error.

### 4. Error code naming convention

`ENTITY_REASON` pattern: `PLANT_NOT_FOUND`, `LOCATION_ALREADY_EXISTS`, `PHOTO_TOO_LARGE`.

Generic codes for cross-cutting concerns: `INTERNAL_ERROR`, `INVALID_REQUEST_BODY`.

### 5. Helper for DB error mapping

A small helper avoids repeating the log-and-convert pattern:

```rust
fn db_error(e: sqlx::Error) -> ApiError {
    tracing::error!("Database error: {e}");
    ApiError::InternalError("INTERNAL_ERROR")
}
```

This keeps each call site to `.map_err(db_error)?`.

## Risks / Trade-offs

- **Breaking API change** -> No external consumers exist; the frontend is the only client and will be updated in the companion change. No migration needed.
- **Losing error detail in 500s** -> The detail moves to server logs. Clients should never see sqlx internals. This is strictly better for security.
- **Large diff touching every API module** -> Mechanical replacement. Each file can be done independently. Tests catch regressions.
- **Code-to-message mapping maintenance** -> Adding a new error code requires adding one constant and one match arm. Low overhead.
