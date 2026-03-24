## 1. Backend types and error code

- [x] 1.1 Add `rejected: Option<bool>` and `rejected_reason: Option<String>` fields to `IdentifyResponse` in `src/ai/types.rs`
- [x] 1.2 Add `AI_IDENTIFY_NOT_A_PLANT` entry to `default_message` in `src/api/error.rs` with message `"The photo does not appear to contain a plant"`
- [x] 1.3 Add unit tests for `IdentifyResponse` deserialization with `rejected: true` / `rejected: false`

## 2. AI provider schema and prompt

- [x] 2.1 Add `rejected` (boolean, required) and `rejected_reason` (string or null, required) to the JSON schema in `OpenAiProvider::identify` in `src/ai/openai.rs`
- [x] 2.2 Update `build_identify_prompt` in `src/ai/prompts.rs` to instruct the model to set `rejected: true` with a reason when the photo does not contain a plant
- [x] 2.3 Add unit test asserting the identify prompt contains the rejection instruction

## 3. Endpoint rejection handling

- [x] 3.1 In `identify_plant` handler in `src/api/ai.rs`, check `result.rejected == Some(true)` after the provider call — log `rejected_reason` at `warn` level and return `ApiError::Validation("AI_IDENTIFY_NOT_A_PLANT")`
- [x] 3.2 Add integration test in `tests/ai_identify.rs` with a mock provider returning `rejected: true` — assert 422 response with `AI_IDENTIFY_NOT_A_PLANT` code
- [x] 3.3 Add integration test with a mock provider returning `rejected: false` with suggestions — assert 200 with suggestions as before

## 4. Frontend i18n and error handling

- [x] 4.1 Add `AI_IDENTIFY_NOT_A_PLANT` to `errorCode` map in `ui/src/lib/i18n/en.ts`, `de.ts`, and `es.ts`
- [x] 4.2 Add UI test in `ui/src/tests/components/IdentifyPanel.test.ts` asserting the not-a-plant error code is displayed correctly

## 5. Verification

- [x] 5.1 Run `npm run check --prefix ui`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`, and `cargo test`
