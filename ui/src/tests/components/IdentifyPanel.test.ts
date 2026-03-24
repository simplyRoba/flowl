import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import IdentifyPanel from "../../lib/components/IdentifyPanel.svelte";
import { ApiError } from "$lib/api";
import type { IdentifyResult } from "$lib/api";

const mockIdentifyPlant = vi.fn();

vi.mock("$lib/api", async () => {
  const actual = await vi.importActual<typeof import("$lib/api")>("$lib/api");
  return {
    ...actual,
    identifyPlant: (...args: unknown[]) => mockIdentifyPlant(...args),
  };
});

function makeSuggestion(
  overrides: Partial<IdentifyResult> = {},
): IdentifyResult {
  return {
    common_name: "Swiss Cheese Plant",
    scientific_name: "Monstera deliciosa",
    confidence: 0.92,
    summary: "A popular tropical houseplant.",
    care_profile: {
      watering_interval_days: 7,
      light_needs: "indirect",
      difficulty: "easy",
      pet_safety: "toxic",
      growth_speed: "moderate",
      soil_type: "standard",
      soil_moisture: "moderate",
    },
    ...overrides,
  };
}

const mockOnapply = vi.fn().mockReturnValue(5);
const mockOnundo = vi.fn();

function renderPanel(
  overrides: Partial<{
    photoFile: File | null;
    photoPreview: string | null;
    existingPhotoUrl: string | null;
  }> = {},
) {
  const photoFile =
    "photoFile" in overrides
      ? overrides.photoFile!
      : new File(["fake"], "plant.jpg", { type: "image/jpeg" });
  const photoPreview =
    "photoPreview" in overrides ? overrides.photoPreview! : "blob:preview";
  return render(IdentifyPanel, {
    props: {
      photoFile,
      photoPreview,
      existingPhotoUrl: overrides.existingPhotoUrl ?? null,
      onapply: mockOnapply,
      onundo: mockOnundo,
    },
  });
}

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  cleanup();
});

describe("IdentifyPanel", () => {
  describe("idle state", () => {
    it("shows the identify button", () => {
      renderPanel();
      expect(screen.getByText("Identify Plant")).toBeTruthy();
    });

    it("shows the main photo preview", () => {
      renderPanel();
      const img = document.querySelector(
        ".extra-photo-main img",
      ) as HTMLImageElement;
      expect(img).toBeTruthy();
      expect(img.src).toContain("blob:preview");
    });

    it("shows extra photo upload slots", () => {
      renderPanel();
      expect(screen.getByText("Close-up")).toBeTruthy();
      expect(screen.getByText("Stem / pot")).toBeTruthy();
    });
  });

  describe("loading state", () => {
    it("shows loading indicator and shimmer lines when identifying", async () => {
      mockIdentifyPlant.mockReturnValue(new Promise(() => {})); // never resolves
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      expect(screen.getByText("Identifying...")).toBeTruthy();
      expect(document.querySelector(".shimmer-lines")).toBeTruthy();
      expect(document.querySelectorAll(".shimmer-lines .shimmer").length).toBe(
        3,
      );
    });
  });

  describe("result state", () => {
    it("shows suggestion with scientific name and confidence", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(screen.getByText("Monstera deliciosa")).toBeTruthy();
        expect(screen.getByText("92%")).toBeTruthy();
      });
    });

    it("shows common name and summary", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(screen.getByText('"Swiss Cheese Plant"')).toBeTruthy();
        expect(screen.getByText("A popular tropical houseplant.")).toBeTruthy();
      });
    });

    it("shows will-fill chips for care profile fields", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(document.querySelectorAll(".fill-chip").length).toBeGreaterThan(
          0,
        );
      });
    });

    it("shows apply and dismiss buttons", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(screen.getByText("Apply to form")).toBeTruthy();
        expect(screen.getByText("Dismiss")).toBeTruthy();
      });
    });

    it("calls onapply and shows applied banner", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));
      await waitFor(() => {
        expect(screen.getByText("Apply to form")).toBeTruthy();
      });

      await user.click(screen.getByText("Apply to form"));

      expect(mockOnapply).toHaveBeenCalledWith(
        expect.objectContaining({ scientific_name: "Monstera deliciosa" }),
      );
      expect(screen.getByText(/5 fields updated/)).toBeTruthy();
    });

    it("dismiss returns to idle", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));
      await waitFor(() => {
        expect(screen.getByText("Dismiss")).toBeTruthy();
      });

      await user.click(screen.getByText("Dismiss"));

      expect(screen.getByText("Identify Plant")).toBeTruthy();
    });
  });

  describe("applied state", () => {
    it("shows undo button and calls onundo", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));
      await waitFor(() => {
        expect(screen.getByText("Apply to form")).toBeTruthy();
      });
      await user.click(screen.getByText("Apply to form"));

      expect(screen.getByText("Undo")).toBeTruthy();

      await user.click(screen.getByText("Undo"));

      expect(mockOnundo).toHaveBeenCalled();
      expect(screen.getByText("Identify Plant")).toBeTruthy();
    });
  });

  describe("error state", () => {
    it("shows error message and retry button", async () => {
      mockIdentifyPlant.mockRejectedValue(new Error("Network error"));
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(screen.getByText("Retry")).toBeTruthy();
      });
    });

    it("retry re-triggers identification", async () => {
      mockIdentifyPlant.mockRejectedValueOnce(new Error("fail"));
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));
      await waitFor(() => {
        expect(screen.getByText("Retry")).toBeTruthy();
      });

      mockIdentifyPlant.mockResolvedValue({
        suggestions: [makeSuggestion()],
      });

      await user.click(screen.getByText("Retry"));

      await waitFor(() => {
        expect(screen.getByText("Monstera deliciosa")).toBeTruthy();
      });
      expect(mockIdentifyPlant).toHaveBeenCalledTimes(2);
    });
  });

  describe("multiple suggestions", () => {
    it("shows navigation when multiple suggestions exist", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [
          makeSuggestion({ scientific_name: "Monstera deliciosa" }),
          makeSuggestion({ scientific_name: "Philodendron bipinnatifidum" }),
        ],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(screen.getByText("Monstera deliciosa")).toBeTruthy();
        expect(screen.getByText("1 / 2")).toBeTruthy();
      });
    });

    it("navigates between suggestions with next/prev buttons", async () => {
      mockIdentifyPlant.mockResolvedValue({
        suggestions: [
          makeSuggestion({ scientific_name: "Monstera deliciosa" }),
          makeSuggestion({ scientific_name: "Philodendron bipinnatifidum" }),
        ],
      });
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));
      await waitFor(() => {
        expect(screen.getByText("Monstera deliciosa")).toBeTruthy();
      });

      const nextBtn = screen.getByLabelText("Next suggestion");
      await user.click(nextBtn);

      await waitFor(() => {
        expect(screen.getByText("Philodendron bipinnatifidum")).toBeTruthy();
        expect(screen.getByText("2 / 2")).toBeTruthy();
      });

      const prevBtn = screen.getByLabelText("Previous suggestion");
      await user.click(prevBtn);

      await waitFor(() => {
        expect(screen.getByText("Monstera deliciosa")).toBeTruthy();
      });
    });
  });

  describe("not-a-plant error", () => {
    it("shows localized not-a-plant message with dismiss instead of retry", async () => {
      mockIdentifyPlant.mockRejectedValue(
        new ApiError(
          422,
          "AI_IDENTIFY_NOT_A_PLANT",
          "The photo does not appear to contain a plant",
        ),
      );
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(
          screen.getByText("The photo does not appear to contain a plant"),
        ).toBeTruthy();
        expect(screen.queryByText("Retry")).toBeNull();
        expect(screen.getByText("Dismiss")).toBeTruthy();
      });
    });

    it("dismiss on non-retryable error returns to idle", async () => {
      mockIdentifyPlant.mockRejectedValue(
        new ApiError(
          422,
          "AI_IDENTIFY_NOT_A_PLANT",
          "The photo does not appear to contain a plant",
        ),
      );
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));
      await waitFor(() => {
        expect(screen.getByText("Dismiss")).toBeTruthy();
      });

      await user.click(screen.getByText("Dismiss"));

      expect(screen.getByText("Identify Plant")).toBeTruthy();
    });

    it("shows retry for retryable server errors", async () => {
      mockIdentifyPlant.mockRejectedValue(
        new ApiError(500, "AI_PROVIDER_FAILED", "AI provider request failed"),
      );
      renderPanel();
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      await waitFor(() => {
        expect(screen.getByText("Retry")).toBeTruthy();
        expect(screen.queryByText("Dismiss")).toBeNull();
      });
    });
  });

  describe("no photo returns to idle", () => {
    it("does not call API when no photo is available", async () => {
      renderPanel({
        photoFile: null,
        photoPreview: null,
        existingPhotoUrl: null,
      });
      const user = userEvent.setup();

      await user.click(screen.getByText("Identify Plant"));

      expect(mockIdentifyPlant).not.toHaveBeenCalled();
      expect(screen.getByText("Identify Plant")).toBeTruthy();
    });
  });
});
