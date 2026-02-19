## Personas
- Developer: Maintains the Rust service, ships Docker images, and exposes plant care data for automation systems.
- End user: Runs the service to get plant care information from e.g. Home Assistant or simple HTTP clients.

## Git workflow
- Always branch from `main`; AI may create branches but must never merge or push to `main`.
- Keep branches short-lived, focused on a single change, and clearly named (e.g., `feat/watering-schedule` rather than `wip`).
- Commits should follow Conventional Commits (`feat`, `fix`, etc.) per https://www.conventionalcommits.org/en/v1.0.0/#specification.

## Review expectations
- Treat every change as pending until a human explicitly reviews it; nothing merges without that approval.
- Before requesting review, run `cargo fmt`, `cargo clippy`, and `cargo test` (includes UI tests) so artifacts and implementation stay in sync.

## Clarifications
- When requirements or intent are unclear, asking for information is mandatory and preferred over proceeding with assumptions. Use **AskUserQuestion tool** if feasable.

## Testing notes
- Test names should describe the state under test, not the desired result; assertions already express expected outcomes.
- `cargo test` runs both Rust and UI tests. The `tests/ui.rs` integration test shells out to `npm run test` in `ui/`. Use `cargo test -- --skip ui_tests` for Rust-only.
- UI test files are co-located with their source: store/utility tests sit next to the source file (e.g. `plants.ts` / `plants.test.ts`), component tests next to the component (e.g. `StatusBadge.svelte` / `StatusBadge.test.ts`), and page tests live under `ui/src/tests/routes/` mirroring the route structure.
- UI tests use vitest, `@testing-library/svelte`, and jsdom. Stores are tested by mocking `$lib/api`; components are rendered with `@testing-library/svelte`.

## Tooling constraints
- Rust work stays on the latest stable toolchain via `rustup`; do not depend on nightly-only features or pin a custom channel in `AGENTS.md`.
- If the CLI is used, prefer the bundled OpenSpec commands (`openspec status`, `openspec instructions`, etc.) that read from the current repo structure.

## Vocabulary
- **Change**: the scoped OpenSpec artifact you are working on (proposal → implementation → verification). A change captures one logical goal, outlines the affected capabilities/apis, and stores only delta requirements.
- **Requirement**: a concrete, testable statement in a change's ADDED/MODIFIED/REMOVED sections. Requirements should use RFC 2119 keywords and Given/When/Then structure so they can be verified during testing.
- **Capability**: the functional area covered by one or more requirements. Capabilities are represented by folders under `openspec/specs/{domain}/{capability}` and help teams scope work before implementation begins.
- **Domain**: the broader grouping for related capabilities (for example `core`, `ui`, `api`). Every new capability must be placed in the correct domain folder.

## Reference materials
- **Design**: `DESIGN.md` — overall application design, architecture, and technical decisions. Read this first to understand the project.
- **Plan**: `PLAN.md` — project roadmap and phased implementation plan. Check this to understand what has been done and what is next.
- **Specs**: `openspec/` — OpenSpec-based specs and change proposals for all features. Check `openspec/specs/` for current capability specs and `openspec/changes/` for active/archived change sets with task checklists.
- **UI Mockups**: `mockups/index.html` — HTML/CSS mockups for all UI screens (dashboard, detail, forms, settings). Open in a browser to view. Always match implemented UI to these mockups.
