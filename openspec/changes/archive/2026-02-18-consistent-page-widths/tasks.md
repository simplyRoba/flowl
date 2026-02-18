## 1. Define CSS custom properties

- [x] 1.1 Add `--content-width-narrow: 640px`, `--content-width-default: 800px`, `--content-width-wide: 1200px` to `:global(:root)` in `ui/src/routes/+layout.svelte`
- [x] 1.2 Override the three variables to `720px`, `960px`, `1400px` inside the existing `@media (min-width: 1280px)` block (using `:global(:root)`)

## 2. Migrate pages to use tokens

- [x] 2.1 Dashboard (`ui/src/routes/+page.svelte`): replace hardcoded `max-width: 1200px` / `1400px` with `var(--content-width-wide)`, remove the page-level `@media (min-width: 1280px)` max-width override
- [x] 2.2 Care Journal (`ui/src/routes/log/+page.svelte`): replace hardcoded `max-width: 800px` / `960px` with `var(--content-width-default)`, remove the page-level `@media (min-width: 1280px)` max-width override
- [x] 2.3 Plant Detail (`ui/src/routes/plants/[id]/+page.svelte`): replace hardcoded `max-width: 800px` / `960px` with `var(--content-width-default)`, remove the page-level `@media (min-width: 1280px)` max-width override
- [x] 2.4 Settings (`ui/src/routes/settings/+page.svelte`): replace hardcoded `max-width: 800px` with `var(--content-width-default)` (gains widescreen expansion)
- [x] 2.5 Edit Plant (`ui/src/routes/plants/[id]/edit/+page.svelte`): replace hardcoded `max-width: 800px` with `var(--content-width-narrow)`, remove the page-level `@media (min-width: 1280px)` max-width override if present
- [x] 2.6 New Plant (`ui/src/routes/plants/new/+page.svelte`): replace hardcoded `max-width: 800px` with `var(--content-width-narrow)`, remove the page-level `@media (min-width: 1280px)` max-width override if present

## 3. Remove redundant inner constraint

- [x] 3.1 Remove `max-width: 640px` and `margin: 0 auto` from `.plant-form` in `ui/src/lib/components/PlantForm.svelte`

## 4. Verify

- [x] 4.1 Run the SvelteKit dev build (`npm run build` in `ui/`) to confirm no build errors
