## Context

The application layout delegates content-width control entirely to individual pages. Each page independently declares its own `max-width` with hardcoded pixel values and `margin: 0 auto` centering. This has led to three inconsistencies:

1. Settings and form pages do not expand on widescreen (≥1280px) while journal and detail pages do.
2. Form pages use a double-constraint: 800px page wrapper with a 640px inner form.
3. No CSS variables exist for widths — values are scattered and drift silently.

The existing `:root` block in `+layout.svelte` already defines semantic CSS variables for typography, radii, and motion. Width tokens are a natural extension of this pattern.

## Goals / Non-Goals

**Goals:**
- Define a three-tier width token system (`narrow`, `default`, `wide`) as CSS custom properties.
- Ensure all content pages use these tokens consistently.
- Fix the inconsistent widescreen expansion behavior.
- Remove the redundant double-constraint on form pages.

**Non-Goals:**
- Changing the dashboard's multi-column grid layout or card sizing.
- Creating a shared wrapper component — pages keep their own wrapper elements, just referencing the variable.
- Changing mobile or tablet layout behavior.
- Adjusting the layout-level `.content` padding — that remains as-is (24px / 32px / 16px).

## Decisions

### 1. Three width tiers as CSS custom properties

**Decision:** Define `--content-width-narrow`, `--content-width-default`, `--content-width-wide` in `:root`, overridden at the `≥1280px` breakpoint.

| Tier | Default | ≥1280px | Use case |
|---|---|---|---|
| `narrow` | `640px` | `720px` | Form-centric pages (new/edit plant) |
| `default` | `800px` | `960px` | Single-column content (journal, detail, settings) |
| `wide` | `1200px` | `1400px` | Multi-column grids (dashboard) |

**Rationale:** Three tiers cover all current pages. Adding variables to the existing `:root` block keeps the pattern consistent with the typography/radii/motion tokens already there. The widescreen values match what journal and detail pages already use, so those pages see no visual change.

**Alternative considered:** A single `--content-width` variable set per-page via inline styles or data attributes. Rejected because it pushes the value back into each page and loses the semantic naming that communicates intent.

### 2. Variables in `:root`, not in `.content`

**Decision:** Place the CSS custom properties in the `:global(:root)` block alongside existing tokens.

**Rationale:** The variables are design tokens, not layout mechanics. They belong with the other tokens. Pages reference them via `var(--content-width-default)`.

### 3. Remove PlantForm inner max-width

**Decision:** Remove `max-width: 640px` from `PlantForm.svelte`. The page wrapper (`--content-width-narrow`) at 640px handles the constraint.

**Rationale:** The double-constraint creates visual inconsistency — forms appear narrower than expected. With the page wrapper at 640px, the form fills its container naturally.

### 4. Page-tier assignment

| Page | Tier | Change from current |
|---|---|---|
| Dashboard (`/`) | `wide` | None |
| Care Journal (`/log`) | `default` | None |
| Plant Detail (`/plants/[id]`) | `default` | None |
| Settings (`/settings`) | `default` | Gains widescreen expansion (800→960) |
| Edit Plant (`/plants/[id]/edit`) | `narrow` | Was 800px outer, now 640px/720px single constraint |
| New Plant (`/plants/new`) | `narrow` | Was 800px outer, now 640px/720px single constraint |

## Risks / Trade-offs

- **Form pages get narrower on default screens** (800px → 640px wrapper). This is intentional — the form content was already 640px via PlantForm's inner constraint, so the visible content width does not change. The only difference is the page wrapper is now tighter, eliminating dead whitespace between the 800px wrapper and the 640px form. → Acceptable, no visual regression.

- **Settings page gets wider on ≥1280px** (800px → 960px). This is the fix, not a risk. All single-column content pages should behave consistently. → Desired outcome.
