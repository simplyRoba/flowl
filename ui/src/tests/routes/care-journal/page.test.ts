import { cleanup, fireEvent, render } from "@testing-library/svelte";
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
const mockGoto = vi.fn();

vi.mock("$lib/api", async () => {
  const actual = await vi.importActual<typeof import("$lib/api")>("$lib/api");
  return {
    ...actual,
    fetchAllCareEvents: (...args: unknown[]) => mockFetchAllCareEvents(...args),
  };
});

let mockUrl = new URL("http://localhost/care-journal");

vi.mock("$app/state", () => ({
  page: {
    get url() {
      return mockUrl;
    },
  },
}));

vi.mock("$app/navigation", () => ({
  goto: (...args: unknown[]) => {
    mockUrl = new URL(String(args[0]), "http://localhost");
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

beforeEach(() => {
  vi.clearAllMocks();
  isOffline.set(false);
  mockUrl = new URL("http://localhost/care-journal");
  mockFetchAllCareEvents.mockResolvedValue({ events: [], has_more: false });
});

afterEach(() => {
  cleanup();
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
  it("loads all events with high limit when URL has no type param", async () => {
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    expect(mockFetchAllCareEvents).toHaveBeenCalledWith(
      10000,
      undefined,
      undefined,
    );
  });

  it("loads with type filter when URL has type params", async () => {
    mockUrl = new URL(
      "http://localhost/care-journal?type=watered&type=fertilized",
    );
    render(Page);

    await vi.waitFor(() => {
      expect(mockFetchAllCareEvents).toHaveBeenCalled();
    });
    expect(mockFetchAllCareEvents).toHaveBeenCalledWith(
      10000,
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
    mockUrl = new URL("http://localhost/care-journal?type=watered&type=pruned");
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
    mockUrl = new URL("http://localhost/care-journal?type=watered");
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

describe("care journal event grouping", () => {
  it("groups consecutive waterings into a collapsible summary", async () => {
    mockFetchAllCareEvents.mockResolvedValue({
      events: [
        makeEvent({ id: 3, occurred_at: "2025-02-01T12:00:00Z" }),
        makeEvent({ id: 2, occurred_at: "2025-02-01T11:00:00Z" }),
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
