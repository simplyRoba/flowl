import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Plant } from "$lib/api";
import { isOffline } from "$lib/stores/network";

vi.mock("$lib/api", async () => {
  const actual = await vi.importActual<typeof import("$lib/api")>("$lib/api");
  return {
    ...actual,
    chatPlant: vi.fn(),
    summarizeChat: vi.fn(),
    createCareEvent: vi.fn(),
    uploadCareEventPhoto: vi.fn(),
  };
});

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: vi.fn(),
}));

import ChatDrawer from "./ChatDrawer.svelte";
import * as api from "$lib/api";

function makePlant(overrides: Partial<Plant> = {}): Plant {
  return {
    id: 1,
    name: "Fern",
    species: "Nephrolepis exaltata",
    icon: "🌿",
    photo_url: null,
    location_id: null,
    location_name: null,
    watering_interval_days: 7,
    watering_status: "ok",
    last_watered: null,
    next_due: null,
    light_needs: "indirect",
    difficulty: null,
    pet_safety: null,
    growth_speed: null,
    soil_type: null,
    soil_moisture: null,
    notes: null,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
    ...overrides,
  };
}

beforeEach(() => {
  vi.clearAllMocks();
  isOffline.set(false);
  HTMLDialogElement.prototype.showModal = vi.fn(function (
    this: HTMLDialogElement,
  ) {
    this.setAttribute("open", "");
  });
  HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
    this.removeAttribute("open");
  });
  Object.defineProperty(window, "matchMedia", {
    configurable: true,
    writable: true,
    value: vi.fn().mockImplementation((query: string) => ({
      matches: false,
      media: query,
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: vi.fn(),
      dispatchEvent: vi.fn(),
    })),
  });
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("ChatDrawer", () => {
  const defaultProps = {
    plant: makePlant(),
    open: true,
    onclose: vi.fn(),
  };

  it("renders the chat input", () => {
    render(ChatDrawer, { props: defaultProps });
    expect(screen.getByPlaceholderText(/Ask about/)).toBeTruthy();
  });

  it("shows suggestion chips when no messages exist", () => {
    render(ChatDrawer, { props: defaultProps });
    expect(screen.getByText("Health check")).toBeTruthy();
    expect(screen.getByText("Watering advice")).toBeTruthy();
    expect(screen.getByText("Light requirements")).toBeTruthy();
  });

  it("releases a staged photo preview when closed", async () => {
    const revokeObjectURL = vi.fn();
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: vi.fn(() => "blob:attached"),
      revokeObjectURL,
    });
    const view = render(ChatDrawer, { props: defaultProps });
    const fileInput = view.container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["photo"], "plant.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", { value: [file] });

    await fireEvent.change(fileInput);
    expect(view.container.querySelector(".photo-preview-strip")).not.toBeNull();

    await fireEvent.click(screen.getByRole("button", { name: "Close chat" }));

    expect(revokeObjectURL).toHaveBeenCalledWith("blob:attached");
    expect(view.container.querySelector(".photo-preview-strip")).toBeNull();
    expect(defaultProps.onclose).toHaveBeenCalled();
  });

  it("releases the save-note photo preview when removed", async () => {
    const revokeObjectURL = vi.fn();
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: vi
        .fn()
        .mockReturnValueOnce("blob:attached")
        .mockReturnValueOnce("blob:last-user-photo"),
      revokeObjectURL,
    });
    vi.mocked(api.chatPlant).mockImplementation(async function* () {
      yield "Advice";
    });
    vi.mocked(api.summarizeChat).mockResolvedValue("Summary");
    const view = render(ChatDrawer, { props: defaultProps });
    const fileInput = view.container.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["photo"], "plant.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", { value: [file] });

    await fireEvent.change(fileInput);
    await fireEvent.input(screen.getByPlaceholderText(/Ask about/), {
      target: { value: "What is wrong?" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Send" }));
    await waitFor(() => expect(screen.getByText("Advice")).toBeTruthy());
    await fireEvent.click(screen.getByText("Create note"));
    await waitFor(() =>
      expect(
        view.container.querySelector(".summary-photo-preview"),
      ).not.toBeNull(),
    );

    await fireEvent.click(screen.getByRole("button", { name: "Remove photo" }));

    expect(revokeObjectURL).toHaveBeenCalledWith("blob:last-user-photo");
    expect(view.container.querySelector(".summary-photo-preview")).toBeNull();
  });

  it("shows 'when to repot' chip when species is known", () => {
    render(ChatDrawer, { props: defaultProps });
    expect(screen.getByText("When to repot?")).toBeTruthy();
  });

  it("shows 'help identify' chip when species is null", () => {
    render(ChatDrawer, {
      props: { ...defaultProps, plant: makePlant({ species: null }) },
    });
    expect(screen.getByText("Help identify")).toBeTruthy();
  });

  it("shows 'why overdue' chip when plant is overdue", () => {
    render(ChatDrawer, {
      props: {
        ...defaultProps,
        plant: makePlant({ watering_status: "overdue" }),
      },
    });
    expect(screen.getByText("Why is it overdue?")).toBeTruthy();
  });

  it("ignores dragged photos while offline", async () => {
    isOffline.set(true);
    const view = render(ChatDrawer, { props: defaultProps });
    const messageArea = view.container.querySelector(".chat-messages");
    const file = new File(["photo"], "plant.jpg", { type: "image/jpeg" });

    expect(messageArea).not.toBeNull();
    await fireEvent.dragEnter(messageArea!, {
      dataTransfer: { files: [file] },
    });
    expect(messageArea!.classList.contains("dragging-file")).toBe(false);

    await fireEvent.drop(messageArea!, {
      dataTransfer: { files: [file] },
    });
    expect(view.container.querySelector(".photo-preview-strip")).toBeNull();
  });
});
