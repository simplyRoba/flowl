import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../../../../../routes/plants/[id]/edit/+page.svelte";
import type { Plant } from "$lib/api";

const mockUpdatePlant = vi.fn();
const mockUploadPhoto = vi.fn();
const mockDeletePhoto = vi.fn();
const mockPushNotification = vi.fn();
const mockGoto = vi.fn();

vi.mock("$app/navigation", () => ({
  goto: (...args: unknown[]) => mockGoto(...args),
}));

vi.mock("$lib/stores/plants", async () => {
  const { writable } = await import("svelte/store");
  const plantsError = writable<string | null>(null);
  return {
    plantsError,
    updatePlant: (...args: unknown[]) => mockUpdatePlant(...args),
    uploadPhoto: (...args: unknown[]) => mockUploadPhoto(...args),
    deletePhoto: (...args: unknown[]) => mockDeletePhoto(...args),
  };
});

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

vi.mock("$lib/components/PlantForm.svelte", async () => {
  const component = await import("../../../../stubs/PlantFormStub.svelte");
  return { default: component.default };
});

function plant(overrides: Partial<Plant> = {}): Plant {
  return {
    id: 1,
    name: "Fern",
    species: null,
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

function renderPage(overrides: Partial<Plant> = {}) {
  return render(Page, {
    props: {
      data: {
        plant: plant(overrides),
        notFound: false,
        loadErrorCode: null,
      },
    },
  });
}

describe("edit plant page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows a toast when update fails", async () => {
    mockUpdatePlant.mockResolvedValue(null);
    renderPage();
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("Save without photo")).toBeTruthy();
    });
    await user.click(screen.getByText("Save without photo"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Edit Plant",
          variant: "error",
          message: "Failed to update plant",
        }),
      );
    });
    expect(mockGoto).not.toHaveBeenCalled();
  });

  it("navigates after updating without a new photo", async () => {
    mockUpdatePlant.mockResolvedValue(plant());
    renderPage();
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("Save without photo")).toBeTruthy();
    });
    await user.click(screen.getByText("Save without photo"));

    await waitFor(() => {
      expect(mockUpdatePlant).toHaveBeenCalledWith(1, { name: "Fern" });
      expect(mockGoto).toHaveBeenCalledWith("/plants/1");
    });
    expect(mockUploadPhoto).not.toHaveBeenCalled();
  });

  it("waits for photo upload before navigating", async () => {
    let resolveUpload: (() => void) | undefined;
    mockUpdatePlant.mockResolvedValue(plant());
    mockUploadPhoto.mockImplementation(
      () =>
        new Promise<Plant>((resolve) => {
          const finishUpload = resolve as (value: Plant) => void;
          resolveUpload = () => finishUpload(plant());
        }),
    );
    renderPage();
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("Save with photo")).toBeTruthy();
    });
    await user.click(screen.getByText("Save with photo"));

    await waitFor(() => {
      expect(mockUploadPhoto).toHaveBeenCalledTimes(1);
    });
    expect(mockGoto).not.toHaveBeenCalled();

    resolveUpload?.();

    await waitFor(() => {
      expect(mockGoto).toHaveBeenCalledWith("/plants/1");
    });
  });

  it("keeps the user on the form when photo upload fails", async () => {
    mockUpdatePlant.mockResolvedValue(plant());
    mockUploadPhoto.mockResolvedValue(null);
    renderPage();
    const user = userEvent.setup();

    await waitFor(() => {
      expect(screen.getByText("Save with photo")).toBeTruthy();
    });
    await user.click(screen.getByText("Save with photo"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Media",
          variant: "error",
          message: "Failed to upload photo",
        }),
      );
    });
    expect(mockGoto).not.toHaveBeenCalled();
  });

  it("resets the form when route data switches to another plant", async () => {
    const view = renderPage({ id: 1, name: "Fern" });

    await waitFor(() => {
      expect(screen.getByTestId("draft-name").textContent).toBe("Fern");
    });

    view.rerender({
      data: {
        plant: plant({ id: 2, name: "Monstera" }),
        notFound: false,
        loadErrorCode: null,
      },
    });

    await waitFor(() => {
      expect(screen.getByTestId("draft-name").textContent).toBe("Monstera");
    });
  });
});
