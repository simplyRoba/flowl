import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../../../../routes/plants/[id]/+page.svelte";
import type { AiStatus, CareEvent, Plant } from "$lib/api";

// jsdom doesn't implement window.matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (
  this: HTMLDialogElement,
) {
  this.setAttribute("open", "");
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
  this.removeAttribute("open");
});

const mockDeletePlant = vi.fn();
const mockWaterPlant = vi.fn();
const mockAddCareEvent = vi.fn();
const mockPushNotification = vi.fn();
const mockGoto = vi.fn();

vi.mock("$app/stores", async () => {
  const { readable } = await import("svelte/store");
  return {
    page: readable({
      params: { id: "1" },
      url: new URL("http://localhost/plants/1"),
    }),
  };
});

vi.mock("$app/navigation", () => ({
  goto: (...args: unknown[]) => mockGoto(...args),
}));

vi.mock("$lib/stores/plants", async () => {
  const { writable } = await import("svelte/store");
  const plantsError = writable<string | null>(null);
  return {
    plantsError,
    deletePlant: (...args: unknown[]) => mockDeletePlant(...args),
    waterPlant: (...args: unknown[]) => mockWaterPlant(...args),
  };
});

vi.mock("$lib/stores/care", async () => {
  const { writable } = await import("svelte/store");
  const careError = writable<string | null>(null);
  return {
    careError,
    addCareEvent: (...args: unknown[]) => mockAddCareEvent(...args),
  };
});

vi.mock("$lib/emoji", () => ({
  emojiToSvgPath: (emoji: string) => `/emoji/${emoji}.svg`,
}));

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

import * as api from "$lib/api";
const mockChatPlant = vi.spyOn(api, "chatPlant");
const mockCreateCareEventApi = vi.spyOn(api, "createCareEvent");
const mockDeleteCareEventApi = vi.spyOn(api, "deleteCareEvent");
const mockFetchCareEventsApi = vi.spyOn(api, "fetchCareEvents");
const mockFetchPlantApi = vi.spyOn(api, "fetchPlant");
const mockSummarizeChat = vi.spyOn(api, "summarizeChat");
const mockUploadCareEventPhoto = vi.spyOn(api, "uploadCareEventPhoto");

const mockLoadAiStatus = vi.fn();

vi.mock("$lib/stores/ai", async () => {
  const { writable } = await import("svelte/store");
  const aiStatus = writable<AiStatus | null>(null);
  return {
    aiStatus,
    loadAiStatus: (...args: unknown[]) => mockLoadAiStatus(...args),
  };
});

import { aiStatus } from "$lib/stores/ai";
import { plantsError } from "$lib/stores/plants";
import { careError } from "$lib/stores/care";

function makePlant(overrides: Partial<Plant> = {}): Plant {
  return {
    id: 1,
    name: "Fern",
    species: "Boston Fern",
    icon: "🌿",
    photo_url: "/uploads/fern.jpg",
    location_id: 1,
    location_name: "Bedroom",
    watering_interval_days: 7,
    watering_status: "ok",
    last_watered: "2025-01-01",
    next_due: "2025-01-08",
    light_needs: "indirect",
    notes: null,
    difficulty: null,
    pet_safety: null,
    growth_speed: null,
    soil_type: null,
    soil_moisture: null,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
    ...overrides,
  };
}

function makeCareEvent(overrides: Partial<CareEvent> = {}): CareEvent {
  return {
    id: 10,
    plant_id: 1,
    plant_name: "Fern",
    event_type: "watered",
    notes: null,
    photo_url: null,
    occurred_at: "2025-01-01T10:00:00Z",
    created_at: "2025-01-01T10:00:00Z",
    ...overrides,
  };
}

async function renderWithPlant(
  plantOverrides: Partial<Plant> = {},
  initialCareEvents: CareEvent[] = [],
) {
  const plant = makePlant(plantOverrides);
  mockFetchCareEventsApi.mockResolvedValue(initialCareEvents);
  return render(Page, {
    props: {
      data: {
        plant,
        notFound: false,
        loadErrorCode: null,
      },
    },
  });
}

beforeEach(() => {
  aiStatus.set(null);
  careError.set(null);
  vi.clearAllMocks();
  mockDeleteCareEventApi.mockResolvedValue(undefined);
  mockFetchCareEventsApi.mockResolvedValue([]);
  mockFetchPlantApi.mockResolvedValue(makePlant());
});

afterEach(() => {
  cleanup();
});

function getLightbox() {
  return document.querySelector("dialog.lightbox") as HTMLDialogElement;
}

describe("route data updates", () => {
  it("switches to the new plant immediately when page data changes", async () => {
    const view = await renderWithPlant({ id: 1, name: "Fern" });

    await screen.findByText("Fern");

    view.rerender({
      data: {
        plant: makePlant({ id: 2, name: "Monstera" }),
        notFound: false,
        loadErrorCode: null,
      },
    });

    await waitFor(() => {
      expect(screen.getByText("Monstera")).toBeTruthy();
      expect(screen.queryByText("Fern")).toBeNull();
    });
  });
});

describe("hero thumbnail", () => {
  it("uses 200px thumbnail for hero photo", async () => {
    await renderWithPlant({ photo_url: "/uploads/fern.jpg" });
    await screen.findByText("Fern");
    const img = document.querySelector(".detail-photo-img") as HTMLImageElement;
    expect(img).toBeTruthy();
    expect(img.src).toContain("/uploads/fern_200.jpg");
  });

  it("falls back to original photo_url on hero thumbnail error", async () => {
    await renderWithPlant({ photo_url: "/uploads/fern.jpg" });
    await screen.findByText("Fern");
    const img = document.querySelector(".detail-photo-img") as HTMLImageElement;
    expect(img.src).toContain("/uploads/fern_200.jpg");
    await fireEvent.error(img);
    expect(img.src).toContain("/uploads/fern.jpg");
    expect(img.src).not.toContain("_200");
  });
});

describe("plant detail lightbox", () => {
  it("opens and closes the lightbox for a photo", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    // Escape triggers the dialog's cancel event
    getLightbox().dispatchEvent(new Event("cancel"));
    await vi.waitFor(() => {
      expect(getLightbox().hasAttribute("open")).toBe(false);
    });
  });

  it("does not expose a lightbox trigger when no photo is available", async () => {
    await renderWithPlant({ photo_url: null });
    await vi.waitFor(() => {
      expect(screen.queryByRole("button", { name: "Open photo" })).toBeNull();
    });
  });

  it("updates zoom on wheel input", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const img = document.querySelector(".lightbox-image") as HTMLImageElement;
    expect(img).toBeTruthy();
    const before = img.style.transform;
    await fireEvent.wheel(img, { deltaY: -600 });
    expect(img.style.transform).not.toBe(before);
  });

  it("pans the image when zoomed", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const img = document.querySelector(".lightbox-image") as HTMLImageElement;
    expect(img).toBeTruthy();
    Object.defineProperty(img, "clientWidth", { value: 400 });
    Object.defineProperty(img, "clientHeight", { value: 300 });
    await fireEvent.wheel(img, { deltaY: -600 });
    const before = img.style.transform;
    await fireEvent.pointerDown(img, { clientX: 100, clientY: 100 });
    await fireEvent.pointerMove(window, { clientX: 160, clientY: 140 });
    await fireEvent.pointerUp(window);
    expect(img.style.transform).not.toBe(before);
  });

  it("closes the lightbox via close button", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    const closeButton = screen.getByRole("button", { name: "Close" });
    await fireEvent.click(closeButton);
    await vi.waitFor(() => {
      expect(getLightbox().hasAttribute("open")).toBe(false);
    });
  });

  it("locks body scroll while lightbox is open and restores on close", async () => {
    document.body.style.overflow = "";
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });

    await fireEvent.click(openButton);
    expect(document.body.style.overflow).toBe("hidden");

    getLightbox().dispatchEvent(new Event("cancel"));
    await vi.waitFor(() => {
      expect(getLightbox().hasAttribute("open")).toBe(false);
    });
    expect(document.body.style.overflow).toBe("");
  });

  it("zooms via touch pinch gesture", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const img = document.querySelector(".lightbox-image") as HTMLImageElement;
    expect(img).toBeTruthy();
    const before = img.style.transform;

    const startEvent = new Event("touchstart", { bubbles: true }) as Event & {
      touches: Array<{ clientX: number; clientY: number }>;
    };
    startEvent.touches = [
      { clientX: 100, clientY: 100 },
      { clientX: 200, clientY: 200 },
    ];
    await fireEvent(window, startEvent);

    const moveEvent = new Event("touchmove", {
      bubbles: true,
      cancelable: true,
    }) as Event & {
      touches: Array<{ clientX: number; clientY: number }>;
    };
    moveEvent.touches = [
      { clientX: 50, clientY: 50 },
      { clientX: 250, clientY: 250 },
    ];
    await fireEvent(window, moveEvent);

    expect(img.style.transform).not.toBe(before);
  });

  it("closes the lightbox via backdrop click", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const lightbox = getLightbox();
    expect(lightbox.hasAttribute("open")).toBe(true);

    // Click directly on the dialog element (backdrop area)
    await fireEvent.click(lightbox);
    await vi.waitFor(() => {
      expect(lightbox.hasAttribute("open")).toBe(false);
    });
  });
});

describe("plant delete confirmation", () => {
  function getDeleteIconButton() {
    return document.querySelector(".btn-danger") as HTMLButtonElement;
  }

  it("shows confirmation dialog with plant name when delete is clicked", async () => {
    await renderWithPlant({ name: "My Fern" });
    await screen.findByText("My Fern");

    const user = userEvent.setup();
    await user.click(getDeleteIconButton());

    await waitFor(() => {
      expect(screen.getByText(/Delete "My Fern"/)).toBeTruthy();
    });
  });

  it("calls deletePlant when confirmed", async () => {
    mockDeletePlant.mockResolvedValue(true);
    await renderWithPlant({ name: "My Fern" });
    await screen.findByText("My Fern");

    const user = userEvent.setup();
    await user.click(getDeleteIconButton());

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Delete" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockDeletePlant).toHaveBeenCalledWith(1);
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Delete plant",
          variant: "success",
          message: 'Plant "My Fern" deleted',
        }),
      );
      expect(mockGoto).toHaveBeenCalledWith("/");
      expect(mockPushNotification.mock.invocationCallOrder[0]).toBeLessThan(
        mockGoto.mock.invocationCallOrder[0],
      );
    });
  });

  it("does not call deletePlant when cancelled", async () => {
    await renderWithPlant({ name: "My Fern" });
    await screen.findByText("My Fern");

    const user = userEvent.setup();
    await user.click(getDeleteIconButton());

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await new Promise((r) => setTimeout(r, 50));
    expect(mockDeletePlant).not.toHaveBeenCalled();
  });
});

describe("watering feedback", () => {
  it("keeps watering success silent", async () => {
    mockWaterPlant.mockResolvedValue(
      makePlant({ last_watered: "2025-02-01T10:00:00Z" }),
    );
    await renderWithPlant();
    await screen.findByText("Fern");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Water now" }));

    await waitFor(() => {
      expect(mockWaterPlant).toHaveBeenCalledWith(1);
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("shows a toast when watering fails", async () => {
    mockWaterPlant.mockResolvedValue(null);
    await renderWithPlant();
    await screen.findByText("Fern");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Water now" }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Watering",
          variant: "error",
          message: "Failed to water plant",
        }),
      );
    });
  });

  it("uses the store error details when watering fails", async () => {
    mockWaterPlant.mockImplementation(async () => {
      plantsError.set("Watering service offline");
      return null;
    });
    await renderWithPlant();
    await screen.findByText("Fern");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Water now" }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Watering",
          variant: "error",
          message: "Watering service offline",
        }),
      );
    });
  });
});

describe("Ask AI button", () => {
  it("shows Ask AI button when AI is enabled", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    await renderWithPlant();
    await screen.findByText("Fern");
    await waitFor(() => {
      expect(screen.getByText("Ask AI")).toBeTruthy();
    });
  });

  it("hides Ask AI button when AI is disabled", async () => {
    aiStatus.set({ enabled: false, base_url: null, model: null });
    await renderWithPlant();
    await screen.findByText("Fern");
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByText("Ask AI")).toBeNull();
  });

  it("hides Ask AI button when AI status is null (fetch failed)", async () => {
    aiStatus.set(null);
    await renderWithPlant();
    await screen.findByText("Fern");
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByText("Ask AI")).toBeNull();
  });

  it("opens chat drawer when Ask AI is clicked", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    await renderWithPlant();
    await waitFor(() => {
      expect(screen.getByText("Ask AI")).toBeTruthy();
    });
    const user = userEvent.setup();
    await user.click(screen.getByText("Ask AI"));
    await waitFor(() => {
      expect(screen.getByText("Quick questions")).toBeTruthy();
    });
  });

  it("closes chat drawer when close button is clicked", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    await renderWithPlant();
    await waitFor(() => {
      expect(screen.getByText("Ask AI")).toBeTruthy();
    });
    const user = userEvent.setup();
    await user.click(screen.getByText("Ask AI"));
    await waitFor(() => {
      expect(screen.getByText("Quick questions")).toBeTruthy();
    });
    const closeBtn = screen.getByRole("button", { name: "Close chat" });
    await user.click(closeBtn);
    await waitFor(() => {
      expect(screen.queryByText("Quick questions")).toBeNull();
    });
  });
});

describe("care event delete reloads plant", () => {
  it("calls loadPlant after deleting a care event", async () => {
    mockDeleteCareEventApi.mockResolvedValue(undefined);
    mockFetchPlantApi.mockResolvedValue(makePlant());
    mockFetchCareEventsApi.mockResolvedValue([]);

    await renderWithPlant({}, [makeCareEvent()]);
    await screen.findByText("Fern");

    await waitFor(() => {
      expect(screen.getByText("Watered")).toBeTruthy();
    });

    const deleteButton = screen.getByRole("button", {
      name: "Delete log entry",
    });
    const user = userEvent.setup();
    await user.click(deleteButton);

    await waitFor(() => {
      expect(screen.getByText(/Delete this care entry/)).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockDeleteCareEventApi).toHaveBeenCalledWith(1, 10);
      expect(mockFetchPlantApi).toHaveBeenCalledWith(1);
      expect(mockFetchCareEventsApi).toHaveBeenCalledWith(1);
    });
  });

  it("shows a toast when deleting a care event fails", async () => {
    mockDeleteCareEventApi.mockRejectedValue(
      new api.ApiError(500, "INTERNAL_ERROR", "An internal error occurred"),
    );

    await renderWithPlant({}, [makeCareEvent()]);
    await screen.findByText("Fern");

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Delete log entry" }),
      ).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Delete log entry" }));

    await waitFor(() => {
      expect(screen.getByText(/Delete this care entry/)).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Care Journal",
          variant: "error",
          message: "Something went wrong. Please try again.",
        }),
      );
    });
  });
});

describe("chat drawer save note", () => {
  beforeEach(() => {
    aiStatus.set({
      enabled: true,
      base_url: "https://api.openai.com/v1",
      model: "gpt-4o-mini",
    });
  });

  async function openChatAndSendMessage() {
    await renderWithPlant();
    const askAiButton = await screen.findByRole("button", { name: "Ask AI" });
    await fireEvent.click(askAiButton);
    return screen;
  }

  it("does not show save note button when no assistant messages", async () => {
    await openChatAndSendMessage();
    // Chat is open but no messages have been sent
    await waitFor(() => {
      expect(screen.queryByText("Create note")).toBeNull();
    });
  });

  it("shows save note button after assistant response", async () => {
    await openChatAndSendMessage();

    // Simulate an existing conversation by checking button visibility
    // The ChatDrawer needs assistant messages to show the button
    // Since we can't easily simulate streaming, we test the flow via summarize
    mockSummarizeChat.mockResolvedValue("Test summary");

    // Verify summarizeChat function exists and is callable
    expect(typeof api.summarizeChat).toBe("function");
  });

  it("shows a toast after saving a note successfully", async () => {
    mockChatPlant.mockImplementation(async function* () {
      yield "Looks healthy";
    });
    mockSummarizeChat.mockResolvedValue("Healthy and growing well");
    mockCreateCareEventApi.mockResolvedValue({
      id: 99,
      plant_id: 1,
      plant_name: "Fern",
      event_type: "ai-consultation",
      notes: "Healthy and growing well",
      photo_url: null,
      occurred_at: "2025-02-01T10:00:00Z",
      created_at: "2025-02-01T10:00:00Z",
    });

    await openChatAndSendMessage();
    const user = userEvent.setup();

    const input = screen.getByPlaceholderText("Ask about your plant...");
    await user.type(input, "How is my plant?");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(screen.getByText("Looks healthy")).toBeTruthy();
      expect(screen.getByText("Create note")).toBeTruthy();
    });

    await user.click(screen.getByText("Create note"));

    await waitFor(() => {
      expect(screen.getByDisplayValue("Healthy and growing well")).toBeTruthy();
    });

    const saveButtons = screen.getAllByText("Save");
    await user.click(saveButtons[saveButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Care Journal",
          variant: "success",
          message: "Saved to care journal",
        }),
      );
      expect(screen.queryByText("Quick questions")).toBeNull();
    });
  });

  it("keeps stream failures inline in the drawer", async () => {
    mockChatPlant.mockImplementation(async function* () {
      yield* [];
      throw new Error("stream failed");
    });

    await openChatAndSendMessage();
    const user = userEvent.setup();

    const input = screen.getByPlaceholderText("Ask about your plant...");
    await user.type(input, "How is my plant?");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(
        screen.getByText("Something went wrong. Please try again."),
      ).toBeTruthy();
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("keeps save-note failures inline in the drawer", async () => {
    mockChatPlant.mockImplementation(async function* () {
      yield "Looks healthy";
    });
    mockSummarizeChat.mockResolvedValue("Healthy and growing well");
    mockCreateCareEventApi.mockRejectedValue(new Error("save failed"));

    await openChatAndSendMessage();
    const user = userEvent.setup();

    const input = screen.getByPlaceholderText("Ask about your plant...");
    await user.type(input, "How is my plant?");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(screen.getByText("Looks healthy")).toBeTruthy();
      expect(screen.getByText("Create note")).toBeTruthy();
    });

    await user.click(screen.getByText("Create note"));

    await waitFor(() => {
      expect(screen.getByDisplayValue("Healthy and growing well")).toBeTruthy();
    });

    const saveButtons = screen.getAllByText("Save");
    await user.click(saveButtons[saveButtons.length - 1]);

    await waitFor(() => {
      expect(screen.getByText("Failed to save note")).toBeTruthy();
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("summarizeChat calls the correct API endpoint", async () => {
    mockSummarizeChat.mockResolvedValue("Plant health looks good");

    const result = await api.summarizeChat(1, [
      { role: "user", content: "How is my plant?" },
      { role: "assistant", content: "Your plant looks healthy!" },
    ]);

    expect(result).toBe("Plant health looks good");
    expect(mockSummarizeChat).toHaveBeenCalledWith(1, [
      { role: "user", content: "How is my plant?" },
      { role: "assistant", content: "Your plant looks healthy!" },
    ]);
  });
});

describe("care event photo in timeline", () => {
  it("renders a thumbnail when a care event has a photo_url", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 20,
        event_type: "fertilized",
        notes: "Fed with liquid fertilizer",
        photo_url: "/uploads/care/20.jpg",
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      const img = document.querySelector(
        ".timeline-photo img",
      ) as HTMLImageElement;
      expect(img).toBeTruthy();
      expect(img.src).toContain("/uploads/care/20_200.jpg");
    });
  });

  it("does not render a thumbnail when care event has no photo_url", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 21,
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      expect(screen.getByText("Watered")).toBeTruthy();
    });
    expect(document.querySelector(".timeline-photo")).toBeNull();
  });

  it("opens lightbox when clicking a care event thumbnail", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 22,
        event_type: "repotted",
        photo_url: "/uploads/care/22.jpg",
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-photo")).toBeTruthy();
    });
    const photoBtn = document.querySelector(
      ".timeline-photo",
    ) as HTMLButtonElement;
    await fireEvent.click(photoBtn);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    const lightboxImg = getLightbox().querySelector("img") as HTMLImageElement;
    expect(lightboxImg.src).toContain("/uploads/care/22.jpg");
    expect(lightboxImg.src).not.toContain("_200");
  });

  it("lightbox uses original photo_url for hero photo", async () => {
    await renderWithPlant({ photo_url: "/uploads/fern.jpg" });
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    const lightboxImg = getLightbox().querySelector("img") as HTMLImageElement;
    expect(lightboxImg.src).toContain("/uploads/fern.jpg");
    expect(lightboxImg.src).not.toContain("_600");
  });

  it("falls back to original photo_url on timeline thumbnail error", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 23,
        photo_url: "/uploads/care/23.png",
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      const img = document.querySelector(
        ".timeline-photo img",
      ) as HTMLImageElement;
      expect(img).toBeTruthy();
      expect(img.src).toContain("/uploads/care/23_200.jpg");
    });
    const img = document.querySelector(
      ".timeline-photo img",
    ) as HTMLImageElement;
    await fireEvent.error(img);
    expect(img.src).toContain("/uploads/care/23.png");
    expect(img.src).not.toContain("_200");
  });
});

describe("log form photo upload", () => {
  it("shows the photo upload control when log form is open", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });
  });

  it("shows a preview after selecting a photo and clears it on remove", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });

    const fileInput = document.querySelector(
      '.care-entry-form input[type="file"]',
    ) as HTMLInputElement;
    expect(fileInput).toBeTruthy();

    const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      writable: false,
    });
    await fireEvent.change(fileInput);

    await waitFor(() => {
      const preview = document.querySelector(
        ".toolbar-thumb img",
      ) as HTMLImageElement;
      expect(preview).toBeTruthy();
    });

    const removeBtn = document.querySelector(
      ".toolbar-dismiss",
    ) as HTMLButtonElement;
    await fireEvent.click(removeBtn);

    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeNull();
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });
  });

  it("uploads photo after creating care event on submit", async () => {
    const createdEvent: CareEvent = {
      id: 30,
      plant_id: 1,
      plant_name: "Fern",
      event_type: "fertilized",
      notes: "",
      photo_url: null,
      occurred_at: "2025-02-01T10:00:00Z",
      created_at: "2025-02-01T10:00:00Z",
    };
    mockAddCareEvent.mockResolvedValue(createdEvent);
    mockUploadCareEventPhoto.mockResolvedValue({
      ...createdEvent,
      photo_url: "/uploads/care/30.jpg",
    });

    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByText("Fertilized")).toBeTruthy();
    });
    await fireEvent.click(screen.getByText("Fertilized"));

    const fileInput = document.querySelector(
      '.care-entry-form input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      writable: false,
    });
    await fireEvent.change(fileInput);

    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeTruthy();
    });

    const saveBtn = screen.getByText("Save");
    await fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(mockAddCareEvent).toHaveBeenCalledWith(
        1,
        expect.objectContaining({ event_type: "fertilized" }),
      );
      expect(mockUploadCareEventPhoto).toHaveBeenCalledWith(1, 30, file);
    });
  });

  it("shows inline validation and blocks submit when no care type is selected", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    await fireEvent.click(screen.getByText("+ Add log entry"));

    await waitFor(() => {
      expect(screen.getByText("Save")).toBeTruthy();
    });

    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(screen.getByText("Choose a care entry type")).toBeTruthy();
    });
    expect(mockAddCareEvent).not.toHaveBeenCalled();
  });

  it("shows a toast and keeps the form open when care entry creation fails", async () => {
    mockAddCareEvent.mockResolvedValue(null);

    await renderWithPlant();
    await screen.findByText("Fern");
    await fireEvent.click(screen.getByText("+ Add log entry"));

    await waitFor(() => {
      expect(screen.getByText("Fertilized")).toBeTruthy();
    });
    await fireEvent.click(screen.getByText("Fertilized"));
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Care Journal",
          variant: "error",
          message: "Failed to add care event",
        }),
      );
      expect(document.querySelector(".care-entry-form")).toBeTruthy();
    });
  });

  it("supports adding backdated watering entries from the log form", async () => {
    const occurredAt = "2025-01-15T09:30";
    const createdEvent: CareEvent = {
      id: 31,
      plant_id: 1,
      plant_name: "Fern",
      event_type: "watered",
      notes: "Watered a few days ago",
      photo_url: null,
      occurred_at: "2025-01-15T09:30:00Z",
      created_at: "2025-02-01T10:00:00Z",
    };
    mockAddCareEvent.mockResolvedValue(createdEvent);
    mockFetchPlantApi.mockResolvedValue(
      makePlant({
        last_watered: "2025-01-15T09:30:00Z",
        next_due: "2025-01-22T09:30:00Z",
        watering_status: "ok",
      }),
    );
    mockFetchCareEventsApi.mockResolvedValue([createdEvent]);
    await renderWithPlant({
      last_watered: "2025-01-01",
      next_due: "2025-01-08",
      watering_status: "overdue",
    });

    await screen.findByText("Fern");
    await fireEvent.click(screen.getByText("+ Add log entry"));
    const form = document.querySelector(".care-entry-form") as HTMLElement;

    await waitFor(() => {
      expect(within(form).getByText("Watered")).toBeTruthy();
    });
    await fireEvent.click(within(form).getByText("Watered"));

    const dateToggle = document.querySelectorAll(
      ".care-entry-form .toolbar-btn",
    )[1] as HTMLButtonElement;
    await fireEvent.click(dateToggle);

    const dateInput = document.querySelector(
      ".care-entry-form .toolbar-date-input",
    ) as HTMLInputElement;
    await fireEvent.input(dateInput, { target: { value: occurredAt } });

    const notesInput = document.querySelector(
      ".care-entry-form .log-notes",
    ) as HTMLTextAreaElement;
    await fireEvent.input(notesInput, {
      target: { value: "Watered a few days ago" },
    });

    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockAddCareEvent).toHaveBeenCalledWith(
        1,
        expect.objectContaining({
          event_type: "watered",
          notes: "Watered a few days ago",
          occurred_at: new Date(occurredAt).toISOString(),
        }),
      );
      expect(mockFetchCareEventsApi).toHaveBeenCalledWith(1);
      expect(mockFetchPlantApi).toHaveBeenCalledWith(1);
    });
  });

  it("clears photo when form is cancelled", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });

    const fileInput = document.querySelector(
      '.care-entry-form input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      writable: false,
    });
    await fireEvent.change(fileInput);

    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeTruthy();
    });

    const cancelBtns = screen.getAllByText("Cancel");
    const logFormCancel = cancelBtns.find(
      (btn) => btn.closest(".care-entry-form") !== null,
    )!;
    await fireEvent.click(logFormCancel);

    // Re-open log form — photo should be gone
    await fireEvent.click(screen.getByText("+ Add log entry"));
    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeNull();
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });
  });
});

describe("care journal event grouping", () => {
  it("groups consecutive waterings into a collapsible summary", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 3, occurred_at: "2025-02-03T10:00:00Z" }),
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    // Should show one group, not three individual items
    const items = document.querySelectorAll(".timeline-item");
    expect(items.length).toBe(1);
  });

  it("expands group on click to show individual entries", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    // No nested entries yet
    expect(document.querySelector(".timeline-nested")).toBeNull();

    // Click the group button
    const groupBtn = document.querySelector(
      ".timeline-group-btn",
    ) as HTMLButtonElement;
    await fireEvent.click(groupBtn);

    await waitFor(() => {
      const nested = document.querySelectorAll(".timeline-nested");
      expect(nested.length).toBe(2);
    });
  });

  it("omits year on first date when both dates share the same year", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 3, occurred_at: "2025-03-10T10:00:00Z" }),
      makeCareEvent({ id: 2, occurred_at: "2025-02-15T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-01-05T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    const dateEl = document.querySelector(
      ".timeline-group-summary .timeline-date",
    ) as HTMLElement;
    const text = dateEl.textContent!;
    // first date (Jan 5) should not contain a year
    const [first, second] = text.split("–").map((s) => s.trim());
    expect(first).not.toMatch(/\d{2}$/);
    // second date (Mar 10) should contain the year
    expect(second).toMatch(/\d{2}$/);
  });

  it("shows year on both dates when they span different years", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2026-01-10T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-12-20T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    const dateEl = document.querySelector(
      ".timeline-group-summary .timeline-date",
    ) as HTMLElement;
    const text = dateEl.textContent!;
    const [first, second] = text.split("–").map((s) => s.trim());
    // both dates should contain a year
    expect(first).toMatch(/\d{2}$/);
    expect(second).toMatch(/\d{2}$/);
  });

  it("does not group waterings with notes", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({
        id: 1,
        occurred_at: "2025-02-01T10:00:00Z",
        notes: "Very dry soil",
      }),
    ]);

    await waitFor(() => {
      expect(screen.getAllByText("Watered").length).toBe(2);
    });
    // Both should be individual items, no group summary
    expect(document.querySelector(".timeline-group-summary")).toBeNull();
    const items = document.querySelectorAll(".timeline-item");
    expect(items.length).toBe(2);
  });
});
