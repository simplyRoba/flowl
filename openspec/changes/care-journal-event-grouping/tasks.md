## 1. Grouping Utility

- [ ] 1.1 Create `ui/src/lib/careGrouping.ts` with `WateringGroup` type and `groupCareEvents()` function that scans events (newest-first), tracks per-plant watering streaks, and returns `Array<CareEvent | WateringGroup>`
- [ ] 1.2 Create `ui/src/lib/careGrouping.test.ts` with tests: consecutive waterings grouped, notes break streak, photos break streak, streak of 1 not grouped, streak of 2 grouped, interleaved plants tracked independently, non-watering event for same plant breaks streak, mixed scenario end-to-end

## 2. Shared Skeleton Loading Styles

- [ ] 2.1 Create `ui/src/lib/styles/skeletons.css` with shared `.shimmer` class and `@keyframes shimmer` (extracted from IdentifyPanel's scoped styles)
- [ ] 2.2 Import `skeletons.css` in `ui/src/routes/+layout.svelte` alongside other shared styles
- [ ] 2.3 Remove the now-redundant `.shimmer` and `@keyframes shimmer` styles from `ui/src/lib/components/IdentifyPanel.svelte`

## 3. i18n

- [ ] 3.1 Add translation keys for group summary text (e.g. "Watered {count} times, {from} - {to}") in en, de, es

## 4. Global Care Journal Integration

- [ ] 4.1 Update `ui/src/routes/care-journal/+page.svelte` to load all events (remove infinite scroll, sentinel observer, cursor pagination) by passing a high limit to `fetchAllCareEvents`
- [ ] 4.2 Add skeleton loading state (shimmer lines) shown while events are being fetched
- [ ] 4.3 Apply `groupCareEvents()` to the event list and render `WateringGroup` items as collapsible summary rows with chevron, count, and date range
- [ ] 4.4 Implement expand/collapse toggle (local state) that reveals individual watering entries inline below the summary

## 5. Plant Detail Integration

- [ ] 5.1 Update `ui/src/routes/plants/[id]/+page.svelte` to apply `groupCareEvents()` to the care events timeline
- [ ] 5.2 Render group summaries in the plant timeline (omit plant name since context is single-plant), with expand/collapse

## 6. Tests and Checks

- [ ] 6.1 Update `ui/src/tests/routes/care-journal/page.test.ts` for load-all behavior, skeleton loading, and group rendering
- [ ] 6.2 Update `ui/src/tests/routes/plants/[id]/page.test.ts` for group rendering in plant detail
- [ ] 6.3 Run `npm run check --prefix ui`, `npm run lint --prefix ui`, `npm run format --prefix ui`, `cargo fmt --check`, `cargo clippy -- -D warnings`
