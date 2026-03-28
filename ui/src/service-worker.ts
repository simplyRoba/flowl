/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, version } from "$service-worker";
import { isCacheableApi, isThumbnail } from "$lib/sw-patterns";

const sw = self as unknown as ServiceWorkerGlobalScope;

const CACHE_NAME = `flowl-cache-${version}`;
const API_CACHE_NAME = `flowl-api-${version}`;
const OFFLINE_PAGE = "/offline.html";

const ASSETS = [...build, ...files];

sw.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS)),
  );
});

sw.addEventListener("activate", (event) => {
  const keepCaches = new Set([CACHE_NAME, API_CACHE_NAME]);
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => !keepCaches.has(key))
            .map((key) => caches.delete(key)),
        ),
      ),
  );

  sw.skipWaiting();
});

sw.addEventListener("fetch", (event) => {
  const { request } = event;

  if (request.method !== "GET") {
    return;
  }

  const url = new URL(request.url);

  // Only handle same-origin requests
  if (url.origin !== sw.location.origin) {
    return;
  }

  // Navigation requests: try cache, then network, fall back to offline page
  if (request.mode === "navigate") {
    event.respondWith(
      caches
        .match(request)
        .then(
          (cached) =>
            cached ||
            fetch(request).catch(
              () => caches.match(OFFLINE_PAGE) as Promise<Response>,
            ),
        ),
    );
    return;
  }

  // Static assets: cache-first
  if (ASSETS.includes(url.pathname)) {
    event.respondWith(
      caches.match(request).then((cached) => cached || fetch(request)),
    );
    return;
  }

  // Cacheable API endpoints: network-first with stale fallback
  if (isCacheableApi(url.pathname)) {
    event.respondWith(
      fetch(request)
        .then((response) => {
          const clone = response.clone();
          caches
            .open(API_CACHE_NAME)
            .then((cache) => cache.put(request, clone));
          return response;
        })
        .catch((err) =>
          caches.match(request).then((cached) => {
            if (cached) return cached;
            throw err;
          }),
        ),
    );
    return;
  }

  // Thumbnails: cache-first
  if (isThumbnail(url.pathname)) {
    event.respondWith(
      caches.open(API_CACHE_NAME).then((cache) =>
        cache.match(request).then(
          (cached) =>
            cached ||
            fetch(request).then((response) => {
              cache.put(request, response.clone());
              return response;
            }),
        ),
      ),
    );
    return;
  }

  // Everything else: pass through to network
});
