/// <reference types="@sveltejs/kit" />
/// <reference no-default-lib="true"/>
/// <reference lib="esnext" />
/// <reference lib="webworker" />

import { build, files, version } from "$service-worker";

const sw = self as unknown as ServiceWorkerGlobalScope;

const CACHE_NAME = `flowl-cache-${version}`;
const OFFLINE_PAGE = "/offline.html";

const ASSETS = [...build, ...files];

sw.addEventListener("install", (event) => {
  event.waitUntil(
    caches.open(CACHE_NAME).then((cache) => cache.addAll(ASSETS)),
  );
});

sw.addEventListener("activate", (event) => {
  event.waitUntil(
    caches
      .keys()
      .then((keys) =>
        Promise.all(
          keys
            .filter((key) => key !== CACHE_NAME)
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

  // Everything else (API, uploads, etc.): pass through to network
});
