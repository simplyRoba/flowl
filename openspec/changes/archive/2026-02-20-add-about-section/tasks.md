## 1. Backend — App Info Endpoint

- [x] 1.1 Add `GET /api/info` handler returning `{ version, repository, license }` using `env!()` compile-time macros
- [x] 1.2 Mount the info route in the API router
- [x] 1.3 Add integration test for `GET /api/info` verifying response shape and status 200

## 2. Frontend — About Section

- [x] 2.1 Add API helper function to fetch `/api/info`
- [x] 2.2 Add About section to the settings page displaying Version, Source (clickable link), and License rows
- [x] 2.3 Handle fetch failure by hiding the About section

## 3. Verification

- [x] 3.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
