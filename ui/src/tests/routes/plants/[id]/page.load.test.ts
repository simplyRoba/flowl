import { describe, expect, it, vi } from "vitest";
import { load } from "../../../../routes/plants/[id]/+page";

describe("plant detail page load", () => {
  it("loads plant details and care events from the route", async () => {
    const plant = { id: 1, name: "Fern" };
    const careEvents = [{ id: 10, event_type: "watered" }];
    const fetch = vi
      .fn()
      .mockResolvedValueOnce({
        ok: true,
        json: async () => plant,
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => careEvents,
      });

    const result = await load({
      fetch,
      params: { id: "1" },
    } as never);

    expect(fetch).toHaveBeenNthCalledWith(1, "/api/plants/1");
    expect(fetch).toHaveBeenNthCalledWith(2, "/api/plants/1/care");
    expect(result).toEqual({
      plant,
      careEvents,
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
      careEvents: [],
      notFound: true,
      loadErrorCode: null,
    });
  });

  it("returns the API error code for non-404 failures", async () => {
    const fetch = vi
      .fn()
      .mockResolvedValueOnce({
        ok: false,
        status: 500,
        statusText: "Internal Server Error",
        json: async () => ({
          code: "INTERNAL_ERROR",
          message: "An internal error occurred",
        }),
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => [],
      });

    const result = await load({
      fetch,
      params: { id: "1" },
    } as never);

    expect(result).toEqual({
      plant: null,
      careEvents: [],
      notFound: false,
      loadErrorCode: "INTERNAL_ERROR",
    });
  });
});
