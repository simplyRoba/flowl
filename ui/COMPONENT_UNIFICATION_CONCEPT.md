# UI Component Unification Concept

## Current State

The codebase has **~25 distinct button styles** and **~6 input styles** defined across 11 files. They share the same design language but diverge in specifics: padding, font-size, hover effects, disabled opacity, border-radius, and transition properties all vary for components that serve the same role.

### Inconsistencies at a Glance

| Issue | Examples |
|---|---|
| **Padding** | Primary buttons use `12px 24px`, `8px 20px`, and `10px 18px` in three places |
| **Font-size** | Hardcoded `13px` vs `--fs-chip` vs `--fs-btn` vs `--fs-input` used interchangeably |
| **Hover mechanism** | `filter: brightness()` vs `background` swap vs `border-color` change |
| **Disabled opacity** | Five values: `0.4`, `0.5`, `0.6`, `0.7`, or none at all |
| **Border-radius** | `16px` hardcoded vs `--radius-pill` vs `10px` hardcoded for chips |
| **Transitions** | Some animate `border-color`, some don't; one hardcodes `0.15s` instead of using `--transition-speed` |
| **Chip active state** | Solid fill (green bg + white text) in some places, 10% tint (light green bg + green text) in others |
| **Icon button size** | `40px` in plant detail vs `36px` in settings for the same role |
| **Input font-size** | `--fs-input` (15px) in forms, `--fs-btn` (14px) in journal, `--fs-chip` (13px) in location chips, hardcoded `18px` in stepper |
| **Input focus transition** | Two inputs animate border-color on focus, three don't |
| **Missing explicit colors** | Some inputs don't set `background`/`color`, relying on inheritance (fragile for dark mode) |

---

## Components to Unify

### 1. Button

All `<button>` and `<a>` elements that look like buttons. Currently 25+ unique style classes.

**Required variants:**

| Variant | Purpose | Current examples |
|---|---|---|
| **primary** | Main CTA (save, add plant) | `.save-btn`, `.add-btn` |
| **water** | Water action | `.water-btn`, `.detail-water-btn` |
| **danger** | Destructive actions | (currently only used as hover state on icon buttons) |
| **outline** | Secondary actions (cancel, backdate) | `.log-cancel`, `.log-when-toggle`, `.media-switch` |
| **ghost** | Minimal, link-like buttons | `.add-log-link` |
| **icon** | Square icon-only button with border | `.action-btn`, `.edit-btn`, `.delete-btn` |
| **icon-ghost** | Small icon-only, no border | `.event-delete`, `.photo-remove-btn` |

**Required sizes:**

| Size | Use case |
|---|---|
| **sm** | Inline/compact contexts (card actions, log form) |
| **md** (default) | Standard buttons (save, cancel) |
| **lg** | Hero actions (detail page water button) |

**Required modifiers:**
- `disabled` — uniform `opacity: 0.6`, `cursor: not-allowed`
- `full-width` — stretches to container (mobile CTAs)
- `icon + text` — consistent gap and alignment

### 2. Chip (selection toggle)

Toggle buttons used for multi-option selection. Currently 7 different style classes.

**Required variants:**

| Variant | Purpose | Current examples |
|---|---|---|
| **chip** (default) | Rounded pill, border, 10% tint when active | `.location-chip`, `.care-option`, `.light-option`, `.interval-preset` |
| **chip-solid** | Rounded pill, solid fill when active | `.type-chip`, `.filter-chip`, `.theme-option` |

Both variants should share: font-size, padding, border-radius, transition, hover effect. They only differ in active state treatment.

### 3. Input

Text inputs and textareas. Currently 6 different style variations.

**Required variants:**

| Variant | Purpose | Current examples |
|---|---|---|
| **input** (default) | Standard text field | `.form-input` (name, species) |
| **input compact** | Smaller padding for inline/tight contexts | `.log-input`, `.log-notes`, `.new-input` |
| **textarea** | Multi-line, resizable | `.form-input.textarea`, `.log-notes` |
| **stepper** | Number input in stepper widget | `.stepper-value` (special, may stay custom) |

**Shared properties that should be consistent:**
- `background: var(--color-surface)`
- `color: var(--color-text)`
- `border: 1px solid var(--color-border)`
- `border-radius: var(--radius-btn)`
- `outline: none`
- `font-family: inherit`
- `transition: border-color var(--transition-speed)`
- Focus: `border-color: var(--color-primary)`
- Error: `border-color: var(--color-danger)`
- `::placeholder { color: var(--color-text-muted) }`

---

## Approach Options

### Option A: Global CSS Classes (Recommended)

Define shared styles as `:global()` classes in a dedicated CSS file (e.g. `ui/src/lib/styles/components.css`) imported in the root layout, or in `:global()` blocks within `+layout.svelte`.

```css
/* Button base */
:global(.btn) {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-weight: 500;
  font-size: var(--fs-btn);
  border-radius: var(--radius-btn);
  padding: 8px 20px;
  border: none;
  cursor: pointer;
  transition: background var(--transition-speed), border-color var(--transition-speed), color var(--transition-speed);
}
:global(.btn:disabled) {
  opacity: 0.6;
  cursor: not-allowed;
}

/* Variants */
:global(.btn-primary) {
  background: var(--color-primary);
  color: var(--color-text-on-primary);
}
:global(.btn-primary:hover:not(:disabled)) {
  background: var(--color-primary-dark);
}

:global(.btn-water) { ... }
:global(.btn-outline) { ... }
:global(.btn-ghost) { ... }
:global(.btn-icon) { ... }

/* Sizes */
:global(.btn-sm) { padding: 6px 14px; font-size: var(--fs-chip); }
:global(.btn-lg) { padding: 10px 24px; }

/* Modifiers */
:global(.btn-full) { width: 100%; justify-content: center; }
```

Usage in components:
```svelte
<button class="btn btn-primary" disabled={saving}>Save</button>
<button class="btn btn-water btn-lg btn-full">Water now</button>
<button class="btn btn-outline btn-sm">Cancel</button>
<button class="btn btn-icon"><Pencil size={16} /></button>
```

**Pros:**
- Zero runtime overhead — pure CSS
- Familiar pattern (BEM-like utility classes)
- Easy to adopt incrementally (migrate one button at a time)
- Works with both `<button>` and `<a>` elements
- No component imports needed, no prop APIs to learn
- Svelte's scoped styles don't interfere since these are `:global()`
- Component-specific layout concerns (margins, positioning) stay in scoped styles

**Cons:**
- Global namespace — class name collisions possible (mitigated with `.btn-` prefix)
- No enforcement — nothing prevents using old one-off classes alongside new ones
- No type safety on variant/size combinations
- Slightly less discoverable than component props (need to know class names)

---

### Option B: Svelte Components

Create `<Button>`, `<Chip>`, `<Input>` components in `$lib/components/` with typed props.

```svelte
<!-- Button.svelte -->
<script lang="ts">
  import type { Snippet } from 'svelte';

  interface Props {
    variant?: 'primary' | 'water' | 'danger' | 'outline' | 'ghost' | 'icon' | 'icon-ghost';
    size?: 'sm' | 'md' | 'lg';
    full?: boolean;
    disabled?: boolean;
    type?: 'button' | 'submit';
    onclick?: (e: MouseEvent) => void;
    children: Snippet;
  }

  let {
    variant = 'primary',
    size = 'md',
    full = false,
    disabled = false,
    type = 'button',
    onclick,
    children,
  }: Props = $props();
</script>

<button
  {type}
  {disabled}
  {onclick}
  class="btn btn-{variant} btn-{size}"
  class:btn-full={full}
>
  {@render children()}
</button>

<style>
  .btn { /* base styles */ }
  .btn-primary { /* ... */ }
  /* etc. */
</style>
```

Usage:
```svelte
<Button variant="primary" disabled={saving}>Save</Button>
<Button variant="water" size="lg" full>Water now</Button>
<Button variant="outline" size="sm">Cancel</Button>
<Button variant="icon" onclick={handleEdit}><Pencil size={16} /></Button>
```

**Pros:**
- Type-safe props — IDE autocomplete and compile-time checks for variants/sizes
- Scoped styles — no global namespace pollution
- Enforced consistency — can only use defined variants
- Single source of truth for both markup structure and styles
- Easier to add behavior (loading states, icons) later

**Cons:**
- Requires importing the component everywhere it's used
- `<a>` styled as buttons need a separate component or an `href` prop with conditional rendering
- Props API surface grows over time (every new attribute needs forwarding: `form`, `aria-label`, etc.)
- Small runtime overhead (component instantiation, though negligible in practice)
- Need to handle event forwarding and attribute spreading carefully in Svelte 5

---

### Option C: Hybrid — CSS Classes + Thin Svelte Wrappers (Alternative)

Define the visual styles as global CSS classes (like Option A) but also provide optional Svelte components that compose those classes for convenience and type safety.

```css
/* components.css — visual styles */
:global(.btn) { ... }
:global(.btn-primary) { ... }
```

```svelte
<!-- Button.svelte — thin wrapper -->
<script lang="ts">
  /* ... typed props ... */
</script>

<button class="btn btn-{variant} btn-{size}" class:btn-full={full} {disabled} {onclick}>
  {@render children()}
</button>

<!-- No <style> block — uses global classes -->
```

Both usages are valid:
```svelte
<!-- With component (recommended for new code) -->
<Button variant="primary">Save</Button>

<!-- With raw classes (for migration, for <a> tags, for edge cases) -->
<a href="/plants/new" class="btn btn-primary">Add plant</a>
```

**Pros:**
- Best of both worlds: type safety when you want it, raw classes when you need them
- `<a>` elements can use the same styles without a wrapper component
- Gradual migration — old code works with classes, new code uses components
- Styles are decoupled from components (easier testing, easier sharing)

**Cons:**
- Two ways to do the same thing — need team discipline to converge
- More files to maintain (CSS file + component file)
- Global CSS classes still have the namespace caveat

---

## Recommendation

**Option A (Global CSS Classes)** is the best fit for this project because:

1. **The project already uses CSS variables as its design system** — global CSS classes are the natural extension of this pattern
2. **No component library is in use** — introducing wrapper components adds a layer of abstraction that doesn't match the current authoring style (plain HTML elements with scoped styles)
3. **Buttons are used as `<a>` tags in several places** (`.add-btn` on the dashboard, `.edit-btn` in detail header) — CSS classes work on any element, components don't
4. **Incremental migration** — each file can be updated independently without changing any imports or component APIs
5. **Minimal overhead** — no new dependencies, no component instantiation, no prop spreading concerns
6. **Small team / solo project signals** — the codebase shows consistent single-author patterns; a lightweight class-based system is easier to maintain than a component API

If the project grows and more people contribute, Option C (hybrid) is a natural evolution: add thin Svelte wrappers on top of the same CSS classes later without changing any existing code.

---

## Implementation Plan

### Step 1: Create the shared styles file

Create `ui/src/lib/styles/components.css` with all button, chip, and input classes.

### Step 2: Import in root layout

```svelte
<!-- +layout.svelte -->
<script>
  import '$lib/styles/components.css';
</script>
```

### Step 3: Define the classes

**Buttons** — base `.btn` + variants `.btn-primary`, `.btn-water`, `.btn-danger`, `.btn-outline`, `.btn-ghost`, `.btn-icon`, `.btn-icon-ghost` + sizes `.btn-sm`, `.btn-lg` + modifiers `.btn-full`.

Standardized values:
- Padding: `8px 20px` (md), `6px 14px` (sm), `10px 24px` (lg)
- Font-size: `var(--fs-btn)` (md/lg), `var(--fs-chip)` (sm)
- Border-radius: `var(--radius-btn)` (default), `var(--radius-pill)` (chips)
- Disabled: `opacity: 0.6; cursor: not-allowed;` (universal)
- Hover: `background` change (never `filter: brightness`)
- Transition: `background var(--transition-speed), border-color var(--transition-speed), color var(--transition-speed)`

**Chips** — base `.chip` + `.chip-solid`. Both pill-shaped, same padding/font/radius, differ only in active state.

Standardized values:
- Padding: `6px 14px`
- Font-size: `var(--fs-chip)`
- Font-weight: `500`
- Border-radius: `var(--radius-pill)`
- Default: `background: var(--color-surface); border: 1px solid var(--color-border); color: var(--color-text-muted)`
- Hover: `border-color: var(--color-primary)`
- Active (`.chip`): `background: color-mix(in srgb, var(--color-primary) 10%, transparent); color: var(--color-primary); border-color: var(--color-primary)`
- Active (`.chip-solid`): `background: var(--color-primary); color: var(--color-text-on-primary); border-color: var(--color-primary)`

**Inputs** — base `.input` + `.input-compact` + `.textarea`.

Standardized values:
- Padding: `10px 12px` (default), `8px 10px` (compact)
- Font-size: `var(--fs-input)`
- Border: `1px solid var(--color-border)`
- Border-radius: `var(--radius-btn)`
- Background: `var(--color-surface)`
- Color: `var(--color-text)`
- Outline: `none`
- Focus: `border-color: var(--color-primary)`
- Error: `.input-error { border-color: var(--color-danger) }`
- Placeholder: `::placeholder { color: var(--color-text-muted) }`
- Transition: `border-color var(--transition-speed)`

### Step 4: Migrate file by file

Replace the per-component button/input styles with the shared classes. Remove the old scoped CSS rules as each component is migrated. Keep component-specific layout styles (margins, positioning, flex behavior) in scoped styles.

Migration order (by number of button styles to replace):
1. `PlantForm.svelte` — 5 button styles + 2 input styles (biggest win)
2. `plants/[id]/+page.svelte` — 8 button styles + 2 input styles
3. `+page.svelte` (dashboard) — 2 button styles
4. `settings/+page.svelte` — 3 button styles + 1 input style
5. `LocationChips.svelte` — 1 chip style + 1 input style
6. `WateringInterval.svelte` — 2 styles
7. `IconPicker.svelte` — 1 style
8. `log/+page.svelte` — 1 chip style
9. `plants/[id]/edit/+page.svelte` — 1 button style
10. `plants/new/+page.svelte` — 1 button style
