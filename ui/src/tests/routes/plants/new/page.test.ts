import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../../../../routes/plants/new/+page.svelte";
import type { Plant } from "$lib/api";

const mockCreatePlant = vi.fn();
const mockUpdatePlant = vi.fn();
const mockUploadPhoto = vi.fn();
const mockPushNotification = vi.fn();
const mockGoto = vi.fn();

vi.mock("$app/navigation", () => ({
  goto: (...args: unknown[]) => mockGoto(...args),
}));

vi.mock("$lib/stores/plants", async () => {
  const { writable } = await import("svelte/store");
  return {
    plants: writable<Plant[]>([]),
    currentPlant: writable<Plant | null>(null),
    plantsError: writable<string | null>(null),
    createPlant: (...args: unknown[]) => mockCreatePlant(...args),
    updatePlant: (...args: unknown[]) => mockUpdatePlant(...args),
    uploadPhoto: (...args: unknown[]) => mockUploadPhoto(...args),
  };
});

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

vi.mock("$lib/components/PlantForm.svelte", async () => {
  const component = await import("../../../stubs/PlantFormStub.svelte");
  return { default: component.default };
});

describe("new plant page", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  afterEach(() => {
    cleanup();
  });

  it("shows a toast when create fails", async () => {
    mockCreatePlant.mockResolvedValue(null);
    render(Page);

    await userEvent.setup().click(screen.getByText("Save without photo"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          variant: "error",
          message: "Failed to create plant",
        }),
      );
    });
    expect(mockGoto).not.toHaveBeenCalled();
  });

  it("keeps the user on the form and reuses the created plant when photo upload fails", async () => {
    mockCreatePlant.mockResolvedValue({ id: 7, name: "Fern" });
    mockUploadPhoto.mockResolvedValueOnce(null).mockResolvedValueOnce({
      id: 7,
      name: "Fern",
      photo_url: "/uploads/fern.jpg",
    });
    mockUpdatePlant.mockResolvedValue({ id: 7, name: "Fern" });
    render(Page);
    const user = userEvent.setup();

    await user.click(screen.getByText("Save with photo"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          variant: "error",
          message: "Failed to upload photo",
        }),
      );
    });
    expect(mockGoto).not.toHaveBeenCalled();

    await user.click(screen.getByText("Save with photo"));

    await waitFor(() => {
      expect(mockUpdatePlant).toHaveBeenCalledWith(7, { name: "Fern" });
      expect(mockGoto).toHaveBeenCalledWith("/plants/7");
    });
  });
});
