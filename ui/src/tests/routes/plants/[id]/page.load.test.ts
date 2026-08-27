import { beforeEach, describe, expect, it, vi } from "vitest";

const { goto } = vi.hoisted(() => ({ goto: vi.fn() }));
vi.mock("$app/navigation", () => ({ goto }));

import { resetAuthNavigationForTests } from "$lib/auth";
import { load } from "../../../../routes/plants/[id]/+page";

beforeEach(() => {
  goto.mockReset();
  resetAuthNavigationForTests();
  window.history.replaceState({}, "", "/plants/1");
});

describe("plant detail page load", () => {
  it("loads plant details from the route", async () => {
    const plant = { id: 1, name: "Fern" };
    const fetch = vi.fn().mockResolvedValueOnce({
      ok: true,
      json: async () => plant,
    });

    const result = await load({
      fetch,
      params: { id: "1" },
    } as never);

    expect(fetch).toHaveBeenCalledWith("/api/plants/1");
    expect(fetch).toHaveBeenCalledTimes(1);
    expect(result).toEqual({
      plant,
      notFound: false,
      loadErrorCode: null,
    });
  });

  it("marks the page as not found when the plant is missing", async () => {
    const fetch = vi.fn().mockResolvedValue({
      ok: false,
      status: 404,
      statusText: "Not Found",
      json: async () => ({ message: "Plant not found" }),
    });

    const result = await load({
      fetch,
      params: { id: "999" },
    } as never);

    expect(result).toEqual({
      plant: null,
      notFound: true,
      loadErrorCode: null,
    });
  });

  it("redirects exact authentication expiry instead of treating it as a missing plant", async () => {
    const result = await load({
      fetch: vi.fn().mockResolvedValue(
        new Response(
          JSON.stringify({
            code: "AUTHENTICATION_REQUIRED",
            message: "Authentication is required",
          }),
          { status: 401 },
        ),
      ),
      params: { id: "1" },
    } as never);

    expect(result).toEqual({
      plant: null,
      notFound: false,
      loadErrorCode: "AUTHENTICATION_REQUIRED",
    });
    expect(goto).toHaveBeenCalledWith("/login?return_to=%2Fplants%2F1", {
      replaceState: true,
    });
  });

  it("keeps route-loader transport failures distinct from authentication expiry", async () => {
    const result = await load({
      fetch: vi.fn().mockRejectedValue(new TypeError("offline")),
      params: { id: "1" },
    } as never);

    expect(result).toEqual({
      plant: null,
      notFound: false,
      loadErrorCode: "UNKNOWN_ERROR",
    });
    expect(goto).not.toHaveBeenCalled();
  });

  it("returns the API error code for non-404 failures", async () => {
    const fetch = vi.fn().mockResolvedValueOnce({
      ok: false,
      status: 500,
      statusText: "Internal Server Error",
      json: async () => ({
        code: "INTERNAL_ERROR",
        message: "An internal error occurred",
      }),
    });

    const result = await load({
      fetch,
      params: { id: "1" },
    } as never);

    expect(result).toEqual({
      plant: null,
      notFound: false,
      loadErrorCode: "INTERNAL_ERROR",
    });
  });
});
