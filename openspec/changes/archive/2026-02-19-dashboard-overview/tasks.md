## 1. Dynamic Greeting Subtitle

- [x] 1.1 Add attention subtitle message pool (plural and singular variants) to `+page.svelte` and derive the subtitle from the attention count when N > 0, keeping the existing random time-of-day subtitle when all plants are ok
- [x] 1.2 Add tests for dynamic subtitle: plants needing attention shows attention message with correct count, all-ok shows existing random subtitle, no plants shows existing subtitle

## 2. Needs Attention Section

- [x] 2.1 Add "Needs Attention" section markup to `+page.svelte` between greeting and "All Plants" grid — filter `$plants` for `due`/`overdue`, sort overdue before due, render attention cards with photo/icon fallback, name, `StatusBadge`, and Water button
- [x] 2.2 Add styles for the attention section: bordered card container, section title with `alert-triangle` icon, 2-column attention card grid (desktop), single column (mobile), Water button with icon+label (desktop) and icon-only (mobile)
- [x] 2.3 Hide the "Needs Attention" section when no plants are due/overdue or when no plants exist

## 3. Inline Water Action

- [x] 3.1 Wire the Water button to the existing `waterPlant` store action with per-card loading state tracking so the button shows a loading indicator while the request is in flight
- [x] 3.2 After watering, the store reactivity removes the plant from the attention section if its status becomes `ok`

## 4. Tests

- [x] 4.1 Add dashboard tests: attention section renders when plants are due/overdue, hidden when all ok, hidden when no plants, overdue cards appear before due cards
- [x] 4.2 Add dashboard tests: Water button calls `waterPlant`, loading state during request, plant removed from attention section after successful watering
- [x] 4.3 ~~Add dashboard tests: responsive layout~~ — skipped: CSS media queries are not testable in jsdom; verified visually via mockup

## 5. Verify

- [x] 5.1 Run `npm run check` in `ui/` and `npm test` in `ui/` to verify no regressions
- [x] 5.2 Run `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test` to verify full build
