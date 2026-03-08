import { afterEach, describe, expect, it, vi } from "vitest";

import {
  calculatePullOffset,
  canStartPullToRefresh,
  getPullIndicatorState,
  getRefreshingPullGestureState,
  hasBlockingPullToRefreshOverlay,
  isPullToRefreshRoute,
  isStandalonePwaSession,
  isTouchCapableDevice,
  MAX_PULL_TO_REFRESH_OFFSET,
  PULL_TO_REFRESH_RELOAD_DELAY_MS,
  PULL_TO_REFRESH_THRESHOLD,
  reloadCurrentPage,
  schedulePullToRefreshReload,
  shouldTriggerPullToRefresh,
} from "./pull-to-refresh";

afterEach(() => {
  document.body.innerHTML = "";
  vi.useRealTimers();
});

describe("pull-to-refresh capability detection", () => {
  it("detects standalone sessions from display mode or iOS home-screen mode", () => {
    expect(
      isStandalonePwaSession({
        displayModeStandalone: true,
        navigatorStandalone: false,
      }),
    ).toBe(true);
    expect(
      isStandalonePwaSession({
        displayModeStandalone: false,
        navigatorStandalone: true,
      }),
    ).toBe(true);
    expect(
      isStandalonePwaSession({
        displayModeStandalone: false,
        navigatorStandalone: false,
      }),
    ).toBe(false);
  });

  it("detects touch-capable devices from coarse pointers and touch APIs", () => {
    expect(
      isTouchCapableDevice({
        coarsePointer: true,
        maxTouchPoints: 0,
        ontouchstart: false,
      }),
    ).toBe(true);
    expect(
      isTouchCapableDevice({
        coarsePointer: false,
        maxTouchPoints: 2,
        ontouchstart: false,
      }),
    ).toBe(true);
    expect(
      isTouchCapableDevice({
        coarsePointer: false,
        maxTouchPoints: 0,
        ontouchstart: true,
      }),
    ).toBe(true);
    expect(
      isTouchCapableDevice({
        coarsePointer: false,
        maxTouchPoints: 0,
        ontouchstart: false,
      }),
    ).toBe(false);
  });
});

describe("pull-to-refresh route allowlist", () => {
  it("allows dashboard, care journal, settings, and plant detail routes", () => {
    expect(isPullToRefreshRoute("/")).toBe(true);
    expect(isPullToRefreshRoute("/care-journal")).toBe(true);
    expect(isPullToRefreshRoute("/settings")).toBe(true);
    expect(isPullToRefreshRoute("/plants/42")).toBe(true);
  });

  it("excludes plant form routes and non-allowlisted paths", () => {
    expect(isPullToRefreshRoute("/plants/new")).toBe(false);
    expect(isPullToRefreshRoute("/plants/42/edit")).toBe(false);
    expect(isPullToRefreshRoute("/plants")).toBe(false);
    expect(isPullToRefreshRoute("/settings/advanced")).toBe(false);
  });
});

describe("pull-to-refresh threshold behavior", () => {
  it("stays in pulling state below the threshold", () => {
    expect(getPullIndicatorState(PULL_TO_REFRESH_THRESHOLD - 1)).toBe(
      "pulling",
    );
    expect(shouldTriggerPullToRefresh(PULL_TO_REFRESH_THRESHOLD - 1)).toBe(
      false,
    );
  });

  it("arms refresh at the threshold and clamps the visual offset", () => {
    expect(getPullIndicatorState(PULL_TO_REFRESH_THRESHOLD)).toBe("release");
    expect(shouldTriggerPullToRefresh(PULL_TO_REFRESH_THRESHOLD)).toBe(true);
    expect(calculatePullOffset(MAX_PULL_TO_REFRESH_OFFSET + 40)).toBe(
      MAX_PULL_TO_REFRESH_OFFSET,
    );
  });

  it("builds a refreshing state after a successful release", () => {
    expect(getRefreshingPullGestureState()).toEqual({
      gestureActive: false,
      touchStartY: null,
      rawPullDistance: PULL_TO_REFRESH_THRESHOLD,
      pullOffset: calculatePullOffset(PULL_TO_REFRESH_THRESHOLD),
      pullIndicatorState: "refreshing",
    });
  });

  it("schedules the reload handoff after a brief delay", () => {
    vi.useFakeTimers();
    const onReload = vi.fn();
    const fakeWindow = {
      setTimeout: vi.fn((callback: () => void, delay?: number) =>
        window.setTimeout(callback, delay),
      ),
    };

    schedulePullToRefreshReload(fakeWindow, onReload);

    expect(fakeWindow.setTimeout).toHaveBeenCalledWith(
      expect.any(Function),
      PULL_TO_REFRESH_RELOAD_DELAY_MS,
    );

    vi.advanceTimersByTime(PULL_TO_REFRESH_RELOAD_DELAY_MS);

    expect(onReload).toHaveBeenCalledOnce();
  });

  it("reloads the current page through the provided window object", () => {
    const reload = vi.fn();

    reloadCurrentPage({
      location: {
        reload,
      },
    });

    expect(reload).toHaveBeenCalledOnce();
  });
});

describe("pull-to-refresh safety gates", () => {
  it("requires standalone mode, touch input, and top-of-page scroll", () => {
    expect(
      canStartPullToRefresh({
        pathname: "/",
        scrollTop: 0,
        standalone: true,
        touchCapable: true,
        overlayOpen: false,
      }),
    ).toBe(true);

    expect(
      canStartPullToRefresh({
        pathname: "/",
        scrollTop: 12,
        standalone: true,
        touchCapable: true,
        overlayOpen: false,
      }),
    ).toBe(false);

    expect(
      canStartPullToRefresh({
        pathname: "/",
        scrollTop: 0,
        standalone: false,
        touchCapable: true,
        overlayOpen: false,
      }),
    ).toBe(false);

    expect(
      canStartPullToRefresh({
        pathname: "/",
        scrollTop: 0,
        standalone: true,
        touchCapable: false,
        overlayOpen: false,
      }),
    ).toBe(false);
  });

  it("detects blocking modal dialogs", () => {
    expect(hasBlockingPullToRefreshOverlay(document)).toBe(false);

    const modal = document.createElement("dialog");
    modal.setAttribute("open", "");
    document.body.append(modal);
    expect(hasBlockingPullToRefreshOverlay(document)).toBe(true);
  });

  it("detects an open lightbox dialog", () => {
    document.body.innerHTML = '<dialog class="lightbox" open></dialog>';
    expect(hasBlockingPullToRefreshOverlay(document)).toBe(true);
  });

  it("detects an open chat drawer", () => {
    document.body.innerHTML = '<div class="chat-drawer"></div>';
    expect(hasBlockingPullToRefreshOverlay(document)).toBe(true);
  });

  it("detects an open inline care entry form", () => {
    document.body.innerHTML = '<div class="care-entry-form"></div>';
    expect(hasBlockingPullToRefreshOverlay(document)).toBe(true);
  });

  it("blocks gesture activation whenever an overlay is open", () => {
    expect(
      canStartPullToRefresh({
        pathname: "/plants/42",
        scrollTop: 0,
        standalone: true,
        touchCapable: true,
        overlayOpen: true,
      }),
    ).toBe(false);
  });
});
