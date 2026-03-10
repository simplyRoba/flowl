import { describe, expect, it } from "vitest";
import { ApiError } from "$lib/api";
import { resolveError } from "./errors";

describe("resolveError", () => {
  it("returns translated message for known ApiError code", () => {
    const error = new ApiError(404, "PLANT_NOT_FOUND", "Plant not found");
    expect(resolveError(error, "loadPlant")).toBe("Plant not found");
  });

  it("returns fallback for ApiError with unknown code", () => {
    const error = new ApiError(500, "SOME_FUTURE_CODE", "something");
    expect(resolveError(error, "loadPlant")).toBe("Failed to load plant");
  });

  it("returns fallback for non-ApiError", () => {
    expect(resolveError(new Error("network fail"), "loadPlants")).toBe(
      "Failed to load plants",
    );
  });

  it("returns fallback for non-Error values", () => {
    expect(resolveError("string error", "createPlant")).toBe(
      "Failed to create plant",
    );
    expect(resolveError(null, "deletePlant")).toBe("Failed to delete plant");
  });
});
