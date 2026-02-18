## Why

Every page sets its own hardcoded `max-width` and responsive expansion independently, resulting in inconsistent content widths across the application. The journal and plant detail expand to 960px on wide screens while settings and form pages stay at 800px for no reason. Form pages have a redundant double-constraint (800px wrapper + 640px inner form). There are no shared CSS variables, so width values drift silently.

## What Changes

- Introduce three CSS custom properties for content width tiers (`narrow`, `default`, `wide`) with corresponding widescreen variants.
- Update the `>=1280px` breakpoint to override these variables to their expanded values.
- Migrate all page wrappers to reference the appropriate CSS variable instead of hardcoded pixel values.
- Remove the redundant inner `max-width` from `PlantForm.svelte` — the page wrapper handles width.
- Fix settings and form pages to expand on wide screens like all other content pages.

## Capabilities

### New Capabilities

_(none — this is a styling consistency fix, not a new capability)_

### Modified Capabilities

- `ui/shell`: Add requirements for global content-width CSS custom properties and the tiered width system that pages must follow.

## Impact

- `ui/src/routes/+layout.svelte` — add CSS custom properties to `:root` / scoped styles
- `ui/src/routes/+page.svelte` — migrate dashboard to `--content-width-wide`
- `ui/src/routes/log/+page.svelte` — migrate journal to `--content-width-default`
- `ui/src/routes/settings/+page.svelte` — migrate settings to `--content-width-default` (gains widescreen expansion)
- `ui/src/routes/plants/[id]/+page.svelte` — migrate detail to `--content-width-default`
- `ui/src/routes/plants/[id]/edit/+page.svelte` — migrate edit page to `--content-width-narrow`
- `ui/src/routes/plants/new/+page.svelte` — migrate new page to `--content-width-narrow`
- `ui/src/lib/components/PlantForm.svelte` — remove inner `max-width: 640px`
