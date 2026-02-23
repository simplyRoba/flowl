## Context

The Settings page already has four optional sections (Appearance, Locations, MQTT, Data, About), each following a consistent pattern: fetch status on mount, hide section on failure, display read-only rows. The MQTT section is the closest analog — it shows connection state, broker host, and topic prefix from a status endpoint. The AI status section mirrors this pattern exactly.

The backend `GET /api/ai/status` endpoint already exists and returns `{ enabled, base_url, model }`.

## Goals / Non-Goals

**Goals:**
- Surface AI configuration status in Settings using the established section pattern
- Support enabled, disabled, and fetch-failure states

**Non-Goals:**
- AI configuration UI (keys/models are env-var only by design)
- Connection testing or health checks beyond the status endpoint
- Any backend changes

## Decisions

### 1. Section placement: between MQTT and Data

The AI section sits after MQTT and before Data. Both MQTT and AI are optional external integrations configured via env vars — grouping them together is logical. Data and About are more general and stay at the bottom.

### 2. Icon: `Sparkles` from lucide-svelte

Consistent with the "AI/magic" metaphor used elsewhere in the plan (identify button, chat button). Other candidates (`Bot`, `Cpu`, `BrainCircuit`) were considered but `Sparkles` aligns with the `✨` motif used throughout the PLAN.

### 3. Display hostname only for base URL

Show `api.openai.com` rather than the full `https://api.openai.com/v1`. The full URL with path is noisy for a status display. Extract hostname via `URL` constructor.

### 4. Disabled state shows env var hint

When AI is not configured, display the text "Set FLOWL_AI_API_KEY to enable" — directly actionable guidance matching the PLAN's wireframe. This is an i18n string with the env var name embedded (not interpolated, since the var name is the same in all locales).

## Risks / Trade-offs

- **Stale status on long-lived sessions** — The status is fetched once on mount. If the server restarts with different env vars, the Settings page shows stale data until refresh. This matches the existing MQTT behavior and is acceptable for a status display. → No mitigation needed.
