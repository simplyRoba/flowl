import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("$app/navigation", () => ({ goto: vi.fn() }));

import {
  MAX_RETURN_TO_BYTES,
  currentReturnTarget,
  fetchAuthConfig,
  navigateToLogin,
  purgeProtectedCaches,
  resetAuthNavigationForTests,
  safeLocalTarget,
} from "./auth";

describe("safeLocalTarget", () => {
  it("preserves a SPA path, query, and fragment", () => {
    expect(safeLocalTarget("/plants/42?tab=care#entry-7")).toBe(
      "/plants/42?tab=care#entry-7",
    );
    expect(
      currentReturnTarget({
        pathname: "/plants/42",
        search: "?tab=care",
        hash: "#entry-7",
      }),
    ).toBe("/plants/42?tab=care#entry-7");
  });

  it.each([
    "https://attacker.example/",
    "//attacker.example/",
    "/\\attacker",
    "/plants/%",
    "/plants/%2fsecret",
    "/plants/%5csecret",
    "/plants/%2e%2e/login",
    "/plants/%252e%252e/login",
    "/log%69n",
    "/a%75th/callback",
    "/login",
    "/login/again",
    "/auth",
    "/auth/callback",
    "/plants/../login",
    "/plants/\u0000bad",
  ])("rejects unsafe target %j", (target) => {
    expect(safeLocalTarget(target)).toBe("/");
  });

  it("preserves safe query escapes while rejecting path ambiguity", () => {
    expect(safeLocalTarget("/plants?query=%2F%23%25")).toBe(
      "/plants?query=%2F%23%25",
    );
    expect(safeLocalTarget("/plants/%252fprivate")).toBe("/");
  });

  it("rejects an oversized target", () => {
    expect(safeLocalTarget(`/${"a".repeat(MAX_RETURN_TO_BYTES)}`)).toBe("/");
  });
});

describe("auth configuration", () => {
  it("accepts the disabled backend payload with a null provider", async () => {
    await expect(
      fetchAuthConfig(
        vi
          .fn()
          .mockResolvedValue(
            new Response(
              JSON.stringify({ enabled: false, provider_name: null }),
            ),
          ),
      ),
    ).resolves.toEqual({ enabled: false, provider_name: null });
  });

  it("accepts an enabled backend payload and rejects invalid combinations", async () => {
    await expect(
      fetchAuthConfig(
        vi
          .fn()
          .mockResolvedValue(
            new Response(
              JSON.stringify({ enabled: true, provider_name: "Example SSO" }),
            ),
          ),
      ),
    ).resolves.toEqual({ enabled: true, provider_name: "Example SSO" });

    await expect(
      fetchAuthConfig(
        vi
          .fn()
          .mockResolvedValue(
            new Response(
              JSON.stringify({ enabled: false, provider_name: "SSO" }),
            ),
          ),
      ),
    ).rejects.toThrow("invalid");
  });
});

describe("protected cache purge", () => {
  const originalCaches = globalThis.caches;
  const originalServiceWorker = navigator.serviceWorker;

  beforeEach(() => {
    vi.useFakeTimers();
  });

  it("falls back to and awaits direct deletion after worker timeout", async () => {
    const deleteCache = vi.fn().mockResolvedValue(true);
    Object.defineProperty(globalThis, "caches", {
      configurable: true,
      value: {
        keys: vi.fn().mockResolvedValue(["flowl-api-v1", "flowl-cache-v1"]),
        delete: deleteCache,
      },
    });
    Object.defineProperty(navigator, "serviceWorker", {
      configurable: true,
      value: { controller: { postMessage: vi.fn() } },
    });

    const purge = purgeProtectedCaches();
    await vi.advanceTimersByTimeAsync(2_000);
    await purge;

    expect(deleteCache).toHaveBeenCalledWith("flowl-api-v1");
    expect(deleteCache).not.toHaveBeenCalledWith("flowl-cache-v1");
  });

  it("waits for worker acknowledgement without deleting caches locally", async () => {
    const deleteCache = vi.fn();
    Object.defineProperty(globalThis, "caches", {
      configurable: true,
      value: {
        keys: vi.fn().mockResolvedValue(["flowl-api-v1"]),
        delete: deleteCache,
      },
    });
    Object.defineProperty(navigator, "serviceWorker", {
      configurable: true,
      value: {
        controller: {
          postMessage: vi.fn((_message: unknown, ports: MessagePort[]) => {
            ports[0]?.postMessage({ type: "PROTECTED_CACHES_PURGED" });
          }),
        },
      },
    });

    await purgeProtectedCaches();

    expect(deleteCache).not.toHaveBeenCalled();
  });

  it("falls back to direct deletion when postMessage throws", async () => {
    const deleteCache = vi.fn().mockResolvedValue(true);
    Object.defineProperty(globalThis, "caches", {
      configurable: true,
      value: {
        keys: vi.fn().mockResolvedValue(["flowl-runtime-v1"]),
        delete: deleteCache,
      },
    });
    Object.defineProperty(navigator, "serviceWorker", {
      configurable: true,
      value: {
        controller: {
          postMessage: vi.fn(() => {
            throw new Error("gone");
          }),
        },
      },
    });

    await purgeProtectedCaches();

    expect(deleteCache).toHaveBeenCalledWith("flowl-runtime-v1");
  });

  it("directly purges all protected cache types without a controller", async () => {
    const deleteCache = vi.fn().mockResolvedValue(true);
    Object.defineProperty(globalThis, "caches", {
      configurable: true,
      value: {
        keys: vi
          .fn()
          .mockResolvedValue([
            "flowl-api-v1",
            "flowl-photo-v1",
            "flowl-runtime-v1",
            "flowl-cache-v1",
            "flowl-sw-version",
          ]),
        delete: deleteCache,
      },
    });
    Object.defineProperty(navigator, "serviceWorker", {
      configurable: true,
      value: { controller: null },
    });

    await purgeProtectedCaches();

    expect(deleteCache).toHaveBeenCalledTimes(3);
    expect(deleteCache).toHaveBeenCalledWith("flowl-api-v1");
    expect(deleteCache).toHaveBeenCalledWith("flowl-photo-v1");
    expect(deleteCache).toHaveBeenCalledWith("flowl-runtime-v1");
    expect(deleteCache).not.toHaveBeenCalledWith("flowl-cache-v1");
    expect(deleteCache).not.toHaveBeenCalledWith("flowl-sw-version");
  });

  afterEach(() => {
    vi.useRealTimers();
    Object.defineProperty(globalThis, "caches", {
      configurable: true,
      value: originalCaches,
    });
    Object.defineProperty(navigator, "serviceWorker", {
      configurable: true,
      value: originalServiceWorker,
    });
  });
});

describe("navigateToLogin", () => {
  beforeEach(resetAuthNavigationForTests);

  it("uses a validated current local URL once", () => {
    const navigate = vi.fn();
    expect(
      navigateToLogin(
        { pathname: "/plants/2", search: "?view=care", hash: "#entry" },
        navigate,
      ),
    ).toBe(true);
    expect(navigate).toHaveBeenCalledWith(
      "/login?return_to=%2Fplants%2F2%3Fview%3Dcare%23entry",
    );
    expect(
      navigateToLogin(
        { pathname: "/plants/3", search: "", hash: "" },
        navigate,
      ),
    ).toBe(false);
  });

  it("does not redirect recursively from public auth routes", () => {
    expect(
      navigateToLogin({ pathname: "/login", search: "", hash: "" }, vi.fn()),
    ).toBe(false);
    expect(
      navigateToLogin(
        { pathname: "/auth/callback", search: "", hash: "" },
        vi.fn(),
      ),
    ).toBe(false);
  });
});
