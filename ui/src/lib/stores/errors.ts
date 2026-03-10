import { get } from "svelte/store";
import { ApiError } from "$lib/api";
import type { Translations } from "$lib/i18n/en";
import { translations } from "./locale";

export function resolveError(
  e: unknown,
  fallbackKey: keyof Translations["error"],
): string {
  const t = get(translations);
  if (e instanceof ApiError && e.code in t.errorCode) {
    return t.errorCode[e.code as keyof typeof t.errorCode];
  }
  return t.error[fallbackKey];
}
