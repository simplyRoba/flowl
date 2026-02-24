## 1. API Client & Types

- [x] 1.1 Add `IdentifyResult` and `CareProfile` TypeScript interfaces to `ui/src/lib/api.ts`
- [x] 1.2 Add `identifyPlant(photos: File[]): Promise<IdentifyResult>` function to `ui/src/lib/api.ts` that sends multipart `POST /api/ai/identify`

## 2. i18n

- [x] 2.1 Add `identify` translation group to `ui/src/lib/i18n/en.ts` with all keys: `identifyPlant`, `identifying`, `extraPhotosHint`, `closeUp`, `stemPot`, `aiSuggestion`, `willFill`, `applyToForm`, `dismiss`, `applied`, `undo`, `errorMessage`, `retry`, `confidence`
- [x] 2.2 Add matching `identify` translation group to `ui/src/lib/i18n/de.ts` and `ui/src/lib/i18n/es.ts`

## 3. PlantForm Identify Section

- [x] 3.1 Add AI status check on mount (`fetchAiStatus()`) and `aiEnabled` state to `PlantForm.svelte`
- [x] 3.2 Add identify state machine (`identifyState: 'idle' | 'loading' | 'result' | 'applied' | 'error'`), `identifyResult` state, and `previousValues` snapshot state
- [x] 3.3 Add extra photo state variables (`extraPhoto1`, `extraPhoto2` with preview URLs) and handlers for selecting/removing extra photos
- [x] 3.4 Add `--color-ai` and `--color-ai-tint` CSS custom properties to the layout root styles
- [x] 3.5 Implement idle state UI: identify button with sparkle icon, extra photo upload slots (72×72 desktop, 64×64 mobile, dashed border, camera icon, labels "Close-up" / "Stem / pot"), visibility gated on `hasPhoto && aiEnabled`
- [x] 3.6 Implement `handleIdentify` function: collect main photo (from `photoFile` or fetch existing `photo_url` as blob), append extras, call `identifyPlant()`, transition states on success/error
- [x] 3.7 Implement loading state UI: spinner, "Identifying..." text, submitted photo thumbnails, shimmer skeleton lines
- [x] 3.8 Implement suggestion card UI: scientific name heading, confidence badge, common name, summary, "will fill" chips computed from valid AI values, "Apply to form" and "Dismiss" buttons
- [x] 3.9 Implement `handleApply` function: snapshot current values, validate each AI value against allowed options, write valid values to form state, count applied fields, transition to `applied` state
- [x] 3.10 Implement applied state UI: success banner with field count and "Undo" button; implement `handleUndo` to restore snapshot and return to idle
- [x] 3.11 Implement error state UI: error message with "Retry" button; "Retry" re-triggers `handleIdentify`
- [x] 3.12 Add responsive styles: mobile (≤ 768px) stacks action buttons full-width with 44px min-height, shrinks extra photo slots to 64×64, proper gaps/margins/padding throughout all states

## 4. Verification

- [x] 4.1 Run `npm run check` in `ui/`, fix any type errors
- [x] 4.2 Run `cargo fmt`, `cargo clippy`, and `cargo test`, fix any issues
- [x] 4.3 Manual test: verify all 6 states render correctly at mobile (375px), tablet (768px), and desktop (1280px) widths
