import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Plant } from "$lib/api";

const mockIdentifyPlant = vi.fn();
const mockLoadAiStatus = vi.fn();
const mockLoadLocations = vi.fn();
const mockFetch = vi.fn();

vi.mock("$lib/api", async () => {
  const actual = await vi.importActual<typeof import("$lib/api")>("$lib/api");
  return {
    ...actual,
    identifyPlant: (...args: unknown[]) => mockIdentifyPlant(...args),
  };
});

vi.mock("$lib/stores/ai", async () => {
  const { writable } = await import("svelte/store");
  const aiStatus = writable({ enabled: true, base_url: null, model: null });
  return {
    aiStatus,
    loadAiStatus: (...args: unknown[]) => mockLoadAiStatus(...args),
  };
});

vi.mock("$lib/stores/locations", async () => {
  const { writable } = await import("svelte/store");
  return {
    locations: writable([]),
    loadLocations: (...args: unknown[]) => mockLoadLocations(...args),
    createLocation: vi.fn(),
  };
});

import PlantForm from "./PlantForm.svelte";
import { aiStatus } from "$lib/stores/ai";

function makePlant(overrides: Partial<Plant> = {}): Plant {
  return {
    id: 1,
    name: "Fern",
    species: null,
    icon: "🌿",
    photo_url: "/uploads/fern.jpg",
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

describe("PlantForm identify feedback", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.stubGlobal("fetch", mockFetch);
    aiStatus.set({ enabled: true, base_url: null, model: null });
    mockLoadAiStatus.mockResolvedValue(undefined);
    mockLoadLocations.mockResolvedValue(undefined);
    mockFetch.mockResolvedValue({
      blob: async () => new Blob(["img"], { type: "image/jpeg" }),
    });
  });

  afterEach(() => {
    cleanup();
    vi.unstubAllGlobals();
  });

  it("keeps identify failures inline", async () => {
    mockIdentifyPlant.mockRejectedValue(new Error("AI unavailable"));
    render(PlantForm, {
      props: {
        initial: makePlant(),
        onsave: vi.fn(),
        showFooterActions: false,
      },
    });

    const user = userEvent.setup();
    await user.click(
      await screen.findByRole("button", { name: "Identify Plant" }),
    );

    await waitFor(() => {
      expect(screen.getByText("AI unavailable")).toBeTruthy();
      expect(screen.getByRole("button", { name: "Retry" })).toBeTruthy();
    });
  });
});
