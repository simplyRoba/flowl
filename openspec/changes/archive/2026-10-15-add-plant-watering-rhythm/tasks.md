# Tasks

This checklist supersedes the completed wave experiment and tracks finalization of the approved care-event timeline.

## 1. History and Layout

- [x] 1.1 Normalize all supported raw care-event types and select the current plant's trailing 28-day history. Test offsets, timezone-less UTC/SQLite/fractional timestamps, invalid/future/old/foreign records, equal instants, notes/photos, and non-mutation of journal data.
- [x] 1.2 Keep decorative icon heights stable and ground geometry independent of event density. Test true time spacing, fixed four-segment ground, exact stem/ground contact, clearance, sparse/dense data, and resize stability.
- [x] 1.3 Prevent overlapping icon footprints for simultaneous, same-day, and nearby events without shifting true time positions. Test deterministic placement, dense-cluster host growth, resize, removal/shrink, ground contact, and unaffected ordinary layout.

## 2. Production Background

- [x] 2.1 Promote the accepted icon/stem/ground renderer to `CareTimeline.svelte`, retaining theme tokens, soft foreground/boundary masking and contact shadows. Add no visible labels, legend, captions, state messages, translation keys, focus targets, or pointer interception.
- [x] 2.2 Add current-time/visibility/source updates, accessible journal/period/loading/error metadata, reduced-motion-safe presentation, and timer/observer/listener cleanup. Test absent/retained history, locale changes, optional foreground bindings, and no graph-owned network requests.
- [x] 2.3 Increase ground curvature slightly and add a 2200ms left-to-right reveal with time-ordered icon pops, finishing around 2600ms. Preserve collision clearance and true x positions, reveal once per plant visit, skip/cancel for reduced motion, and test update/teardown cleanup.

## 3. Integration and Cleanup

- [x] 3.1 Make the timeline the default plant-detail background. Retarget route regressions to care markers while preserving request/form ownership, successful authoritative plant updates, journal CRUD/grouping/notes/photos, AI, offline restrictions, and global-journal behavior.
- [x] 3.2 Remove the former legend's component/footer allowance and mobile heading padding. Verify normal hero-to-card spacing, growing wrapped headings, desktop stacked/mobile side-by-side actions, photo/fallback and AI enabled/disabled at supported widths.
- [x] 3.3 Remove all superseded wave components, models, geometry and tests, preview variants, query switches, and stale feature-specific references. Confirm no unused imports, obsolete code paths, or new dependencies remain.

## 4. Verification

- [x] 4.1 Inspect the default page in Chromium at 320, 390, 768, 1024, and 1440px in light/dark themes and English/German/Spanish. Check sparse/dense histories, stable ground shape, foreground readability, no legend gap, keyboard/touch controls, photo/AI dialogs, and retained-history/offline behavior.
- [x] 4.2 Validate both deltas and run the pre-review gate in order: `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `npm run format:check --prefix ui`, `npm run lint --prefix ui`, `npm run check --prefix ui`, and `cargo test`. Review the final diff for scope and spec consistency before requesting human review.
