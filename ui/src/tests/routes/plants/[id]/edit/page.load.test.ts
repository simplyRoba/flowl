import { describe, expect, it, vi } from "vitest";
import { load } from "../../../../../routes/plants/[id]/edit/+page";

describe("plant edit page load", () => {
  it("loads the plant from the route", async () => {
    const plant = { id: 1, name: "Fern" };
    const fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => plant,
    });

    const result = await load({
      fetch,
      params: { id: "1" },
    } as never);

    expect(fetch).toHaveBeenCalledWith("/api/plants/1");
    expect(result).toEqual({
      plant,
      notFound: false,
      loadError: null,
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
      loadError: null,
    });
  });
});
