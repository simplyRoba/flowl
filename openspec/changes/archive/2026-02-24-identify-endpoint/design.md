## Context

The `AiProvider::identify` method is fully implemented in `OpenAiProvider` — it accepts image byte slices, encodes them as base64, sends them to the OpenAI-compatible API with JSON mode, and deserializes the response into `IdentifyResult`. However, there is no HTTP endpoint exposing this to the frontend. The existing photo upload endpoint (`POST /plants/{id}/photo`) already demonstrates the multipart extraction pattern using `axum::extract::Multipart`.

The response types `IdentifyResult` and `CareProfile` currently only derive `Deserialize` (used for parsing the AI response). They need `Serialize` to be returned as JSON from an HTTP handler.

`ApiError` has variants for `NotFound`, `Validation`, `Conflict`, `BadRequest`, and `ServiceUnavailable`, but lacks a generic internal error variant for unexpected failures like AI API errors or deserialization failures.

## Goals / Non-Goals

**Goals:**
- Expose `POST /api/ai/identify` accepting multipart photo uploads and returning the `IdentifyResult` as JSON
- Handle all error cases with appropriate HTTP status codes
- Follow existing patterns from the photo upload handler for multipart extraction
- Keep the endpoint stateless — no database interaction, no stored images

**Non-Goals:**
- Image storage or persistence (photos are only forwarded to the AI API)
- Image resizing or optimization before sending to the AI
- Rate limiting or usage tracking
- Authentication/authorization (the app has no auth layer)
- Frontend UI for identification (Phase 4)

## Decisions

### 1. Multipart field name: `photos`

Use `photos` as the multipart field name, matching the PLAN.md API spec (`photos[]: <file>`). Accept 1–3 files. The handler iterates `Multipart::next_field()` collecting all fields named `photos` or `photo`, reading each into a `Vec<u8>`.

**Why not JSON with base64?** Multipart is the standard for file uploads, avoids 33% base64 size overhead on the wire, and matches the existing `upload_photo` pattern. The base64 encoding happens server-side only when building the AI API payload.

### 2. Reuse existing `Multipart` extractor from axum

Axum's built-in `Multipart` extractor (already used in `photos.rs`) handles streaming multipart parsing. No new dependencies needed.

### 3. Add `InternalError` variant to `ApiError`

Add `InternalError(String)` mapping to HTTP 500. This covers AI API failures (network errors, non-200 responses) and deserialization errors (AI returned unparseable JSON). The existing `ServiceUnavailable` is used when AI is not configured.

**Why not reuse `BadRequest`?** A 400 implies the client sent something wrong. When the AI API fails or returns garbage, that's a server-side issue — 500 is semantically correct.

### 4. Add `Serialize` to `IdentifyResult` and `CareProfile`

Add `#[derive(Serialize)]` alongside the existing `Deserialize`. These types serve double duty: deserialize the AI response, then serialize as the HTTP response. No structural changes.

### 5. Body size limit

Apply `DefaultBodyLimit::max(30 * 1024 * 1024)` (30 MB) to the identify route. With up to 3 photos at ~5 MB each plus multipart overhead, 30 MB is sufficient. This follows the same pattern used for the import endpoint.

### 6. Content type validation

Accept `image/jpeg`, `image/png`, and `image/webp` — the same set as the existing photo upload handler. Reject other content types with a 422 validation error. This prevents sending non-image data to the vision API.

### 7. Error mapping

| Condition | HTTP Status | ApiError variant |
|-----------|-------------|-----------------|
| AI not configured (no API key) | 503 | `ServiceUnavailable` |
| No photos in request | 422 | `Validation` |
| Invalid content type | 422 | `Validation` |
| AI API network/HTTP error | 500 | `InternalError` |
| AI response not parseable | 500 | `InternalError` |

## Risks / Trade-offs

**[Large payloads to AI API]** → Up to 3 images are base64-encoded and sent in a single API request. This could be slow on large images. Mitigation: the 5 MB per-file limit from content type validation keeps individual images reasonable; the AI API itself enforces its own limits.

**[AI API latency]** → The identify call blocks until the AI API responds, which can be several seconds. Mitigation: this is inherent to the feature; the frontend will show a loading state (Phase 4). No timeout is added now — `reqwest`'s default is sufficient and the AI API may legitimately take time for vision tasks.

**[No retry on failure]** → If the AI API returns a transient error, the request fails. Mitigation: the user can simply retry from the UI. Adding retry logic would add complexity for minimal gain in this interactive use case.
