import { describe, expect, it } from "vitest";
import { isCacheableApi, isThumbnail } from "$lib/sw-patterns";

describe("isCacheableApi", () => {
  it("matches /api/plants", () => {
    expect(isCacheableApi("/api/plants")).toBe(true);
  });

  it("matches /api/stats", () => {
    expect(isCacheableApi("/api/stats")).toBe(true);
  });

  it("matches /api/locations", () => {
    expect(isCacheableApi("/api/locations")).toBe(true);
  });

  it("matches /api/plants/{id}", () => {
    expect(isCacheableApi("/api/plants/1")).toBe(true);
    expect(isCacheableApi("/api/plants/42")).toBe(true);
    expect(isCacheableApi("/api/plants/999")).toBe(true);
  });

  it("matches /api/plants/{id}/care", () => {
    expect(isCacheableApi("/api/plants/1/care")).toBe(true);
    expect(isCacheableApi("/api/plants/42/care")).toBe(true);
  });

  it("does not match /api/care", () => {
    expect(isCacheableApi("/api/care")).toBe(false);
  });

  it("does not match /api/settings", () => {
    expect(isCacheableApi("/api/settings")).toBe(false);
  });

  it("does not match /api/mqtt/status", () => {
    expect(isCacheableApi("/api/mqtt/status")).toBe(false);
  });

  it("does not match /api/ai/status", () => {
    expect(isCacheableApi("/api/ai/status")).toBe(false);
  });

  it("does not match /api/ai/chat", () => {
    expect(isCacheableApi("/api/ai/chat")).toBe(false);
  });

  it("does not match /api/info", () => {
    expect(isCacheableApi("/api/info")).toBe(false);
  });

  it("does not match /api/plants with trailing slash", () => {
    expect(isCacheableApi("/api/plants/")).toBe(false);
  });

  it("does not match /api/plants/{id}/photo", () => {
    expect(isCacheableApi("/api/plants/1/photo")).toBe(false);
  });

  it("does not match /api/plants/new", () => {
    expect(isCacheableApi("/api/plants/new")).toBe(false);
  });

  it("does not match /api/data/export", () => {
    expect(isCacheableApi("/api/data/export")).toBe(false);
  });
});

describe("isThumbnail", () => {
  it("matches 200px thumbnails", () => {
    expect(isThumbnail("/uploads/abc123_200.jpg")).toBe(true);
  });

  it("matches 600px thumbnails", () => {
    expect(isThumbnail("/uploads/abc123_600.jpg")).toBe(true);
  });

  it("matches 1000px thumbnails", () => {
    expect(isThumbnail("/uploads/abc123_1000.jpg")).toBe(true);
  });

  it("matches thumbnails with long hashes", () => {
    expect(isThumbnail("/uploads/a1b2c3d4e5f6_200.jpg")).toBe(true);
  });

  it("does not match original uploads", () => {
    expect(isThumbnail("/uploads/abc123.png")).toBe(false);
    expect(isThumbnail("/uploads/abc123.jpg")).toBe(false);
  });

  it("does not match unsupported sizes", () => {
    expect(isThumbnail("/uploads/abc123_300.jpg")).toBe(false);
    expect(isThumbnail("/uploads/abc123_500.jpg")).toBe(false);
  });

  it("does not match non-upload paths", () => {
    expect(isThumbnail("/api/plants/1_200.jpg")).toBe(false);
  });

  it("does not match non-jpg thumbnails", () => {
    expect(isThumbnail("/uploads/abc123_200.png")).toBe(false);
  });
});
