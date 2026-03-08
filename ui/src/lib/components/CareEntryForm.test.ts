import {
  cleanup,
  render,
  screen,
  fireEvent,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const mockAddCareEvent = vi.fn();
const mockPushNotification = vi.fn();

vi.mock("$lib/stores/care", () => ({
  addCareEvent: (...args: unknown[]) => mockAddCareEvent(...args),
}));

vi.mock("$lib/api", () => ({
  uploadCareEventPhoto: vi.fn(),
}));

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

import CareEntryForm from "./CareEntryForm.svelte";

beforeEach(() => {
  vi.clearAllMocks();
});

afterEach(() => {
  cleanup();
});

describe("CareEntryForm", () => {
  const defaultProps = {
    plantId: 1,
    onsubmit: vi.fn(),
    oncancel: vi.fn(),
  };

  it("renders all five event type chips", () => {
    render(CareEntryForm, { props: defaultProps });
    expect(screen.getByText("Watered")).toBeTruthy();
    expect(screen.getByText("Fertilized")).toBeTruthy();
    expect(screen.getByText("Repotted")).toBeTruthy();
    expect(screen.getByText("Pruned")).toBeTruthy();
    expect(screen.getByText("Custom")).toBeTruthy();
  });

  it("shows validation error when submitting without selecting event type", async () => {
    render(CareEntryForm, { props: defaultProps });
    await fireEvent.click(screen.getByText("Save"));
    expect(screen.getByText("Choose a care entry type")).toBeTruthy();
    expect(mockAddCareEvent).not.toHaveBeenCalled();
  });

  it("clears validation error when selecting an event type", async () => {
    render(CareEntryForm, { props: defaultProps });
    await fireEvent.click(screen.getByText("Save"));
    expect(screen.getByText("Choose a care entry type")).toBeTruthy();
    await fireEvent.click(screen.getByText("Watered"));
    expect(screen.queryByText("Choose a care entry type")).toBeNull();
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
});
