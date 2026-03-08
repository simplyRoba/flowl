## Context

flowl already supports standalone PWA installation, but the installed app removes browser chrome and with it the normal refresh affordances users rely on in mobile browsers. The UI shell currently uses document-body scrolling via `ui/src/routes/+layout.svelte`, while the allowlisted routes load data through a mix of page-local fetches, store-driven loads, and route `load` functions. That mixed loading model makes a shared soft-refresh abstraction possible, but more invasive than the user problem requires.

The user need is narrow: on phones and tablets, give installed-PWA users a familiar pull-to-refresh interaction on browse routes only, while keeping form-heavy flows safe from accidental refresh.

## Goals / Non-Goals

**Goals:**

- Enable pull-to-refresh for standalone PWA sessions on touch devices.
- Restrict the gesture to the browse routes `/`, `/care-journal`, `/settings`, and `/plants/[id]`.
- Reuse each route's existing load path by triggering a full page reload after a successful pull gesture.
- Provide visible, lightweight gesture feedback so the interaction feels deliberate rather than surprising.
- Avoid accidental activation while the page is not scrolled to the top or while transient overlays are open.

**Non-Goals:**

- No pull-to-refresh on `/plants/new`, `/plants/[id]/edit`, or any `PlantForm`-driven flow.
- No refactor to unify all route data loading behind a shared refresh abstraction.
- No offline caching, service worker refresh strategy, or stale-while-revalidate behavior.
- No custom pull-to-refresh in normal browser-tab mode where the browser may already provide native refresh affordances.

## Decisions

### 1. Gate the feature to standalone PWA mode on touch devices

**Decision:** Only enable the gesture when the app is running in standalone display mode, including iOS home-screen mode.

**Why:** The problem exists primarily in installed mode. Limiting the feature there avoids competing with browser-native pull-to-refresh and keeps desktop and in-browser behavior unchanged.

**Alternatives considered:**

- Enable on all mobile browsers: rejected because it risks fighting native browser refresh behavior.
- Enable on all viewports: rejected because desktop users already have clear reload affordances.

### 2. Implement the gesture in the shared shell with a route allowlist

**Decision:** Attach pull detection at the app shell level and gate activation through an allowlist covering `/`, `/care-journal`, `/settings`, and `/plants/[id]`.

**Why:** The shell already has awareness of the current route and owns the main scrolling context. Centralizing the gesture prevents duplicate logic across pages while still keeping route eligibility explicit.

**Alternatives considered:**

- Implement separately in every page component: rejected because it duplicates touch logic and increases drift.
- Add a new nested scroll container just for pull handling: rejected because the shell intentionally uses document-body scrolling today.

### 3. Trigger a full page reload instead of a route-specific soft refresh

**Decision:** When the user releases beyond the pull threshold, perform a full reload of the current route.

**Why:** The current routes do not share a single refresh contract. A hard reload reuses the existing route-specific loading behavior for dashboard, care journal, settings, and plant detail without refactoring those pages first.

**Alternatives considered:**

- Route-specific soft refresh callbacks: smoother, but requires new refresh plumbing for each route and more coordination between shell and page state.
- Global invalidation: insufficient on its own because not every route derives its state from SvelteKit invalidation-compatible data sources.

### 4. Only arm the gesture at true top-of-scroll and suspend it during overlays

**Decision:** The gesture only becomes active when the document is scrolled to the top and no transient overlay is consuming the user's attention.

**Why:** This reduces false triggers during normal scrolling and protects routes like plant detail, which can open dialogs, lightboxes, chat drawers, and inline care flows.

**Alternatives considered:**

- Allow activation near the top: rejected because it would cause accidental reloads during ordinary upward scrolls.
- Ignore overlay state: rejected because it creates confusing gesture conflicts.

### 5. Keep the feedback lightweight and native-leaning

**Decision:** Show a simple pull indicator that communicates three states: idle, pulling, and release-to-refresh, then briefly transitions into a loading state while the reload begins.

**Why:** The feature should solve discoverability without introducing a large new visual system. A minimal indicator is enough for confirmation and keeps the change small.

**Alternatives considered:**

- No indicator: rejected because the interaction would feel invisible and accidental.
- Heavy animated component: rejected because it increases scope and test surface without changing the core value.

## Risks / Trade-offs

- **[Risk] iOS and Android standalone detection differ** -> Mitigation: support both standards-based display-mode checks and iOS home-screen detection so the feature does not silently disappear on one platform.
- **[Risk] Body-scroll pull detection is sensitive to rubber-band overscroll** -> Mitigation: require a clear threshold, cap the visual pull distance, and only arm the gesture from an exact top-of-scroll state.
- **[Risk] Full reload clears transient page state** -> Mitigation: limit the feature to browse routes where losing ephemeral UI state is acceptable, and keep edit/new flows excluded.
- **[Risk] Plant detail contains modal and drawer states** -> Mitigation: suppress pull-to-refresh while destructive dialogs, lightbox, chat drawer, or inline care entry UI is open.
- **[Risk] Mixed route load strategies make future polish harder** -> Mitigation: accept hard reload for v1; if the gesture proves valuable, a later change can unify route refresh contracts.

## Migration Plan

- No backend migration is required.
- Ship as a frontend-only change behind route and standalone gating.
- Rollback is straightforward: remove the shell-level gesture handling without affecting stored data or API contracts.

## Open Questions

- Whether tablets should be determined strictly by touch capability or also by viewport width thresholds.
- Whether the loading indicator should remain visible until `beforeunload`/navigation handoff or disappear immediately after the threshold release.
