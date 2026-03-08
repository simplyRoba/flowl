import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Plant } from "$lib/api";

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
});
