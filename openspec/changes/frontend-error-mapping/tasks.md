## 1. API error parsing

- [ ] 1.1 Add `code` property to `ApiError` class in `ui/src/lib/api.ts`
- [ ] 1.2 Parse `code` from JSON error responses in the `request()` function, defaulting to `"UNKNOWN_ERROR"` when absent

## 2. i18n error code group

- [ ] 2.1 Add `errorCode` group to English dictionary with all backend error codes
- [ ] 2.2 Add `errorCode` group to German dictionary
- [ ] 2.3 Add `errorCode` group to Spanish dictionary

## 3. Error resolution helper

- [ ] 3.1 Create a `resolveError(e: unknown, fallbackKey)` helper that maps `ApiError.code` to `t.errorCode[code]` with fallback
- [ ] 3.2 Replace `e instanceof Error ? e.message : t.error.xxx` pattern in `plants.ts` store
- [ ] 3.3 Replace pattern in `care.ts` store
- [ ] 3.4 Replace pattern in `locations.ts` store and remove `localizeLocationError`

## 4. Tests and verification

- [ ] 4.1 Add tests for `resolveError` with known code, unknown code, and non-ApiError
- [ ] 4.2 Update existing store tests if they assert on error message content
- [ ] 4.3 Run `npm run check --prefix ui`, `npm run lint --prefix ui`, and `npm test --prefix ui`
- [ ] 4.4 Mark U1 as done in `REVIEW.md`
- [ ] 4.5 Remove the "Error Handling & Localization" section from `TODO.md`
