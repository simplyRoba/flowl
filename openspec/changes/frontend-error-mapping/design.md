## Context

The backend (after `backend-error-codes`) returns `{"code": "PLANT_NAME_REQUIRED", "message": "Plant name is required"}` on every error. The frontend currently ignores any structured error data and shows `e.message` (the raw English text from the response body) or falls back to a generic i18n key when the error isn't an `Error` instance.

The `localizeLocationError` function in `locations.ts` is the only attempt at translation, and it works by regex-matching the English message string.

## Goals / Non-Goals

**Goals:**

- Every API error shown to the user is displayed in the active locale.
- A single `resolveError` helper replaces the scattered `e instanceof Error ? e.message : t.error.xxx` pattern.
- The `localizeLocationError` hack is removed.
- Unknown error codes fall back to a generic localized message, never raw English.

**Non-Goals:**

- Parameterized error messages (e.g., interpolating a location name into `LOCATION_ALREADY_EXISTS`). The generic translated string is sufficient for now.
- Changing error display patterns (inline vs toast). That's covered by the notifications spec.
- Backend changes (covered by `backend-error-codes`).

## Decisions

### 1. `ApiError` gains a `code` property

```typescript
class ApiError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}
```

The `request()` function parses `code` from the JSON error body. If `code` is missing (e.g., non-JSON error), it defaults to `"UNKNOWN_ERROR"`.

### 2. `errorCode` i18n group

A new top-level group in each locale dictionary maps codes to translated strings:

```typescript
errorCode: {
  PLANT_NOT_FOUND: "Plant not found",
  PLANT_NAME_REQUIRED: "Plant name is required",
  LOCATION_ALREADY_EXISTS: "A location with this name already exists",
  INTERNAL_ERROR: "Something went wrong. Please try again.",
  // ...all codes from the backend catalog
}
```

### 3. `resolveError` helper

A shared function used by all stores:

```typescript
function resolveError(e: unknown, fallbackKey: keyof typeof translations.error): string {
  const t = get(translations);
  if (e instanceof ApiError && e.code in t.errorCode) {
    return t.errorCode[e.code as keyof typeof t.errorCode];
  }
  return t.error[fallbackKey];
}
```

This replaces every `e instanceof Error ? e.message : t.error.xxx` with `resolveError(e, 'xxx')`.

### 4. Fallback strategy

If an `ApiError` has a code not in `errorCode`, the fallback key is used. This handles:
- Future backend codes not yet added to i18n
- Non-API errors (network failures, etc.)
- Unexpected error shapes

## Risks / Trade-offs

- **Keeping errorCode dict in sync with backend** -> Low risk since codes are added rarely and tests can verify coverage.
- **Losing the location name in conflict errors** -> The current regex extracts the name for display. The new generic message ("A location with this name already exists") is slightly less specific but works across all locales without string parsing. Acceptable trade-off.
- **Large i18n dict addition** -> ~30 keys per locale. Manageable.
