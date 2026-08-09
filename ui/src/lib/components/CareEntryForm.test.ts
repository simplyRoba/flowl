import {
  cleanup,
  render,
  screen,
  fireEvent,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mockAddCareEvent = vi.fn();
const mockUpdateCareEvent = vi.fn();
const mockUploadCareEventPhoto = vi.fn();
const mockDeleteCareEventPhoto = vi.fn();
const mockPushNotification = vi.fn();

vi.mock("$lib/stores/care", () => ({
  addCareEvent: (...args: unknown[]) => mockAddCareEvent(...args),
}));

vi.mock("$lib/api", () => ({
  updateCareEvent: (...args: unknown[]) => mockUpdateCareEvent(...args),
  uploadCareEventPhoto: (...args: unknown[]) =>
    mockUploadCareEventPhoto(...args),
  deleteCareEventPhoto: (...args: unknown[]) =>
    mockDeleteCareEventPhoto(...args),
}));

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

import CareEntryForm from "./CareEntryForm.svelte";
import type { CareEvent } from "$lib/api";
import { isOffline } from "$lib/stores/network";

beforeEach(() => {
  vi.clearAllMocks();
  isOffline.set(false);
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("CareEntryForm", () => {
  function makeCareEvent(overrides: Partial<CareEvent> = {}): CareEvent {
    return {
      id: 12,
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

  const defaultProps = {
    plantId: 1,
    onsubmit: vi.fn(),
    oncancel: vi.fn(),
  };

  it("renders all five event type chips with their selected state", () => {
    render(CareEntryForm, { props: defaultProps });
    expect(screen.getByText("Watered")).toBeTruthy();
    expect(screen.getByText("Fertilized")).toBeTruthy();
    expect(screen.getByText("Repotted")).toBeTruthy();
    expect(screen.getByText("Pruned")).toBeTruthy();
    expect(screen.getByText("Custom")).toBeTruthy();
    expect(
      screen
        .getByRole("button", { name: "Watered" })
        .getAttribute("aria-pressed"),
    ).toBe("false");
  });

  it("disables save until an event type is selected", async () => {
    render(CareEntryForm, { props: defaultProps });
    const save = screen.getByText("Save");
    expect(save).toHaveProperty("disabled", true);
    expect(mockAddCareEvent).not.toHaveBeenCalled();

    await fireEvent.click(screen.getByText("Watered"));
    expect(save).toHaveProperty("disabled", false);
    expect(
      screen
        .getByRole("button", { name: "Watered" })
        .getAttribute("aria-pressed"),
    ).toBe("true");
  });

  it("submits with selected event type and calls onsubmit", async () => {
    const onsubmit = vi.fn();
    mockAddCareEvent.mockResolvedValue({
      id: 1,
      plant_id: 1,
      event_type: "watered",
      notes: null,
      photo_url: null,
      occurred_at: "2025-01-01T00:00:00Z",
      created_at: "2025-01-01T00:00:00Z",
    });
    render(CareEntryForm, {
      props: { ...defaultProps, onsubmit },
    });
    await fireEvent.click(screen.getByText("Watered"));
    await fireEvent.click(screen.getByText("Save"));
    await waitFor(() => {
      expect(mockAddCareEvent).toHaveBeenCalledWith(1, {
        event_type: "watered",
        notes: undefined,
        occurred_at: undefined,
      });
      expect(onsubmit).toHaveBeenCalled();
    });
  });

  it("calls oncancel when cancel is clicked", async () => {
    const oncancel = vi.fn();
    render(CareEntryForm, { props: { ...defaultProps, oncancel } });
    await fireEvent.click(screen.getByText("Cancel"));
    expect(oncancel).toHaveBeenCalled();
  });

  it("shows error notification when addCareEvent returns null", async () => {
    mockAddCareEvent.mockResolvedValue(null);
    render(CareEntryForm, { props: defaultProps });
    await fireEvent.click(screen.getByText("Fertilized"));
    await fireEvent.click(screen.getByText("Save"));
    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({ variant: "error" }),
      );
    });
  });

  it("initializes edit mode with the existing event values and photo", () => {
    const event = makeCareEvent({
      event_type: "pruned",
      notes: "Trimmed damaged leaves",
      photo_url: "/uploads/care/12.jpg",
    });
    render(CareEntryForm, { props: { ...defaultProps, existingEvent: event } });

    expect(screen.getByText("Pruned").classList.contains("active")).toBe(true);
    expect(screen.getByDisplayValue("Trimmed damaged leaves")).toBeTruthy();
    expect(
      document.querySelector(".toolbar-thumb img")?.getAttribute("src"),
    ).toBe("/uploads/care/12.jpg");
    expect(document.querySelector('input[type="datetime-local"]')).toBeTruthy();
  });

  it("shows translated saving text and disables edit save while update is pending", async () => {
    const event = makeCareEvent();
    const onsubmit = vi.fn();
    let resolveUpdate!: (value: CareEvent) => void;
    mockUpdateCareEvent.mockReturnValue(
      new Promise<CareEvent>((resolve) => {
        resolveUpdate = resolve;
      }),
    );
    render(CareEntryForm, {
      props: { ...defaultProps, existingEvent: event, onsubmit },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() => {
      const save = screen.getByRole("button", { name: "Saving..." });
      expect(save).toHaveProperty("disabled", true);
      expect(mockUpdateCareEvent).toHaveBeenCalledWith(
        1,
        event.id,
        expect.anything(),
      );
    });

    resolveUpdate(event);
    await waitFor(() => {
      expect(onsubmit).toHaveBeenCalledWith(event);
      expect(screen.getByRole("button", { name: "Save" })).toHaveProperty(
        "disabled",
        false,
      );
    });
  });

  it("updates fields and retains an unchanged photo", async () => {
    const event = makeCareEvent({ photo_url: "/uploads/care/12.jpg" });
    const onsubmit = vi.fn();
    mockUpdateCareEvent.mockResolvedValue(event);
    render(CareEntryForm, {
      props: { ...defaultProps, existingEvent: event, onsubmit },
    });

    const dateInput = document.querySelector(
      'input[type="datetime-local"]',
    ) as HTMLInputElement;
    const notesInput = document.querySelector(
      ".log-notes",
    ) as HTMLTextAreaElement;
    await fireEvent.input(notesInput, { target: { value: "Fresh notes" } });
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockUpdateCareEvent).toHaveBeenCalledWith(1, event.id, {
        event_type: "watered",
        notes: "Fresh notes",
        occurred_at: new Date(dateInput.value).toISOString(),
      });
      expect(mockDeleteCareEventPhoto).not.toHaveBeenCalled();
      expect(mockUploadCareEventPhoto).not.toHaveBeenCalled();
      expect(onsubmit).toHaveBeenCalledWith(event);
    });
  });

  it("removes an existing photo only after a successful field update", async () => {
    const event = makeCareEvent({ photo_url: "/uploads/care/12.jpg" });
    mockUpdateCareEvent.mockResolvedValue(event);
    mockDeleteCareEventPhoto.mockResolvedValue(undefined);
    render(CareEntryForm, { props: { ...defaultProps, existingEvent: event } });

    await fireEvent.click(screen.getByRole("button", { name: "Remove photo" }));
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockUpdateCareEvent).toHaveBeenCalledWith(
        1,
        event.id,
        expect.anything(),
      );
      expect(mockDeleteCareEventPhoto).toHaveBeenCalledWith(1, event.id);
      expect(mockUpdateCareEvent.mock.invocationCallOrder[0]).toBeLessThan(
        mockDeleteCareEventPhoto.mock.invocationCallOrder[0],
      );
    });
  });

  it("uploads a replacement after the field update without deleting the existing photo", async () => {
    const event = makeCareEvent({ photo_url: "/uploads/care/12.jpg" });
    const uploaded = {
      ...event,
      photo_url: "/uploads/care/12-replacement.jpg",
    };
    const onsubmit = vi.fn();
    const file = new File(["photo"], "replacement.jpg", {
      type: "image/jpeg",
    });
    mockUpdateCareEvent.mockResolvedValue(event);
    mockUploadCareEventPhoto.mockResolvedValue(uploaded);
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: vi.fn(() => "blob:replacement"),
      revokeObjectURL: vi.fn(),
    });
    render(CareEntryForm, {
      props: { ...defaultProps, existingEvent: event, onsubmit },
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    Object.defineProperty(fileInput, "files", { value: [file] });
    await fireEvent.change(fileInput);
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockUpdateCareEvent.mock.invocationCallOrder[0]).toBeLessThan(
        mockUploadCareEventPhoto.mock.invocationCallOrder[0],
      );
      expect(mockDeleteCareEventPhoto).not.toHaveBeenCalled();
      expect(mockUploadCareEventPhoto).toHaveBeenCalledWith(1, event.id, file);
      expect(onsubmit).toHaveBeenCalledWith(uploaded);
    });
  });

  it("keeps edit input and selected replacement after a failed photo upload", async () => {
    const event = makeCareEvent({ photo_url: "/uploads/care/12.jpg" });
    const file = new File(["photo"], "replacement.jpg", {
      type: "image/jpeg",
    });
    mockUpdateCareEvent.mockResolvedValue(event);
    mockUploadCareEventPhoto.mockRejectedValue(new Error("upload failed"));
    vi.stubGlobal("URL", {
      ...URL,
      createObjectURL: vi.fn(() => "blob:replacement"),
      revokeObjectURL: vi.fn(),
    });
    render(CareEntryForm, { props: { ...defaultProps, existingEvent: event } });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    Object.defineProperty(fileInput, "files", { value: [file] });
    await fireEvent.change(fileInput);
    await fireEvent.input(document.querySelector(".log-notes")!, {
      target: { value: "Keep this note" },
    });
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          message: "Care entry was saved, but the photo change failed",
        }),
      );
      expect(screen.getByDisplayValue("Keep this note")).toBeTruthy();
      expect(
        document.querySelector(".toolbar-thumb img")?.getAttribute("src"),
      ).toBe("blob:replacement");
      expect(mockUploadCareEventPhoto).toHaveBeenCalledWith(1, event.id, file);
    });
  });

  it("keeps photo removal pending after deletion fails and retries it on save", async () => {
    const event = makeCareEvent({ photo_url: "/uploads/care/12.jpg" });
    const onsubmit = vi.fn();
    mockUpdateCareEvent.mockResolvedValue(event);
    mockDeleteCareEventPhoto.mockRejectedValueOnce(new Error("delete failed"));
    render(CareEntryForm, {
      props: { ...defaultProps, existingEvent: event, onsubmit },
    });

    await fireEvent.click(screen.getByRole("button", { name: "Remove photo" }));
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          message: "Care entry was saved, but the photo change failed",
        }),
      );
      expect(mockDeleteCareEventPhoto).toHaveBeenCalledTimes(1);
      expect(onsubmit).not.toHaveBeenCalled();
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });

    mockDeleteCareEventPhoto.mockResolvedValue(undefined);
    await fireEvent.click(screen.getByText("Save"));
    await waitFor(() => {
      expect(mockUpdateCareEvent).toHaveBeenCalledTimes(2);
      expect(mockDeleteCareEventPhoto).toHaveBeenCalledTimes(2);
      expect(onsubmit).toHaveBeenCalledWith({ ...event, photo_url: null });
    });
  });

  it("treats timezone-less API timestamps as UTC", async () => {
    vi.stubEnv("TZ", "America/New_York");
    const event = makeCareEvent({ occurred_at: "2025-01-01T10:00:00" });
    mockUpdateCareEvent.mockResolvedValue(event);

    try {
      expect(new Date("2025-01-01T10:00:00Z").getHours()).toBe(5);
      render(CareEntryForm, {
        props: { ...defaultProps, existingEvent: event },
      });
      const dateInput = document.querySelector(
        'input[type="datetime-local"]',
      ) as HTMLInputElement;
      const expectedInstant = "2025-01-01T10:00:00.000Z";
      expect(new Date(dateInput.value).toISOString()).toBe(expectedInstant);

      await fireEvent.click(screen.getByText("Save"));
      await waitFor(() => {
        expect(mockUpdateCareEvent).toHaveBeenCalledWith(
          1,
          event.id,
          expect.objectContaining({ occurred_at: expectedInstant }),
        );
      });
    } finally {
      vi.unstubAllEnvs();
    }
  });

  it.each([
    ["UTC", "2025-01-01T10:00:00Z", "2025-01-01T10:00:00Z"],
    ["explicit offset", "2025-01-01T12:00:00+02:00", "2025-01-01T10:00:00Z"],
  ])(
    "preserves the instant from %s API timestamps",
    async (_format, occurredAt, expectedIso) => {
      const event = makeCareEvent({ occurred_at: occurredAt });
      mockUpdateCareEvent.mockResolvedValue(event);
      render(CareEntryForm, {
        props: { ...defaultProps, existingEvent: event },
      });

      const dateInput = document.querySelector(
        'input[type="datetime-local"]',
      ) as HTMLInputElement;
      const expectedInstant = new Date(expectedIso).toISOString();
      expect(new Date(dateInput.value).toISOString()).toBe(expectedInstant);

      await fireEvent.click(screen.getByText("Save"));
      await waitFor(() => {
        expect(mockUpdateCareEvent).toHaveBeenCalledWith(
          1,
          event.id,
          expect.objectContaining({ occurred_at: expectedInstant }),
        );
      });
    },
  );

  it("blocks a future edit occurrence time without sending an update", async () => {
    const event = makeCareEvent();
    render(CareEntryForm, { props: { ...defaultProps, existingEvent: event } });
    const dateInput = document.querySelector(
      'input[type="datetime-local"]',
    ) as HTMLInputElement;
    const future = new Date(Date.now() + 60 * 60 * 1000);
    const pad = (value: number) => String(value).padStart(2, "0");
    await fireEvent.input(dateInput, {
      target: {
        value: `${future.getFullYear()}-${pad(future.getMonth() + 1)}-${pad(future.getDate())}T${pad(future.getHours())}:${pad(future.getMinutes())}`,
      },
    });
    await fireEvent.click(screen.getByText("Save"));

    expect(
      screen.getByText("Occurrence time cannot be in the future"),
    ).toBeTruthy();
    expect(mockUpdateCareEvent).not.toHaveBeenCalled();
  });

  it("disables edit save while offline without clearing its values", () => {
    isOffline.set(true);
    const event = makeCareEvent({ notes: "Keep me" });
    render(CareEntryForm, { props: { ...defaultProps, existingEvent: event } });

    expect(screen.getByText("Save")).toHaveProperty("disabled", true);
    expect(screen.getByDisplayValue("Keep me")).toBeTruthy();
  });
});
