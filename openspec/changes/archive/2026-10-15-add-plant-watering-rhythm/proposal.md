# Proposal

## Why

The plant journal contains the care history, but a quiet visual overview makes event timing and gaps easier to scan. The approved design is a background care-event timeline: floating icons on dotted stems, planted on a gently winding ground path. It replaces the rejected watering-level wave experiment.

## What Changes

- Show all supported care-event types from the last 28 days, positioned horizontally by their actual occurrence times.
- Give icons stable, decorative heights. Height and ground depth carry no numeric, moisture, quantity, scheduling, or health meaning.
- Use one gentle ground path independent of event count at a fixed container size. Adding events adds stems, not bends; stems meet that path with faint contact shadows.
- Keep nearby icons apart without shifting time positions; grow the host only when a dense cluster needs more vertical space.
- Reveal the ground left to right over 2200ms and stagger gentle icon pops, once per plant visit. Respect reduced motion.
- Keep the timeline a transparent, noninteractive hero background, with soft foreground protection and existing theme colors.
- Preserve desktop stacked actions and mobile side-by-side actions below the growing heading host.
- Add no visible legend, date-axis labels, captions, state messages, or translation keys. Remove the space reserved for the former legend.
- Preserve both journals, event CRUD and photos, AI, offline restrictions, request ownership guards, and authoritative watering status.
- Make the chosen timeline the default. Remove wave calculations/renderers, unused prototype variants, and URL preview switches.

## Capabilities

### New Capabilities

- `ui-care-timeline`: Recorded care-event positioning, decorative heights, density-independent ground, themed hero presentation, and history lifecycle.

### Modified Capabilities

- `ui-plant-detail`: Default background timeline and responsive hero action placement, preserving existing detail behavior.

## Impact

Frontend only: `CareTimeline.svelte`, its focused history/layout helpers and tests, and plant-detail integration. No new dependencies, APIs, database changes, MQTT behavior, global-journal behavior, or watering-status calculation. The existing change directory is retained; its superseded `ui-watering-rhythm` delta is replaced by `ui-care-timeline`.
