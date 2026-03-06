## 1. Backend: multi-type filtering

- [x] 1.1 Add `axum-extra` dependency with the `query` feature to `Cargo.toml`
- [x] 1.2 Update `GlobalCareQuery` to use `Vec<String>` for `event_type` and switch `list_all_care_events` to `axum_extra::extract::Query`
- [x] 1.3 Update `list_all_care_events` to validate each type in the vec and build a dynamic `WHERE ce.event_type IN (?, ...)` clause
- [x] 1.4 Add/update Rust tests for multi-type filtering (single type, multiple types, invalid type in list, empty list)

## 2. Frontend: API client

- [x] 2.1 Change `fetchAllCareEvents` signature from `type?: string` to `types?: string[]` and append a `type` param per entry
- [x] 2.2 Update `api.test.ts` for the new `fetchAllCareEvents` signature

## 3. Frontend: care journal page multi-select + URL persistence

- [x] 3.1 Replace `activeFilter: string` with a `Set<string>` derived from `$page.url.searchParams.getAll('type')`
- [x] 3.2 Implement `toggleFilter(type)` that updates the URL via `goto()` with `replaceState: true`
- [x] 3.3 Implement "All" chip toggle logic (no filters → select all 6; some/all selected → clear)
- [x] 3.4 Wire the reactive filter set to `loadPage()` so it passes the types array to `fetchAllCareEvents`
- [x] 3.5 Update chip rendering to support multiple active chips and correct "All" chip appearance
- [x] 3.6 Update care journal component tests for multi-select, URL persistence, and "All" toggle behavior

## 4. Verification

- [x] 4.1 Run `npm run check` (in `ui/`), `cargo fmt`, `cargo clippy`, and `cargo test`
