## Context

The app already uses several feedback patterns:

- Field-inline errors for direct correction, such as location rename conflicts and plant name validation.
- Section-inline feedback for contextual flows such as identify errors and chat errors.
- Page-level error text for route data fetch failures.
- Silent success paths for some actions.
- An invisible or poorly placed error path for some quick actions, most notably dashboard watering from the attention cards.

That mix is not wrong by itself. The problem is that the rules are implicit, inconsistent, and sometimes work against the user. This change defines the decision model first, then maps it onto the current screens.

## Goals / Non-Goals

**Goals**

- Define one shared feedback taxonomy for the whole UI.
- Make toast/snackbar usage explicit rather than ad hoc.
- Preserve strong inline patterns where the user needs nearby context or correction.
- Define responsive notification placement that works with the fixed bottom nav and mobile chat drawer.
- Produce a route-by-route audit that is easy to confirm or adjust before implementation.

**Non-Goals**

- Implement the notification system in this design step.
- Replace every inline message with a toast.
- Introduce long-lived inbox/history notifications.
- Specify backend error semantics beyond existing API message behavior.

## Decision Model

Use feedback based on what the user must do next.

```text
FEEDBACK LADDER

Destructive / blocking / must acknowledge
    -> modal dialog

User must correct a nearby input
    -> inline field error

User must understand context inside a specific section
    -> inline section message

Action completed or failed, but no local correction UI is required
    -> toast/snackbar

The page cannot really function yet
    -> page-level error state
```

### Placement Rules

**Use inline field feedback when:**

- the problem belongs to one input or control
- the user needs to edit that exact area to recover
- the message may need to remain visible while they correct it

**Use inline section feedback when:**

- the feedback only makes sense inside that feature area
- the action result includes detail, retry, or multiple data points
- removing the message from the local context would make it harder to understand

**Use toast/snackbar when:**

- the message is understandable out of context
- the source element may disappear, navigate away, or be offscreen
- the user does not need a nearby corrective input to recover
- the message can stay short

**Use page-level feedback when:**

- the route's primary data failed to load
- the page cannot present its normal content yet
- a toast alone would be too temporary or easy to miss

## Toast / Snackbar Concept

### Role

The toast system is a global acknowledgement layer, not the primary error surface for forms or page load failures.

### Variants

- `success`: action completed
- `info`: neutral acknowledgement
- `warning`: caution, partial completion, or recoverable problem
- `error`: action failed and no inline corrective UI is required

### Behavior

- `success` and `info` auto-dismiss after a short timeout.
- `warning` stays longer and is manually dismissible.
- `error` is manually dismissible by default.
- A toast may include at most one short action button such as `Retry` or `Undo`.
- Toast body text should stay short enough to understand without scanning the rest of the page.

### Accessibility

- Non-error toasts should use polite announcement semantics.
- Error toasts should use assertive announcement semantics.
- Toasts should be keyboard reachable and dismissible.
- Inline messages remain the primary accessible surface when the user must correct local state.

## Responsive Placement

The shell already has two important constraints:

- a fixed mobile bottom nav in `ui/src/routes/+layout.svelte`
- a mobile chat drawer that behaves like a bottom sheet

Because of that, mobile bottom snackbars are a poor fit here.

### Desktop / Tablet (> 768px)

- Toast stack sits at the bottom-right of the viewport.
- The stack uses card-like surfaces, not edge-to-edge bars.
- Show up to three visible toasts before queueing.
- New toasts appear above the older ones.

### Mobile (<= 768px)

- Toast stack sits at the top of the viewport, below the safe-area inset.
- Toasts span nearly full width with page gutters.
- Do not anchor to the bottom because that conflicts with the fixed nav and the chat drawer gesture area.
- The stack should avoid covering the first tappable control on a page for too long.

## Visual Mockups

Interactive mockups live in `openspec/changes/add-ui-notification-patterns/mockups/notifications.html`.

The HTML artifact includes these scenarios:

- desktop dashboard with bottom-right toast stack
- mobile screen with top-anchored toast stack
- field-inline error example
- section-inline contextual error example
- settings action toast example
- page-level error example

That artifact should be treated as the visual source of truth for this change rather than ASCII sketches embedded in markdown.

## Surface Audit

This is the main review artifact for confirmation or adjustment.

| Surface | Current behavior | Recommended primary pattern | Why | Secondary option |
|---|---|---|---|---|
| Dashboard: initial `loadPlants()` failure | Page-level error text far above empty state | Page-level error | The route cannot render its core content | Optional toast only as a supplement, not primary |
| Dashboard: attention-card `waterPlant()` success | No explicit acknowledgement besides list changes | Toast | Trigger may be high on page, card may disappear after success, message is short | None |
| Dashboard: attention-card `waterPlant()` failure | Error falls into `$plantsError` far away | Toast | This is the review bug; local action needs visible feedback near the time of action | Inline card error if you want a more anchored feel |
| Settings: rename location conflict | Inline below input | Keep inline field error | User must fix the typed value in that exact input | None |
| Settings: delete location success | Silent row disappearance | Toast | The row disappears, so a short acknowledgement helps | Silent is acceptable if you prefer minimalism |
| Settings: delete location failure | Store-level error text at section level | Toast | The failed action is row-scoped but not tied to editable input | Section-inline list error if you want more persistence |
| Settings: MQTT repair success | Inline result text next to action | Toast | The user needs acknowledgement, not persistent row text; toast keeps the row compact across breakpoints | Silent success if you want less noise |
| Settings: MQTT repair failure | Inline error next to action | Toast | Failure does not require local correction input, and row-inline text is cramped on mobile | Modal alert only for unusually severe failures |
| Settings: import success | Inline result with imported counts | Toast | Import completion is a short acknowledgement; the page itself can reflect the new data via refreshed stats | Toast can include a compact count summary if desired |
| Settings: import failure | Inline error | Toast | No local corrective field exists, and toast gives cleaner, more consistent feedback | Modal alert for version-mismatch-style hard stops if stronger emphasis is desired |
| Settings: export success | Native download only | Usually no toast | Browser download behavior is normally sufficient and success may not be detectable reliably | Toast only if implementation can detect a meaningful failure or completion state |
| Settings: export failure | Usually browser/network level only | Toast if detectable | Failure is global and not tied to a nearby editable control | Inline fallback if the export row later gains richer local status |
| Plants new: field validation | Inline field errors | Keep inline field errors | Direct correction flow | None |
| Plants new: create failure | Currently effectively silent at page level because the route does not render `$plantsError` | Inline form-level error near save area | The user must remain in the form and retry | Toast should not be the primary surface |
| Plants new/edit: photo upload failure after successful save | Failure can be easy to miss, especially if navigation happens | Toast after navigation or persistent detail-page banner | The source screen may disappear | If implementation is simplified, keep user on form and show inline |
| Plant detail: initial `loadPlant()` failure | Page-level error / not-found handling | Keep page-level | Route cannot proceed normally | None |
| Plant detail: `waterPlant()` success | Visual state refresh only | Usually no toast needed | The result is visible in place via status/date changes | Toast if you want parity with dashboard watering |
| Plant detail: `waterPlant()` failure | Store error outside action area | Inline section or toast | The action is local, but current error placement is weak | Toast is simpler if no dedicated inline slot is added |
| Plant detail: delete plant success | Navigation away only | Toast on destination page | The source page disappears after success | Silent redirect is acceptable but less explicit |
| Plant detail: delete care event success | Silent item removal | Usually no toast | The deletion is obvious in the visible list | Toast only if users need stronger reassurance |
| Plant detail: delete care event failure | Weakly surfaced through shared store | Inline journal-section error or toast | Failure belongs to the journal area, not whole route | Toast if implementation cost should stay low |
| Care entry form: submission failure | Not clearly surfaced in-form | Inline form/toolbar error | User remains inside the form and may retry immediately | Toast only as supplement |
| Care journal route: initial load failure | Page-level error | Keep page-level | Route content cannot load | None |
| Plant identify: identify request failure | Inline error state with retry | Keep inline section feedback | Retry belongs to the identify panel | None |
| Chat drawer: stream failure | Message appears inside chat transcript | Keep inline section feedback | The failure belongs to the conversation context | None |
| Chat drawer: save-note failure | Inline note status | Keep inline section feedback | Retry/edit action is in the drawer | Optional toast only as supplement |
| Chat drawer: save-note success | Drawer closes; acknowledgement may disappear | Toast | Context closes immediately, so acknowledgement should survive closure | Silent close if you want fewer notifications |

## Initial Recommendation Set

If we want a minimal first implementation with high value and low churn, the default rollout should be:

1. Add the global toast host and taxonomy.
2. Fix the dashboard watering feedback with toast success/error.
3. Add toast support for settings actions and actions that navigate away or remove their own context:
   - MQTT repair success/failure
   - import success/failure
   - delete plant success
   - delete location success
   - chat save-note success
4. Keep current inline patterns for:
   - rename conflicts
   - identify errors
   - chat stream errors
   - route load failures
5. Add missing inline form-level errors where failures are currently weak or silent:
   - plant create failure
   - care entry submit failure

## Review Checkpoints

These are the places most worth confirming before implementation:

1. Should delete-success flows stay silent, or should they toast?
2. Should dashboard watering success toast, or only failure?
3. Should import and MQTT repair use concise toast copy only, or should either one escalate to a modal alert for specific hard failures?
4. Should plant-detail watering stay purely in-place, or match dashboard watering with toast feedback?
5. For photo-upload-after-save failures, do we prefer post-navigation toast or keeping the user on the originating form?

## Risks / Trade-offs

- Too many success toasts can make the app feel noisy, especially on mobile.
- Converting contextual failures to toasts can reduce clarity if the user still needs a nearby retry/input.
- A global system is easy to overuse once it exists; the taxonomy must remain the gatekeeper.
- If warning/error toasts persist too aggressively, they can become clutter; if they auto-dismiss too quickly, they become easy to miss.
