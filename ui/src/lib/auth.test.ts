import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { goto } from "$app/navigation";
import returnVectors from "../../../tests/fixtures/auth/return-targets.json";

vi.mock("$app/navigation", () => ({ goto: vi.fn() }));

import {
  MAX_RETURN_TO_BYTES,
  currentReturnTarget,
  fetchAuthConfig,
  loginUrl,
  navigateToLogin,
  postLogout,
  purgeProtectedCaches,
  resetAuthNavigationForTests,
  safeLocalTarget,
} from "./auth";

describe("safeLocalTarget", () => {
  it.each(returnVectors.vectors)("common return fixture: $id", (vector) => {
    const input = vector.input;
    const value =
      input === null
        ? null
        : typeof input === "string"
          ? input
          : input.prefix + input.repeat.repeat(input.count) + input.suffix;
    const expected = vector.target.flowl ? (value ?? "/") : "/";
    expect(safeLocalTarget(value)).toBe(expected);
    // Match the peer harness's query round trip; accepted serialization is unchanged.
    const wire =
      value === null
        ? ""
        : new URLSearchParams({ return_to: value }).toString();
    expect(safeLocalTarget(new URLSearchParams(wire).get("return_to"))).toBe(
      expected,
    );
  });

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

  it("constructs login URLs without decoding or normalizing accepted targets", () => {
    const target = "/plants/42?label=a%20b&ratio=100%25#entry%20one";
    const url = new URL(loginUrl(target), "https://flowl.invalid");
    expect(url.pathname).toBe("/login");
    expect(url.searchParams.get("return_to")).toBe(target);
    expect(new URL(loginUrl("/api"), url).searchParams.get("return_to")).toBe(
      "/",
    );
  });

  it("rejects an oversized target", () => {
    expect(safeLocalTarget(`/${"a".repeat(MAX_RETURN_TO_BYTES)}`)).toBe("/");
  });
});

describe("auth configuration", () => {
  it("accepts the disabled backend payload without a provider", async () => {
    await expect(
      fetchAuthConfig(
        vi
          .fn()
          .mockResolvedValue(new Response(JSON.stringify({ enabled: false }))),
      ),
    ).resolves.toEqual({ enabled: false });
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

describe("postLogout", () => {
  beforeEach(() => vi.mocked(goto).mockClear());

  function opaqueResponse(type: "opaqueredirect" | "opaque"): Response {
    return Object.defineProperty(Response.error(), "type", { value: type });
  }

  it.each([
    [
      "visible 303",
      () =>
        new Response(null, {
          status: 303,
          headers: { Location: "/login?logged_out=1" },
        }),
    ],
    ["browser manual redirect", () => opaqueResponse("opaqueredirect")],
  ] as const)(
    "POSTs logout and hard-navigates only after %s",
    async (_kind, response) => {
      let finish!: (response: Response) => void;
      const fetchFn = vi.fn(
        () =>
          new Promise<Response>((resolve) => {
            finish = resolve;
          }),
      );
      const navigate = vi.fn();
      const logout = postLogout(fetchFn, navigate);

      expect(fetchFn).toHaveBeenCalledExactlyOnceWith("/auth/logout", {
        method: "POST",
        credentials: "same-origin",
        cache: "no-store",
        redirect: "manual",
      });
      expect(navigate).not.toHaveBeenCalled();
      finish(response());
      await logout;
      expect(navigate).toHaveBeenCalledExactlyOnceWith("/login?logged_out=1");
    },
  );

  it.each([
    [
      "HTTP 500",
      () => new Response("provider-secret-sentinel", { status: 500 }),
    ],
    [
      "HTTP 401",
      () =>
        new Response(JSON.stringify({ code: "AUTHENTICATION_REQUIRED" }), {
          status: 401,
        }),
    ],
    ["HTTP 200", () => new Response("provider-secret-sentinel")],
    ["opaque non-redirect", () => opaqueResponse("opaque")],
    [
      "external redirect",
      () =>
        new Response(null, {
          status: 303,
          headers: {
            Location: "https://provider.invalid/provider-secret-sentinel",
          },
        }),
    ],
    [
      "unexpected local redirect",
      () =>
        new Response(null, {
          status: 303,
          headers: { Location: "/auth/login" },
        }),
    ],
    ["missing redirect target", () => new Response(null, { status: 303 })],
    [
      "wrong redirect status",
      () =>
        new Response(null, {
          status: 302,
          headers: { Location: "/login?logged_out=1" },
        }),
    ],
  ] as const)(
    "fails closed with a fixed error for %s",
    async (_kind, response) => {
      const received = response();
      const readBody = vi.spyOn(received, "text");
      const navigate = vi.fn();
      await expect(
        postLogout(vi.fn().mockResolvedValue(received), navigate),
      ).rejects.toEqual(new Error("Sign out failed. Please try again."));
      expect(readBody).not.toHaveBeenCalled();
      expect(received.bodyUsed).toBe(false);
      expect(navigate).not.toHaveBeenCalled();
      expect(goto).not.toHaveBeenCalled();
    },
  );

  it("does not reflect network errors or navigate after transport failure", async () => {
    const navigate = vi.fn();
    await expect(
      postLogout(
        vi.fn().mockRejectedValue(new Error("provider-secret-sentinel")),
        navigate,
      ),
    ).rejects.toEqual(new Error("Sign out failed. Please try again."));
    expect(navigate).not.toHaveBeenCalled();
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
  beforeEach(() => {
    resetAuthNavigationForTests();
    vi.mocked(goto).mockClear();
  });

  it("resolves the default login navigation with the complete encoded return target", () => {
    expect(
      navigateToLogin({
        pathname: "/plants/42",
        search: "?tab=care&filter=%2F%23%25&name=Monstera%20deliciosa",
        hash: "#entry-7",
      }),
    ).toBe(true);
    expect(goto).toHaveBeenCalledExactlyOnceWith(
      `/login?return_to=${encodeURIComponent("/plants/42?tab=care&filter=%2F%23%25&name=Monstera%20deliciosa#entry-7")}`,
      { replace: true },
    );
  });

  it("uses the safe fallback with the default navigator for an unsafe current path", () => {
    expect(
      navigateToLogin({
        pathname: "/plants/%2fsecret",
        search: "",
        hash: "",
      }),
    ).toBe(true);
    expect(goto).toHaveBeenCalledExactlyOnceWith("/login?return_to=%2F", {
      replace: true,
    });
  });

  it("does not invoke the default navigator from a public auth route", () => {
    expect(navigateToLogin({ pathname: "/login", search: "", hash: "" })).toBe(
      false,
    );
    expect(goto).not.toHaveBeenCalled();
  });

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
