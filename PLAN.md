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

Single model for all AI tasks (identification, chat, summarization). Most modern models support both text and vision input.

Configuration is entirely via environment variables — no settings UI for keys or models.

| Variable | Default | Description |
|----------|---------|-------------|
| `FLOWL_AI_API_KEY` | — | OpenAI API key (required to enable AI) |
| `FLOWL_AI_BASE_URL` | `https://api.openai.com/v1` | API base URL (change for compatible services) |
| `FLOWL_AI_MODEL` | `gpt-4o-mini` | Model for all AI tasks (must support vision) |

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

### 2. AI Chat (Health Check, Care Q&A, Watering Advice)

**What:** A unified conversational interface for all plant AI interactions. The user can ask care questions, share photos of problems, get health assessments, and request watering advice — all in one chat. The AI has full context about the plant (species, care profile, watering history, care log). At the end, a "Save note" button asks the AI to summarize the conversation into a care journal entry.

**Where:** Slide-out drawer on desktop, bottom sheet on mobile. Triggered by an "Ask AI" button on the Plant Detail hero section.

**Desktop — drawer slides in from the right:**

```
┌──── Plant Detail ─────────────┬── AI Chat ───────────────────┐
│                               │                              │
│  ┌────────┐ Monstera          │  ✨ Monstera            [✕]  │
│  │ photo  │ M. deliciosa      │                              │
│  │        │ 📍 Living Room    │  Quick questions:            │
│  │        │ ● Ok — in 3 days  │  [Health check] [Watering?]  │
│  └────────┘                   │  [When to repot?]            │
│  [💧 Water] [✨ Ask AI]       │                              │
│                               │  You: The lower leaves are   │
│  Watering                     │  turning yellow              │
│  Every 7 days                 │  ┌──────────┐                │
│  ...                          │  │ 📷 photo │                │
│                               │  └──────────┘                │
│  Care Info                    │                              │
│  ...                          │  flowl: Based on your photo  │
│                               │  and watering every 7 days,  │
│  (dimmed behind drawer)       │  this looks like             │
│                               │  overwatering. The yellowing │
│                               │  pattern on lower leaves is  │
│                               │  typical. Try extending to   │
│                               │  10-12 days between          │
│                               │  waterings.                  │
│                               │                              │
│                               │  ┌──────────────────┐ [📷]   │
│                               │  │ Ask...           │ [Send] │
│                               │  └──────────────────┘        │
│                               │           [Save note]        │
└───────────────────────────────┴──────────────────────────────┘
```

**Mobile — bottom sheet slides up (~90% height):**

```
┌─────────────────────────────┐
│  Plant Detail (dimmed)      │
│  ┌────────┐ Monstera        │
│  │ photo  │ M. deliciosa    │
│  └────────┘                 │
├─── ▬▬▬ drag handle ▬▬▬ ────┤
│  ✨ Monstera            [✕] │
│                             │
│  You: Lower leaves are      │
│  turning yellow             │
│  ┌──────────┐               │
│  │ 📷 photo │               │
│  └──────────┘               │
│                             │
│  flowl: Based on your       │
│  photo and watering every   │
│  7 days, this looks like    │
│  overwatering...            │
│                             │
│                             │
│  ┌───────────────────┐      │
│  │ Ask...            │ [>]  │
│  └───────────────────┘      │
│  [📷]          [Save note]  │
└─────────────────────────────┘
   ▬▬▬▬ nav bar ▬▬▬▬▬▬▬▬▬▬▬▬
```

**Behavior:**
- "Ask AI" button on Plant Detail hero section opens the drawer/sheet
- Plant context header at the top (name, species, status) — compact, not a full card
- Quick-question chips shown when chat is empty (context-aware — e.g. "Why overdue?" if plant is overdue, "Health check" always available)
- **Inline photo upload:** attach a photo to any message (file picker or camera on mobile via `capture="environment"`). Photos are sent to the vision model alongside the text. Photos are ephemeral — not stored on the plant or in the DB
- Streaming responses with typing indicator
- Markdown rendering for AI responses (bold, lists, etc.)
- Chat history persists for the session only (not stored in DB)
- Closing and reopening the drawer within the same page visit preserves the conversation
- Swipe down to dismiss on mobile

**"Save note" flow:**
1. User taps "Save note" at the bottom of the chat
2. An additional AI call summarizes the conversation into 1–3 sentences
3. Summary appears in an editable text field for the user to review/adjust
4. User confirms → saved as a care journal entry with a new event type `ai-consultation`
5. The saved entry appears in the plant's care journal and the global care journal timeline

**New care event type: `ai-consultation`**
- Icon: sparkle/wand (matching the "Ask AI" button)
- Color: purple (distinct from water=blue, fertilize=orange, repot=green, prune=gray)
- Shows the AI-generated summary as the event notes
- Displayed in care journal timeline like any other event

**Context sent to AI (system prompt):**

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

**API endpoints:**

```
POST /api/ai/chat
Content-Type: application/json

{
  "plant_id": 42,
  "message": "The lower leaves are turning yellow",
  "image": "<base64>",           (optional — attached photo)
  "history": [
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." }
  ]
}

→ 200 (streamed, text/event-stream)
data: {"delta": "Based on your photo "}
data: {"delta": "and watering every 7 days, "}
data: {"delta": "this looks like overwatering..."}
data: {"done": true}
```

```
POST /api/ai/summarize
Content-Type: application/json

{
  "plant_id": 42,
  "history": [
    { "role": "user", "content": "..." },
    { "role": "assistant", "content": "..." }
  ]
}

→ 200 {
    "summary": "Diagnosed yellowing lower leaves as overwatering. Recommended extending watering interval from 7 to 10-12 days and letting soil dry between waterings."
  }
```

---

## Settings — AI Status

New section in the Settings page, between "MQTT" and "Data". Read-only status indicator — mirrors the MQTT section pattern. All configuration lives in env vars.

```
┌ AI Assistant ────────────────────────────┐
│                                          │
│  ● Enabled                               │
│  Provider   api.openai.com               │
│  Model      gpt-4o-mini                  │
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
GET /api/ai/status → { "enabled": true, "base_url": "https://api.openai.com/v1", "model": "gpt-4o-mini" }
```

---

## Data Model Changes

### New care event type

Add `ai-consultation` to the set of valid event types (`watered`, `fertilized`, `repotted`, `pruned`, `custom`, **`ai-consultation`**). No schema migration needed — `event_type` is a free-text column. Frontend needs the new type in its filter chips, icon map, and color map.

---

## Backend Design

### AI Provider Trait

```rust
#[async_trait]
trait AiProvider: Send + Sync {
    /// Identify a plant from photos. Returns species + care profile + summary.
    async fn identify(&self, images: Vec<&[u8]>) -> Result<IdentifyResult>;

    /// Chat about a specific plant. Supports text + optional image.
    /// Streams response tokens via a channel.
    async fn chat(&self, context: &PlantContext, message: &str, image: Option<&[u8]>, history: &[ChatMessage]) -> Result<ChatResponseStream>;

    /// Summarize a chat conversation into a short care journal note.
    async fn summarize(&self, context: &PlantContext, history: &[ChatMessage]) -> Result<String>;
}
```

One implementation for now: `OpenAiProvider` (works with any OpenAI-compatible API). Ollama provider can be added later behind the same trait.

All three methods use the same model (`FLOWL_AI_MODEL`):
- `identify` — JSON mode, multi-image input
- `chat` — plain text streaming, optional image input
- `summarize` — JSON mode, text only

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
- Chat responses (feature 2) don't use JSON mode — they stream plain text via SSE
- Summarize endpoint uses JSON mode to return a structured `{ "summary": "..." }` response

### New Rust Dependencies

| Crate | Purpose |
|-------|---------|
| `reqwest` | HTTP client for AI API calls |
| `async-trait` | Trait with async methods |
| `base64` | Encode images for API payloads |
| `tokio-stream` | SSE streaming for chat responses |

---

## Implementation Phases

### Phase 1 — Backend AI Foundation

Env vars, provider trait, OpenAI client. No UI yet — testable via curl.

- [x] AI config in `Config` struct (env vars: API key, base URL, model)
- [x] AI provider trait definition (`identify`, `chat`, `summarize`)
- [x] OpenAI provider: `identify` method (JSON mode, multi-image)
- [x] `Option<Arc<dyn AiProvider>>` in `AppState` (None when no API key)
- [x] `GET /api/ai/status` endpoint

### Phase 2 — Settings UI + Status

AI status indicator in the Settings page. First visible AI presence in the app.

- [x] AI status section in Settings UI (enabled/disabled, provider, model)
- [x] i18n keys for AI status labels

### Phase 3 — Identify Endpoint

Backend endpoint for plant identification. Testable via curl with a photo.

- [x] `POST /api/ai/identify` endpoint (accepts multi-photo multipart)
- [x] Response deserialization with serde (IdentifyResult, CareProfile)
- [x] Error handling (AI disabled, bad response, API errors)

### Phase 4 — Identify UI

"Identify Plant" button and suggestion card in PlantForm.

- [x] "Identify Plant" button in `PlantForm` (visible when photo present + AI enabled)
- [x] Optional extra photo slots (leaf close-up, stem/pot)
- [x] Loading state (shimmer/spinner)
- [x] Suggestion card with field preview + "Apply to form" / "Dismiss"
- [x] Auto-fill form fields including notes summary
- [x] i18n keys for identification UI

### Phase 4.1 — Multiple Identification Suggestions

Return 3 ranked suggestions in a single API call instead of one. Carousel UI to slide between them.

- [x] `IdentifyResponse { suggestions: Vec<IdentifyResult> }` wrapper type
- [x] Provider trait + OpenAI provider return `IdentifyResponse`
- [x] Updated prompt requesting top 3 identifications
- [x] Updated JSON schema wrapping results in `{ "suggestions": [...] }`
- [x] `POST /api/ai/identify` returns `Json<IdentifyResponse>`
- [x] Frontend `identifyPlant()` returns array of suggestions
- [x] Carousel UI in suggestion card (chevron nav, dot indicators, counter)
- [x] Touch swipe navigation on mobile
- [x] i18n keys for suggestion navigation (en/de/es)

### Phase 5 — Chat Backend

Streaming chat endpoint + summarize endpoint. Testable via curl.

- [ ] OpenAI provider: `chat` method (SSE streaming, optional image support)
- [ ] OpenAI provider: `summarize` method (JSON mode)
- [ ] `POST /api/ai/chat` endpoint (SSE streaming, accepts text + optional base64 image)
- [ ] `POST /api/ai/summarize` endpoint
- [ ] Plant context builder (loads plant + recent care events for system prompt)

### Phase 6 — Chat Drawer UI

Drawer/bottom sheet component with text-only chat. Core chat experience.

- [ ] Chat drawer component (slides from right on desktop)
- [ ] Bottom sheet component (slides up on mobile, drag to dismiss)
- [ ] "Ask AI" button on Plant Detail hero section
- [ ] Chat message list (user messages, AI responses)
- [ ] Text input + send button
- [ ] Streaming response rendering with typing indicator
- [ ] Quick-question chips (context-aware)

### Phase 7 — Chat Photos + Save Note

Photo attachments in chat messages and journal save flow.

- [ ] Inline photo upload in chat (file picker + camera capture on mobile)
- [ ] Photo preview in sent messages
- [ ] Markdown rendering for AI responses
- [ ] New care event type `ai-consultation` (icon, color, i18n)
- [ ] "Save note" button in chat
- [ ] AI summary generation → editable text field → save as care journal entry

### Future — Additional Providers

- [ ] Ollama provider implementation (behind the same trait)
- [ ] Provider selection via env var or Settings UI
