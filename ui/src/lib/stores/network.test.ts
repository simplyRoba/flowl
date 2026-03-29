import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import { isOffline, recheckHealth, startNetworkMonitor } from "./network";

describe("network store", () => {
  let cleanup: () => void;

  beforeEach(() => {
    vi.useFakeTimers();
    isOffline.set(false);
  });

  afterEach(() => {
    cleanup?.();
    vi.useRealTimers();
    vi.restoreAllMocks();
    isOffline.set(false);
  });

  it("sets isOffline to false when /health returns ok on startup", async () => {
    isOffline.set(true);
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
    );

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);

    expect(get(isOffline)).toBe(false);
  });

  it("sets isOffline to true when /health fetch throws on startup", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("network error"));

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);

    expect(get(isOffline)).toBe(true);
  });

  it("sets isOffline to true when /health returns non-ok status", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ status: "unhealthy" }), { status: 503 }),
    );

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);

    expect(get(isOffline)).toBe(true);
  });

  it("starts recovery polling when offline is detected on startup", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockRejectedValue(new Error("unreachable"));

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(true);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    // Recovery poll fires after 10s
    await vi.advanceTimersByTimeAsync(10_000);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });

  it("does not poll when online", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    // No polling after 60s since we're online
    await vi.advanceTimersByTimeAsync(60_000);
    expect(fetchSpy).toHaveBeenCalledTimes(1);
  });

  it("stops recovery polling once health succeeds", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockRejectedValue(new Error("unreachable"));

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(true);

    // Recovery: health succeeds
    fetchSpy.mockResolvedValue(
      new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
    );
    await vi.advanceTimersByTimeAsync(10_000);
    expect(get(isOffline)).toBe(false);
    expect(fetchSpy).toHaveBeenCalledTimes(2);

    // No more polling after recovery
    await vi.advanceTimersByTimeAsync(30_000);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });

  it("rechecks health on browser online event", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    window.dispatchEvent(new Event("online"));
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });

  it("sets offline immediately on browser offline event and starts recovery poll", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(false);

    window.dispatchEvent(new Event("offline"));
    expect(get(isOffline)).toBe(true);

    // Recovery poll should be running
    await vi.advanceTimersByTimeAsync(10_000);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });

  it("recheckHealth triggers an immediate health check", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("unreachable"));

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(true);

    // Simulate recovery
    vi.mocked(globalThis.fetch).mockResolvedValue(
      new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
    );
    recheckHealth();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(false);
  });

  it("recheckHealth starts recovery poll when offline detected", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(false);

    // Goes offline
    fetchSpy.mockRejectedValue(new Error("unreachable"));
    recheckHealth();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(true);

    // Recovery poll fires
    await vi.advanceTimersByTimeAsync(10_000);
    expect(fetchSpy.mock.calls.length).toBeGreaterThanOrEqual(3);
  });

  it("cleanup stops recovery polling and removes event listeners", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockRejectedValue(new Error("unreachable"));

    cleanup = startNetworkMonitor();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(true);
    const callsAfterInit = fetchSpy.mock.calls.length;

    cleanup();

    await vi.advanceTimersByTimeAsync(30_000);
    expect(fetchSpy).toHaveBeenCalledTimes(callsAfterInit);
  });
});
