# ui-care-timeline Specification

## Purpose

Show recorded care as a quiet, decorative plant-detail background without implying measured water levels or replacing the journals.

## Requirements

### Requirement: Recorded care-event background

The plant-detail hero SHALL display eligible raw care events from the current plant's last 28 days using existing event-type icons. All supported types and events with notes/photos SHALL participate independently of journal grouping. Invalid, foreign, out-of-window, and future records SHALL NOT create markers.

#### Scenario: Mixed care history

- **WHEN** eligible watering, fertilizing, repotting, pruning, custom, or saved AI-consultation events are loaded
- **THEN** each event has its corresponding icon
- **AND** notes, photos, or collapsed journal groups do not remove markers

#### Scenario: Unusable or absent history

- **WHEN** no event is eligible in the displayed window
- **THEN** the background SVG is omitted
- **AND** no timeline-specific empty, loading, or error message is added

### Requirement: True horizontal time

Event x-coordinates SHALL represent proportional elapsed time within the trailing 28-day period ending at the actual current time. Supported timestamp representations SHALL normalize to actual instants, with timezone-less timestamps interpreted as UTC. Equal-instant records SHALL retain separate identities. The timeline SHALL NOT draw a forecast.

#### Scenario: Equivalent timestamps

- **WHEN** records express the same instant using supported offsets or UTC formats
- **THEN** their x-coordinates match regardless of response order
- **AND** their separate journal records remain unchanged

#### Scenario: Current-time eligibility

- **WHEN** time advances while visible or the page becomes visible again
- **THEN** window positions and eligibility update without a history reload
- **AND** a future event appears only after real time reaches its occurrence

### Requirement: Decorative vertical placement

Icon heights SHALL be deterministic decorative layout with collision avoidance, not a numeric scale. Height SHALL NOT encode moisture, care amount, schedule compliance, or health. Icons whose horizontal footprints intersect SHALL have vertical clearance, including simultaneous and same-day events. Ordinary redraws SHALL NOT randomly reposition icons, and interval edits SHALL NOT reshape them.

#### Scenario: Stable layout

- **WHEN** history is reordered, refreshed, or displayed in another language
- **THEN** unchanged events keep their decorative placement for the same history, time, and size
- **AND** collision avoidance may adjust heights when neighboring events change

#### Scenario: Same-day event collisions

- **WHEN** events occur at the same instant or close enough for their horizontal icon footprints to intersect
- **THEN** their icons occupy separate vertical positions with visible clearance
- **AND** their exact time x-coordinates remain unchanged
- **AND** a cluster too dense for the normal hero height expands the host rather than hiding, merging, or overlapping icons

#### Scenario: Interval edit

- **WHEN** the configured watering interval changes without changing care records
- **THEN** event layout remains unchanged
- **AND** the existing API-backed watering badge remains authoritative

### Requirement: Density-independent ground trail

The background SHALL contain one gently winding ground path built from five fixed horizontal anchors and four smooth segments, independent of event count. Dotted stems SHALL connect raised icons to that path at each event's true x-coordinate. Faint contact marks/shadows MAY suggest depth without adding a filled ground plane.

#### Scenario: More or fewer events

- **WHEN** events are added or removed for the same plant and container size
- **THEN** the ground path remains unchanged
- **AND** event density does not add ground anchors or segments

#### Scenario: Ground contact

- **WHEN** an event is rendered between ground anchors
- **THEN** its stem ends on the rendered ground curve
- **AND** the icon remains above its contact point

#### Scenario: Resize

- **WHEN** the background size changes
- **THEN** event time fractions and the seeded ground contour are preserved
- **AND** heights and stem endpoints adapt without overflow

### Requirement: Subdued responsive presentation

The timeline SHALL remain a transparent, noninteractive hero background in both themes. It SHALL use existing theme colors, dotted strokes, soft boundary fades, and content-aware foreground protection. It SHALL NOT add a card, visible axes/date legend, captions, or legend-reserved whitespace. Desktop actions SHALL stack; mobile actions SHALL remain below the growing heading host and side-by-side when both exist.

#### Scenario: Desktop hero

- **WHEN** the viewport is wider than 768px
- **THEN** the timeline occupies the information area beside the photo or fallback
- **AND** text, status, and vertically stacked actions remain readable and operable

#### Scenario: Mobile hero

- **WHEN** the viewport is 768px or narrower, including 320px
- **THEN** the timeline stays behind the heading information rather than in a separate card or band
- **AND** wrapped content can grow without clipping
- **AND** actions remain outside and below that host

#### Scenario: Legend removal

- **WHEN** the finalized timeline is displayed
- **THEN** no bottom time labels are rendered
- **AND** component bounds and mobile heading padding do not reserve space for the removed legend

#### Scenario: Foreground changes

- **WHEN** language, text wrapping, optional information, or AI availability changes
- **THEN** masking follows currently occupied foreground bounds
- **AND** disconnected bindings leave no stale masks or errors

#### Scenario: Theme changes

- **WHEN** the active theme changes
- **THEN** the timeline blends into that theme without fixed-color background patches
- **AND** event positions remain unchanged

### Requirement: Progressive entrance

The timeline SHALL reveal the ground from left to right once per plant visit over approximately 2200ms, with stems and icons appearing in time order as the reveal reaches their positions. Appearance animation SHALL NOT move recorded time coordinates. Ordinary updates SHALL NOT replay it. Reduced motion SHALL display the final appearance immediately.

#### Scenario: First eligible history

- **WHEN** eligible events first become available during a plant visit
- **AND** reduced motion is not requested
- **THEN** the ground progressively appears from left to right
- **AND** event icons gently pop into place as their time position is reached
- **AND** the complete entrance finishes in approximately 2600ms

#### Scenario: Redraws during a visit

- **WHEN** history, clock, size, language, or theme updates after the entrance has started
- **THEN** it does not restart
- **AND** navigating to another plant establishes a new visit

#### Scenario: Reduced motion and cleanup

- **WHEN** reduced motion is requested, including during an entrance
- **THEN** the final timeline appears without entrance effects
- **AND** leaving the component clears entrance timers and preference listeners

### Requirement: Shared history lifecycle

The timeline SHALL reuse existing care history and mutation refreshes without network polling. Superseded requests SHALL NOT overwrite newer plant visits, history, errors, or loading state. Failed refreshes SHALL retain loaded markers with existing error feedback. Cached history SHALL render offline without enabling mutations.

#### Scenario: Care event changes

- **WHEN** care is recorded, deleted, retimed, or changed to another supported type
- **AND** refreshed history is available
- **THEN** markers reflect the resulting events and icons
- **AND** the ground path stays unchanged when container dimensions stay unchanged

#### Scenario: Navigation and overlapping refreshes

- **WHEN** a different plant is opened or a newer same-plant refresh supersedes an earlier request
- **THEN** obsolete completions cannot change the active view
- **AND** the previous plant's markers are not displayed for the new plant

#### Scenario: Failed history reconciliation

- **GIVEN** markers were already loaded
- **WHEN** care history refresh fails
- **THEN** those markers remain with existing page error feedback and accessible error metadata
- **AND** a successful current authoritative plant fetch can still update details/status

#### Scenario: Cached offline history

- **WHEN** cached plant and care events are available offline
- **THEN** the timeline can render and progress its time window
- **AND** existing offline mutation restrictions remain unchanged

### Requirement: Accessible context and journal preservation

The timeline SHALL reuse existing localized journal/loading/error labels and formatted period dates for SVG metadata, without new translation keys or visible copy. It SHALL NOT intercept pointer, touch, or keyboard controls. Both journals, grouping, individual actions, notes/photos, AI behavior, and watering-status calculation SHALL remain unchanged.

#### Scenario: Nonvisual context

- **WHEN** assistive technology accesses the background
- **THEN** its journal context and displayed period are available without interpreting decorative height or color
- **AND** detailed care records remain accessible through the journal

#### Scenario: Existing interactions

- **WHEN** the timeline is displayed
- **THEN** photo/lightbox, watering, edit/delete, care forms, and Ask AI remain operable
- **AND** the background adds no focus targets or interactive controls

#### Scenario: Localization

- **WHEN** language changes among English, German, and Spanish
- **THEN** existing accessible labels and period formatting follow that language
- **AND** no visible timeline captions or legends appear

#### Scenario: Finalized default

- **WHEN** a plant detail page is opened without preview query parameters
- **THEN** the selected icon/stem/ground timeline is used
- **AND** no wave renderer, alternate preview variant, or background query switch remains
