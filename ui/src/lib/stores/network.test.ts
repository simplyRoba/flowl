import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { get } from "svelte/store";
import { isOffline, recheckHealth, startHealthPolling } from "./network";

describe("network store health polling", () => {
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

  it("sets isOffline to false when /health returns ok", async () => {
    isOffline.set(true);
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
    );

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);

    expect(get(isOffline)).toBe(false);
  });

  it("sets isOffline to true when /health fetch throws", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("network error"));

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);

    expect(get(isOffline)).toBe(true);
  });

  it("sets isOffline to true when /health returns non-ok status", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ status: "unhealthy" }), { status: 503 }),
    );

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);

    expect(get(isOffline)).toBe(true);
  });

  it("polls every 60 seconds", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);

    // Initial call
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    // After 60s
    await vi.advanceTimersByTimeAsync(60_000);
    expect(fetchSpy).toHaveBeenCalledTimes(2);

    // After another 60s
    await vi.advanceTimersByTimeAsync(60_000);
    expect(fetchSpy).toHaveBeenCalledTimes(3);
  });

  it("rechecks health on browser online event", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    window.dispatchEvent(new Event("online"));
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchSpy).toHaveBeenCalledTimes(2);
  });

  it("sets offline immediately on browser offline event", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
    );

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(false);

    window.dispatchEvent(new Event("offline"));
    expect(get(isOffline)).toBe(true);
  });

  it("recheckHealth triggers an immediate health check", async () => {
    vi.spyOn(globalThis, "fetch").mockRejectedValue(new Error("unreachable"));

    cleanup = startHealthPolling();
    // Let initial (failing) check complete
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

  it("recheckHealth sets offline when health endpoint is unreachable", async () => {
    vi.spyOn(globalThis, "fetch").mockResolvedValue(
      new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
    );
    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(false);

    // Health endpoint becomes unreachable
    vi.mocked(globalThis.fetch).mockRejectedValue(new Error("unreachable"));
    recheckHealth();
    await vi.advanceTimersByTimeAsync(0);
    expect(get(isOffline)).toBe(true);
  });

  it("stops polling on cleanup", async () => {
    const fetchSpy = vi
      .spyOn(globalThis, "fetch")
      .mockResolvedValue(
        new Response(JSON.stringify({ status: "ok" }), { status: 200 }),
      );

    cleanup = startHealthPolling();
    await vi.advanceTimersByTimeAsync(0);
    expect(fetchSpy).toHaveBeenCalledTimes(1);

    cleanup();

    await vi.advanceTimersByTimeAsync(60_000);
    expect(fetchSpy).toHaveBeenCalledTimes(1);
  });
});
