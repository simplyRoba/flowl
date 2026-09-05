import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { SvelteURL } from "svelte/reactivity";
import * as pullToRefresh from "$lib/pull-to-refresh";
import * as notifications from "$lib/stores/notifications";
import { isOffline } from "$lib/stores/network";

import LayoutHarness from "./LayoutHarness.svelte";

const {
  mockFetchSettings,
  mockFetchAuthConfig,
  mockSetServiceWorkerAuthMode,
  mockSetWorkerAuthMode,
  mockStartNetworkMonitor,
  mockStopNetworkMonitor,
  mockEnvironment,
} = vi.hoisted(() => ({
  mockFetchSettings: vi.fn(),
  mockFetchAuthConfig: vi.fn(),
  mockSetServiceWorkerAuthMode: vi.fn(),
  mockSetWorkerAuthMode: vi.fn(),
  mockStartNetworkMonitor: vi.fn(),
  mockStopNetworkMonitor: vi.fn(),
  mockEnvironment: { dev: false },
}));

let mockUrl: URL = new SvelteURL("http://localhost/");
let serviceWorkerRegister = vi.fn();
let serviceWorkerRemoveEventListener = vi.fn();

vi.mock("$app/environment", () => ({
  get dev() {
    return mockEnvironment.dev;
  },
}));

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

vi.mock("$lib/stores/network", async () => {
  const { writable } = await import("svelte/store");
  return {
    isOffline: writable(false),
    recheckHealth: vi.fn(),
    startNetworkMonitor: () => {
      mockStartNetworkMonitor();
      return mockStopNetworkMonitor;
    },
  };
});

vi.mock("$lib/auth", () => ({
  fetchAuthConfig: (...args: unknown[]) => mockFetchAuthConfig(...args),
  setServiceWorkerAuthMode: (...args: unknown[]) =>
    mockSetServiceWorkerAuthMode(...args),
  setWorkerAuthMode: (...args: unknown[]) => mockSetWorkerAuthMode(...args),
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

function installIdleServiceWorker() {
  serviceWorkerRegister = vi.fn().mockResolvedValue({
    active: null,
    installing: null,
    waiting: null,
    addEventListener: vi.fn(),
  });
  serviceWorkerRemoveEventListener = vi.fn();
  Object.defineProperty(window.navigator, "serviceWorker", {
    configurable: true,
    value: {
      register: serviceWorkerRegister,
      controller: null,
      addEventListener: vi.fn(),
      removeEventListener: serviceWorkerRemoveEventListener,
    },
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

describe("app layout route isolation", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockEnvironment.dev = false;
    mockUrl = new SvelteURL("http://localhost/login");
    mockFetchSettings.mockResolvedValue({ theme: "system", locale: "en" });
    mockFetchAuthConfig.mockResolvedValue({
      enabled: false,
      provider_name: null,
    });
    mockMatchMedia({ standalone: false, coarsePointer: false });
    installIdleServiceWorker();
  });

  afterEach(() => {
    cleanup();
    Reflect.deleteProperty(window.navigator, "serviceWorker");
    vi.restoreAllMocks();
  });

  it("does not bootstrap protected shell work on initial login", async () => {
    const addEventListener = vi.spyOn(window, "addEventListener");
    render(LayoutHarness);
    await Promise.resolve();

    expect(mockFetchSettings).not.toHaveBeenCalled();
    expect(mockFetchAuthConfig).not.toHaveBeenCalled();
    expect(mockStartNetworkMonitor).not.toHaveBeenCalled();
    expect(serviceWorkerRegister).not.toHaveBeenCalled();
    expect(window.matchMedia).not.toHaveBeenCalledWith(
      "(display-mode: standalone)",
    );
    expect(window.matchMedia).not.toHaveBeenCalledWith("(pointer: coarse)");
    for (const event of [
      "touchstart",
      "touchmove",
      "touchend",
      "touchcancel",
    ]) {
      expect(addEventListener).not.toHaveBeenCalledWith(
        event,
        expect.anything(),
      );
    }
    expect(document.querySelector(".sidebar")).toBeNull();
    expect(document.querySelector(".pull-indicator")).toBeNull();
  });

  it("does not register the service worker in development", async () => {
    mockEnvironment.dev = true;
    mockUrl = new SvelteURL("http://localhost/");

    render(LayoutHarness);

    await waitFor(() =>
      expect(mockStartNetworkMonitor).toHaveBeenCalledTimes(1),
    );
    expect(serviceWorkerRegister).not.toHaveBeenCalled();
    expect(mockFetchAuthConfig).not.toHaveBeenCalled();
  });

  it("starts and tears down protected network, service-worker, and pull setup across login transitions", async () => {
    const removeEventListener = vi.spyOn(window, "removeEventListener");
    render(LayoutHarness);
    mockUrl.href = "http://localhost/";

    await waitFor(() => expect(mockFetchSettings).toHaveBeenCalledTimes(1));
    await waitFor(() =>
      expect(mockStartNetworkMonitor).toHaveBeenCalledTimes(1),
    );
    await waitFor(() => expect(mockFetchAuthConfig).toHaveBeenCalledTimes(1));
    await waitFor(() => expect(serviceWorkerRegister).toHaveBeenCalledTimes(1));
    expect(mockSetServiceWorkerAuthMode).toHaveBeenCalledWith(false);
    expect(document.querySelector(".sidebar")).not.toBeNull();

    mockUrl.href = "http://localhost/login";
    await waitFor(() => expect(document.querySelector(".sidebar")).toBeNull());
    await waitFor(() =>
      expect(mockStopNetworkMonitor).toHaveBeenCalledTimes(1),
    );
    expect(serviceWorkerRemoveEventListener).toHaveBeenCalledWith(
      "controllerchange",
      expect.anything(),
    );
    for (const event of [
      "touchstart",
      "touchmove",
      "touchend",
      "touchcancel",
    ]) {
      expect(removeEventListener).toHaveBeenCalledWith(
        event,
        expect.anything(),
      );
    }
    expect(document.querySelector(".pull-indicator")).toBeNull();
  });

  it("does not create an effect update loop across repeated login transitions", async () => {
    const consoleError = vi
      .spyOn(console, "error")
      .mockImplementation(() => {});
    render(LayoutHarness);

    for (const path of ["/", "/login", "/settings", "/login"]) {
      mockUrl.href = `http://localhost${path}`;
      await Promise.resolve();
    }

    expect(consoleError.mock.calls.flat().join(" ")).not.toContain(
      "effect_update_depth_exceeded",
    );
  });
});

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
  let cacheStore: Map<string, Response>;

  beforeEach(() => {
    vi.clearAllMocks();
    cacheStore = new Map();
    isOffline.set(false);
    mockUrl = new URL("http://localhost/");
    mockFetchSettings.mockResolvedValue({ theme: "system", locale: "en" });
    mockFetchAuthConfig.mockResolvedValue({
      enabled: false,
      provider_name: null,
    });
    mockMatchMedia({ standalone: false, coarsePointer: false });

    // Mock Cache API (not available in jsdom)
    Object.defineProperty(window, "caches", {
      configurable: true,
      value: {
        open: vi.fn().mockResolvedValue({
          match: vi.fn((url: string) =>
            Promise.resolve(cacheStore.get(url) ?? undefined),
          ),
          put: vi.fn((url: string, response: Response) => {
            cacheStore.set(url, response);
            return Promise.resolve();
          }),
        }),
      },
    });
  });

  afterEach(() => {
    cleanup();
    isOffline.set(false);
    vi.restoreAllMocks();
  });

  function createVersionWorker(version: string) {
    return {
      postMessage: vi.fn((data: { type: string }, ports: MessagePort[]) => {
        if (data.type === "GET_VERSION") {
          ports[0]?.postMessage({ type: "VERSION", version });
        }
      }),
    };
  }

  function mockServiceWorker({
    activeVersion,
    newVersion,
  }: {
    activeVersion: string | null;
    newVersion: string;
  }) {
    let updateFoundHandler: (() => void) | null = null;
    let stateChangeHandler: (() => void) | null = null;

    const activeWorker = activeVersion
      ? createVersionWorker(activeVersion)
      : null;

    const installingWorker = {
      ...createVersionWorker(newVersion),
      state: "installing" as string,
      addEventListener: vi.fn((event: string, handler: () => void) => {
        if (event === "statechange") {
          stateChangeHandler = handler;
        }
      }),
    };

    const waitingWorker = createVersionWorker("waiting");
    const registration = {
      active: activeWorker,
      installing: installingWorker,
      waiting: waitingWorker,
      addEventListener: vi.fn((event: string, handler: () => void) => {
        if (event === "updatefound") {
          updateFoundHandler = handler;
        }
      }),
    };

    let controllerChangeHandler: (() => void) | null = null;
    const controllerWorker = createVersionWorker("controller");
    const sw = {
      register: vi.fn().mockResolvedValue(registration),
      controller: controllerWorker,
      addEventListener: vi.fn((event: string, handler: () => void) => {
        if (event === "controllerchange") controllerChangeHandler = handler;
      }),
      removeEventListener: vi.fn(),
    };

    Object.defineProperty(window.navigator, "serviceWorker", {
      configurable: true,
      value: sw,
    });

    return {
      triggerUpdate: () => {
        updateFoundHandler!();
      },
      activateNewWorker: () => {
        installingWorker.state = "activated";
        stateChangeHandler!();
      },
      triggerControllerChange: () => controllerChangeHandler?.(),
      workers: {
        activeWorker,
        installingWorker,
        waitingWorker,
        controllerWorker,
      },
    };
  }

  it("sends disabled auth mode to every worker lifecycle target", async () => {
    const { triggerControllerChange, workers } = mockServiceWorker({
      activeVersion: "v1",
      newVersion: "v2",
    });
    render(LayoutHarness);

    await waitFor(() => {
      expect(mockSetServiceWorkerAuthMode).toHaveBeenCalledWith(false);
      expect(mockSetWorkerAuthMode).toHaveBeenCalledWith(
        workers.activeWorker,
        false,
      );
      expect(mockSetWorkerAuthMode).toHaveBeenCalledWith(
        workers.installingWorker,
        false,
      );
      expect(mockSetWorkerAuthMode).toHaveBeenCalledWith(
        workers.waitingWorker,
        false,
      );
    });

    triggerControllerChange();
    expect(mockSetServiceWorkerAuthMode).toHaveBeenCalledTimes(3);
  });

  it("shows update toast when the service worker version actually changed", async () => {
    const { triggerUpdate, activateNewWorker } = mockServiceWorker({
      activeVersion: "v1",
      newVersion: "v2",
    });
    const pushSpy = vi.spyOn(notifications, "pushNotification");

    render(LayoutHarness);

    await waitFor(() => {
      expect(cacheStore.size).toBeGreaterThan(0);
    });

    triggerUpdate();
    activateNewWorker();

    await waitFor(() => {
      expect(pushSpy).toHaveBeenCalledWith(
        expect.objectContaining({
          variant: "info",
        }),
      );
    });
  });

  it("does not show update toast when the version is unchanged", async () => {
    const { triggerUpdate, activateNewWorker } = mockServiceWorker({
      activeVersion: "v1",
      newVersion: "v1",
    });
    const pushSpy = vi.spyOn(notifications, "pushNotification");

    render(LayoutHarness);

    await waitFor(() => {
      expect(cacheStore.size).toBeGreaterThan(0);
    });

    triggerUpdate();
    activateNewWorker();

    await new Promise((r) => setTimeout(r, 50));

    expect(pushSpy).not.toHaveBeenCalled();
  });

  it("does not show update toast on first installation", async () => {
    const { triggerUpdate } = mockServiceWorker({
      activeVersion: null,
      newVersion: "v1",
    });
    const pushSpy = vi.spyOn(notifications, "pushNotification");

    render(LayoutHarness);

    await new Promise((r) => setTimeout(r, 50));

    triggerUpdate();

    expect(pushSpy).not.toHaveBeenCalled();
  });
});
