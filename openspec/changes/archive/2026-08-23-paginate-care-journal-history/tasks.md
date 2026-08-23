## 1. Branch and Test-First Coverage

- [x] 1.1 Create a short-lived `feat/paginate-care-journal-history` branch from `main` without merging or pushing, and verify it with `git branch --show-current`.
- [x] 1.2 Add backend integration tests for malformed create timestamps, a 500-event page, the 500-event maximum, `has_more`, an unknown cursor, backdated IDs, equal timestamps, supported offset/legacy timestamp representations, and filtered continuation; verify the new tests fail against the previous validation/cursor behavior with `cargo test --test care_events`.
- [x] 1.3 Add grouping unit tests for partial multi-member and one-member boundary streaks, completion at a breaker/end of history, stable newest-event keys, and expansion-relevant identity after appending; verify the new tests initially fail with `npm run test --prefix ui -- ui/src/lib/careGrouping.test.ts`.
- [x] 1.4 Add route tests for the initial `limit=500` request, manual continuation without document overflow, append/end-of-history behavior, partial group text, and preserved loaded content during continuation; verify the new tests initially fail with `npm run test --prefix ui -- ui/src/tests/routes/care-journal/page.test.ts`.
- [x] 1.5 Add route tests with mocked intersection/resize observers for overflow-triggered loading, leave/re-enter rearming, duplicate-trigger guarding, observer cleanup, continuation failure/manual retry, and stale responses after filter changes; verify the new tests initially fail with the single care-journal test command.

## 2. Chronological Backend Pagination

- [x] 2.1 Add an append-only expression-index migration for chronological `occurred_at` plus `id` ordering, and verify a fresh test database applies all migrations through `cargo test --test care_events global_respects_limit`.
- [x] 2.2 Validate explicit occurrence timestamps on creation, raise the global endpoint maximum page size to 500, and resolve `before` to its normalized chronological `occurred_at` plus `id` boundary, returning a JSON HTTP 422 validation error for an unknown event; verify the focused create, limit, cursor, timestamp-representation, backdated, equal-timestamp, and filter tests pass.
- [x] 2.3 Confirm the backend still defaults to 20 events, fetches one extra row for `has_more`, and preserves the existing request/response shape; verify the complete `cargo test --test care_events` target passes.

## 3. Partial Watering Groups

- [x] 3.1 Extend `WateringGroup` and `groupCareEvents` with a newest-event-based stable key, partial state, and awareness of whether older history remains; verify all grouping unit tests pass, including unchanged complete grouping on plant detail data.
- [x] 3.2 Emit unresolved one-member and multi-member boundary streaks as partial groups, then recompute them correctly when pages append or history ends; verify tests cover every per-plant boundary transition without regressing interleaved-plant behavior.
- [x] 3.3 Add English, German, and Spanish translations for partial counts/continuation and continuation loading, retry, refresh, and “Load older entries” labels; verify locale typing and translation tests pass with `npm run check --prefix ui` and the relevant UI tests.

## 4. Hybrid Journal Loading UI

- [x] 4.1 Replace `loadAllEvents` with a 500-event reset/continuation loader that tracks `has_more`, separate loading states, continuation errors, request generations, and defensive ID deduplication; verify route tests cover append order, filter reset, stale-response rejection, and concurrent-trigger guards.
- [x] 4.2 Render a keyboard-accessible continuation button whenever `has_more` is true, preserving the loaded timeline and showing busy/error/retry or deleted-cursor refresh states during continuation; verify manual loading and refresh recovery work with a non-overflowing document and the control disappears at end of history.
- [x] 4.3 Use the continuation button as an `IntersectionObserver` sentinel only while the document scrolling element overflows, recalculate on page append/group expansion/viewport resize, and clean up observers/listeners on reset and destroy; verify observer tests prove compacted non-scrolling content never auto-drains pages.
- [x] 4.4 Pass continuation state into grouping, render partial summaries with `{count}+` and loaded date ranges, and key expansion by the stable group key; verify an expanded partial group stays expanded and gains members after continuation.
- [x] 4.5 Run the complete care-journal route and grouping test files and verify initial skeletons, filters, day grouping, complete groups, partial groups, manual fallback, infinite scrolling, retry behavior, and empty/end states all pass.

## 5. Documentation and Integration Review

- [x] 5.1 Review `README.md` for any end-user statement affected by progressive journal loading, update only if necessary, and verify any documentation describes behavior rather than implementation details.
- [x] 5.2 Review the final API/UI implementation against both delta specs, including the unchanged public `before` parameter and response fields, and verify `openspec validate paginate-care-journal-history --strict` passes.

## 6. Pre-Review Gate

- [x] 6.1 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, and `npm run check --prefix ui`; resolve every reported issue and rerun until all commands pass.
- [x] 6.2 Run `cargo test` as the final full-suite gate, confirm the working tree contains only intentional change files, and leave all changes pending for human review without merging or pushing to `main`.

## 7. Verification Follow-ups

- [x] 7.1 Add deterministic ordering for malformed historical timestamps in the index, cursor, and query, with a regression test proving every row remains reachable.
- [x] 7.2 Render and test complete loaded `from`–`to` date ranges for exact and partial global watering groups in all supported locales.
- [x] 7.3 Align the delta specs with the intentionally chevron-based disclosure control and the actual repeated-query-key parser behavior.
- [x] 7.4 Add multi-type filtered pagination coverage and assert rejected malformed creation leaves the care-event list empty.
- [x] 7.5 Re-run strict OpenSpec validation and the complete format, lint, type-check, and test gates.
