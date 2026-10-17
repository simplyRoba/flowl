import { goto } from "$app/navigation";
import { resolve } from "$app/paths";
import { isProtectedCacheName } from "#lib/sw-policy.js";
import { parseAuthConfig, type AuthConfig } from "#lib/auth-config.js";
import {
  loginUrl as commonLoginUrl,
  returnToFromQuery as commonReturnToFromQuery,
  safeLocalTarget as commonSafeLocalTarget,
} from "#lib/auth-core.js";
export { MAX_RETURN_TO_BYTES } from "#lib/auth-return.js";
export type {
  AuthConfig,
  EnabledAuthConfig,
  DisabledAuthConfig,
} from "#lib/auth-config.js";

export type FetchLike = typeof fetch;

/** Applies Flowl's managed-upload exclusion to the common return rules. */
export function safeLocalTarget(target: string | null | undefined): string {
  return commonSafeLocalTarget(target, ["/uploads"]);
}

/** Strictly reads the login query with Flowl's managed-upload exclusion. */
export function returnToFromQuery(rawQuery: string): string {
  return commonReturnToFromQuery(rawQuery, ["/uploads"]);
}

export function currentReturnTarget(
  location: Pick<Location, "pathname" | "search" | "hash">,
): string {
  return safeLocalTarget(
    `${location.pathname}${location.search}${location.hash}`,
  );
}

export function loginUrl(target: string): `/login?return_to=${string}` {
  return commonLoginUrl(target, ["/uploads"]) as `/login?return_to=${string}`;
}

export async function fetchAuthConfig(
  fetchFn: FetchLike = fetch,
): Promise<AuthConfig> {
  const response = await fetchFn("/auth/config");
  if (!response.ok) {
    throw new Error("Authentication configuration is unavailable");
  }

  const config = parseAuthConfig(await response.json());
  if (config === null) {
    throw new Error("Authentication configuration is invalid");
  }
  return config;
}

let authNavigationPending = false;

export function resetAuthNavigationForTests(): void {
  authNavigationPending = false;
}

export function navigateToLogin(
  location: Pick<Location, "pathname" | "search" | "hash"> = window.location,
  navigate: (url: ReturnType<typeof loginUrl>) => void = (url) => {
    // The query comes from safeLocalTarget; /auth routes are backend-only.
    void goto(resolve(url), { replace: true });
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

export async function postLogout(
  fetchFn: FetchLike = fetch,
  navigate: (url: "/login?logged_out=1") => void = (url) =>
    window.location.assign(url),
): Promise<void> {
  let response: Response;
  try {
    response = await fetchFn("/auth/logout", {
      method: "POST",
      credentials: "same-origin",
      cache: "no-store",
      redirect: "manual",
    });
  } catch {
    throw new Error("Sign out failed. Please try again.");
  }

  // Browsers hide manual redirects as opaqueredirect (status 0), including same-origin 303s.
  if (
    (response.type === "opaqueredirect" && response.status === 0) ||
    (response.status === 303 &&
      response.headers.get("Location") === "/login?logged_out=1")
  ) {
    navigate("/login?logged_out=1");
    return;
  }
  throw new Error("Sign out failed. Please try again.");
}
