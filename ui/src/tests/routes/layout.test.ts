import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import * as pullToRefresh from "$lib/pull-to-refresh";
import * as notifications from "$lib/stores/notifications";
import { isOffline } from "$lib/stores/network";

import LayoutHarness from "./LayoutHarness.svelte";

const mockFetchSettings = vi.fn();

let mockUrl = new URL("http://localhost/");

vi.mock("$app/paths", () => ({
  resolve: (value: string) => value,
}));

vi.mock("$app/state", () => ({
  page: {
    get url() {
      return mockUrl;
    },
  },
}));

vi.mock("$lib/api", () => ({
  fetchSettings: (...args: unknown[]) => mockFetchSettings(...args),
}));

function mockMatchMedia({
  standalone,
  coarsePointer,
}: {
  standalone: boolean;
  coarsePointer: boolean;
}) {
  Object.defineProperty(window, "matchMedia", {
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches:
        query === "(display-mode: standalone)"
          ? standalone
          : query === "(pointer: coarse)"
            ? coarsePointer
            : false,
      media: query,
      onchange: null,
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      addListener: vi.fn(),
      removeListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
}

function buildTouchEvent(
  type: string,
  yPositions: number[],
  cancelable = false,
) {
  const event = new Event(type, {
    bubbles: true,
    cancelable,
  }) as Event & {
    touches: Array<{ clientX: number; clientY: number }>;
  };

  event.touches = yPositions.map((clientY) => ({ clientX: 0, clientY }));

  return event;
}

async function performPull(distance: number) {
  await fireEvent(window, buildTouchEvent("touchstart", [120]));
  await fireEvent(window, buildTouchEvent("touchmove", [120 + distance], true));
}

describe("app layout pull-to-refresh", () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.clearAllMocks();
    mockUrl = new URL("http://localhost/");
    mockFetchSettings.mockResolvedValue({ theme: "system", locale: "en" });
    mockMatchMedia({ standalone: true, coarsePointer: true });
    Object.defineProperty(window, "scrollY", { configurable: true, value: 0 });
    Object.defineProperty(window.navigator, "maxTouchPoints", {
      configurable: true,
      value: 2,
    });
    Object.defineProperty(window.navigator, "standalone", {
      configurable: true,
      value: true,
    });
    document.body.scrollTop = 0;
    document.documentElement.scrollTop = 0;
  });

  afterEach(() => {
    cleanup();
    vi.runOnlyPendingTimers();
    vi.useRealTimers();
    vi.restoreAllMocks();
  });

  it.each([
    ["modal dialog", "dialog"],
    ["lightbox", "lightbox"],
    ["chat drawer", "chat"],
    ["inline care entry", "care-entry"],
  ] as const)(
    "does not arm on plant detail while a %s overlay is open",
    async (_label, overlay) => {
      mockUrl = new URL("http://localhost/plants/42");
      const reloadSpy = vi
        .spyOn(pullToRefresh, "reloadCurrentPage")
        .mockImplementation(() => undefined);

      render(LayoutHarness, { overlay });

      const indicator = screen.getByTestId("pull-to-refresh-indicator");

      await performPull(pullToRefresh.PULL_TO_REFRESH_THRESHOLD + 12);
      await fireEvent(window, new Event("touchend", { bubbles: true }));
      await vi.advanceTimersByTimeAsync(120);

      expect(indicator.getAttribute("aria-hidden")).toBe("true");
      expect(reloadSpy).not.toHaveBeenCalled();
    },
  );
});

describe("app layout offline indicator", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    isOffline.set(false);
    mockUrl = new URL("http://localhost/");
    mockFetchSettings.mockResolvedValue({ theme: "system", locale: "en" });
    mockMatchMedia({ standalone: false, coarsePointer: false });
  });

  afterEach(() => {
    cleanup();
    isOffline.set(false);
    vi.restoreAllMocks();
  });

  it("shows offline dot when isOffline store is true", async () => {
    isOffline.set(true);

    render(LayoutHarness);

    await waitFor(() => {
      expect(document.querySelector(".offline-dot")).not.toBeNull();
    });
  });

  it("does not show offline dot when isOffline store is false", async () => {
    isOffline.set(false);

    render(LayoutHarness);

    await waitFor(() => {
      expect(document.querySelector(".offline-dot")).toBeNull();
    });
  });

  it("shows offline dot when store transitions to offline", async () => {
    isOffline.set(false);

    render(LayoutHarness);

    expect(document.querySelector(".offline-dot")).toBeNull();

    isOffline.set(true);

    await waitFor(() => {
      expect(document.querySelector(".offline-dot")).not.toBeNull();
    });
  });

  it("hides offline dot when store transitions to online", async () => {
    isOffline.set(true);

    render(LayoutHarness);

    await waitFor(() => {
      expect(document.querySelector(".offline-dot")).not.toBeNull();
    });

    isOffline.set(false);

    await waitFor(() => {
      expect(document.querySelector(".offline-dot")).toBeNull();
    });
  });
});

describe("app layout service worker update notification", () => {
  let controllerChangeHandler: (() => void) | null = null;

  beforeEach(() => {
    vi.clearAllMocks();
    controllerChangeHandler = null;
    isOffline.set(false);
    mockUrl = new URL("http://localhost/");
    mockFetchSettings.mockResolvedValue({ theme: "system", locale: "en" });
    mockMatchMedia({ standalone: false, coarsePointer: false });
  });

  afterEach(() => {
    cleanup();
    isOffline.set(false);
    vi.restoreAllMocks();
  });

  function mockServiceWorker(hasController: boolean) {
    const sw = {
      register: vi.fn().mockResolvedValue(undefined),
      controller: hasController ? {} : null,
      addEventListener: vi.fn((event: string, handler: () => void) => {
        if (event === "controllerchange") {
          controllerChangeHandler = handler;
        }
      }),
    };

    Object.defineProperty(window.navigator, "serviceWorker", {
      configurable: true,
      value: sw,
    });

    return sw;
  }

  it("shows update toast on controllerchange when previous controller existed", async () => {
    mockServiceWorker(true);
    const pushSpy = vi.spyOn(notifications, "pushNotification");

    render(LayoutHarness);

    expect(controllerChangeHandler).not.toBeNull();
    controllerChangeHandler!();

    await waitFor(() => {
      expect(pushSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          variant: "info",
        }),
      );
    });
  });

  it("does not show update toast on controllerchange for first registration", async () => {
    mockServiceWorker(false);
    const pushSpy = vi.spyOn(notifications, "pushNotification");

    render(LayoutHarness);

    expect(controllerChangeHandler).not.toBeNull();
    controllerChangeHandler!();

    expect(pushSpy).not.toHaveBeenCalled();
  });
});
