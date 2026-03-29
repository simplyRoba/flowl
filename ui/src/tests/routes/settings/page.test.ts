import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../../../routes/settings/+page.svelte";
import { get } from "svelte/store";
import { setThemePreference, THEME_STORAGE_KEY } from "$lib/stores/theme";
import {
  locale,
  setLocale,
  destroyLocale,
  LOCALE_STORAGE_KEY,
} from "$lib/stores/locale";
import { locations, locationsError } from "$lib/stores/locations";
import { isOffline } from "$lib/stores/network";
import * as api from "$lib/api";

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (
  this: HTMLDialogElement,
) {
  this.setAttribute("open", "");
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
  this.removeAttribute("open");
});

const mockDeleteLocation = vi.fn();
const mockLoadLocations = vi.fn();
const mockUpdateLocation = vi.fn();
const mockPushNotification = vi.fn();

vi.mock("$lib/stores/network", async () => {
  const { writable } = await import("svelte/store");
  return {
    isOffline: writable(false),
    recheckHealth: vi.fn(),
    startNetworkMonitor: vi.fn(() => () => {}),
  };
});

vi.mock("$lib/stores/locations", async () => {
  const { writable } = await import("svelte/store");
  return {
    locations: writable([]),
    locationsError: writable(null),
    loadLocations: (...args: unknown[]) => mockLoadLocations(...args),
    deleteLocation: (...args: unknown[]) => mockDeleteLocation(...args),
    updateLocation: (...args: unknown[]) => mockUpdateLocation(...args),
  };
});

vi.mock("$lib/stores/notifications", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

beforeEach(() => {
  localStorage.clear();
  setThemePreference("system");
  destroyLocale();
  setLocale("en");
  locations.set([]);
  locationsError.set(null);
  isOffline.set(false);
  vi.clearAllMocks();
  mockLoadLocations.mockResolvedValue(undefined);
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

describe("settings appearance theme selector", () => {
  it("shows appearance section with light, dark, and system options", () => {
    render(Page);

    expect(screen.getByText("Appearance")).toBeTruthy();
    expect(screen.getByRole("radiogroup", { name: "Theme" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Light" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Dark" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "System" })).toBeTruthy();
  });

  it("persists selection and reflects active state", async () => {
    const user = userEvent.setup();
    render(Page);

    const darkButton = screen.getByRole("button", { name: "Dark" });
    await user.click(darkButton);

    expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe("dark");
    expect(darkButton.classList.contains("active")).toBe(true);
  });
});

describe("settings language selector", () => {
  it("shows language section with English, Deutsch, and Español options", () => {
    render(Page);

    expect(screen.getByRole("radiogroup", { name: "Language" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "English" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Deutsch" })).toBeTruthy();
    expect(screen.getByRole("button", { name: "Español" })).toBeTruthy();
  });

  it("updates locale store and persists when selecting a language", async () => {
    const user = userEvent.setup();
    render(Page);

    const deutschButton = screen.getByRole("button", { name: "Deutsch" });
    await user.click(deutschButton);

    expect(get(locale)).toBe("de");
    expect(localStorage.getItem(LOCALE_STORAGE_KEY)).toBe("de");
    expect(deutschButton.classList.contains("active")).toBe(true);
  });

  it("reactively updates UI text when language changes", async () => {
    const user = userEvent.setup();
    render(Page);

    expect(screen.getByText("Appearance")).toBeTruthy();

    await user.click(screen.getByRole("button", { name: "Deutsch" }));

    await waitFor(() => {
      expect(screen.getByText("Darstellung")).toBeTruthy();
    });
  });
});

describe("settings locations section", () => {
  it("shows Locations heading", () => {
    render(Page);
    expect(screen.getByText("Locations")).toBeTruthy();
  });

  it("shows empty state when no locations", () => {
    render(Page);
    expect(
      screen.getByText(
        "No locations yet. Create locations when adding plants.",
      ),
    ).toBeTruthy();
  });

  it("renders location list", () => {
    locations.set([
      { id: 1, name: "Bedroom", plant_count: 2 },
      { id: 2, name: "Kitchen", plant_count: 0 },
    ]);
    render(Page);
    expect(screen.getByText("Bedroom")).toBeTruthy();
    expect(screen.getByText("Kitchen")).toBeTruthy();
  });

  it("shows plant count badge for locations with plants", () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 3 }]);
    render(Page);
    expect(screen.getByText("3 plants")).toBeTruthy();
  });

  it("shows singular plant count", () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 1 }]);
    render(Page);
    expect(screen.getByText("1 plant")).toBeTruthy();
  });

  it("shows error when locationsError is set", () => {
    locationsError.set("Failed to load");
    render(Page);
    expect(screen.getByText("Failed to load")).toBeTruthy();
  });

  it("keeps rename conflicts inline", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 0 }]);
    mockUpdateLocation.mockResolvedValue({ error: "Location already exists" });
    render(Page);

    const user = userEvent.setup();
    const editButton = document.querySelector(
      ".location-actions .btn-icon",
    ) as HTMLButtonElement;
    await user.click(editButton);

    const input = document.querySelector(".edit-input") as HTMLInputElement;
    await user.clear(input);
    await user.type(input, "Kitchen");

    const confirmButton = document.querySelector(
      ".edit-row .btn-icon",
    ) as HTMLButtonElement;
    await user.click(confirmButton);

    await waitFor(() => {
      expect(screen.getByText("Location already exists")).toBeTruthy();
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("cancels inline editing without saving", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 0 }]);
    render(Page);

    const user = userEvent.setup();
    const editButton = document.querySelector(
      ".location-actions .btn-icon",
    ) as HTMLButtonElement;
    await user.click(editButton);

    const input = document.querySelector(".edit-input") as HTMLInputElement;
    await user.clear(input);
    await user.type(input, "Kitchen");

    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await waitFor(() => {
      expect(screen.queryByRole("textbox")).toBeNull();
    });
    expect(screen.getByText("Bedroom")).toBeTruthy();
    expect(mockUpdateLocation).not.toHaveBeenCalled();
  });
});

describe("settings data section export/import", () => {
  beforeEach(() => {
    vi.spyOn(api, "fetchStats").mockResolvedValue({
      plant_count: 5,
      care_event_count: 10,
      location_count: 2,
      photo_count: 3,
    });
    vi.spyOn(api, "fetchAppInfo").mockRejectedValue(new Error("skip"));
    vi.spyOn(api, "fetchMqttStatus").mockRejectedValue(new Error("skip"));
  });

  it("shows Export and Import buttons on same row when stats load", async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByText("Data")).toBeTruthy();
    });
    expect(screen.getByText("Backup")).toBeTruthy();
    expect(screen.getByRole("button", { name: /Export/ })).toBeTruthy();
    expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
  });

  it("export button navigates to export URL", async () => {
    const exportSpy = vi.spyOn(api, "exportData").mockResolvedValue();

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Export/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Export/ }));

    await waitFor(() => {
      expect(exportSpy).toHaveBeenCalledTimes(1);
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("shows a toast when export fails before download starts", async () => {
    vi.spyOn(api, "exportData").mockRejectedValue(
      new Error("Export unavailable"),
    );

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Export/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Export/ }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Export",
          variant: "error",
          message: "Export failed",
        }),
      );
    });
  });

  it("import button opens file picker", async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    expect(fileInput).toBeTruthy();
    expect(fileInput.accept).toBe(".zip");
  });

  it("shows import confirmation dialog with file name", async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["zip"], "test.zip", { type: "application/zip" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      configurable: true,
    });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));

    await waitFor(() => {
      expect(screen.getByText(/test\.zip/)).toBeTruthy();
      expect(screen.getByText(/replaced/)).toBeTruthy();
    });
  });

  it("shows translated import error for known ApiError codes", async () => {
    setLocale("de");
    vi.spyOn(api, "importData").mockRejectedValue(
      new api.ApiError(
        400,
        "IMPORT_VERSION_MISMATCH",
        "Incompatible export version",
      ),
    );

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["zip"], "test.zip", { type: "application/zip" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      configurable: true,
    });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));

    // Confirm in dialog
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Importieren" })).toBeTruthy();
    });
    const user = userEvent.setup();
    const importButtons = screen.getAllByRole("button", {
      name: "Importieren",
    });
    await user.click(importButtons[importButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Daten importieren",
          variant: "error",
          message: "Inkompatible Exportversion",
        }),
      );
    });
  });

  it("shows success message after import", async () => {
    vi.spyOn(api, "importData").mockResolvedValue({
      locations: 1,
      plants: 3,
      care_events: 5,
      photos: 2,
    });

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["zip"], "test.zip", { type: "application/zip" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      configurable: true,
    });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));

    // Confirm in dialog
    await waitFor(() => {
      expect(screen.getByText(/test\.zip/)).toBeTruthy();
    });
    const user = userEvent.setup();
    const importButtons = screen.getAllByRole("button", { name: "Import" });
    await user.click(importButtons[importButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Import data",
          variant: "success",
          message: expect.stringMatching(/Imported 3 plants/),
        }),
      );
    });
  });

  it("refreshes stats and locations after import succeeds", async () => {
    const fetchStatsSpy = vi
      .spyOn(api, "fetchStats")
      .mockResolvedValueOnce({
        plant_count: 5,
        care_event_count: 10,
        location_count: 2,
        photo_count: 3,
      })
      .mockResolvedValueOnce({
        plant_count: 8,
        care_event_count: 12,
        location_count: 4,
        photo_count: 6,
      });
    vi.spyOn(api, "importData").mockResolvedValue({
      locations: 4,
      plants: 8,
      care_events: 12,
      photos: 6,
    });

    render(Page);
    await waitFor(() => {
      expect(screen.getByText(/5 plants, 3 photos/)).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["zip"], "refresh.zip", { type: "application/zip" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      configurable: true,
    });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));

    await waitFor(() => {
      expect(screen.getByText(/refresh\.zip/)).toBeTruthy();
    });

    const user = userEvent.setup();
    const importButtons = screen.getAllByRole("button", { name: "Import" });
    await user.click(importButtons[importButtons.length - 1]);

    await waitFor(() => {
      expect(fetchStatsSpy).toHaveBeenCalledTimes(2);
      expect(mockLoadLocations).toHaveBeenCalledTimes(2);
      expect(
        screen.getByText(
          /8 plants, 6 photos, 12 care journal entries, 4 locations/,
        ),
      ).toBeTruthy();
    });
  });

  it("shows a follow-up error when totals cannot be refreshed after import", async () => {
    vi.spyOn(api, "fetchStats")
      .mockResolvedValueOnce({
        plant_count: 5,
        care_event_count: 10,
        location_count: 2,
        photo_count: 3,
      })
      .mockRejectedValueOnce(new Error("refresh failed"));
    vi.spyOn(api, "importData").mockResolvedValue({
      locations: 1,
      plants: 3,
      care_events: 5,
      photos: 2,
    });

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["zip"], "test.zip", { type: "application/zip" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      configurable: true,
    });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));

    await waitFor(() => {
      expect(screen.getByText(/test\.zip/)).toBeTruthy();
    });

    const user = userEvent.setup();
    const importButtons = screen.getAllByRole("button", { name: "Import" });
    await user.click(importButtons[importButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Import data",
          variant: "error",
          message:
            "Import completed, but the refreshed totals could not be loaded.",
        }),
      );
    });
  });

  it("does not import when dialog is cancelled", async () => {
    const importSpy = vi.spyOn(api, "importData");

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Import/ })).toBeTruthy();
    });

    const fileInput = document.querySelector(
      'input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["zip"], "test.zip", { type: "application/zip" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      configurable: true,
    });
    fileInput.dispatchEvent(new Event("change", { bubbles: true }));

    // Cancel in dialog
    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    });
    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await new Promise((r) => setTimeout(r, 50));
    expect(importSpy).not.toHaveBeenCalled();
  });
});

describe("settings delete location confirmation", () => {
  function getDeleteButtons() {
    return document.querySelectorAll(
      ".btn-danger",
    ) as NodeListOf<HTMLButtonElement>;
  }

  it("deletes immediately when location has no plants", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 0 }]);
    mockDeleteLocation.mockResolvedValue(true);
    render(Page);

    const user = userEvent.setup();
    await user.click(getDeleteButtons()[0]);

    await waitFor(() => {
      expect(mockDeleteLocation).toHaveBeenCalledWith(1);
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Locations",
          variant: "success",
          message: 'Location "Bedroom" deleted',
        }),
      );
    });
  });

  it("shows confirmation dialog with location name when location has plants", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 3 }]);
    render(Page);

    const user = userEvent.setup();
    await user.click(getDeleteButtons()[0]);

    await waitFor(() => {
      expect(screen.getByText(/Delete "Bedroom"/)).toBeTruthy();
      expect(
        screen.getByText(/3 plants will lose their location/),
      ).toBeTruthy();
    });
  });

  it("calls deleteLocation when confirmed", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 2 }]);
    mockDeleteLocation.mockResolvedValue(true);
    render(Page);

    const user = userEvent.setup();
    await user.click(getDeleteButtons()[0]);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Delete" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockDeleteLocation).toHaveBeenCalledWith(1);
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Locations",
          variant: "success",
          message: 'Location "Bedroom" deleted',
        }),
      );
    });
  });

  it("shows a toast when deleting a location fails", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 0 }]);
    mockDeleteLocation.mockImplementation(async () => {
      locationsError.set("Failed to delete location");
      return false;
    });
    render(Page);

    const user = userEvent.setup();
    await user.click(getDeleteButtons()[0]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Locations",
          variant: "error",
          message: "Failed to delete location",
        }),
      );
    });
  });

  it("does not delete when dialog is cancelled", async () => {
    locations.set([{ id: 1, name: "Bedroom", plant_count: 1 }]);
    render(Page);

    const user = userEvent.setup();
    await user.click(getDeleteButtons()[0]);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await new Promise((r) => setTimeout(r, 50));
    expect(mockDeleteLocation).not.toHaveBeenCalled();
  });
});

describe("settings MQTT repair confirmation", () => {
  beforeEach(() => {
    vi.spyOn(api, "fetchStats").mockRejectedValue(new Error("skip"));
    vi.spyOn(api, "fetchAppInfo").mockRejectedValue(new Error("skip"));
    vi.spyOn(api, "fetchMqttStatus").mockResolvedValue({
      status: "connected",
      broker: "mqtt://localhost",
      topic_prefix: "flowl",
    });
  });

  it("shows confirmation dialog when repair is clicked", async () => {
    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Repair/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Repair/ }));

    await waitFor(() => {
      expect(screen.getByText(/Clear all retained MQTT topics/)).toBeTruthy();
    });
  });

  it("calls repairMqtt when confirmed", async () => {
    vi.spyOn(api, "repairMqtt").mockResolvedValue({ cleared: 5, published: 3 });

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Repair/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Repair/ }));

    await waitFor(() => {
      expect(screen.getByText(/Clear all retained MQTT topics/)).toBeTruthy();
    });
    // Two "Repair" buttons: toolbar and dialog confirm — click the dialog one
    const repairButtons = screen.getAllByRole("button", { name: "Repair" });
    await user.click(repairButtons[repairButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Repair MQTT",
          variant: "success",
          message: "Cleared 5, published 3",
        }),
      );
    });
  });

  it("restores the repair button after the request finishes", async () => {
    let resolveRepair:
      | ((value: { cleared: number; published: number }) => void)
      | undefined;
    vi.spyOn(api, "repairMqtt").mockImplementation(
      () =>
        new Promise((resolve) => {
          resolveRepair = resolve;
        }),
    );

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Repair/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Repair/ }));
    await waitFor(() => {
      expect(screen.getByText(/Clear all retained MQTT topics/)).toBeTruthy();
    });

    const repairButtons = screen.getAllByRole("button", { name: "Repair" });
    await user.click(repairButtons[repairButtons.length - 1]);

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Repairing..." })).toBeTruthy();
    });

    resolveRepair?.({ cleared: 2, published: 1 });

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Repair" })).toBeTruthy();
    });
  });

  it("shows a toast when repair fails", async () => {
    vi.spyOn(api, "repairMqtt").mockRejectedValue(
      new Error("Repair unavailable"),
    );

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Repair/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Repair/ }));

    await waitFor(() => {
      expect(screen.getByText(/Clear all retained MQTT topics/)).toBeTruthy();
    });
    const repairButtons = screen.getAllByRole("button", { name: "Repair" });
    await user.click(repairButtons[repairButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Repair MQTT",
          variant: "error",
          message: "Repair failed",
        }),
      );
    });
  });

  it("does not repair when dialog is cancelled", async () => {
    const repairSpy = vi.spyOn(api, "repairMqtt");

    render(Page);
    await waitFor(() => {
      expect(screen.getByRole("button", { name: /Repair/ })).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: /Repair/ }));

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await new Promise((r) => setTimeout(r, 50));
    expect(repairSpy).not.toHaveBeenCalled();
  });
});

describe("settings offline message", () => {
  it("shows offline message and hides all API sections when offline", async () => {
    isOffline.set(true);

    render(Page);

    await waitFor(() => {
      expect(
        screen.getByText(
          "You're offline. Connect to the internet to view this page.",
        ),
      ).toBeTruthy();
    });

    // Appearance and Language sections should still be present
    expect(screen.getByText("Appearance")).toBeTruthy();
    expect(screen.getByText("Language")).toBeTruthy();

    // All API-dependent sections should be hidden
    expect(screen.queryByText("Locations")).toBeNull();
    expect(screen.queryByText("MQTT")).toBeNull();
    expect(screen.queryByText("Data")).toBeNull();
    expect(screen.queryByText("About")).toBeNull();
  });

  it("hides stale API sections when transitioning to offline", async () => {
    isOffline.set(false);

    // Load with data available
    const fetchStatsSpy = vi.spyOn(api, "fetchStats").mockResolvedValue({
      plant_count: 3,
      photo_count: 1,
      care_event_count: 5,
      location_count: 2,
    });
    const fetchInfoSpy = vi.spyOn(api, "fetchAppInfo").mockResolvedValue({
      version: "1.0.0",
      repository: "https://github.com/test",
      license: "MIT",
    });
    const fetchMqttSpy = vi.spyOn(api, "fetchMqttStatus").mockResolvedValue({
      status: "connected",
      broker: "localhost",
      topic_prefix: "flowl",
    });

    render(Page);

    await waitFor(() => {
      expect(screen.getByText("Data")).toBeTruthy();
      expect(screen.getByText("About")).toBeTruthy();
    });

    // Go offline - stale sections should disappear
    isOffline.set(true);

    await waitFor(() => {
      expect(
        screen.getByText(
          "You're offline. Connect to the internet to view this page.",
        ),
      ).toBeTruthy();
      expect(screen.queryByText("Data")).toBeNull();
      expect(screen.queryByText("About")).toBeNull();
    });

    fetchStatsSpy.mockRestore();
    fetchInfoSpy.mockRestore();
    fetchMqttSpy.mockRestore();
  });

  it("does not show offline message when online", async () => {
    isOffline.set(false);

    const fetchStatsSpy = vi
      .spyOn(api, "fetchStats")
      .mockRejectedValue(new Error("server error"));
    const fetchInfoSpy = vi
      .spyOn(api, "fetchAppInfo")
      .mockRejectedValue(new Error("server error"));
    const fetchMqttSpy = vi
      .spyOn(api, "fetchMqttStatus")
      .mockRejectedValue(new Error("server error"));

    render(Page);

    // Wait for all promises to settle
    await new Promise((r) => setTimeout(r, 50));

    expect(
      screen.queryByText(
        "You're offline. Connect to the internet to view this page.",
      ),
    ).toBeNull();

    fetchStatsSpy.mockRestore();
    fetchInfoSpy.mockRestore();
    fetchMqttSpy.mockRestore();
  });
});
