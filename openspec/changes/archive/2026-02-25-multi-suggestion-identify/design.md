## Context

The plant identification feature currently makes a single API call to an OpenAI-compatible model and returns one `IdentifyResult`. The model's responses are non-deterministic — the same photo often yields different species across calls. Users must dismiss and retry until they get the correct match.

The existing pipeline: `PlantForm → identifyPlant() → POST /api/ai/identify → OpenAiProvider::identify() → single IdentifyResult → suggestion card with Apply/Dismiss`.

## Goals / Non-Goals

**Goals:**
- Return up to 3 ranked identification suggestions in a single API call
- Let users browse suggestions via a carousel before applying one
- Keep the change backward-compatible (no new dependencies, no DB changes)

**Non-Goals:**
- Pagination or lazy-loading of suggestions
- Caching identification results across sessions
- Changing the number of suggestions dynamically (fixed at 3)

## Decisions

### 1. Single API call with 3 suggestions vs. 3 separate calls

**Decision:** Single call asking for 3 suggestions in one structured response.

**Rationale:** 3 separate calls would triple latency and cost. Vision tokens (the photos) are the expensive part and would be sent 3 times. A single call with "provide your top 3 identifications" is well within the model's capability and produces diverse results because the prompt explicitly asks for ranked alternatives.

**Alternative considered:** Parallel fan-out of 3 calls with `tokio::join!`. Rejected for cost/latency reasons.

### 2. Response envelope: `IdentifyResponse { suggestions }` vs. raw `Vec<IdentifyResult>`

**Decision:** Wrap in `IdentifyResponse` struct with a `suggestions` field.

**Rationale:** A named wrapper is extensible (could add `metadata` later), matches the JSON schema requirement for strict mode (root must be an object, not an array), and is more self-documenting at the API boundary.

### 3. Frontend navigation: swipe carousel vs. tabs vs. dropdown

**Decision:** Carousel with chevron buttons, dot indicators, and touch swipe.

**Rationale:** Matches the existing card-based UI pattern. Tabs would add visual clutter for 3 items. A dropdown hides the alternatives. The carousel keeps the existing card layout and adds minimal navigation chrome. Touch swipe is essential for mobile — pointer events (pointerdown/pointermove/pointerup) with a 50px threshold provide cross-device support without a library.

### 4. `willFillChips` derivation

**Decision:** Derive from `activeSuggestion` (computed from `identifyResults[currentSuggestion]`) instead of a single `identifyResult`.

**Rationale:** The chips must update when the user navigates between suggestions. Making `activeSuggestion` a `$derived` value and basing chips on it keeps reactivity simple.

## Risks / Trade-offs

- **[Model may return fewer than 3 suggestions]** → The frontend handles any length (1–3). Dot indicators and navigation buttons adapt to the actual count. If only 1 suggestion is returned, no navigation controls appear (same as current behavior).
- **[Slightly higher token usage per call]** → The model now produces ~3x the output tokens. Input tokens (photos) stay the same. Net cost increase is marginal since output tokens are cheap relative to vision input.
- **[Strict JSON schema for arrays]** → OpenAI's strict mode requires `additionalProperties: false` and all fields in `required`. The wrapper object `{ "suggestions": [...] }` satisfies this. The inner array uses `items` with the existing `IdentifyResult` schema.
