/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, version } from "$service-worker";
import { isCacheableApi, isThumbnail } from "$lib/sw-patterns";
import {
  activationKeepCaches,
  cachedShellOrOffline,
  createAuthModeController,
  disabledApiNetworkFirst,
  disabledNavigationCacheFirst,
  disabledThumbnailCacheFirst,
  isAuthPath,
  installPublicAssets,
  networkOnly,
  protectedNavigationNetworkFirst,
  protectedNetworkFirst,
  shouldCacheProtectedThumbnailResponse,
  purgeProtectedCaches,
  removeObsoleteCaches,
} from "$lib/sw-policy";

const sw = self as unknown as ServiceWorkerGlobalScope;
const CACHE_NAME = `flowl-cache-${version}`;
const API_CACHE_NAME = `flowl-api-${version}`;
const PHOTO_CACHE_NAME = `flowl-photo-${version}`;
const OFFLINE_PAGE = "/offline.html";
const ASSETS = [...build, ...files];

// A worker starts unknown and therefore protected. Disabled mode is established
// only by a fresh network-only backend config response, never by client state.
const authMode = createAuthModeController();

function cachedNavigationFallback(): Promise<Response> {
  return cachedShellOrOffline(caches, "/index.html", OFFLINE_PAGE);
}

function fetchFreshAuthConfig(): Promise<Response> {
  return fetch(
    new Request(new URL("/auth/config", sw.location.origin), {
      cache: "no-store",
      redirect: "error",
    }),
  );
}

function withAuthMode(
  disabledPolicy: () => Promise<Response>,
  protectedPolicy: () => Promise<Response>,
): Promise<Response> {
  return authMode
    .policyMode(fetchFreshAuthConfig)
    .then((mode) =>
      mode === "disabled" ? disabledPolicy() : protectedPolicy(),
    );
}

sw.addEventListener("install", (event) => {
  event.waitUntil(installPublicAssets(caches, CACHE_NAME, ASSETS));
  sw.skipWaiting();
});

sw.addEventListener("activate", (event) => {
  const keepCaches = activationKeepCaches(
    CACHE_NAME,
    API_CACHE_NAME,
    PHOTO_CACHE_NAME,
  );
  event.waitUntil(
    removeObsoleteCaches(caches, keepCaches).then(() => sw.clients.claim()),
  );
});

sw.addEventListener("fetch", (event) => {
  const { request } = event;
  if (request.method !== "GET") return;
  const url = new URL(request.url);
  if (url.origin !== sw.location.origin) return;

  // OIDC config, redirects, callback, and logout responses are always network-only.
  if (isAuthPath(url.pathname)) {
    event.respondWith(networkOnly(request, fetch));
    return;
  }

  if (request.mode === "navigate") {
    event.respondWith(
      withAuthMode(
        () =>
          disabledNavigationCacheFirst(
            request,
            fetch,
            caches,
            cachedNavigationFallback,
          ),
        () =>
          protectedNavigationNetworkFirst(
            request,
            fetch,
            cachedNavigationFallback,
          ),
      ),
    );
    return;
  }

  if (ASSETS.includes(url.pathname)) {
    event.respondWith(
      caches.match(request).then((cached) => cached ?? fetch(request)),
    );
    return;
  }

  if (isCacheableApi(url.pathname)) {
    event.respondWith(
      withAuthMode(
        () => disabledApiNetworkFirst(request, fetch, caches, API_CACHE_NAME),
        () => protectedNetworkFirst(request, fetch, caches, API_CACHE_NAME),
      ),
    );
    return;
  }

  if (isThumbnail(url.pathname)) {
    event.respondWith(
      withAuthMode(
        () =>
          disabledThumbnailCacheFirst(request, fetch, caches, PHOTO_CACHE_NAME),
        () =>
          protectedNetworkFirst(
            request,
            fetch,
            caches,
            PHOTO_CACHE_NAME,
            shouldCacheProtectedThumbnailResponse,
          ),
      ),
    );
  }
});

sw.addEventListener("message", (event) => {
  if (event.data?.type === "GET_VERSION") {
    event.ports[0]?.postMessage({ type: "VERSION", version });
    return;
  }
  if (event.data?.type === "SET_AUTH_ENABLED") {
    authMode.updateFromClient(event.data.enabled === true);
    return;
  }
  if (event.data?.type === "PURGE_PROTECTED_CACHES") {
    event.waitUntil(
      purgeProtectedCaches(caches).then(() => {
        event.ports[0]?.postMessage({ type: "PROTECTED_CACHES_PURGED" });
      }),
    );
  }
});
