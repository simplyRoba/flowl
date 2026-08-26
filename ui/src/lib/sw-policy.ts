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

export function isAuthPath(pathname: string): boolean {
  return pathname === "/auth" || pathname.startsWith("/auth/");
}

export function shouldCacheProtectedResponse(response: Response): boolean {
  return response.status === 200 && !response.redirected;
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
): Promise<Response> {
  try {
    const response = await fetchFn(request);
    if (shouldCacheProtectedResponse(response)) {
      const cache = await cacheStorage.open(cacheName);
      await cache.put(request, response.clone());
    }
    return response;
  } catch (error) {
    return staleOnTransportFailure(request, cacheStorage, error);
  }
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
