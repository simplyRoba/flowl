import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import ToastHost from "./ToastHost.svelte";
import {
  clearNotifications,
  dismissNotification,
  pushNotification,
} from "$lib/stores/notifications";

describe("ToastHost", () => {
  beforeEach(() => {
    clearNotifications();
    vi.useFakeTimers();
  });

  afterEach(() => {
    cleanup();
    clearNotifications();
    vi.useRealTimers();
  });

  it("renders notifications and uses live-region semantics", async () => {
    render(ToastHost);
    pushNotification({ variant: "error", message: "Upload failed" });
    pushNotification({ variant: "success", message: "Saved" });

    await waitFor(() => {
      expect(screen.getByText("Upload failed")).toBeTruthy();
      expect(screen.getByText("Saved")).toBeTruthy();
    });

    expect(screen.getByRole("alert").textContent).toContain("Upload failed");
    expect(screen.getByRole("status").textContent).toContain("Saved");
  });

  it("auto-dismisses success notifications", async () => {
    render(ToastHost);
    pushNotification({ variant: "success", message: "Saved" });

    await waitFor(() => {
      expect(screen.getByText("Saved")).toBeTruthy();
    });

    vi.advanceTimersByTime(3500);

    await waitFor(() => {
      expect(screen.queryByText("Saved")).toBeNull();
    });
  });

  it("pauses auto-dismiss while hovered and resumes on mouse leave", async () => {
    render(ToastHost);
    pushNotification({ variant: "success", message: "Saved" });

    const toast = await screen.findByText("Saved");
    await fireEvent.mouseEnter(toast.closest(".toast")!);

    vi.advanceTimersByTime(10000);
    expect(screen.getByText("Saved")).toBeTruthy();

    await fireEvent.mouseLeave(toast.closest(".toast")!);
    vi.advanceTimersByTime(3500);

    await waitFor(() => {
      expect(screen.queryByText("Saved")).toBeNull();
    });
  });

  it("does not auto-dismiss error notifications", async () => {
    render(ToastHost);
    pushNotification({ variant: "error", message: "Upload failed" });

    await waitFor(() => {
      expect(screen.getByText("Upload failed")).toBeTruthy();
    });

    vi.advanceTimersByTime(10000);

    expect(screen.getByText("Upload failed")).toBeTruthy();
  });

  it("shows at most three notifications at once", async () => {
    render(ToastHost);
    const ids = [
      pushNotification({ message: "One" }),
      pushNotification({ message: "Two" }),
      pushNotification({ message: "Three" }),
      pushNotification({ message: "Four" }),
    ];

    await waitFor(() => {
      expect(screen.getByText("Four")).toBeTruthy();
      expect(screen.getByText("Three")).toBeTruthy();
      expect(screen.getByText("Two")).toBeTruthy();
    });

    expect(screen.queryByText("One")).toBeNull();

    dismissNotification(ids[3]);

    await waitFor(() => {
      expect(screen.getByText("One")).toBeTruthy();
    });
  });
});
