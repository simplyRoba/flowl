## Why

The care journal fills up with repetitive watering entries. A user who waters 15 plants weekly sees 60+ identical "Watered" rows per month, burying interesting events (repots, notes, photos, AI consultations). Grouping consecutive note-less, photo-less watering events per plant into collapsible summaries reduces noise while keeping the data accessible via expand/collapse.

## What Changes

- Add a client-side grouping utility that collapses consecutive watering events (same plant, no notes, no photos) into summary items showing count and date range
- The global care journal switches from infinite-scroll pagination to loading all events, enabling accurate grouping
- Both the global care journal and plant detail timeline use the same grouping utility
- Grouped summaries are expandable/collapsible to reveal individual entries
- Only `watered` events are grouped; all other event types always render individually
- Events with notes or photos break the grouping streak and render individually

## Capabilities

### New Capabilities

_(none)_

### Modified Capabilities

- `care-journal`: Add watering event grouping with expand/collapse, replace infinite scroll with load-all on the global page, share grouping logic with plant detail timeline

## Impact

- `ui/src/routes/care-journal/+page.svelte` — remove infinite scroll, apply grouping, add expand/collapse UI
- `ui/src/routes/plants/[id]/+page.svelte` — apply same grouping to the care journal section
- `ui/src/lib/` — new grouping utility module (pure function, no API changes)
- `ui/src/lib/i18n/` — new translation keys for group summary text
- No backend/API changes required
