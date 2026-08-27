import { cleanup, fireEvent, render } from "@testing-library/svelte";
import { SvelteURL } from "svelte/reactivity";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../../../routes/care-journal/+page.svelte";
import { ApiError, type CareEvent } from "$lib/api";
import { isOffline } from "$lib/stores/network";

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (
  this: HTMLDialogElement,
) {
  this.setAttribute("open", "");
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
  this.removeAttribute("open");
});

const mockFetchAllCareEvents = vi.fn();
const mockUpdateCareEvent = vi.fn();
const mockGoto = vi.fn();

vi.mock("$lib/api", async () => {
  const actual = await vi.importActual<typeof import("$lib/api")>("$lib/api");
  return {
    ...actual,
    fetchAllCareEvents: (...args: unknown[]) => mockFetchAllCareEvents(...args),
    updateCareEvent: (...args: unknown[]) => mockUpdateCareEvent(...args),
  };
});

let mockUrl = new SvelteURL("http://localhost/care-journal");

vi.mock("$app/state", () => ({
  page: {
    get url() {
      return mockUrl;
    },
  },
}));

vi.mock("$app/navigation", () => ({
  goto: (...args: unknown[]) => {
    mockUrl.href = new URL(String(args[0]), "http://localhost").href;
    return mockGoto(...args);
  },
}));

function makeEvent(overrides: Partial<CareEvent> = {}): CareEvent {
  return {
    id: 1,
    plant_id: 1,
    plant_name: "Fern",
    event_type: "watered",
    notes: null,
    photo_url: null,
    occurred_at: "2025-02-01T10:00:00Z",
    created_at: "2025-02-01T10:00:00Z",
    ...overrides,
  };
}

function shortDate(value: string): string {
  return new Date(value).toLocaleDateString(undefined, {
    month: "short",
    day: "numeric",
    year: "2-digit",
  });
}

class MockIntersectionObserver {
  static instances: MockIntersectionObserver[] = [];

  readonly targets = new Set<Element>();
  readonly observe = vi.fn((target: Element) => this.targets.add(target));
  readonly unobserve = vi.fn((target: Element) => this.targets.delete(target));
  readonly disconnect = vi.fn(() => this.targets.clear());

  constructor(private readonly callback: IntersectionObserverCallback) {
    MockIntersectionObserver.instances.push(this);
  }

  trigger(isIntersecting = true) {
    this.callback(
      [...this.targets].map(
        (target) => ({ isIntersecting, target }) as IntersectionObserverEntry,
      ),
      this as unknown as IntersectionObserver,
    );
  }
}

class MockResizeObserver {
  static instances: MockResizeObserver[] = [];

  readonly targets = new Set<Element>();
  readonly observe = vi.fn((target: Element) => this.targets.add(target));
  readonly unobserve = vi.fn((target: Element) => this.targets.delete(target));
  readonly disconnect = vi.fn(() => this.targets.clear());

  constructor(private readonly callback: ResizeObserverCallback) {
    MockResizeObserver.instances.push(this);
  }

  trigger() {
    this.callback([], this as unknown as ResizeObserver);
  }
}

const documentSize = { scrollHeight: 500, clientHeight: 1000 };
const originalScrollHeight = Object.getOwnPropertyDescriptor(
  document.documentElement,
  "scrollHeight",
);
const originalClientHeight = Object.getOwnPropertyDescriptor(
  document.documentElement,
  "clientHeight",
);
const originalScrollingElement = Object.getOwnPropertyDescriptor(
  document,
  "scrollingElement",
);

function setDocumentOverflow(overflows: boolean) {
  documentSize.scrollHeight = overflows ? 1002 : 500;
  documentSize.clientHeight = 1000;
}

function restoreProperty(
  target: object,
  property: PropertyKey,
  descriptor: PropertyDescriptor | undefined,
) {
  if (descriptor) {
    Object.defineProperty(target, property, descriptor);
  } else {
    Reflect.deleteProperty(target, property);
  }
}

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (reason?: unknown) => void;
  const promise = new Promise<T>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise;
    reject = rejectPromise;
  });
  return { promise, resolve, reject };
}

beforeEach(() => {
  vi.clearAllMocks();
  mockFetchAllCareEvents.mockReset();
  mockUpdateCareEvent.mockReset();
  MockIntersectionObserver.instances = [];
  MockResizeObserver.instances = [];
  vi.stubGlobal("IntersectionObserver", MockIntersectionObserver);
  vi.stubGlobal("ResizeObserver", MockResizeObserver);
  Object.defineProperties(document.documentElement, {
    scrollHeight: {
      configurable: true,
      get: () => documentSize.scrollHeight,
    },
    clientHeight: {
      configurable: true,
      get: () => documentSize.clientHeight,
    },
  });
  Object.defineProperty(document, "scrollingElement", {
    configurable: true,
    get: () => document.documentElement,
  });
  setDocumentOverflow(false);
  isOffline.set(false);
  mockUrl = new SvelteURL("http://localhost/care-journal");
  mockFetchAllCareEvents.mockResolvedValue({ events: [], has_more: false });
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
  restoreProperty(
    document.documentElement,
    "scrollHeight",
    originalScrollHeight,
  );
  restoreProperty(
    document.documentElement,
    "clientHeight",
    originalClientHeight,
  );
  restoreProperty(document, "scrollingElement", originalScrollingElement);
});

describe("care journal thumbnails", () => {
  it("uses 200px thumbnail for event photo", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 1, photo_url: "/uploads/care/1.jpg" })],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      const img = document.querySelector(
        ".log-entry-photo img",
      ) as HTMLImageElement;
      expect(img).toBeTruthy();
      expect(img.src).toContain("/uploads/care/1_200.jpg");
    });
  });

  it("falls back to original photo_url on thumbnail error", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 2, photo_url: "/uploads/care/2.png" })],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-entry-photo img")).toBeTruthy();
    });
    const img = document.querySelector(
      ".log-entry-photo img",
    ) as HTMLImageElement;
    expect(img.src).toContain("/uploads/care/2_200.jpg");
    await fireEvent.error(img);
    expect(img.src).toContain("/uploads/care/2.png");
    expect(img.src).not.toContain("_200");
  });

  it("opens lightbox with original photo_url when clicking thumbnail", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 3, photo_url: "/uploads/care/3.jpg" })],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-entry-photo")).toBeTruthy();
    });
    const photoBtn = document.querySelector(
      ".log-entry-photo",
    ) as HTMLButtonElement;
    await fireEvent.click(photoBtn);

    const lightbox = document.querySelector(
      "dialog.lightbox",
    ) as HTMLDialogElement;
    expect(lightbox.hasAttribute("open")).toBe(true);
    const lightboxImg = lightbox.querySelector("img") as HTMLImageElement;
    expect(lightboxImg.src).toContain("/uploads/care/3.jpg");
    expect(lightboxImg.src).not.toContain("_200");
  });

  it("does not render photo element when event has no photo_url", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 4, photo_url: null })],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-entry")).toBeTruthy();
    });
    expect(document.querySelector(".log-entry-photo")).toBeNull();
  });
});

describe("care journal filters", () => {
  it("loads the newest bounded page when URL has no type param", async () => {
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    expect(mockFetchAllCareEvents).toHaveBeenCalledWith(
      500,
      undefined,
      undefined,
    );
  });

  it("loads with type filter when URL has type params", async () => {
    mockUrl = new SvelteURL(
      "http://localhost/care-journal?type=watered&type=fertilized",
    );
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    expect(mockFetchAllCareEvents).toHaveBeenCalledWith(
      500,
      undefined,
      expect.arrayContaining(["watered", "fertilized"]),
    );
  });

  it("shows All chip as active when no filters are set", async () => {
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    const chips = document.querySelectorAll(".chip");
    const allChip = chips[0];
    expect(allChip.classList.contains("active")).toBe(true);
  });

  it("toggles a type filter on click", async () => {
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    // Click "Watered" chip (second chip, after "All")
    const chips = document.querySelectorAll(".chip");
    await fireEvent.click(chips[1]); // watered

    expect(mockGoto).toHaveBeenCalled();
    const gotoUrl = mockGoto.mock.calls[0][0] as string;
    expect(gotoUrl).toContain("type=watered");
    expect(mockGoto.mock.calls[0][1]).toEqual(
      expect.objectContaining({ replaceState: true }),
    );
  });

  it("All chip selects all types when no filters are active", async () => {
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    const allChip = document.querySelectorAll(".chip")[0];
    await fireEvent.click(allChip);

    expect(mockGoto).toHaveBeenCalled();
    const gotoUrl = mockGoto.mock.calls[0][0] as string;
    for (const t of [
      "watered",
      "fertilized",
      "repotted",
      "pruned",
      "custom",
      "ai-consultation",
    ]) {
      expect(gotoUrl).toContain(`type=${t}`);
    }
  });

  it("All chip clears filters when some are active", async () => {
    mockUrl = new SvelteURL(
      "http://localhost/care-journal?type=watered&type=pruned",
    );
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    const allChip = document.querySelectorAll(".chip")[0];
    await fireEvent.click(allChip);

    expect(mockGoto).toHaveBeenCalled();
    const gotoUrl = mockGoto.mock.calls[0][0] as string;
    expect(gotoUrl).not.toContain("type=");
  });

  it("toggling off the last active type returns to unfiltered state", async () => {
    mockUrl = new SvelteURL("http://localhost/care-journal?type=watered");
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    // Click "Watered" chip to toggle it off (second chip, after "All")
    const chips = document.querySelectorAll(".chip");
    await fireEvent.click(chips[1]);

    expect(mockGoto).toHaveBeenCalled();
    const gotoUrl = mockGoto.mock.calls[0][0] as string;
    expect(gotoUrl).not.toContain("type=");
  });
});

describe("care journal skeleton loading", () => {
  it("shows skeleton shimmer while loading", async () => {
    let resolveEvents: (value: unknown) => void;
    mockFetchAllCareEvents.mockReturnValue(
      new Promise((resolve) => {
        resolveEvents = resolve;
      }),
    );
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".skeleton-list")).toBeTruthy();
    });
    expect(document.querySelectorAll(".skeleton-entry").length).toBe(6);

    resolveEvents!({ events: [], has_more: false });

    await vi.waitFor(() => {
      expect(document.querySelector(".skeleton-list")).toBeNull();
    });
  });
});

describe("care journal history loading", () => {
  it("manually loads older entries without document overflow using the last raw event ID", async () => {
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 5, event_type: "fertilized", notes: "Newest" }),
          makeEvent({ id: 4, event_type: "fertilized", notes: "Boundary" }),
        ],
        has_more: true,
      })
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 4, event_type: "fertilized", notes: "Boundary" }),
          makeEvent({ id: 3, event_type: "fertilized", notes: "Older" }),
        ],
        has_more: false,
      });
    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByRole("button", { name: "Load older entries" }),
      ).toBeTruthy();
    });
    expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(1);
    expect(MockIntersectionObserver.instances).toHaveLength(0);

    await fireEvent.click(
      view.getByRole("button", { name: "Load older entries" }),
    );

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenLastCalledWith(
        500,
        4,
        undefined,
      );
    });
    expect(
      [...document.querySelectorAll(".log-entry-note")].map(
        (entry) => entry.textContent,
      ),
    ).toEqual(["Newest", "Boundary", "Older"]);
    expect(
      view.queryByRole("button", { name: "Load older entries" }),
    ).toBeNull();
  });

  it("keeps the loaded timeline visible while an older page is loading", async () => {
    const olderPage = deferred<{ events: CareEvent[]; has_more: boolean }>();
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 4, event_type: "fertilized", notes: "Retained" }),
        ],
        has_more: true,
      })
      .mockReturnValueOnce(olderPage.promise);
    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByRole("button", { name: "Load older entries" }),
      ).toBeTruthy();
    });
    await fireEvent.click(
      view.getByRole("button", { name: "Load older entries" }),
    );

    expect(view.getByText("Retained")).toBeTruthy();
    expect(view.queryByText("Loading older entries…")).toBeTruthy();
    expect(document.querySelector(".skeleton-list")).toBeNull();

    olderPage.resolve({ events: [], has_more: false });
    await vi.waitFor(() => {
      expect(
        view.queryByRole("button", { name: "Load older entries" }),
      ).toBeNull();
    });
  });

  it("labels unresolved watering groups with an inexact loaded count and continuation copy", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [
        makeEvent({ id: 2, occurred_at: "2025-02-02T12:00:00Z" }),
        makeEvent({ id: 1, occurred_at: "2025-02-01T12:00:00Z" }),
      ],
      has_more: true,
    });
    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByText(
          `Watered 2+ times, ${shortDate("2025-02-01T12:00:00Z")} – ${shortDate("2025-02-02T12:00:00Z")}`,
        ),
      ).toBeTruthy();
    });
    expect(
      view.getByText("Older entries may continue this watering streak."),
    ).toBeTruthy();
  });

  it("loads older entries when an overflowing document exposes the sentinel", async () => {
    setDocumentOverflow(true);
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 4, event_type: "fertilized", notes: "Newest" }),
        ],
        has_more: true,
      })
      .mockResolvedValueOnce({ events: [], has_more: false });
    render(Page);

    await vi.waitFor(() => {
      expect(MockIntersectionObserver.instances).toHaveLength(1);
    });
    MockIntersectionObserver.instances[0].trigger();

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenLastCalledWith(
        500,
        4,
        undefined,
      );
    });
  });

  it("requires the sentinel to leave and re-enter before another automatic page", async () => {
    setDocumentOverflow(true);
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [makeEvent({ id: 4 })],
        has_more: true,
      })
      .mockResolvedValueOnce({
        events: [makeEvent({ id: 3 })],
        has_more: true,
      })
      .mockResolvedValueOnce({ events: [], has_more: false });
    render(Page);

    await vi.waitFor(() => {
      expect(MockIntersectionObserver.instances).toHaveLength(1);
    });
    MockIntersectionObserver.instances[0].trigger(true);
    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(2);
      expect(MockIntersectionObserver.instances.length).toBeGreaterThan(1);
    });

    const nextObserver = MockIntersectionObserver.instances.at(-1)!;
    nextObserver.trigger(true);
    expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(2);

    nextObserver.trigger(false);
    nextObserver.trigger(true);
    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(3);
    });
  });

  it("does not observe or auto-load a visible sentinel until the document overflows", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 4, event_type: "fertilized", notes: "Manual" })],
      has_more: true,
    });
    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByRole("button", { name: "Load older entries" }),
      ).toBeTruthy();
    });
    expect(MockIntersectionObserver.instances).toHaveLength(0);
    expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(1);

    setDocumentOverflow(true);
    MockResizeObserver.instances[0].trigger();

    await vi.waitFor(() => {
      expect(MockIntersectionObserver.instances).toHaveLength(1);
    });
  });

  it("guards duplicate observer and click continuation triggers", async () => {
    setDocumentOverflow(true);
    const olderPage = deferred<{ events: CareEvent[]; has_more: boolean }>();
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [makeEvent({ id: 4, event_type: "fertilized" })],
        has_more: true,
      })
      .mockReturnValueOnce(olderPage.promise);
    const view = render(Page);

    await vi.waitFor(() => {
      expect(MockIntersectionObserver.instances).toHaveLength(1);
    });
    const observer = MockIntersectionObserver.instances[0];
    observer.trigger();
    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(2);
    });
    await fireEvent.click(
      view.getByRole("button", { name: "Loading older entries…" }),
    );
    observer.trigger();
    expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(2);

    olderPage.resolve({ events: [], has_more: false });
    await vi.waitFor(() => {
      expect(
        view.queryByRole("button", { name: "Load older entries" }),
      ).toBeNull();
    });
  });

  it("keeps only one active monitor when refreshes overlap", async () => {
    setDocumentOverflow(true);
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 4 }), makeEvent({ id: 3 })],
      has_more: true,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(
        MockIntersectionObserver.instances.filter(
          (observer) => observer.targets.size > 0,
        ),
      ).toHaveLength(1);
      expect(
        MockResizeObserver.instances.filter(
          (observer) => observer.targets.size > 0,
        ),
      ).toHaveLength(1);
    });

    const toggle = document.querySelector(
      ".log-group-toggle",
    ) as HTMLButtonElement;
    toggle.click();
    toggle.click();

    await vi.waitFor(() => {
      expect(
        MockIntersectionObserver.instances.filter(
          (observer) => observer.targets.size > 0,
        ),
      ).toHaveLength(1);
      expect(
        MockResizeObserver.instances.filter(
          (observer) => observer.targets.size > 0,
        ),
      ).toHaveLength(1);
    });
  });

  it("cleans up continuation observers and the resize listener when destroyed", async () => {
    setDocumentOverflow(true);
    mockFetchAllCareEvents.mockResolvedValue({
      events: [makeEvent({ id: 4, event_type: "fertilized" })],
      has_more: true,
    });
    const removeEventListener = vi.spyOn(window, "removeEventListener");
    const view = render(Page);

    await vi.waitFor(() => {
      expect(MockIntersectionObserver.instances).toHaveLength(1);
      expect(MockResizeObserver.instances).toHaveLength(1);
    });
    const intersectionObserver = MockIntersectionObserver.instances[0];
    const resizeObserver = MockResizeObserver.instances[0];

    view.unmount();

    expect(intersectionObserver.disconnect).toHaveBeenCalledOnce();
    expect(resizeObserver.disconnect).toHaveBeenCalledOnce();
    expect(removeEventListener).toHaveBeenCalledWith(
      "resize",
      expect.any(Function),
    );
  });

  it("preserves data after a continuation error, pauses auto-retry, and retries manually from the same cursor", async () => {
    setDocumentOverflow(true);
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 4, event_type: "fertilized", notes: "Retained" }),
        ],
        has_more: true,
      })
      .mockRejectedValueOnce(new Error("network error"))
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 3, event_type: "fertilized", notes: "Retried" }),
        ],
        has_more: false,
      });
    const view = render(Page);

    await vi.waitFor(() => {
      expect(MockIntersectionObserver.instances).toHaveLength(1);
    });
    const observer = MockIntersectionObserver.instances[0];
    observer.trigger();

    await vi.waitFor(() => {
      expect(
        view.getByRole("button", { name: "Retry loading older entries" }),
      ).toBeTruthy();
    });
    expect(view.getByText("Retained")).toBeTruthy();
    expect(mockFetchAllCareEvents).toHaveBeenLastCalledWith(500, 4, undefined);

    observer.trigger();
    expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(2);

    await fireEvent.click(
      view.getByRole("button", { name: "Retry loading older entries" }),
    );
    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenLastCalledWith(
        500,
        4,
        undefined,
      );
      expect(view.getByText("Retried")).toBeTruthy();
    });
  });

  it("ignores a stale response after filters restart the journal from the newest page", async () => {
    const unfiltered = deferred<{ events: CareEvent[]; has_more: boolean }>();
    const filtered = deferred<{ events: CareEvent[]; has_more: boolean }>();
    mockFetchAllCareEvents
      .mockReturnValueOnce(unfiltered.promise)
      .mockReturnValueOnce(filtered.promise);
    const view = render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalledTimes(1);
    });
    await fireEvent.click(document.querySelectorAll(".chip")[1]);
    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenLastCalledWith(500, undefined, [
        "watered",
      ]);
    });

    filtered.resolve({
      events: [makeEvent({ id: 2, event_type: "watered", notes: "Filtered" })],
      has_more: false,
    });
    await vi.waitFor(() => {
      expect(view.getByText("Filtered")).toBeTruthy();
    });

    unfiltered.resolve({
      events: [
        makeEvent({ id: 1, event_type: "fertilized", notes: "Stale event" }),
      ],
      has_more: false,
    });
    await vi.waitFor(() => {
      expect(view.getByText("Filtered")).toBeTruthy();
    });
    expect(view.queryByText("Stale event")).toBeNull();
  });

  it("refreshes from the newest page when the cursor event was deleted", async () => {
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 4, event_type: "fertilized", notes: "Retained" }),
        ],
        has_more: true,
      })
      .mockRejectedValueOnce(
        new ApiError(422, "CARE_EVENT_NOT_FOUND", "Care event not found"),
      )
      .mockResolvedValueOnce({
        events: [
          makeEvent({ id: 5, event_type: "fertilized", notes: "Refreshed" }),
        ],
        has_more: false,
      });
    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByRole("button", { name: "Load older entries" }),
      ).toBeTruthy();
    });
    await fireEvent.click(
      view.getByRole("button", { name: "Load older entries" }),
    );

    await vi.waitFor(() => {
      expect(
        view.getByRole("button", { name: "Refresh journal" }),
      ).toBeTruthy();
    });
    expect(view.getByText("Retained")).toBeTruthy();

    await fireEvent.click(
      view.getByRole("button", { name: "Refresh journal" }),
    );
    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenLastCalledWith(
        500,
        undefined,
        undefined,
      );
      expect(view.getByText("Refreshed")).toBeTruthy();
    });
    expect(view.queryByText("Retained")).toBeNull();
  });

  it("keeps an expanded partial group open and appends its older members", async () => {
    mockFetchAllCareEvents
      .mockResolvedValueOnce({
        events: [makeEvent({ id: 4 }), makeEvent({ id: 3 })],
        has_more: true,
      })
      .mockResolvedValueOnce({
        events: [makeEvent({ id: 2 })],
        has_more: false,
      });
    const view = render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-group-summary")).toBeTruthy();
      expect(
        view.getByRole("button", { name: "Load older entries" }),
      ).toBeTruthy();
    });
    await fireEvent.click(
      document.querySelector(".log-group-toggle") as HTMLButtonElement,
    );
    expect(
      document.querySelectorAll(".log-group-expanded .log-entry-nested"),
    ).toHaveLength(2);

    await fireEvent.click(
      view.getByRole("button", { name: "Load older entries" }),
    );
    await vi.waitFor(() => {
      expect(
        document.querySelectorAll(".log-group-expanded .log-entry-nested"),
      ).toHaveLength(3);
    });
    expect(
      (document.querySelector(".log-group-toggle") as HTMLButtonElement)
        .ariaExpanded,
    ).toBe("true");
  });
});

describe("care journal event grouping", () => {
  it("groups consecutive waterings into a collapsible summary", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [
        makeEvent({ id: 3, occurred_at: "2025-02-03T12:00:00Z" }),
        makeEvent({ id: 2, occurred_at: "2025-02-02T11:00:00Z" }),
        makeEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
      ],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-group-summary")).toBeTruthy();
    });

    // Should show one group, not three individual entries
    expect(document.querySelectorAll(".log-entry").length).toBe(1);
    expect(document.querySelector(".log-group-chevron")).toBeTruthy();
    expect(document.querySelector(".log-entry-action")?.textContent).toBe(
      `Watered 3 times, ${shortDate("2025-02-01T10:00:00Z")} – ${shortDate("2025-02-03T12:00:00Z")}`,
    );
  });

  it("expands group on click to show individual entries", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [
        makeEvent({ id: 3, occurred_at: "2025-02-01T12:00:00Z" }),
        makeEvent({ id: 2, occurred_at: "2025-02-01T11:00:00Z" }),
      ],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-group-summary")).toBeTruthy();
    });

    // No expanded entries yet
    expect(document.querySelector(".log-group-expanded")).toBeNull();

    // Click the toggle button to expand
    const toggle = document.querySelector(".log-group-toggle") as HTMLElement;
    await fireEvent.click(toggle);

    expect(document.querySelector(".log-group-expanded")).toBeTruthy();
    const nested = document.querySelectorAll(
      ".log-group-expanded .log-entry-nested",
    );
    expect(nested.length).toBe(2);
  });

  it("does not group waterings with notes", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [
        makeEvent({ id: 2, occurred_at: "2025-02-01T11:00:00Z" }),
        makeEvent({
          id: 1,
          occurred_at: "2025-02-01T10:00:00Z",
          notes: "Very dry",
        }),
      ],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelectorAll(".log-entry").length).toBe(2);
    });
    // No group summary — both are individual
    expect(document.querySelector(".log-group-summary")).toBeNull();
  });
});

describe("care journal remains read-only", () => {
  it("does not expose editing for individual or expanded grouped entries", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [
        makeEvent({ id: 2, occurred_at: "2025-02-01T12:00:00Z" }),
        makeEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
      ],
      has_more: false,
    });
    render(Page);

    await vi.waitFor(() => {
      expect(document.querySelector(".log-group-summary")).toBeTruthy();
    });
    await fireEvent.click(
      document.querySelector(".log-group-toggle") as HTMLButtonElement,
    );

    expect(document.querySelector(".care-entry-form")).toBeNull();
    expect(document.querySelector('[aria-label="Edit log entry"]')).toBeNull();
    expect(mockUpdateCareEvent).not.toHaveBeenCalled();
  });
});

describe("care journal errors", () => {
  it("shows translated error for known ApiError codes", async () => {
    mockFetchAllCareEvents.mockRejectedValue(
      new ApiError(500, "INTERNAL_ERROR", "An internal error occurred"),
    );

    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByText("Something went wrong. Please try again."),
      ).toBeTruthy();
    });
  });
});

describe("care journal offline message", () => {
  it("shows offline message when fetch fails and offline", async () => {
    isOffline.set(true);
    mockFetchAllCareEvents.mockRejectedValue(new Error("fetch failed"));

    const view = render(Page);

    await vi.waitFor(() => {
      expect(
        view.getByText(
          "You're offline. Connect to the internet to view this page.",
        ),
      ).toBeTruthy();
    });
  });

  it("shows generic error when fetch fails and online", async () => {
    isOffline.set(false);
    mockFetchAllCareEvents.mockRejectedValue(new Error("server error"));

    const view = render(Page);

    await vi.waitFor(() => {
      expect(view.getByText("Failed to load care events")).toBeTruthy();
    });
  });
});
