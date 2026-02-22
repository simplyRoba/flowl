# flowl — AI Integration Concept

## Current State

flowl is a fully functional self-hosted plant care manager with:
- Plant CRUD with photos, icons, species, locations, and care metadata
- Watering schedules with due/overdue tracking
- Care journal (watered, fertilized, repotted, pruned, custom events)
- Dashboard with "needs attention" overview
- MQTT integration for Home Assistant auto-discovery
- Import/export (ZIP backup)
- i18n (English, German, Spanish)
- Light/dark theme

**Tech stack:** Rust (Axum, SQLite, rumqttc) + SvelteKit SPA embedded via rust-embed, single Docker container.

---

## AI Integration — Overview

Add optional AI capabilities that enhance the plant care experience without replacing manual control. All AI features are opt-in and the app remains fully functional without them.

### Provider

**OpenAI-compatible API** — the only provider for now. This covers OpenAI itself, plus any service that exposes an OpenAI-compatible endpoint (Azure OpenAI, local proxies, LM Studio, vLLM, etc.). A provider trait in the backend keeps the door open for a dedicated Ollama integration later.

Two model slots with sensible defaults:

| Slot | Env Var | Default | Used for |
|------|---------|---------|----------|
| **Vision** | `FLOWL_AI_VISION_MODEL` | `gpt-4o` | Plant identification, health check (image input) |
| **Chat** | `FLOWL_AI_CHAT_MODEL` | `gpt-4o-mini` | Care assistant, watering suggestions (text only) |

Configuration is entirely via environment variables — no settings UI for keys or models.

| Variable | Default | Description |
|----------|---------|-------------|
| `FLOWL_AI_API_KEY` | — | OpenAI API key (required to enable AI) |
| `FLOWL_AI_BASE_URL` | `https://api.openai.com/v1` | API base URL (change for compatible services) |
| `FLOWL_AI_VISION_MODEL` | `gpt-4o` | Model for vision tasks |
| `FLOWL_AI_CHAT_MODEL` | `gpt-4o-mini` | Model for text tasks |

AI is **disabled** when `FLOWL_AI_API_KEY` is not set. No settings table needed.

### Architecture Addition

```
┌─────────────────────────────────────┐
│            Docker                   │
│  ┌───────────────────────────────┐  │
│  │      flowl (Rust binary)      │  │
│  │  ┌─────────┐ ┌──────┐        │  │
│  │  │  Axum   │ │ MQTT │──────────────▶ Mosquitto
│  │  │  HTTP   │ │client│        │  │
│  │  └────┬────┘ └──────┘        │  │
│  │       │                      │  │
│  │  ┌────┴────┐  ┌───────────┐  │  │
│  │  │ SQLite  │  │ AI Client │──┼──┼──▶ OpenAI-compatible API
│  │  └─────────┘  └───────────┘  │  │
│  └───────────────────────────────┘  │
│       │                             │
│  volume: /data/flowl.db             │
└─────────────────────────────────────┘
```

---

## Features

### 1. Plant Identification + Care Profile

**What:** AI identifies the species from photos and returns a complete care profile. The user's already-uploaded plant photo is used as the primary image, with the option to add 1–2 extra shots (e.g. leaf close-up, stem detail) for better accuracy. Returns common name, scientific name, confidence level, and all care fields.

**Where:** `PlantForm` component (used on both Add Plant and Edit Plant screens).

```
┌─────────────────────────────────────────────┐
│  Add Plant                     Cancel  Save  │
│─────────────────────────────────────────────│
│                                              │
│  ┌──────────────────────────────────┐        │
│  │                                  │        │
│  │         [uploaded photo]         │        │
│  │                                  │        │
│  └──────────────────────────────────┘        │
│                                              │
│  ┌──────────────────────────────────┐        │
│  │ ✨ Identify Plant                │        │
│  │                                  │        │
│  │  Add more photos for better      │        │
│  │  accuracy (optional):            │        │
│  │  [+ leaf close-up] [+ stem/pot]  │        │
│  └──────────────────────────────────┘        │
│                                              │
│  ┌ AI Suggestion ───────────────────┐        │
│  │                                  │        │
│  │  🪴 Monstera deliciosa           │        │
│  │  "Swiss Cheese Plant"            │        │
│  │  Confidence: 94%                 │        │
│  │                                  │        │
│  │  Will fill:                      │        │
│  │  · Species · Watering (10d)      │        │
│  │  · Light (indirect) · Difficulty │        │
│  │  · Pet safety (toxic)            │        │
│  │                                  │        │
│  │  [Apply to form]  [Dismiss]      │        │
│  └──────────────────────────────────┘        │
│                                              │
│  Name     [                          ]       │
│  Species  [                          ]       │
│  ...                                         │
└─────────────────────────────────────────────┘
```

**Behavior:**
- "Identify Plant" button appears below the photo upload area only when a photo is present and AI is configured
- Clicking it shows optional extra photo slots (leaf close-up, stem/pot) — small upload targets that don't replace the main plant photo
- Extra photos are used only for identification, not stored on the plant
- Sends all photos to the vision API in a single request
- Shows a result card with species info, confidence, and a preview of which fields will be filled
- "Apply to form" fills in: `species`, `name` (if empty), `notes` (short species summary), `light_needs`, `watering_interval_days`, `difficulty`, `pet_safety`, `growth_speed`, `soil_type`, `soil_moisture`
- User can still edit any field after applying — AI suggestions are never forced

**Fields populated:**

| Field | Example | Source |
|-------|---------|--------|
| Name | "Monstera" | Common name (only if name field is empty) |
| Species | "Monstera deliciosa" | Scientific name |
| Notes | "Tropical climbing plant native to..." | Short species summary (only if notes field is empty) |
| Watering interval | 10 days | Species-typical |
| Light needs | Indirect | Species-typical |
| Difficulty | Easy | Species-typical |
| Pet safety | Toxic | Known toxicity data |
| Growth speed | Moderate | Species-typical |
| Soil type | Standard | Species-typical |
| Soil moisture | Moderate | Species-typical |

**API endpoint:**

```
POST /api/ai/identify
Content-Type: multipart/form-data

photos[]: <file>       (the main plant photo)
photos[]: <file>       (optional extra 1)
photos[]: <file>       (optional extra 2)

→ 200 {
    "common_name": "Swiss Cheese Plant",
    "scientific_name": "Monstera deliciosa",
    "confidence": 0.94,
    "summary": "Tropical climbing plant native to Central American rainforests. Known for its distinctive split leaves. Prefers warm, humid conditions.",
    "care_profile": {
      "watering_interval_days": 10,
      "light_needs": "indirect",
      "difficulty": "easy",
      "pet_safety": "toxic",
      "growth_speed": "moderate",
      "soil_type": "standard",
      "soil_moisture": "moderate"
    }
  }
```

---

### 2. Plant Health Check

**What:** Analyze a photo of a plant for visible health issues — yellowing leaves, brown spots, pests, drooping, root rot signs, etc. Returns a health assessment with actionable advice.

**Where:** Plant Detail page — new action button in the hero section, next to "Water now".

```
┌─────────────────────────────────────────────┐
│  ← Back                        ✏️  🗑️        │
│─────────────────────────────────────────────│
│                                              │
│  ┌────────┐  Monstera                        │
│  │ photo  │  Monstera deliciosa              │
│  │        │  📍 Living Room                  │
│  │        │  ● Ok — in 3 days                │
│  └────────┘                                  │
│              [💧 Water now] [🔍 Health check] │
│                                              │
│  ┌ Health Report ───────────────────┐        │
│  │                                  │        │
│  │  Overall: ⚠️ Needs attention     │        │
│  │                                  │        │
│  │  Findings:                       │        │
│  │  • Yellowing lower leaves        │        │
│  │    → Likely overwatering. Let     │        │
│  │      soil dry out between         │        │
│  │      waterings.                   │        │
│  │                                  │        │
│  │  • Brown leaf tips                │        │
│  │    → Low humidity. Consider       │        │
│  │      misting or a pebble tray.    │        │
│  │                                  │        │
│  │  Suggested actions:               │        │
│  │  ☐ Reduce watering to every 14d  │        │
│  │  ☐ Increase humidity             │        │
│  │                                  │        │
│  │  [Dismiss]                        │        │
│  └──────────────────────────────────┘        │
│                                              │
│  Watering                                    │
│  ...                                         │
└─────────────────────────────────────────────┘
```

**Behavior:**
- "Health check" button opens a flow: either analyze the existing plant photo, or upload/take a new one
- Camera capture option on mobile (`capture="environment"` on file input)
- Results appear inline on the detail page as a collapsible report card
- Health status levels: `healthy`, `needs-attention`, `unhealthy`
- Each finding has a short description and a recommendation
- Suggested actions can include watering interval changes — with a one-tap "Apply" that updates the plant's schedule

**API endpoint:**

```
POST /api/ai/health-check
Content-Type: multipart/form-data

photo: <file>
plant_id: 42  (optional — includes plant context: species, care history, watering schedule)

→ 200 {
    "status": "needs-attention",
    "findings": [
      {
        "issue": "Yellowing lower leaves",
        "severity": "moderate",
        "cause": "Likely overwatering",
        "recommendation": "Let soil dry out between waterings."
      },
      {
        "issue": "Brown leaf tips",
        "severity": "mild",
        "cause": "Low humidity",
        "recommendation": "Consider misting or using a pebble tray."
      }
    ],
    "suggested_actions": [
      {
        "type": "update_watering_interval",
        "value": 14,
        "reason": "Current interval may be too frequent for this species in indirect light."
      }
    ]
  }
```

---

### 3. Care Assistant (Chat)

**What:** A conversational interface where users can ask plant care questions. The AI has full context about the specific plant — its species, care profile, watering history, care log, and current health.

**Where:** Plant Detail page — a collapsible panel at the bottom, or a floating action button that opens a chat drawer.

```
┌─────────────────────────────────────────────┐
│  ← Back                        ✏️  🗑️        │
│─────────────────────────────────────────────│
│  ... (hero, watering info, care info) ...    │
│                                              │
│  ┌ Ask about this plant ────────────┐        │
│  │                                  │        │
│  │  Quick questions:                │        │
│  │  [When to repot?]               │        │
│  │  [Why are leaves drooping?]     │        │
│  │  [Fertilizer schedule?]         │        │
│  │                                  │        │
│  │  ┌──────────────────────┐ [Ask]  │        │
│  │  │ Type your question   │        │        │
│  │  └──────────────────────┘        │        │
│  │                                  │        │
│  │  ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─ ─  │        │
│  │                                  │        │
│  │  You: When should I repot this?  │        │
│  │                                  │        │
│  │  flowl: Monstera deliciosa       │        │
│  │  typically needs repotting every  │        │
│  │  1-2 years, or when roots start  │        │
│  │  growing through drainage holes. │        │
│  │  Based on your care log, the     │        │
│  │  last repotting was 14 months    │        │
│  │  ago — it may be time soon.      │        │
│  │  Look for roots circling the     │        │
│  │  pot or slowed growth as signs.  │        │
│  │                                  │        │
│  └──────────────────────────────────┘        │
│                                              │
│  Care Journal                                │
│  ...                                         │
└─────────────────────────────────────────────┘
```

**Behavior:**
- Section titled "Ask about this plant" with a text input and send button
- Quick-question chips for common queries (context-aware — e.g. shows "Why overdue?" if plant is overdue)
- Responses are plant-specific: the AI receives the full plant profile, recent care history, and current watering status as context
- Chat history persists for the session only (not stored in DB) — keeps the feature lightweight
- Markdown rendering for AI responses (bold, lists, etc.)
- Streaming responses with a typing indicator

**Context sent to AI:**

```json
{
  "plant": {
    "name": "Monstera",
    "species": "Monstera deliciosa",
    "location": "Living Room",
    "light_needs": "indirect",
    "watering_interval_days": 10,
    "watering_status": "ok",
    "last_watered": "2026-02-20",
    "soil_type": "standard",
    "difficulty": "easy",
    "pet_safety": "toxic",
    "notes": "Bought from local nursery, seems healthy"
  },
  "recent_care_events": [
    { "type": "watered", "date": "2026-02-20" },
    { "type": "fertilized", "date": "2026-02-01", "notes": "Liquid fertilizer" },
    { "type": "repotted", "date": "2024-12-15" }
  ]
}
```

**API endpoint:**

```
POST /api/ai/chat
Content-Type: application/json

{
  "plant_id": 42,
  "message": "When should I repot this?",
  "history": [
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." }
  ]
}

→ 200 (streamed, text/event-stream)
data: {"delta": "Monstera deliciosa "}
data: {"delta": "typically needs "}
data: {"delta": "repotting every 1-2 years..."}
data: {"done": true}
```

---

### 4. Smart Watering Suggestions

**What:** AI analyzes the plant's species, location, light conditions, and care history to suggest an optimal watering interval. Appears as a non-intrusive hint — never auto-changes settings.

**Where:** Plant Detail page — inline in the Watering section when the current interval differs significantly from what's typical for the species.

```
┌ Watering ────────────────────────────────┐
│                                          │
│  Interval      Every 7 days              │
│  Last watered  Feb 20, 2026              │
│  Next due      Feb 27, 2026              │
│                                          │
│  ┌ 💡 Suggestion ─────────────────────┐  │
│  │ Monstera in indirect light          │  │
│  │ typically needs water every 10-14   │  │
│  │ days. Your current 7-day interval   │  │
│  │ may be too frequent.                │  │
│  │                                     │  │
│  │ [Apply 12 days]  [Dismiss]          │  │
│  └─────────────────────────────────────┘  │
│                                          │
└──────────────────────────────────────────┘
```

**Behavior:**
- Suggestion is fetched once when the plant detail page loads (if AI is configured and species is known)
- Cached per plant — re-fetched only when species or light_needs change
- Dismissing hides the suggestion until the plant profile changes
- "Apply" updates the watering interval immediately
- Suggestion stored in a new DB column `ai_watering_suggestion` on the plants table (nullable JSON) so it doesn't re-query every page load

**API endpoint:**

```
POST /api/ai/suggest-watering
Content-Type: application/json

{
  "plant_id": 42
}

→ 200 {
    "suggested_interval_days": 12,
    "reasoning": "Monstera in indirect light typically needs water every 10-14 days.",
    "confidence": 0.82
  }
```

---

## Settings — AI Status

New section in the Settings page, between "MQTT" and "Data". Read-only status indicator — mirrors the MQTT section pattern. All configuration lives in env vars.

```
┌ AI Assistant ────────────────────────────┐
│                                          │
│  ● Enabled                               │
│  Provider   OpenAI (api.openai.com)      │
│  Vision     gpt-4o                       │
│  Chat       gpt-4o-mini                  │
│                                          │
└──────────────────────────────────────────┘
```

When AI is not configured (no API key):

```
┌ AI Assistant ────────────────────────────┐
│                                          │
│  ○ Disabled                              │
│  Set FLOWL_AI_API_KEY to enable.         │
│                                          │
└──────────────────────────────────────────┘
```

**API endpoint:**

```
GET /api/ai/status → { "enabled": true, "base_url": "https://api.openai.com/v1", "vision_model": "gpt-4o", "chat_model": "gpt-4o-mini" }
```

---

## Data Model Changes

### New column on `plants`

```sql
ALTER TABLE plants ADD COLUMN ai_watering_suggestion TEXT;
-- JSON: {"interval_days": 12, "reasoning": "...", "confidence": 0.82, "generated_at": "..."}
```

---

## Backend Design

### AI Provider Trait

```rust
#[async_trait]
trait AiProvider: Send + Sync {
    /// Identify a plant from a photo. Returns species + care profile.
    async fn identify_plant(&self, image: &[u8]) -> Result<IdentifyResult>;

    /// Analyze a photo for health issues. Plant context is optional.
    async fn health_check(&self, image: &[u8], context: Option<&PlantContext>) -> Result<HealthReport>;

    /// Chat about a specific plant. Receives full plant context + history.
    async fn chat(&self, context: &PlantContext, message: &str, history: &[ChatMessage]) -> Result<ChatResponseStream>;

    /// Suggest a watering interval based on species and conditions.
    async fn suggest_watering(&self, context: &PlantContext) -> Result<WateringSuggestion>;
}
```

One implementation for now: `OpenAiProvider` (works with any OpenAI-compatible API). Ollama provider can be added later behind the same trait.

### Response Deserialization

All structured AI responses (identify, health check, watering suggestion) use OpenAI's **JSON mode** (`response_format: { "type": "json_object" }`). This guarantees the model outputs valid JSON. The expected schema is described in the system prompt; serde handles deserialization on the Rust side.

```rust
#[derive(Deserialize)]
struct IdentifyResponse {
    pub common_name: String,
    pub scientific_name: String,
    #[serde(default)]
    pub confidence: Option<f32>,
    pub summary: Option<String>,
    #[serde(default)]
    pub care_profile: Option<CareProfile>,
}
```

- Required fields (`common_name`, `scientific_name`) — if missing, serde returns an error and the endpoint responds with a clear "AI returned an unexpected response" message
- Optional fields (`confidence`, `summary`, `care_profile`) — gracefully absent, the frontend just doesn't show them
- JSON mode is widely supported across OpenAI-compatible APIs (OpenAI, Azure, vLLM, LM Studio, etc.)
- Chat responses (feature 3) don't use JSON mode — they stream plain text via SSE

### New Rust Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client for AI API calls |
| `async-trait` | Trait with async methods |
| `base64` | Encode images for API payloads |
| `tokio-stream` | SSE streaming for chat responses |

---

## Implementation Phases

### Phase A — Foundation

- [ ] AI config in `Config` struct (env vars: API key, base URL, vision model, chat model)
- [ ] AI provider trait definition
- [ ] OpenAI provider implementation (reqwest-based, vision + chat)
- [ ] `Option<Arc<dyn AiProvider>>` in `AppState` (None when no API key)
- [ ] `GET /api/ai/status` endpoint
- [ ] AI status section in Settings UI (enabled/disabled indicator, models)

### Phase B — Plant Identification

- [ ] `POST /api/ai/identify` endpoint
- [ ] "Identify Plant" button in `PlantForm` (below photo upload)
- [ ] Suggestion card with "Apply to form" / "Dismiss"
- [ ] Auto-fill form fields from care profile

### Phase C — Health Check

- [ ] `POST /api/ai/health-check` endpoint
- [ ] "Health check" button on Plant Detail hero section
- [ ] Photo selection flow (use existing photo or upload new)
- [ ] Health report card with findings and recommendations
- [ ] "Apply" action for suggested watering changes

### Phase D — Care Assistant

- [ ] `POST /api/ai/chat` endpoint with SSE streaming
- [ ] Chat section on Plant Detail page
- [ ] Quick-question chips (context-aware)
- [ ] Streaming response rendering with typing indicator
- [ ] Markdown rendering for responses

### Phase E — Smart Suggestions

- [ ] `POST /api/ai/suggest-watering` endpoint
- [ ] `ai_watering_suggestion` column + migration
- [ ] Inline suggestion card in Watering section on Plant Detail
- [ ] Dismiss + apply flow

### Future — Additional Providers

- [ ] Ollama provider implementation (behind the same trait)
- [ ] Provider selection via env var or Settings UI
