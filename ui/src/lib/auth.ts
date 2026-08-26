import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { isProtectedCacheName } from "$lib/sw-policy";

export const MAX_RETURN_TO_BYTES = 2048;

export interface EnabledAuthConfig {
  enabled: true;
  provider_name: string;
}

export interface DisabledAuthConfig {
  enabled: false;
  provider_name: null;
}

export type AuthConfig = EnabledAuthConfig | DisabledAuthConfig;

export type FetchLike = typeof fetch;

function byteLength(value: string): number {
  return new TextEncoder().encode(value).byteLength;
}

function hasControlCharacter(value: string): boolean {
  return [...value].some((character) => {
    const code = character.charCodeAt(0);
    return code <= 0x1f || code === 0x7f;
  });
}

/** Returns a bounded, non-recursive local navigation target or `/`. */
export function safeLocalTarget(target: string | null | undefined): string {
  if (!target || byteLength(target) > MAX_RETURN_TO_BYTES) return "/";
  if (!target.startsWith("/") || target.startsWith("//")) return "/";
  if (/\\/.test(target) || hasControlCharacter(target)) return "/";
  let decoded: string;
  try {
    decoded = decodeURIComponent(target);
  } catch {
    return "/";
  }
  if (/\\/.test(decoded) || hasControlCharacter(decoded)) return "/";

  const pathEnd = target.search(/[?#]/);
  const rawPath = pathEnd === -1 ? target : target.slice(0, pathEnd);
  let decodedPath: string;
  try {
    decodedPath = decodeURIComponent(rawPath);
  } catch {
    return "/";
  }

  if (decodedPath.includes("%") || /%(?:2f|5c|3f|23)/i.test(rawPath)) {
    return "/";
  }
  if (
    decodedPath
      .split("/")
      .some((segment) => segment === "." || segment === "..") ||
    decodedPath === "/login" ||
    decodedPath.startsWith("/login/") ||
    decodedPath === "/auth" ||
    decodedPath.startsWith("/auth/")
  ) {
    return "/";
  }

  try {
    if (
      new URL(target, "https://flowl.invalid").origin !==
      "https://flowl.invalid"
    ) {
      return "/";
    }
  } catch {
    return "/";
  }

  return target;
}

export function currentReturnTarget(
  location: Pick<Location, "pathname" | "search" | "hash">,
): string {
  return safeLocalTarget(
    `${location.pathname}${location.search}${location.hash}`,
  );
}

export function loginUrl(target: string): string {
  return `/login?return_to=${encodeURIComponent(safeLocalTarget(target))}`;
}

export async function fetchAuthConfig(
  fetchFn: FetchLike = fetch,
): Promise<AuthConfig> {
  const response = await fetchFn("/auth/config");
  if (!response.ok) {
    throw new Error("Authentication configuration is unavailable");
  }

  const data: unknown = await response.json();
  if (
    typeof data !== "object" ||
    data === null ||
    typeof (data as { enabled?: unknown }).enabled !== "boolean"
  ) {
    throw new Error("Authentication configuration is invalid");
  }

  const config = data as { enabled: boolean; provider_name?: unknown };
  if (
    (config.enabled && typeof config.provider_name !== "string") ||
    (!config.enabled && config.provider_name !== null)
  ) {
    throw new Error("Authentication configuration is invalid");
  }

  return config as AuthConfig;
}

let authNavigationPending = false;

export function resetAuthNavigationForTests(): void {
  authNavigationPending = false;
}

export function navigateToLogin(
  location: Pick<Location, "pathname" | "search" | "hash"> = window.location,
  navigate: (url: string) => void = (url) => {
    // The query comes from safeLocalTarget; /auth routes are backend-only.
    // eslint-disable-next-line svelte/no-navigation-without-resolve
    void goto(`${resolve("/login")}${url.slice("/login".length)}`, {
      replaceState: true,
    });
  },
): boolean {
  if (
    authNavigationPending ||
    location.pathname === "/login" ||
    location.pathname === "/auth" ||
    location.pathname.startsWith("/auth/")
  ) {
    return false;
  }

  authNavigationPending = true;
  navigate(loginUrl(currentReturnTarget(location)));
  return true;
}

async function purgeCachesDirectly(): Promise<void> {
  if (typeof caches === "undefined") return;
  const names = await caches.keys();
  await Promise.all(
    names.filter(isProtectedCacheName).map((name) => caches.delete(name)),
  );
}

/** Clears protected offline data, waiting for a controlling worker when present. */
export async function purgeProtectedCaches(): Promise<void> {
  const controller = navigator.serviceWorker?.controller;
  if (!controller || typeof MessageChannel === "undefined") {
    await purgeCachesDirectly();
    return;
  }

  const acknowledged = await new Promise<boolean>((resolve) => {
    const channel = new MessageChannel();
    const timeout = setTimeout(() => resolve(false), 2_000);
    channel.port1.onmessage = (event) => {
      if (event.data?.type === "PROTECTED_CACHES_PURGED") {
        clearTimeout(timeout);
        resolve(true);
      }
    };
    try {
      controller.postMessage({ type: "PURGE_PROTECTED_CACHES" }, [
        channel.port2,
      ]);
    } catch {
      clearTimeout(timeout);
      resolve(false);
    }
  });

  if (!acknowledged) await purgeCachesDirectly();
}

export function setWorkerAuthMode(
  worker: Pick<ServiceWorker, "postMessage"> | null | undefined,
  enabled: boolean,
): void {
  worker?.postMessage({ type: "SET_AUTH_ENABLED", enabled });
}

export function setServiceWorkerAuthMode(enabled: boolean): void {
  setWorkerAuthMode(navigator.serviceWorker?.controller, enabled);
}

/** Submits a real same-origin form so the backend owns logout navigation. */
export function postLogout(): void {
  const form = document.createElement("form");
  form.method = "POST";
  form.action = "/auth/logout";
  form.style.display = "none";
  document.body.append(form);
  form.submit();
}
