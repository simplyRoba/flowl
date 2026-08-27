import { describe, expect, it, vi } from "vitest";

import {
  activationKeepCaches,
  cachedShellOrOffline,
  createAuthModeController,
  disabledApiNetworkFirst,
  disabledNavigationCacheFirst,
  disabledThumbnailCacheFirst,
  isAuthPath,
  installPublicAssets,
  isProtectedCacheName,
  networkOnly,
  protectedNavigationNetworkFirst,
  protectedNetworkFirst,
  shouldCacheProtectedThumbnailResponse,
  purgeProtectedCaches,
  removeObsoleteCaches,
} from "./sw-policy";

function request(path = "/api/plants"): Request {
  return new Request(`http://localhost${path}`);
}

function response(
  status: number,
  body = String(status),
  headers?: HeadersInit,
): Response {
  return new Response(body, { status, headers });
}

function cacheStorage(cached?: Response) {
  const cache = {
    addAll: vi.fn().mockResolvedValue(undefined),
    match: vi.fn().mockResolvedValue(cached),
    put: vi.fn().mockResolvedValue(undefined),
  };
  return {
    match: vi.fn().mockResolvedValue(cached),
    open: vi.fn().mockResolvedValue(cache),
    keys: vi.fn().mockResolvedValue([]),
    delete: vi.fn().mockResolvedValue(true),
    cache,
  };
}

describe("authentication service-worker policy", () => {
  it.each(["/auth/config", "/auth/login", "/auth/callback", "/auth/logout"])(
    "keeps %s network-only without touching application caches",
    async (path) => {
      const fetchFn = vi.fn().mockResolvedValue(response(200));
      const result = await networkOnly(request(path), fetchFn);

      expect(isAuthPath(path)).toBe(true);
      expect(result.status).toBe(200);
      expect(fetchFn).toHaveBeenCalledOnce();
    },
  );

  it("does not treat normal application paths as auth endpoints", () => {
    expect(isAuthPath("/login")).toBe(false);
    expect(isAuthPath("/api/plants")).toBe(false);
  });

  it.each(["/auth/login", "/auth/callback", "/auth/logout"])(
    "returns received login result or redirect from %s without caching it",
    async (path) => {
      const received = response(302, "redirect");
      const result = await networkOnly(
        request(path),
        vi.fn().mockResolvedValue(received),
      );

      expect(result).toBe(received);
      expect(result.status).toBe(302);
    },
  );

  it("caches only a 200 API response in auth mode", async () => {
    const storage = cacheStorage();
    const result = await protectedNetworkFirst(
      request(),
      vi.fn().mockResolvedValue(response(200, "fresh")),
      storage,
      "flowl-api-v1",
    );

    expect(await result.text()).toBe("fresh");
    expect(storage.cache.put).toHaveBeenCalledOnce();
  });

  it.each([401, 500])(
    "returns a received API %i directly despite stale data and does not overwrite it",
    async (status) => {
      const stale = response(200, "stale");
      const storage = cacheStorage(stale);
      const result = await protectedNetworkFirst(
        request(),
        vi.fn().mockResolvedValue(response(status, "network")),
        storage,
        "flowl-api-v1",
      );

      expect(result.status).toBe(status);
      expect(await result.text()).toBe("network");
      expect(storage.match).not.toHaveBeenCalled();
      expect(storage.cache.put).not.toHaveBeenCalled();
    },
  );

  it.each(["open", "put"] as const)(
    "returns a fresh API response without consulting stale data when cache.%s fails",
    async (operation) => {
      const stale = response(200, "stale");
      const storage = cacheStorage(stale);
      if (operation === "open") {
        storage.open.mockRejectedValue(new Error("cache unavailable"));
      } else {
        storage.cache.put.mockRejectedValue(new Error("cache unavailable"));
      }

      const result = await protectedNetworkFirst(
        request(),
        vi.fn().mockResolvedValue(response(200, "fresh")),
        storage,
        "flowl-api-v1",
      );

      expect(await result.text()).toBe("fresh");
      expect(storage.match).not.toHaveBeenCalled();
    },
  );

  it("uses a stale API response only after a rejected request", async () => {
    const stale = response(200, "stale");
    const storage = cacheStorage(stale);
    const result = await protectedNetworkFirst(
      request(),
      vi.fn().mockRejectedValue(new TypeError("offline")),
      storage,
      "flowl-api-v1",
    );

    expect(await result.text()).toBe("stale");
    expect(storage.cache.put).not.toHaveBeenCalled();
  });

  it("does not cache a fresh HTML thumbnail or consult stale data", async () => {
    const storage = cacheStorage(response(200, "stale image"));
    const result = await protectedNetworkFirst(
      request("/uploads/fern_200.jpg"),
      vi
        .fn()
        .mockResolvedValue(
          response(200, "login document", { "Content-Type": "text/html" }),
        ),
      storage,
      "flowl-photo-v1",
      shouldCacheProtectedThumbnailResponse,
    );

    expect(await result.text()).toBe("login document");
    expect(storage.match).not.toHaveBeenCalled();
    expect(storage.cache.put).not.toHaveBeenCalled();
  });

  it("caches a fresh JPEG thumbnail in auth mode", async () => {
    const storage = cacheStorage();
    const result = await protectedNetworkFirst(
      request("/uploads/fern_200.jpg"),
      vi.fn().mockResolvedValue(
        response(200, "image", {
          "Content-Type": "IMAGE/JPEG; charset=binary",
        }),
      ),
      storage,
      "flowl-photo-v1",
      shouldCacheProtectedThumbnailResponse,
    );

    expect(await result.text()).toBe("image");
    expect(storage.cache.put).toHaveBeenCalledOnce();
  });

  it.each([
    ["authentication failure", response(401, "expired")],
    [
      "redirect",
      Object.defineProperty(response(200, "redirect"), "redirected", {
        value: true,
      }),
    ],
  ])(
    "returns a received thumbnail %s directly instead of stale data",
    async (_kind, received) => {
      const storage = cacheStorage(response(200, "stale image"));
      const result = await protectedNetworkFirst(
        request("/uploads/fern_200.jpg"),
        vi.fn().mockResolvedValue(received),
        storage,
        "flowl-photo-v1",
      );

      expect(result).toBe(received);
      expect(storage.match).not.toHaveBeenCalled();
      expect(storage.cache.put).not.toHaveBeenCalled();
    },
  );

  it("uses a stale thumbnail only after a transport rejection", async () => {
    const storage = cacheStorage(response(200, "stale image"));
    const result = await protectedNetworkFirst(
      request("/uploads/fern_200.jpg"),
      vi.fn().mockRejectedValue(new TypeError("offline")),
      storage,
      "flowl-photo-v1",
    );

    expect(await result.text()).toBe("stale image");
    expect(storage.cache.put).not.toHaveBeenCalled();
  });

  it("establishes disabled mode from fresh config after a worker restart", async () => {
    const firstWorker = createAuthModeController();
    await expect(
      firstWorker.policyMode(
        vi.fn().mockResolvedValue(
          response(200, JSON.stringify({ enabled: false }), {
            "Content-Type": "application/json",
          }),
        ),
      ),
    ).resolves.toBe("disabled");

    const restartedWorker = createAuthModeController();
    const fetchConfig = vi.fn().mockResolvedValue(
      response(200, JSON.stringify({ enabled: false }), {
        "Content-Type": "application/json",
      }),
    );
    await expect(restartedWorker.policyMode(fetchConfig)).resolves.toBe(
      "disabled",
    );
    expect(fetchConfig).toHaveBeenCalledOnce();
  });

  it("moves an established disabled worker to protected mode after enabled config", async () => {
    const worker = createAuthModeController();
    const fetchConfig = vi
      .fn()
      .mockResolvedValueOnce(
        response(200, JSON.stringify({ enabled: false }), {
          "Content-Type": "application/json",
        }),
      )
      .mockResolvedValueOnce(
        response(200, JSON.stringify({ enabled: true }), {
          "Content-Type": "application/json",
        }),
      );

    await expect(worker.policyMode(fetchConfig)).resolves.toBe("disabled");
    await expect(worker.policyMode(fetchConfig)).resolves.toBe("enabled");
    await expect(
      worker.policyMode(vi.fn().mockRejectedValue(new TypeError("offline"))),
    ).resolves.toBe("enabled");
    expect(fetchConfig).toHaveBeenCalledTimes(2);
  });

  it("keeps established disabled mode available when config revalidation is offline", async () => {
    const worker = createAuthModeController();
    await expect(
      worker.policyMode(
        vi.fn().mockResolvedValue(
          response(200, JSON.stringify({ enabled: false }), {
            "Content-Type": "application/json",
          }),
        ),
      ),
    ).resolves.toBe("disabled");

    await expect(
      worker.policyMode(vi.fn().mockRejectedValue(new TypeError("offline"))),
    ).resolves.toBe("disabled");
  });

  it("fails closed and retries until disabled mode is freshly confirmed", async () => {
    const worker = createAuthModeController();
    worker.updateFromClient(false);
    const fetchConfig = vi
      .fn()
      .mockRejectedValueOnce(new TypeError("offline"))
      .mockResolvedValueOnce(
        response(200, JSON.stringify({ enabled: false }), {
          "Content-Type": "application/json",
        }),
      );

    await expect(worker.policyMode(fetchConfig)).resolves.toBe("enabled");
    await expect(worker.policyMode(fetchConfig)).resolves.toBe("disabled");
    expect(fetchConfig).toHaveBeenCalledTimes(2);
  });

  it("does not accept a redirected config response as authoritative", async () => {
    const worker = createAuthModeController();
    const redirected = Object.defineProperty(
      response(200, JSON.stringify({ enabled: false }), {
        "Content-Type": "application/json",
      }),
      "redirected",
      { value: true },
    );

    await expect(
      worker.policyMode(vi.fn().mockResolvedValue(redirected)),
    ).resolves.toBe("enabled");
  });

  it("does not let a stale disabled config response override enabled mode", async () => {
    const worker = createAuthModeController();
    let resolveConfig: (value: Response) => void;
    const pendingConfig = new Promise<Response>((resolve) => {
      resolveConfig = resolve;
    });

    const policyMode = worker.policyMode(() => pendingConfig);
    worker.updateFromClient(true);
    resolveConfig!(
      response(200, JSON.stringify({ enabled: false }), {
        "Content-Type": "application/json",
      }),
    );

    await expect(policyMode).resolves.toBe("enabled");
  });

  it("always fetches protected navigation before a cached index and returns redirects", async () => {
    const fetchFn = vi.fn().mockResolvedValue(response(302, "login redirect"));
    const fallback = vi.fn().mockResolvedValue(response(200, "cached index"));
    const result = await protectedNavigationNetworkFirst(
      request("/plants/7"),
      fetchFn,
      fallback,
    );

    expect(result.status).toBe(302);
    expect(fetchFn).toHaveBeenCalledOnce();
    expect(fallback).not.toHaveBeenCalled();
  });

  it("uses the cached shell, then the offline page, only after navigation rejection", async () => {
    const fetchFn = vi.fn().mockRejectedValue(new TypeError("offline"));
    const shellStorage = cacheStorage();
    shellStorage.match
      .mockResolvedValueOnce(response(200, "cached shell"))
      .mockResolvedValueOnce(response(200, "offline"));
    const shellResult = await protectedNavigationNetworkFirst(
      request("/plants/7"),
      fetchFn,
      () => cachedShellOrOffline(shellStorage, "/index.html", "/offline.html"),
    );
    expect(await shellResult.text()).toBe("cached shell");

    const offlineStorage = cacheStorage();
    offlineStorage.match
      .mockResolvedValueOnce(undefined)
      .mockResolvedValueOnce(response(200, "offline"));
    const offlineResult = await cachedShellOrOffline(
      offlineStorage,
      "/index.html",
      "/offline.html",
    );
    expect(await offlineResult.text()).toBe("offline");
  });

  it("uses an existing disabled-mode navigation cache before the network", async () => {
    const storage = cacheStorage(response(200, "cached dashboard"));
    const fetchFn = vi.fn();
    const fallback = vi.fn();

    const result = await disabledNavigationCacheFirst(
      request("/plants/7"),
      fetchFn,
      storage,
      fallback,
    );

    expect(await result.text()).toBe("cached dashboard");
    expect(fetchFn).not.toHaveBeenCalled();
    expect(fallback).not.toHaveBeenCalled();
  });

  it("uses the public offline fallback for an uncached disabled-mode navigation outage", async () => {
    const storage = cacheStorage();
    const fallback = vi.fn().mockResolvedValue(response(200, "offline page"));

    const result = await disabledNavigationCacheFirst(
      request("/plants/7"),
      vi.fn().mockRejectedValue(new TypeError("offline")),
      storage,
      fallback,
    );

    expect(await result.text()).toBe("offline page");
    expect(fallback).toHaveBeenCalledOnce();
  });

  it("retains disabled-mode API network-first and thumbnail cache-first behavior", async () => {
    const apiStorage = cacheStorage(response(200, "stale API"));
    const apiResult = await disabledApiNetworkFirst(
      request(),
      vi.fn().mockResolvedValue(response(500, "network API")),
      apiStorage,
      "flowl-api-v1",
    );
    expect(await apiResult.text()).toBe("network API");
    expect(apiStorage.cache.put).toHaveBeenCalledOnce();

    const thumbnailStorage = cacheStorage(response(200, "cached thumbnail"));
    const fetchThumbnail = vi.fn();
    const thumbnailResult = await disabledThumbnailCacheFirst(
      request("/uploads/fern_200.jpg"),
      fetchThumbnail,
      thumbnailStorage,
      "flowl-photo-v1",
    );
    expect(await thumbnailResult.text()).toBe("cached thumbnail");
    expect(fetchThumbnail).not.toHaveBeenCalled();
  });

  it("does not cache or purge protected data on expiry or an ordinary outage", async () => {
    const storage = cacheStorage(response(200, "stale"));
    await protectedNetworkFirst(
      request(),
      vi.fn().mockResolvedValue(response(401, "expired")),
      storage,
      "flowl-api-v1",
    );
    await protectedNetworkFirst(
      request(),
      vi.fn().mockRejectedValue(new TypeError("offline")),
      storage,
      "flowl-api-v1",
    );

    expect(storage.cache.put).not.toHaveBeenCalled();
    expect(storage.delete).not.toHaveBeenCalled();
  });

  it("installs public resources and retains every current cache on activation", async () => {
    const storage = cacheStorage();
    const publicAssets = ["/offline.html", "/_app/immutable/login.js"];

    await installPublicAssets(storage, "flowl-cache-v2", publicAssets);
    expect(storage.cache.addAll).toHaveBeenCalledWith(publicAssets);

    storage.keys.mockResolvedValue([
      "flowl-cache-v1",
      "flowl-api-v1",
      "flowl-photo-v1",
      "flowl-cache-v2",
      "flowl-api-v2",
      "flowl-photo-v2",
      "flowl-sw-version",
    ]);
    await expect(
      removeObsoleteCaches(
        storage,
        activationKeepCaches(
          "flowl-cache-v2",
          "flowl-api-v2",
          "flowl-photo-v2",
        ),
      ),
    ).resolves.toEqual(["flowl-cache-v1", "flowl-api-v1", "flowl-photo-v1"]);
    expect(storage.delete).toHaveBeenCalledTimes(3);
    expect(storage.delete).toHaveBeenCalledWith("flowl-cache-v1");
    expect(storage.delete).toHaveBeenCalledWith("flowl-api-v1");
    expect(storage.delete).toHaveBeenCalledWith("flowl-photo-v1");
  });

  it("purges only current and obsolete protected API, photo, and runtime caches", async () => {
    const storage = cacheStorage();
    storage.keys.mockResolvedValue([
      "flowl-api-v1",
      "flowl-api-v2",
      "flowl-photo-v1",
      "flowl-runtime-v1",
      "flowl-cache-v2",
      "flowl-sw-version",
      "workbox-precache-v1",
    ]);

    await expect(purgeProtectedCaches(storage)).resolves.toEqual([
      "flowl-api-v1",
      "flowl-api-v2",
      "flowl-photo-v1",
      "flowl-runtime-v1",
    ]);
    expect(storage.delete).toHaveBeenCalledTimes(4);
    expect(isProtectedCacheName("flowl-cache-v2")).toBe(false);
    expect(isProtectedCacheName("flowl-sw-version")).toBe(false);
  });
});
