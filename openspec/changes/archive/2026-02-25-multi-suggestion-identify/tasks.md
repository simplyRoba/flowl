## 1. Backend types and trait

- [x] 1.1 Add `IdentifyResponse` wrapper struct to `src/ai/types.rs` with `suggestions: Vec<IdentifyResult>`, deriving `Serialize`/`Deserialize`
- [x] 1.2 Add unit tests for `IdentifyResponse` deserialization (multiple suggestions, single suggestion, round-trip)
- [x] 1.3 Change `identify` return type in `AiProvider` trait (`src/ai/provider.rs`) from `IdentifyResult` to `IdentifyResponse`

## 2. OpenAI provider

- [x] 2.1 Update `OpenAiProvider::identify` prompt to request top 3 most likely identifications ranked by confidence
- [x] 2.2 Update JSON schema in `OpenAiProvider::identify` to wrap results in `{ "suggestions": [...] }` envelope with array of `IdentifyResult` items
- [x] 2.3 Deserialize response into `IdentifyResponse` instead of `IdentifyResult`
- [x] 2.4 Update `identify_request_payload_structure` test to verify new schema shape

## 3. API endpoint

- [x] 3.1 Change `identify_plant` handler in `src/api/ai.rs` to return `Json<IdentifyResponse>`
- [x] 3.2 Update debug logging to log suggestion count and first suggestion's name/confidence

## 4. Frontend API layer

- [x] 4.1 Add `IdentifyResponse` interface to `ui/src/lib/api.ts`
- [x] 4.2 Update `identifyPlant()` return type from `Promise<IdentifyResult>` to `Promise<IdentifyResponse>`

## 5. PlantForm carousel state and logic

- [x] 5.1 Replace `identifyResult` state with `identifyResults: IdentifyResult[]` array and `currentSuggestion` index
- [x] 5.2 Add `activeSuggestion` derived value from `identifyResults[currentSuggestion]`
- [x] 5.3 Update `willFillChips` derivation to use `activeSuggestion`
- [x] 5.4 Update `handleIdentify` to store `response.suggestions` and set `currentSuggestion = 0`
- [x] 5.5 Update `handleApply` to read from `activeSuggestion`
- [x] 5.6 Update `handleDismiss` to clear the results array
- [x] 5.7 Add `prevSuggestion()` / `nextSuggestion()` navigation handlers with wrap-around

## 6. PlantForm carousel UI

- [x] 6.1 Add suggestion counter ("1 / 3") to the suggestion header
- [x] 6.2 Add left/right chevron navigation buttons around dot indicators
- [x] 6.3 Add dot indicators (filled for active, outline for inactive, clickable)
- [x] 6.4 Hide navigation controls when only 1 suggestion is returned
- [x] 6.5 Add touch swipe handling with pointer events (50px threshold)
- [x] 6.6 Style carousel navigation (dots, chevrons, counter) with existing CSS variables

## 7. i18n

- [x] 7.1 Add `suggestionCount`, `prevSuggestion`, `nextSuggestion` keys to `en.ts`
- [x] 7.2 Add corresponding keys to `de.ts`
- [x] 7.3 Add corresponding keys to `es.ts`

## 8. Verification

- [x] 8.1 Run `cargo fmt`, `cargo clippy`, and `cargo test`
- [x] 8.2 Run `cd ui && npm run check`
