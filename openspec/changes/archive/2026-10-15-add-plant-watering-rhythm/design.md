# Design

## Accepted Direction

A quiet care-event timeline behind the plant-detail hero. Time is real; vertical placement is decorative. Event icons rise on dotted stems from a gently winding ground path with faint contact shadows. There is no water-level curve, visible axis legend, numeric vertical scale, or inferred plant status.

The journal remains the full, interactive history. This background is an overview, not a replacement.

## Decisions

### Event history and time

Use the parent's raw care-event array before journal grouping. Include every supported event type, including entries with notes/photos, and filter by plant identity. Normalize actual instants, interpreting accepted timezone-less and SQLite timestamps as UTC. Preserve equal-instant records separately, sort by time and ID, and ignore invalid timestamps without changing journal data.

Display the trailing 28 days ending at the real current time. Do not seed events from outside the window or plot future records. Normalize on source changes; update eligibility/window position once per minute while visible and immediately on visibility resume. Resample the real clock when fresh history arrives. The timeline owns no requests or polling.

### Decorative geometry

Use deterministic event-ID preferences with vertical collision avoidance; never call `Math.random` during rendering. Keep 14px icons at least 4px apart when their horizontal footprints intersect, preserving exact time x-coordinates. Reflow heights only as needed for changed neighbors or container dimensions. Dense clusters grow the hero's minimum height rather than hide, merge, or overlap records; shrink that minimum when the cluster clears. Do not encode care amount, moisture, interval compliance, or health in height. Editing the watering interval does not reshape the timeline.

Build the ground from five plant-seeded anchors at fixed horizontal fractions, joined by four smooth cubic segments. Use a decorative depth range capped at 36px or 18% of host height for a slightly more curved trail; collision minimum-height planning accounts for that same range. Ground shape depends on plant identity and container size, never event count or event IDs. Sample that same curve at each event's actual x-coordinate to locate its stem foot. Keep at least 18px between icon and ground. Use a tiny contact dot and flattened translucent shadow to suggest depth without painting a ground plane.

### Background presentation

Use native SVG and existing theme tokens: watering blue, primary for fertilizing, secondary for repotting, muted color for other manual care, and the AI token for saved consultations. Icons are 14px; stems/ground are fine rounded dotted strokes. Keep the accepted subdued opacity levels and faint ground shadow.

Softly fade both outer boundaries. Protect actual text/button bounds with luminance-mask cutouts using 12px padding and 12px Gaussian blur. Achromatic mask colors are not visible background patches. Re-measure after resizing, wrapping, locale changes, and optional-content changes; disconnect observers/listeners on teardown. Optional DOM bindings may become null.

The background never receives focus or intercepts pointer/touch events. Once per plant visit, reveal the ground left to right over 2200ms. Fade in stems at their time position, then gently pop icons into place with a short scale animation; finish the sequence within about 2600ms. Animate appearance only, never shift recorded time coordinates. Clock, resize, locale/theme, and history updates do not replay the entrance. Reduced motion skips it, including when enabled during a reveal; clean up its timer and media listener on teardown.

### Responsive layout and spacing

Retain the 768px breakpoint. Desktop uses the information area beside the photo/fallback; hero actions stack vertically. Mobile keeps the timeline behind the name, species, optional location and status in a growing heading host; actions stay outside/below and side-by-side when both exist.

Remove visible date labels and their reserved padding. Place the ground relative to the host bottom without the former 32px legend allowance; retain only small visual breathing room. Remove the mobile heading's 24px legend padding, not the page's ordinary card gutter or photo proportions. Confirm actual hero/card bounds in a browser rather than preserving accidental whitespace.

### History ownership and failure states

Preserve local plant-visit, care-request, and form-session ownership guards. Superseded success/failure/loading completions must not overwrite a newer view or request. Successful authoritative plant refreshes can update status/details even when independent history reconciliation fails; retain the loaded event markers and existing error feedback.

Omit the SVG when no event is eligible in the displayed window. Add no graph-specific empty/loading/error UI. Cached history renders offline with existing mutation restrictions. Reuse journal/loading/error labels and localized period dates for SVG accessibility metadata; add no translation keys or visible copy. Detailed events remain accessible through the journal.

## Cleanup and Scope

Promote the accepted renderer to `CareTimeline.svelte`. Remove `CareTimelinePreview`, the thread/rail variants, `background` query switches, `WateringRhythm`, watering-level normalization/cycle utilities, wave geometry, and superseded tests. Retarget relevant asynchronous and journal regressions rather than discarding them.

No changes to backend contracts, persistence, dependencies, global theme tokens, AI behavior, journal grouping, or watering-status calculation. The feature remains subject to human review; no automatic archive, merge, or push.
