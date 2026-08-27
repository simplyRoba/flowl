export const PROTECTED_CACHE_PREFIXES = [
  "flowl-api-",
  "flowl-photo-",
  "flowl-runtime-",
];

export interface CacheLike {
  match(request: RequestInfo): Promise<Response | undefined>;
  put(request: RequestInfo, response: Response): Promise<void>;
}

interface PublicAssetCache {
  addAll(requests: RequestInfo[]): Promise<void>;
}

interface PublicAssetCacheStorage {
  open(name: string): Promise<PublicAssetCache>;
}

export interface CacheStorageLike {
  match(request: RequestInfo): Promise<Response | undefined>;
  open(name: string): Promise<CacheLike>;
  keys(): Promise<string[]>;
  delete(name: string): Promise<boolean>;
}

export type WorkerFetch = (request: Request) => Promise<Response>;
export type AuthMode = "unknown" | "enabled" | "disabled";
export type AuthConfigFetch = () => Promise<Response>;

export interface AuthModeController {
  policyMode(fetchAuthConfig: AuthConfigFetch): Promise<AuthMode>;
  updateFromClient(enabled: boolean): void;
}

export function isAuthPath(pathname: string): boolean {
  return pathname === "/auth" || pathname.startsWith("/auth/");
}

export function shouldCacheProtectedResponse(response: Response): boolean {
  return response.status === 200 && !response.redirected;
}

/** Accepts only successful, non-redirected image media types for photo caches. */
export function shouldCacheProtectedThumbnailResponse(
  response: Response,
): boolean {
  const mediaType = response.headers
    .get("content-type")
    ?.split(";", 1)[0]
    .trim();
  return (
    shouldCacheProtectedResponse(response) &&
    mediaType !== undefined &&
    /^image\/[!#$%&'*+\-.^_`|~0-9a-z]+$/i.test(mediaType)
  );
}

/**
 * Keeps the worker fail-closed until a fresh auth configuration confirms that
 * authentication is disabled. A disabled worker rechecks before every policy
 * that could expose cached application data; an outage retains established
 * disabled-mode offline behavior.
 */
export function createAuthModeController(): AuthModeController {
  let mode: AuthMode = "unknown";
  let revision = 0;

  async function freshMode(
    fetchAuthConfig: AuthConfigFetch,
  ): Promise<AuthMode> {
    const response = await fetchAuthConfig();
    if (!response.ok || response.redirected) {
      throw new Error("Authentication configuration is unavailable");
    }

    const config: unknown = await response.json();
    if (
      typeof config !== "object" ||
      config === null ||
      typeof (config as { enabled?: unknown }).enabled !== "boolean"
    ) {
      throw new Error("Authentication configuration is invalid");
    }

    return (config as { enabled: boolean }).enabled ? "enabled" : "disabled";
  }

  return {
    async policyMode(fetchAuthConfig) {
      while (mode !== "enabled") {
        const checkedMode = mode;
        const checkedRevision = revision;
        let freshAuthMode: AuthMode;
        try {
          freshAuthMode = await freshMode(fetchAuthConfig);
        } catch {
          // Do not turn an unknown state into an authoritative enabled state:
          // fail closed for this request, then retry discovery next time.
          if (revision !== checkedRevision) continue;
          return checkedMode === "disabled" ? "disabled" : "enabled";
        }

        // Do not let an earlier config response override a client invalidation
        // or a stricter observation that arrived while it was in flight.
        if (revision !== checkedRevision) continue;
        mode = freshAuthMode;
        revision += 1;
        return mode;
      }
      return mode;
    },
    updateFromClient(enabled) {
      // A client may make the worker stricter, but cannot establish disabled
      // mode without a fresh backend response.
      mode = enabled ? "enabled" : "unknown";
      revision += 1;
    },
  };
}

export function isProtectedCacheName(name: string): boolean {
  return PROTECTED_CACHE_PREFIXES.some((prefix) => name.startsWith(prefix));
}

export function activationKeepCaches(
  shellCacheName: string,
  apiCacheName: string,
  photoCacheName: string,
): Set<string> {
  return new Set([
    shellCacheName,
    apiCacheName,
    photoCacheName,
    "flowl-sw-version",
  ]);
}

/** Runs auth endpoints without consulting or modifying application caches. */
export function networkOnly(
  request: Request,
  fetchFn: WorkerFetch,
): Promise<Response> {
  return fetchFn(request);
}

async function staleOnTransportFailure(
  request: Request,
  cacheStorage: Pick<CacheStorageLike, "match">,
  error: unknown,
): Promise<Response> {
  const cached = await cacheStorage.match(request);
  if (cached) return cached;
  throw error;
}

/** Auth-mode API and thumbnail policy: network first, stale only after rejection. */
export async function protectedNetworkFirst(
  request: Request,
  fetchFn: WorkerFetch,
  cacheStorage: Pick<CacheStorageLike, "match" | "open">,
  cacheName: string,
  shouldCache: (response: Response) => boolean = shouldCacheProtectedResponse,
): Promise<Response> {
  let response: Response;
  try {
    response = await fetchFn(request);
  } catch (error) {
    return staleOnTransportFailure(request, cacheStorage, error);
  }

  if (shouldCache(response)) {
    try {
      const cache = await cacheStorage.open(cacheName);
      await cache.put(request, response.clone());
    } catch {
      // A cache write must never replace a received network response with stale data.
    }
  }
  return response;
}

/** Preserves auth-disabled navigation cache-first behavior. */
export async function disabledNavigationCacheFirst(
  request: Request,
  fetchFn: WorkerFetch,
  cacheStorage: Pick<CacheStorageLike, "match">,
  fallback: () => Promise<Response>,
): Promise<Response> {
  const cached = await cacheStorage.match(request);
  if (cached) return cached;

  try {
    return await fetchFn(request);
  } catch {
    return fallback();
  }
}

/** Preserves auth-disabled API network-first behavior. */
export async function disabledApiNetworkFirst(
  request: Request,
  fetchFn: WorkerFetch,
  cacheStorage: CacheStorageLike,
  cacheName: string,
): Promise<Response> {
  try {
    const response = await fetchFn(request);
    const cache = await cacheStorage.open(cacheName);
    await cache.put(request, response.clone());
    return response;
  } catch (error) {
    return staleOnTransportFailure(request, cacheStorage, error);
  }
}

/** Preserves auth-disabled thumbnail cache-first behavior. */
export async function disabledThumbnailCacheFirst(
  request: Request,
  fetchFn: WorkerFetch,
  cacheStorage: Pick<CacheStorageLike, "open">,
  cacheName: string,
): Promise<Response> {
  const cache = await cacheStorage.open(cacheName);
  const cached = await cache.match(request);
  if (cached) return cached;
  const response = await fetchFn(request);
  await cache.put(request, response.clone());
  return response;
}

/** Auth-mode navigations always reach the backend before an offline fallback. */
export async function protectedNavigationNetworkFirst(
  request: Request,
  fetchFn: WorkerFetch,
  fallback: () => Promise<Response>,
): Promise<Response> {
  try {
    return await fetchFn(request);
  } catch {
    return fallback();
  }
}

/** Uses the cached application shell before the branded offline page. */
export async function cachedShellOrOffline(
  cacheStorage: Pick<CacheStorageLike, "match">,
  shellPath: string,
  offlinePath: string,
): Promise<Response> {
  return (
    (await cacheStorage.match(shellPath)) ??
    (await cacheStorage.match(offlinePath)) ??
    new Response("Offline", { status: 503, statusText: "Offline" })
  );
}

/** Installs only the immutable public assets required before authentication. */
export async function installPublicAssets(
  cacheStorage: PublicAssetCacheStorage,
  cacheName: string,
  assets: RequestInfo[],
): Promise<void> {
  const cache = await cacheStorage.open(cacheName);
  await cache.addAll(assets);
}

/** Removes outdated worker caches while retaining the current public resources. */
export async function removeObsoleteCaches(
  cacheStorage: Pick<CacheStorageLike, "keys" | "delete">,
  keepCaches: ReadonlySet<string>,
): Promise<string[]> {
  const obsolete = (await cacheStorage.keys()).filter(
    (name) => !keepCaches.has(name),
  );
  await Promise.all(obsolete.map((name) => cacheStorage.delete(name)));
  return obsolete;
}

export async function purgeProtectedCaches(
  cacheStorage: Pick<CacheStorageLike, "keys" | "delete">,
): Promise<string[]> {
  const names = (await cacheStorage.keys()).filter(isProtectedCacheName);
  await Promise.all(names.map((name) => cacheStorage.delete(name)));
  return names;
}
