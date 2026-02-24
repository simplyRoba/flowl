## Context

The `POST /api/ai/identify` endpoint is live and returns `IdentifyResult` as JSON (common name, scientific name, confidence, summary, care profile). The `PlantForm` component handles both Add Plant (`/plants/new`) and Edit Plant (`/plants/[id]/edit`) screens. It already manages photo upload, icon picker, and all care fields. The `fetchAiStatus()` API function exists and the Settings page already consumes it. No frontend code calls the identify endpoint yet.

The form uses Svelte 5 runes (`$state`, `$derived`, `$effect`) and follows a section-based layout with `.section` cards. CSS uses the app's design tokens (CSS custom properties). The app supports three locales (en, de, es) via a translation store.

## Goals / Non-Goals

**Goals:**
- Add an identify UI section to PlantForm that calls `POST /api/ai/identify` and lets the user apply results to the form
- Support 1–3 photos (the main plant photo plus up to 2 optional extras)
- Show clear loading, success, error, and "applied" states
- Auto-fill form fields from the AI response when the user clicks "Apply"
- Work responsively across mobile, tablet, and desktop
- Add i18n keys for all new UI strings

**Non-Goals:**
- No backend changes — the endpoint and types are already done
- No image resizing or compression before upload
- No persistence of extra photos — they are ephemeral, used only for the identify call
- No identify history or caching of results
- No standalone identify page — this lives entirely within PlantForm

## Decisions

### 1. Identify section placement: inside the media section, below photo actions

The identify section renders inside the media `<section>` card, directly below the photo preview and its action buttons (Replace photo / Use icon instead). It appears only when two conditions are met: a photo is present (`hasPhoto` is true) and AI is enabled.

**Why here?** The identify action is contextually tied to the photo — "you uploaded a photo, now identify what's in it." Placing it below the photo makes the connection obvious. Placing it as a separate section would disconnect it from the image.

**AI status check:** On mount, the form calls `fetchAiStatus()` and stores `aiEnabled` as a `$state` boolean. This is a single lightweight GET request that the Settings page already makes; no new API surface.

### 2. Extra photo slots: inline thumbnails within the identify section

Two optional upload targets (88×88px on desktop, 80×80px on mobile) labeled "Close-up" and "Stem / pot" appear below the identify button. They use dashed borders when empty and show a thumbnail preview when filled. Each has a remove button.

**Why inline, not a modal?** Inline slots are lower friction — the user sees them immediately and can optionally fill them without navigating away. A modal would add interaction cost for an optional feature.

**Why fixed labels?** Two labeled slots ("Close-up" and "Stem / pot") are clearer than a generic "+ Add photo" that leaves the user guessing what to add. The labels guide the user toward photos that improve identification accuracy.

Extra photos are stored as `$state<File | null>` variables (`extraPhoto1`, `extraPhoto2`) with corresponding preview URLs. They are never uploaded to the server as plant photos — they exist only for the identify call.

### 3. State machine: five visual states

The identify section cycles through states managed by a `$state` variable:

| State | Visual | Transitions to |
|-------|--------|----------------|
| `idle` | Identify button + extra photo slots | `loading` (on click) |
| `loading` | Spinner, submitted photo thumbnails, shimmer skeleton | `result`, `error` |
| `result` | Suggestion card with Apply/Dismiss | `applied` (Apply), `idle` (Dismiss) |
| `applied` | Success banner with Undo | `idle` (Undo) |
| `error` | Error message with Retry | `loading` (Retry), `idle` (dismiss) |

**Why a string state instead of multiple booleans?** A single `identifyState: 'idle' | 'loading' | 'result' | 'applied' | 'error'` is easier to reason about and prevents impossible combinations (e.g., loading + error).

### 4. API call: multipart FormData from the frontend

The new `identifyPlant(photos: File[])` function in `api.ts` builds a `FormData` with each file appended under the field name `photos`, then calls `POST /api/ai/identify`. It returns the parsed `IdentifyResult` or throws.

```typescript
export interface IdentifyResult {
  common_name: string;
  scientific_name: string;
  confidence: number | null;
  summary: string | null;
  care_profile: CareProfile | null;
}

export interface CareProfile {
  watering_interval_days: number | null;
  light_needs: string | null;
  difficulty: string | null;
  pet_safety: string | null;
  growth_speed: string | null;
  soil_type: string | null;
  soil_moisture: string | null;
}
```

The main photo (from `photoFile` or fetched from `initial.photo_url`) must be included. If `initial.photo_url` exists but `photoFile` is null (editing an existing plant), the photo is fetched as a blob first. Extra photos are appended if present.

### 5. "Apply to form" behavior

Clicking "Apply" writes the AI result into the form's `$state` variables:

| AI field | Form variable | Condition |
|----------|---------------|-----------|
| `scientific_name` | `species` | Always |
| `common_name` | `name` | Only if `name` is empty |
| `summary` | `notes` | Only if `notes` is empty |
| `care_profile.watering_interval_days` | `wateringDays` | If present |
| `care_profile.light_needs` | `lightNeeds` | If present and valid (`direct`/`indirect`/`low`) |
| `care_profile.difficulty` | `difficulty` | If present and valid |
| `care_profile.pet_safety` | `petSafety` | If present and valid |
| `care_profile.growth_speed` | `growthSpeed` | If present and valid |
| `care_profile.soil_type` | `soilType` | If present and valid |
| `care_profile.soil_moisture` | `soilMoisture` | If present and valid |

**Validation of AI values:** The AI may return values that don't match the form's allowed options (e.g., `light_needs: "bright"` instead of `"indirect"`). Each value is checked against the form's valid set before applying. Invalid values are silently skipped.

**Undo:** Before applying, the current form values are snapshot into a `previousValues` object. "Undo" restores them. The snapshot is cleared when the user manually edits any field or navigates away.

### 6. "Will fill" preview chips

The suggestion card shows green chips for each field that will be set. The chips are computed from the AI result: for each field in the mapping above, if the AI returned a value and that value is valid, a chip is shown with the field label and the value in parentheses (e.g., "Watering (10d)", "Light (indirect)").

This lets the user see exactly what will change before clicking "Apply."

### 7. Suggestion card styling: purple/violet AI accent

The identify section uses a distinct `--color-ai` (purple/violet) to visually separate AI features from the green primary color. This follows the PLAN.md convention where AI features use a sparkle/purple theme. The color is defined as a new CSS custom property in the layout styles.

### 8. i18n approach

New keys are added under a new `identify` group in the translation dictionaries:

```
identify.identifyPlant       → "Identify Plant"
identify.identifying         → "Identifying..."
identify.extraPhotosHint     → "Add more photos for better accuracy (optional):"
identify.closeUp             → "Close-up"
identify.stemPot             → "Stem / pot"
identify.aiSuggestion        → "AI Suggestion"
identify.willFill            → "Will fill:"
identify.applyToForm         → "Apply to form"
identify.dismiss             → "Dismiss"
identify.applied             → "AI suggestion applied — {n} fields updated"
identify.undo                → "Undo"
identify.errorMessage        → "Identification failed. The AI service might be temporarily unavailable."
identify.retry               → "Retry"
identify.confidence          → "{n}%"
```

### 9. Responsive behavior

| Breakpoint | Changes |
|------------|---------|
| Desktop (> 768px) | Extra photo slots 88×88px, suggestion actions side by side, "will fill" chips wrap naturally |
| Mobile (≤ 768px) | Extra photo slots 80×80px, suggestion actions stack full-width with 44px min-height touch targets, photo preview 200×200px |
| Wide (≥ 1280px) | No additional changes beyond existing form width increase |

### 10. Fetching existing photo for edit form

When editing a plant that has an existing `photo_url` but no new `photoFile`, the identify function needs the photo bytes to send to the API. The approach: `fetch(initial.photo_url)` → `.blob()` → construct a `File` from the blob. This avoids duplicating the photo upload logic and reuses the same `identifyPlant(photos)` call.

## Risks / Trade-offs

**[Large photo uploads over slow connections]** → The identify call sends raw photos (up to 3 × ~5 MB) to the backend, which then base64-encodes and forwards them to the AI API. On slow connections this could take a while. Mitigation: the loading state with shimmer gives visual feedback; the 30 MB backend limit prevents runaway requests. No client-side compression is added (non-goal) to keep complexity low.

**[AI returns unexpected care values]** → The AI might return values like `"bright indirect"` instead of `"indirect"` for light needs. Mitigation: each value is validated against the known option set before applying. Invalid values are silently skipped and the corresponding "will fill" chip is not shown. This is a graceful degradation — the user still gets the species name and summary even if some care fields don't match.

**[Undo complexity]** → Storing and restoring a snapshot of all form fields adds state management surface. Mitigation: the snapshot is a simple object of primitive values, not deeply nested. It is cleared aggressively (on any manual edit or navigation) to avoid stale state. If undo proves too complex during implementation, it can be cut — the user can always manually edit fields after applying.

**[Fetching existing photo_url on edit]** → Fetching the plant's existing photo as a blob to send to the identify API requires an extra HTTP round-trip. Mitigation: this only happens on the edit form when the user has not selected a new photo, and the photo is served from the local server (fast). The fetch is initiated only when the user clicks "Identify," not on page load.
