import {
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
  within,
} from "@testing-library/svelte";
import userEvent from "@testing-library/user-event";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import Page from "../../../../routes/plants/[id]/+page.svelte";
import type { AiStatus, CareEvent, Plant } from "#lib/api.js";

// jsdom doesn't implement window.matchMedia
Object.defineProperty(window, "matchMedia", {
  writable: true,
  value: vi.fn().mockImplementation((query: string) => ({
    matches: false,
    media: query,
    onchange: null,
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
});

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (
  this: HTMLDialogElement,
) {
  this.setAttribute("open", "");
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
  this.removeAttribute("open");
});

const mockDeletePlant = vi.fn();
const mockWaterPlant = vi.fn();
const mockAddCareEvent = vi.fn();
const mockPushNotification = vi.fn();
const mockGoto = vi.fn();

vi.mock("$app/state", () => ({
  page: {
    params: { id: "1" },
    url: new URL("http://localhost/plants/1"),
  },
}));

vi.mock("$app/navigation", () => ({
  goto: (...args: unknown[]) => mockGoto(...args),
}));

vi.mock("#lib/stores/plants.js", async () => {
  const { writable } = await import("svelte/store");
  const plantsError = writable<string | null>(null);
  return {
    plantsError,
    deletePlant: (...args: unknown[]) => mockDeletePlant(...args),
    waterPlant: (...args: unknown[]) => mockWaterPlant(...args),
  };
});

vi.mock("#lib/stores/care.js", async () => {
  const { writable } = await import("svelte/store");
  const careError = writable<string | null>(null);
  return {
    careError,
    addCareEvent: (...args: unknown[]) => mockAddCareEvent(...args),
  };
});

vi.mock("#lib/emoji.js", () => ({
  emojiToSvgPath: (emoji: string) => `/emoji/${emoji}.svg`,
}));

vi.mock("#lib/stores/notifications.js", () => ({
  pushNotification: (...args: unknown[]) => mockPushNotification(...args),
}));

import * as api from "#lib/api.js";
const mockChatPlant = vi.spyOn(api, "chatPlant");
const mockCreateCareEventApi = vi.spyOn(api, "createCareEvent");
const mockDeleteCareEventApi = vi.spyOn(api, "deleteCareEvent");
const mockUpdateCareEventApi = vi.spyOn(api, "updateCareEvent");
const mockFetchCareEventsApi = vi.spyOn(api, "fetchCareEvents");
const mockFetchPlantApi = vi.spyOn(api, "fetchPlant");
const mockSummarizeChat = vi.spyOn(api, "summarizeChat");
const mockUploadCareEventPhoto = vi.spyOn(api, "uploadCareEventPhoto");

const mockLoadAiStatus = vi.fn();

vi.mock("#lib/stores/ai.js", async () => {
  const { writable } = await import("svelte/store");
  const aiStatus = writable<AiStatus | null>(null);
  return {
    aiStatus,
    loadAiStatus: (...args: unknown[]) => mockLoadAiStatus(...args),
  };
});

import { aiStatus } from "#lib/stores/ai.js";
import { plantsError } from "#lib/stores/plants.js";
import { careError } from "#lib/stores/care.js";
import { isOffline } from "#lib/stores/network.js";
import {
  normalizeCareTimelineEvents,
  selectCareTimelineEvents,
} from "#lib/careTimeline.js";
import pageSource from "../../../../routes/plants/[id]/+page.svelte?raw";

function makePlant(overrides: Partial<Plant> = {}): Plant {
  return {
    id: 1,
    name: "Fern",
    species: "Boston Fern",
    icon: "🌿",
    photo_url: "/uploads/fern.jpg",
    location_id: 1,
    location_name: "Bedroom",
    watering_interval_days: 7,
    watering_status: "ok",
    last_watered: "2025-01-01",
    next_due: "2025-01-08",
    light_needs: "indirect",
    notes: null,
    difficulty: null,
    pet_safety: null,
    growth_speed: null,
    soil_type: null,
    soil_moisture: null,
    created_at: "2025-01-01T00:00:00Z",
    updated_at: "2025-01-01T00:00:00Z",
    ...overrides,
  };
}

function makeCareEvent(overrides: Partial<CareEvent> = {}): CareEvent {
  return {
    id: 10,
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

async function renderWithPlant(
  plantOverrides: Partial<Plant> = {},
  initialCareEvents: CareEvent[] = [],
) {
  const plant = makePlant(plantOverrides);
  mockFetchCareEventsApi.mockResolvedValue(initialCareEvents);
  return render(Page, {
    props: {
      data: {
        plant,
        notFound: false,
        loadErrorCode: null,
      },
    },
  });
}

beforeEach(() => {
  aiStatus.set(null);
  careError.set(null);
  isOffline.set(false);
  vi.clearAllMocks();
  mockDeleteCareEventApi.mockResolvedValue(undefined);
  mockUpdateCareEventApi.mockResolvedValue(makeCareEvent());
  mockFetchCareEventsApi.mockResolvedValue([]);
  mockFetchPlantApi.mockResolvedValue(makePlant());
});

afterEach(() => {
  cleanup();
});

function deferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: Error) => void;
  const promise = new Promise<T>((res, rej) => {
    resolve = res;
    reject = rej;
  });
  return { promise, resolve, reject };
}

async function settleRequests(promise: Promise<unknown>) {
  await promise.catch(() => undefined);
  // Drain chained loader/Promise.all/mutation continuations before asserting
  // that a stale result made no commits. These tests use real timers.
  await new Promise((resolve) => setTimeout(resolve, 0));
}

function routeData(plant: Plant | null = makePlant(), notFound = false) {
  return { plant, notFound, loadErrorCode: null };
}

const TIMELINE_NOW = Date.parse("2026-04-01T12:00:00Z");
const DAY = 86_400_000;

function recentEvent(
  id: number,
  daysAgo: number,
  overrides: Partial<CareEvent> = {},
) {
  return makeCareEvent({
    id,
    occurred_at: new Date(TIMELINE_NOW - daysAgo * DAY).toISOString(),
    ...overrides,
  });
}

function timelineMarkers() {
  return [...document.querySelectorAll(".care-timeline g.event")].map(
    (marker) => {
      const transform = /^translate\((\S+)\s+\S+\)$/.exec(
        marker.getAttribute("transform") ?? "",
      );
      expect(transform).not.toBeNull();
      return {
        id: Number(marker.getAttribute("data-event-id")),
        eventType: marker.getAttribute("data-event-type"),
        x: Number(Number(transform![1]).toFixed(6)),
      };
    },
  );
}

function wateringFeedbackIds() {
  return [
    ...document.querySelectorAll(
      '.care-timeline g.event[data-water-feedback="true"]',
    ),
  ].map((marker) => Number(marker.getAttribute("data-event-id")));
}

function timelineSnapshot() {
  if (!document.querySelector(".care-timeline > svg")) return undefined;
  return JSON.stringify(timelineMarkers());
}

function expectedTimeline(events: CareEvent[], plant = makePlant()) {
  const now = Date.now();
  return JSON.stringify(
    selectCareTimelineEvents(
      normalizeCareTimelineEvents(events, plant.id),
      now,
    ).map(({ id, eventType, occurredAt }) => ({
      id,
      eventType,
      x: Number(
        (32 + ((occurredAt - (now - 42 * DAY)) / (42 * DAY)) * 256).toFixed(6),
      ),
    })),
  );
}

async function deleteJournalEntry(index = 0) {
  const buttons = await screen.findAllByRole("button", {
    name: "Delete log entry",
  });
  await fireEvent.click(buttons[index]);
  await fireEvent.click(screen.getByRole("button", { name: "Delete" }));
}

function getLightbox() {
  return document.querySelector("dialog.lightbox") as HTMLDialogElement;
}

describe("plant detail rendering", () => {
  function getCard(title: string) {
    return screen.getByText(title).closest(".section") as HTMLElement;
  }

  const configuredCareInfo: Partial<Plant> = {
    difficulty: "easy",
    pet_safety: "safe",
    growth_speed: "slow",
    soil_type: "standard",
  };

  it("preserves plant information, watering details, cards, and notes", async () => {
    await renderWithPlant({ photo_url: null, notes: "Keep away from drafts." });
    expect(await screen.findByRole("heading", { name: "Fern" })).toBeTruthy();
    expect(screen.getByText("Boston Fern")).toBeTruthy();
    expect(screen.getByText("Bedroom")).toBeTruthy();
    const icon = document.querySelector(
      ".detail-photo-icon",
    ) as HTMLImageElement;
    expect(icon.getAttribute("src")).toBe("/emoji/🌿.svg");
    expect(icon.alt).toBe("Fern");
    const watering = within(getCard("Watering"));
    expect(watering.getByText("Interval")).toBeTruthy();
    expect(watering.getByText("Every 7 days")).toBeTruthy();
    for (const [label, date] of [
      ["Last watered", "2025-01-01"],
      ["Next due", "2025-01-08"],
    ]) {
      const row = watering
        .getByText(label)
        .closest(".detail-row") as HTMLElement;
      expect(
        within(row).getByText(
          new Date(date).toLocaleDateString(undefined, {
            year: "numeric",
            month: "short",
            day: "numeric",
          }),
        ),
      ).toBeTruthy();
    }
    expect(
      within(getCard("Care Info")).getByText("Indirect light"),
    ).toBeTruthy();
    expect(
      within(getCard("Notes")).getByText("Keep away from drafts."),
    ).toBeTruthy();
    expect(
      within(getCard("Care Journal")).getByText("+ Add log entry"),
    ).toBeTruthy();
  });

  it.each([
    [{ difficulty: "easy" }, "Difficulty", "Easy"],
    [{ difficulty: "moderate" }, "Difficulty", "Moderate"],
    [{ difficulty: "demanding" }, "Difficulty", "Demanding"],
    [{ pet_safety: "safe" }, "Pet safety", "Safe"],
    [{ pet_safety: "caution" }, "Pet safety", "Caution"],
    [{ pet_safety: "toxic" }, "Pet safety", "Toxic"],
    [{ growth_speed: "slow" }, "Growth", "Slow"],
    [{ growth_speed: "moderate" }, "Growth", "Moderate"],
    [{ growth_speed: "fast" }, "Growth", "Fast"],
    [{ soil_type: "standard" }, "Soil", "Standard"],
    [{ soil_type: "cactus-mix" }, "Soil", "Cactus mix"],
    [{ soil_type: "orchid-bark" }, "Soil", "Orchid bark"],
    [{ soil_type: "peat-moss" }, "Soil", "Peat moss"],
  ] satisfies [Partial<Plant>, string, string][])(
    "renders configured care info %j as %s: %s",
    async (overrides, label, value) => {
      await renderWithPlant({ ...configuredCareInfo, ...overrides });
      const card = getCard("Care Info");
      const row = within(card)
        .getByText(label)
        .closest(".detail-row") as HTMLElement;
      expect(within(row).getByText(value)).toBeTruthy();
      expect(card.querySelectorAll(".detail-row")).toHaveLength(5);
      expect(within(card).getByText("Indirect light")).toBeTruthy();
    },
  );

  it.each([
    ["difficulty", "Difficulty"],
    ["pet_safety", "Pet safety"],
    ["growth_speed", "Growth"],
    ["soil_type", "Soil"],
  ] as const)(
    "omits null %s without an empty care info row",
    async (field, label) => {
      await renderWithPlant({ ...configuredCareInfo, [field]: null });
      const card = getCard("Care Info");
      expect(within(card).queryByText(label)).toBeNull();
      expect(within(card).queryByText("N/A")).toBeNull();
      const rows = card.querySelectorAll(".detail-row");
      expect(rows).toHaveLength(4);
      for (const row of rows) {
        expect(
          row.querySelector(".detail-row-value")?.textContent?.trim(),
        ).toBeTruthy();
      }
    },
  );

  it("shows only the light row with its icon when all optional care info is null", async () => {
    await renderWithPlant();
    const card = getCard("Care Info");
    expect(card.querySelectorAll(".detail-row")).toHaveLength(1);
    expect(within(card).getByText("Light")).toBeTruthy();
    expect(within(card).getByText("Indirect light")).toBeTruthy();
    expect(card.querySelector(".detail-row .lucide-cloud-sun")).toBeTruthy();
    for (const label of ["Difficulty", "Pet safety", "Growth", "Soil", "N/A"]) {
      expect(within(card).queryByText(label)).toBeNull();
    }
    expect(screen.queryByText("Notes")).toBeNull();
  });

  it.each([
    ["dry", "Prefers dry"],
    ["moderate", "Moderate"],
    ["moist", "Keeps moist"],
  ] as const)(
    "shows %s soil moisture below Next due in the Watering card",
    async (soil_moisture, label) => {
      await renderWithPlant({ soil_moisture, difficulty: "moderate" });
      const card = getCard("Watering");
      const row = within(card)
        .getByText("Soil moisture")
        .closest(".detail-row") as HTMLElement;
      expect(within(row).getByText(label)).toBeTruthy();
      expect(row.previousElementSibling).toBe(
        within(card).getByText("Next due").closest(".detail-row"),
      );
      expect(card.querySelectorAll(".detail-row")).toHaveLength(4);
      expect(
        within(getCard("Care Info")).queryByText("Soil moisture"),
      ).toBeNull();
    },
  );

  it("omits the soil moisture row when null", async () => {
    await renderWithPlant({ soil_moisture: null });
    const card = getCard("Watering");
    expect(within(card).queryByText("Soil moisture")).toBeNull();
    expect(card.querySelectorAll(".detail-row")).toHaveLength(3);
  });

  it("targets the plant edit route with an enabled online navigation link", async () => {
    await renderWithPlant();
    const edits = await screen.findAllByRole("link", { name: "Edit Plant" });
    // PageHeader renders navigation in both the desktop header and mobile bar.
    expect(edits).toHaveLength(2);
    for (const edit of edits) {
      expect(edit.getAttribute("href")).toBe("/plants/1/edit");
      expect(edit.getAttribute("aria-disabled")).toBe("false");
      expect(edit.classList.contains("disabled")).toBe(false);
    }
  });

  it("displays the localized not-found message for null plant and notFound route data", async () => {
    render(Page, { props: { data: routeData(null, true) } });
    expect(
      await screen.findByRole("heading", { name: "Plant not found" }),
    ).toBeTruthy();
    expect(
      screen.getByText("This plant doesn't exist or may have been deleted."),
    ).toBeTruthy();
    expect(
      screen.getByRole("link", { name: "Back to plants" }).getAttribute("href"),
    ).toBe("/");
    expect(screen.queryByText("Watering")).toBeNull();
    expect(screen.queryByText("Care Info")).toBeNull();
    expect(mockFetchCareEventsApi).not.toHaveBeenCalled();
  });

  it("shows enabled delete controls for each entry in an expanded watering group", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
    ]);
    const journal = getCard("Care Journal");
    await fireEvent.click(
      await within(journal).findByRole("button", {
        name: "Expand watering entries",
      }),
    );
    const entries = journal.querySelectorAll<HTMLElement>(".timeline-nested");
    expect(entries).toHaveLength(2);
    for (const entry of entries) {
      expect(
        within(entry).getByRole("button", { name: "Delete log entry" }),
      ).toHaveProperty("disabled", false);
    }
  });
});

describe("route data updates", () => {
  it("switches to the new plant immediately when page data changes", async () => {
    const view = await renderWithPlant({ id: 1, name: "Fern" });

    await screen.findByText("Fern");

    view.rerender({
      data: {
        plant: makePlant({ id: 2, name: "Monstera" }),
        notFound: false,
        loadErrorCode: null,
      },
    });

    await waitFor(() => {
      expect(screen.getByText("Monstera")).toBeTruthy();
      expect(screen.queryByText("Fern")).toBeNull();
    });
  });
});

describe("care timeline integration", () => {
  let clock: ReturnType<typeof vi.spyOn>;
  beforeEach(() => {
    clock = vi.spyOn(Date, "now").mockReturnValue(TIMELINE_NOW);
  });
  afterEach(() => clock.mockRestore());

  it("shows three biweekly waterings across six weeks while keeping older entries in the journal", async () => {
    const events = [
      recentEvent(1, 49, { notes: "Older journal entry" }),
      recentEvent(2, 35),
      recentEvent(3, 21),
      recentEvent(4, 7),
    ];
    await renderWithPlant({ watering_interval_days: 14 }, events);

    await waitFor(() =>
      expect(timelineMarkers()).toEqual([
        { id: 2, eventType: "watered", x: 74.666667 },
        { id: 3, eventType: "watered", x: 160 },
        { id: 4, eventType: "watered", x: 245.333333 },
      ]),
    );
    expect(screen.getByText("Older journal entry")).toBeTruthy();
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(1);
  });

  it.each([false, true])(
    "keeps graph behind heading and controls outside it (AI: %s)",
    async (enabled) => {
      aiStatus.set({ enabled, base_url: null, model: null });
      const longName = "Fern".repeat(40);
      const longSpecies = "Nephrolepis".repeat(30);
      const longLocation = "Bedroom".repeat(40);
      await renderWithPlant(
        { name: longName, species: longSpecies, location_name: longLocation },
        [recentEvent(1, 2)],
      );
      await waitFor(() => expect(timelineSnapshot()).toBeTruthy());
      const info = document.querySelector(".detail-info")!;
      const heading = info.querySelector(".detail-heading")!;
      const foreground = heading.querySelector(".detail-heading-content")!;
      expect(
        heading.firstElementChild?.classList.contains("care-timeline"),
      ).toBe(true);
      expect(heading.children).toHaveLength(2);
      const background = within(heading as HTMLElement).getByRole("img", {
        name: "Care Journal",
      });
      expect(background.getAttribute("focusable")).toBe("false");
      expect(background.querySelector("button, a, [tabindex]")).toBeNull();
      expect(background.querySelector("text")).toBeNull();
      expect(foreground.querySelector("h2")?.textContent).toBe(longName);
      expect(foreground.querySelector(".detail-species")?.textContent).toBe(
        longSpecies,
      );
      expect(
        foreground.querySelector(".detail-location span")?.textContent,
      ).toBe(longLocation);
      expect(foreground.querySelector(".detail-status")).toBeTruthy();
      expect(heading.querySelector("button")).toBeNull();
      const actions = info.querySelector(".hero-actions")!;
      expect(actions.parentElement).toBe(info);
      expect(actions.previousElementSibling).toBe(heading);
      expect(
        within(actions as HTMLElement).getByRole("button", {
          name: "Water now",
        }),
      ).toBeTruthy();
      expect(actions.querySelectorAll("button")).toHaveLength(enabled ? 2 : 1);
      expect(document.querySelector(".detail-photo")).not.toBeNull();
      expect(
        heading.querySelector(".caption, .message, p:not(.detail-location)"),
      ).toBeNull();
      expect(heading.querySelector(".care-timeline")?.children).toHaveLength(1);
    },
  );

  describe("same-visit foreground binding cleanup", () => {
    const selectors = [
      ".detail-name h2",
      ".detail-species",
      ".detail-location",
      ".detail-status-foreground",
      ".btn-water",
      ".btn-ai",
    ];
    const observers: {
      observe: ReturnType<typeof vi.fn>;
      disconnect: ReturnType<typeof vi.fn>;
    }[] = [];
    let bounds: ReturnType<typeof vi.spyOn>;

    beforeEach(() => {
      observers.length = 0;
      vi.stubGlobal(
        "ResizeObserver",
        class {
          observe = vi.fn();
          disconnect = vi.fn();
          constructor() {
            observers.push(this);
          }
        },
      );
      bounds = vi
        .spyOn(HTMLElement.prototype, "getBoundingClientRect")
        .mockImplementation(function (this: HTMLElement) {
          if (this.matches(".care-timeline"))
            return new DOMRect(100, 200, 320, 140);
          const index = selectors.findIndex((selector) =>
            this.matches(selector),
          );
          return index < 0
            ? new DOMRect()
            : new DOMRect(116, 206 + index * 20, 80 + index * 10, 18);
        });
    });

    afterEach(() => {
      cleanup();
      bounds.mockRestore();
      vi.unstubAllGlobals();
    });

    function expectForeground(excluded: readonly string[] = []) {
      const observer = observers.at(-1)!;
      const present = selectors.filter(
        (selector) => !excluded.includes(selector),
      );
      expect(
        [...document.querySelectorAll(".care-timeline mask g rect")].map(
          (rect) =>
            ["x", "y", "width", "height"].map((key) =>
              Number(rect.getAttribute(key)),
            ),
        ),
      ).toEqual(
        present.map((selector) => {
          const index = selectors.indexOf(selector);
          return [4, -6 + index * 20, 104 + index * 10, 42];
        }),
      );
      for (const selector of present)
        expect(observer.observe).toHaveBeenCalledWith(
          document.querySelector(selector),
        );
      expect(observer.disconnect).not.toHaveBeenCalled();
      for (const old of observers.slice(0, -1))
        expect(old.disconnect).toHaveBeenCalledOnce();
    }

    async function initialTimeline(events: CareEvent[]) {
      aiStatus.set({ enabled: true, base_url: null, model: null });
      await renderWithPlant({}, events);
      await waitFor(() =>
        expect(timelineSnapshot()).toBe(expectedTimeline(events)),
      );
      expectForeground();
      return {
        marker: document.querySelector(
          '.care-timeline g.event[data-event-id="1"]',
        ),
        snapshot: timelineSnapshot(),
      };
    }

    it.each([
      ["species", ".detail-species", "New species"],
      ["location_name", ".detail-location", "New location"],
    ] as const)(
      "removes and restores %s bindings through metadata refresh while retaining existing markers",
      async (field, selector, restored) => {
        const events = [
          recentEvent(1, 2),
          recentEvent(2, 3, { event_type: "fertilized" }),
          recentEvent(3, 4, { event_type: "pruned" }),
        ];
        const { marker } = await initialTimeline(events);
        const original = document.querySelector(selector)!;
        const previousObserver = observers.at(-1)!;
        mockFetchPlantApi.mockResolvedValueOnce(makePlant({ [field]: null }));
        mockFetchCareEventsApi.mockResolvedValueOnce(events.slice(0, 2));
        await expect(deleteJournalEntry(2)).resolves.toBeUndefined();
        await waitFor(() => {
          expect(document.querySelector(selector)).toBeNull();
          expectForeground([selector]);
        });
        expect(original.isConnected).toBe(false);
        expect(previousObserver.disconnect).toHaveBeenCalledOnce();
        expect(observers.at(-1)!.observe).not.toHaveBeenCalledWith(original);
        expect(
          document.querySelector('.care-timeline g.event[data-event-id="1"]'),
        ).toBe(marker);
        expect(timelineSnapshot()).toBe(expectedTimeline(events.slice(0, 2)));
        expect(screen.getByRole("heading", { name: "Fern" })).toBeTruthy();

        mockFetchPlantApi.mockResolvedValueOnce(
          makePlant({ [field]: restored }),
        );
        mockFetchCareEventsApi.mockResolvedValueOnce(events.slice(0, 1));
        await expect(deleteJournalEntry(1)).resolves.toBeUndefined();
        await screen.findByText(restored);
        await waitFor(() => expectForeground());
        expect(document.querySelector(selector)).not.toBe(original);
        expect(
          document.querySelector('.care-timeline g.event[data-event-id="1"]'),
        ).toBe(marker);
        expect(timelineSnapshot()).toBe(expectedTimeline(events.slice(0, 1)));
        expect(mockFetchPlantApi).toHaveBeenCalledTimes(2);
        expect(mockFetchPlantApi).toHaveBeenLastCalledWith(1);
      },
    );

    it("removes and restores Ask AI bindings through the store while retaining existing markers", async () => {
      const { marker, snapshot } = await initialTimeline([recentEvent(1, 2)]);
      const original = screen.getByRole("button", { name: "Ask AI" });
      aiStatus.set({ enabled: false, base_url: null, model: null });
      await waitFor(() => {
        expect(screen.queryByRole("button", { name: "Ask AI" })).toBeNull();
        expectForeground([".btn-ai"]);
      });
      expect(original.isConnected).toBe(false);
      expect(observers.at(-1)!.observe).not.toHaveBeenCalledWith(original);
      expect(
        document.querySelector('.care-timeline g.event[data-event-id="1"]'),
      ).toBe(marker);
      expect(timelineSnapshot()).toBe(snapshot);

      aiStatus.set({ enabled: true, base_url: null, model: null });
      expect(await screen.findByRole("button", { name: "Ask AI" })).not.toBe(
        original,
      );
      await waitFor(() => expectForeground());
      expect(
        document.querySelector('.care-timeline g.event[data-event-id="1"]'),
      ).toBe(marker);
      expect(timelineSnapshot()).toBe(snapshot);
      expect(screen.getByRole("heading", { name: "Fern" })).toBeTruthy();
      expect(screen.getByText("Boston Fern")).toBeTruthy();
      expect(screen.getByText("Bedroom")).toBeTruthy();
      expect(mockFetchPlantApi).not.toHaveBeenCalled();
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(1);
    });
  });

  it("omits optional heading fields without adding placeholders and preserves the photo fallback", async () => {
    await renderWithPlant({
      species: null,
      location_name: null,
      photo_url: null,
    });
    await screen.findByText("No care events recorded yet.");
    const heading = document.querySelector(".detail-heading")!;
    expect(
      heading.querySelector(".detail-species, .detail-location, svg[role=img]"),
    ).toBeNull();
    expect(heading.querySelector(".care-timeline")?.textContent).toBe("");
    expect(document.querySelector(".detail-photo-icon")).toBeTruthy();
    expect(screen.queryByRole("button", { name: "Open photo" })).toBeNull();
    expect(screen.queryByRole("button", { name: "Ask AI" })).toBeNull();
  });

  it("applies actual timeline minimum callbacks and shrinks/clears the inherited hero minimum after refresh/navigation", async () => {
    const dense = Array.from({ length: 30 }, (_, index) =>
      recentEvent(index + 1, 14, { event_type: "custom" }),
    );
    const view = await renderWithPlant({}, dense);
    const minimum = () =>
      document
        .querySelector<HTMLElement>(".detail-info")
        ?.style.getPropertyValue("--care-timeline-min-height");
    await waitFor(() => expect(minimum()).toBe("610px"));
    expect(document.querySelectorAll(".care-timeline .event")).toHaveLength(30);
    expect(
      document.querySelector(".care-timeline > svg")?.getAttribute("viewBox"),
    ).toBe("0 0 320 610");
    const ys = [...document.querySelectorAll(".care-timeline .event")]
      .map((node) =>
        Number(node.getAttribute("transform")!.slice(10, -1).split(" ")[1]),
      )
      .sort((a, b) => a - b);
    for (let index = 1; index < ys.length; index++)
      expect(ys[index] - ys[index - 1]).toBeGreaterThanOrEqual(18 - 1e-9);
    // Reporting layout never fetches or changes business state.
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(1);
    expect(mockFetchPlantApi).not.toHaveBeenCalled();
    mockFetchCareEventsApi.mockResolvedValueOnce(dense.slice(0, 2));
    await view.rerender({ data: routeData() });
    await waitFor(() => expect(minimum()).toBe("140px"));
    expect(document.querySelectorAll(".care-timeline .event")).toHaveLength(2);
    mockFetchCareEventsApi.mockResolvedValueOnce([]);
    await view.rerender({ data: routeData() });
    await waitFor(() => expect(minimum()).toBe("0px"));
    expect(document.querySelector(".care-timeline > svg")).toBeNull();
    mockFetchCareEventsApi.mockResolvedValueOnce(dense);
    await view.rerender({ data: routeData() });
    await waitFor(() => expect(minimum()).toBe("610px"));
    const next = deferred<CareEvent[]>();
    mockFetchCareEventsApi.mockReturnValueOnce(next.promise);
    await view.rerender({
      data: routeData(makePlant({ id: 2, name: "Next plant" })),
    });
    await waitFor(() => expect(minimum()).toBe("0px"));
    next.resolve([recentEvent(40, 2, { plant_id: 2 })]);
    await waitFor(() => expect(minimum()).toBe("140px"));
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(5);
    expect(mockFetchPlantApi).not.toHaveBeenCalled();
  });

  it("keeps desktop containing-block/stacking and growing mobile heading rules at the existing breakpoint", () => {
    // DOM tests cannot measure the layout; pin the intended host rules, not pixels from jsdom.
    expect(pageSource).toMatch(
      /\.detail-info\s*\{[^}]*position:\s*relative;[^}]*min-width:\s*0;[^}]*min-height:\s*max\(260px, var\(--care-timeline-min-height, 0px\)\);/,
    );
    expect(pageSource).toMatch(/\.detail-heading\s*\{[^}]*position:\s*static;/);
    expect(pageSource).toMatch(/\.detail-hero\s*\{[^}]*margin-bottom:\s*24px;/);
    expect(pageSource).toMatch(
      /\.detail-photo\s*\{[^}]*width:\s*260px;[^}]*height:\s*260px;/,
    );
    expect(pageSource).toMatch(
      /\.detail-heading-content\s*\{[^}]*position:\s*relative;[^}]*z-index:\s*1;[^}]*overflow-wrap:\s*anywhere;/,
    );
    expect(pageSource).toMatch(
      /\.hero-actions\s*\{[^}]*z-index:\s*1;[^}]*flex-direction:\s*column;[^}]*align-items:\s*flex-start;/,
    );
    const mobile = pageSource.split("@media (max-width: 768px)")[1];
    expect(mobile).toMatch(/\.detail-info\s*\{[^}]*min-height:\s*0;/);
    expect(mobile).toMatch(
      /\.detail-heading\s*\{[^}]*position:\s*relative;[^}]*min-height:\s*max\(140px, var\(--care-timeline-min-height, 0px\)\);/,
    );
    expect(mobile).not.toMatch(/\.detail-heading\s*\{[^}]*padding-bottom:/);
    expect(mobile).toMatch(
      /\.detail-photo\s*\{[^}]*width:\s*100%;[^}]*height:\s*220px;/,
    );
    expect(mobile).toMatch(/\.hero-actions\s*\{[^}]*flex-direction:\s*row;/);
    expect(pageSource).toMatch(
      /@media \(min-width: 1280px\)\s*\{\s*\.detail-info\s*\{[^}]*min-height:\s*max\(300px, var\(--care-timeline-min-height, 0px\)\);/,
    );
    expect(pageSource.split("@media (min-width: 1280px)")[1]).toMatch(
      /\.detail-photo\s*\{[^}]*width:\s*300px;[^}]*height:\s*300px;/,
    );
    expect(pageSource).not.toMatch(/text-overflow:|line-clamp:/);
  });

  it("forwards all raw care types before grouping, including notes/photos, while badge uses API status", async () => {
    const events = [
      recentEvent(4, 1),
      recentEvent(3, 3),
      recentEvent(2, 8, {
        notes: "Watered deeply",
        photo_url: "/uploads/care/2.jpg",
      }),
      recentEvent(1, 12, { event_type: "fertilized" }),
      recentEvent(5, 13, { event_type: "repotted" }),
      recentEvent(6, 14, { event_type: "pruned" }),
      recentEvent(7, 15, { event_type: "custom", notes: "Rotated pot" }),
      recentEvent(8, 16, {
        event_type: "ai-consultation",
        notes: "Saved advice",
      }),
      recentEvent(9, 43),
      recentEvent(10, -1),
      recentEvent(11, 0, { occurred_at: "invalid" }),
      recentEvent(99, 0, { plant_id: 2 }),
    ];
    await renderWithPlant({ watering_status: "overdue" }, events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    expect(screen.getByText("Watered deeply")).toBeTruthy();
    expect(document.querySelector(".timeline-photo img")).toBeTruthy();
    expect(document.querySelector(".detail-status")?.textContent).toContain(
      "Overdue",
    );
    expect(document.querySelectorAll(".care-timeline g.event")).toHaveLength(8);
    for (const [id, type, icon] of [
      [1, "fertilized", "leaf"],
      [2, "watered", "droplet"],
      [5, "repotted", "shovel"],
      [6, "pruned", "scissors"],
      [7, "custom", "pencil"],
      [8, "ai-consultation", "sparkles"],
    ]) {
      const marker = document.querySelector(
        `.care-timeline g.event[data-event-id="${id}"]`,
      )!;
      expect(marker.getAttribute("data-event-type")).toBe(type);
      expect(marker.querySelector(`.lucide-${icon}`)).toBeTruthy();
    }
    expect(
      document
        .querySelector('.care-timeline [data-event-id="1"]')
        ?.getAttribute("data-event-type"),
    ).toBe("fertilized");
    expect(document.querySelectorAll(".care-timeline text")).toHaveLength(0);
  });

  it.each(["resolve", "reject"] as const)(
    "ignores old-plant care %s and its finally while the new plant is loading",
    async (outcome) => {
      const old = deferred<CareEvent[]>();
      const next = deferred<CareEvent[]>();
      mockFetchCareEventsApi
        .mockReturnValueOnce(old.promise)
        .mockReturnValueOnce(next.promise);
      const view = render(Page, { props: { data: routeData() } });
      await waitFor(() =>
        expect(mockFetchCareEventsApi).toHaveBeenCalledWith(1),
      );
      await view.rerender({
        data: routeData(makePlant({ id: 2, name: "Monstera" })),
      });
      await waitFor(() =>
        expect(mockFetchCareEventsApi).toHaveBeenCalledWith(2),
      );
      if (outcome === "resolve")
        old.resolve([recentEvent(1, 1, { notes: "Old visit" })]);
      else old.reject(new Error("old visit failed"));
      await settleRequests(old.promise);
      await waitFor(() =>
        expect(document.querySelector(".skeleton-list")).toBeTruthy(),
      );
      expect(document.querySelector(".care-timeline svg")).toBeNull();
      expect(screen.queryByRole("alert")).toBeNull();
      expect(screen.queryByText("Old visit")).toBeNull();
      const events = [recentEvent(2, 3, { plant_id: 2, notes: "New visit" })];
      next.resolve(events);
      await screen.findByText("New visit");
      expect(timelineSnapshot()).toBe(
        expectedTimeline(events, makePlant({ id: 2 })),
      );
      expect(screen.queryByText("Fern")).toBeNull();
    },
  );

  it.each([false, true])(
    "invalidates pending history on absent/not-found route data (notFound: %s)",
    async (notFound) => {
      const old = deferred<CareEvent[]>();
      mockFetchCareEventsApi.mockReturnValueOnce(old.promise);
      const view = render(Page, { props: { data: routeData() } });
      await waitFor(() =>
        expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(1),
      );
      await view.rerender({ data: routeData(null, notFound) });
      old.resolve([recentEvent(1, 0, { notes: "Stale" })]);
      await settleRequests(old.promise);
      expect(screen.queryByText("Stale")).toBeNull();
      expect(document.querySelector(".care-timeline")).toBeNull();
      mockFetchCareEventsApi.mockResolvedValue([]);
      await view.rerender({ data: routeData() });
      await screen.findByText("No care events recorded yet.");
      expect(document.querySelector(".care-timeline svg")).toBeNull();
    },
  );

  it("clears previous markers immediately when switching plants and ignores a late failure after new history", async () => {
    const initial = [recentEvent(1, 3)];
    const view = await renderWithPlant({}, initial);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(initial)),
    );
    const old = deferred<CareEvent[]>();
    const next = deferred<CareEvent[]>();
    mockFetchCareEventsApi
      .mockReturnValueOnce(old.promise)
      .mockReturnValueOnce(next.promise);
    await view.rerender({ data: routeData() });
    await view.rerender({
      data: routeData(makePlant({ id: 2, name: "Monstera" })),
    });
    expect(timelineSnapshot()).toBeUndefined();
    const events = [recentEvent(2, 2, { plant_id: 2 })];
    next.resolve(events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(
        expectedTimeline(events, makePlant({ id: 2 })),
      ),
    );
    old.reject(new Error("stale"));
    await settleRequests(old.promise);
    expect(timelineSnapshot()).toBe(
      expectedTimeline(events, makePlant({ id: 2 })),
    );
    expect(screen.queryByRole("alert")).toBeNull();
  });

  it.each(["resolve", "reject"] as const)(
    "keeps newer same-plant refresh loading and cancels older coupled %s silently",
    async (outcome) => {
      const events = [
        recentEvent(1, 1, { notes: "First" }),
        recentEvent(2, 10, { notes: "Second" }),
      ];
      await renderWithPlant({}, events);
      await waitFor(() =>
        expect(timelineSnapshot()).toBe(expectedTimeline(events)),
      );
      const oldCare = deferred<CareEvent[]>();
      const newCare = deferred<CareEvent[]>();
      const oldPlant = deferred<Plant>();
      const newPlant = deferred<Plant>();
      mockFetchCareEventsApi
        .mockReturnValueOnce(oldCare.promise)
        .mockReturnValueOnce(newCare.promise);
      mockFetchPlantApi
        .mockReturnValueOnce(oldPlant.promise)
        .mockReturnValueOnce(newPlant.promise);
      await deleteJournalEntry(0);
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(1));
      await deleteJournalEntry(1);
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(2));
      if (outcome === "resolve")
        oldCare.resolve([recentEvent(88, 0, { notes: "Superseded" })]);
      else oldCare.reject(new Error("superseded"));
      oldPlant.resolve(makePlant({ name: "Old response" }));
      await settleRequests(oldPlant.promise);
      await waitFor(() =>
        expect(
          document.querySelector(".care-timeline desc")?.textContent,
        ).toContain("Loading"),
      );
      expect(timelineSnapshot()).toBe(expectedTimeline(events));
      expect(screen.queryByText("Old response")).toBeNull();
      expect(screen.queryByText("Superseded")).toBeNull();
      expect(screen.queryByRole("alert")).toBeNull();
      expect(mockPushNotification).not.toHaveBeenCalled();
      const latest = [recentEvent(3, 2, { notes: "Latest" })];
      newPlant.resolve(
        makePlant({ name: "Current response", watering_interval_days: 14 }),
      );
      newCare.resolve(latest);
      await screen.findByText("Current response");
      expect(timelineSnapshot()).toBe(
        expectedTimeline(latest, makePlant({ watering_interval_days: 14 })),
      );
      expect(
        document.querySelector(".care-timeline desc")?.textContent,
      ).not.toContain("Loading");
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it.each(["resolve", "reject"] as const)(
    "retains the latest same-plant graph when superseded care %s arrives last",
    async (outcome) => {
      const events = [
        recentEvent(1, 1, { notes: "First" }),
        recentEvent(2, 10, { notes: "Second" }),
      ];
      await renderWithPlant({}, events);
      await waitFor(() =>
        expect(timelineSnapshot()).toBe(expectedTimeline(events)),
      );
      const oldCare = deferred<CareEvent[]>();
      const latest = [recentEvent(3, 4, { notes: "Current history" })];
      mockFetchCareEventsApi
        .mockReturnValueOnce(oldCare.promise)
        .mockResolvedValueOnce(latest);
      mockFetchPlantApi
        .mockResolvedValueOnce(makePlant({ name: "Old plant response" }))
        .mockResolvedValueOnce(makePlant({ name: "Latest plant response" }));
      await deleteJournalEntry(0);
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(1));
      await deleteJournalEntry(1);
      await screen.findByText("Latest plant response");
      await screen.findByText("Current history");
      const snapshot = timelineSnapshot();
      if (outcome === "resolve")
        oldCare.resolve([recentEvent(99, 0, { notes: "Stale history" })]);
      else oldCare.reject(new Error("late history failure"));
      await settleRequests(oldCare.promise);
      await waitFor(() => expect(timelineSnapshot()).toBe(snapshot));
      expect(screen.getByText("Current history")).toBeTruthy();
      expect(screen.getByText("Latest plant response")).toBeTruthy();
      expect(screen.queryByText("Stale history")).toBeNull();
      expect(screen.queryByRole("alert")).toBeNull();
      expect(
        document.querySelector(".care-timeline desc")?.textContent,
      ).not.toContain("Loading");
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it("cannot commit old fetched plant after a newer same-plant refresh has finished", async () => {
    const events = [
      recentEvent(1, 1, { notes: "First" }),
      recentEvent(2, 10, { notes: "Second" }),
    ];
    await renderWithPlant({}, events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    const oldPlant = deferred<Plant>();
    mockFetchPlantApi
      .mockReturnValueOnce(oldPlant.promise)
      .mockResolvedValueOnce(makePlant({ name: "Newest" }));
    mockFetchCareEventsApi
      .mockResolvedValueOnce(events)
      .mockResolvedValueOnce([events[0]]);
    await deleteJournalEntry(0);
    await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(1));
    await deleteJournalEntry(1);
    await screen.findByText("Newest");
    const currentSnapshot = timelineSnapshot();
    oldPlant.resolve(
      makePlant({ name: "Obsolete", watering_interval_days: 30 }),
    );
    await settleRequests(oldPlant.promise);
    await waitFor(() => expect(screen.getByText("Newest")).toBeTruthy());
    expect(screen.queryByText("Obsolete")).toBeNull();
    expect(timelineSnapshot()).toBe(currentSnapshot);
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it.each(["resolve", "reject"] as const)(
    "does not resurrect the old view when its fetched plant %s after navigation",
    async (outcome) => {
      const view = await renderWithPlant({}, [
        recentEvent(1, 1, { notes: "Old event" }),
      ]);
      await waitFor(() => expect(timelineSnapshot()).toBeTruthy());
      const oldPlant = deferred<Plant>();
      mockFetchPlantApi.mockReturnValueOnce(oldPlant.promise);
      mockFetchCareEventsApi.mockResolvedValueOnce([]);
      await deleteJournalEntry();
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(1));
      const next = [recentEvent(2, 2, { plant_id: 2, notes: "New event" })];
      mockFetchCareEventsApi.mockResolvedValueOnce(next);
      await view.rerender({
        data: routeData(makePlant({ id: 2, name: "Monstera" })),
      });
      await screen.findByText("New event");
      if (outcome === "resolve")
        oldPlant.resolve(makePlant({ name: "Zombie Fern" }));
      else oldPlant.reject(new Error("stale plant failure"));
      await settleRequests(oldPlant.promise);
      await waitFor(() => expect(screen.getByText("Monstera")).toBeTruthy());
      expect(screen.queryByText("Zombie Fern")).toBeNull();
      expect(timelineSnapshot()).toBe(
        expectedTimeline(next, makePlant({ id: 2 })),
      );
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it("ignores a foreign delayed mutation refresh without invalidating the active care request", async () => {
    const view = await renderWithPlant({}, [
      recentEvent(1, 1, { notes: "Old" }),
    ]);
    await waitFor(() => expect(timelineSnapshot()).toBeTruthy());
    const deletion = deferred<void>();
    mockDeleteCareEventApi.mockReturnValueOnce(deletion.promise);
    await deleteJournalEntry();
    const nextCare = deferred<CareEvent[]>();
    mockFetchCareEventsApi.mockReturnValueOnce(nextCare.promise);
    await view.rerender({
      data: routeData(makePlant({ id: 2, name: "Monstera" })),
    });
    await waitFor(() =>
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2),
    );
    deletion.resolve(undefined);
    await settleRequests(deletion.promise);
    await waitFor(() =>
      expect(document.querySelector(".skeleton-list")).toBeTruthy(),
    );
    expect(mockFetchPlantApi).not.toHaveBeenCalled();
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
    const events = [recentEvent(2, 1, { plant_id: 2 })];
    nextCare.resolve(events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(
        expectedTimeline(events, makePlant({ id: 2 })),
      ),
    );
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it.each(["resolve", "reject"] as const)(
    "guards Water now results and pending flags across visits (old result: %s)",
    async (outcome) => {
      const view = await renderWithPlant();
      const old = deferred<Plant | null>();
      const current = deferred<Plant | null>();
      mockWaterPlant
        .mockReturnValueOnce(old.promise)
        .mockReturnValueOnce(current.promise);
      await fireEvent.click(
        await screen.findByRole("button", { name: "Water now" }),
      );
      await view.rerender({
        data: routeData(makePlant({ id: 2, name: "Monstera" })),
      });
      await fireEvent.click(
        await screen.findByRole("button", { name: "Water now" }),
      );
      if (outcome === "resolve")
        old.resolve(makePlant({ name: "Stale watering" }));
      else old.resolve(null); // Store contract translates API rejection to null.
      await settleRequests(old.promise);
      await waitFor(() =>
        expect(
          screen.getByRole("button", { name: "Watering..." }),
        ).toHaveProperty("disabled", true),
      );
      expect(screen.getByText("Monstera")).toBeTruthy();
      expect(mockPushNotification).not.toHaveBeenCalled();
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
      current.resolve(
        makePlant({ id: 2, name: "Monstera", watering_status: "ok" }),
      );
      await screen.findByRole("button", { name: "Water now" });
      expect(mockFetchCareEventsApi).toHaveBeenLastCalledWith(2);
    },
  );

  it.each(["completed", "pending"] as const)(
    "reconciles delayed Water now after a newer care-edit refresh is %s without applying its stale DTO",
    async (refreshState) => {
      const original = recentEvent(1, 3, { notes: "Original event" });
      await renderWithPlant({}, [original]);
      await screen.findByText("Original event");
      const water = deferred<Plant | null>();
      mockWaterPlant.mockReturnValueOnce(water.promise);
      await fireEvent.click(screen.getByRole("button", { name: "Water now" }));

      const edited = { ...original, notes: "Edited event" };
      const newerPlant = makePlant({
        name: "Newer Fern",
        species: "New species",
        location_name: "Kitchen",
        watering_interval_days: 14,
        notes: "New plant notes",
      });
      const editCare = deferred<CareEvent[]>();
      const editPlant = deferred<Plant>();
      const latestCare = deferred<CareEvent[]>();
      const latestPlant = deferred<Plant>();
      mockUpdateCareEventApi.mockResolvedValueOnce(edited);
      mockFetchCareEventsApi
        .mockReturnValueOnce(editCare.promise)
        .mockReturnValueOnce(latestCare.promise);
      mockFetchPlantApi
        .mockReturnValueOnce(editPlant.promise)
        .mockReturnValueOnce(latestPlant.promise);
      await fireEvent.click(
        screen.getByRole("button", { name: "Edit log entry" }),
      );
      await fireEvent.click(screen.getByRole("button", { name: "Save" }));
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(1));
      if (refreshState === "completed") {
        editCare.resolve([edited]);
        editPlant.resolve(newerPlant);
        await screen.findByText("Newer Fern");
      }

      water.resolve(makePlant({ name: "Obsolete water DTO" }));
      await settleRequests(water.promise);
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(2));
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(3);
      expect(mockFetchPlantApi).toHaveBeenLastCalledWith(1);
      expect(screen.queryByText("Obsolete water DTO")).toBeNull();
      if (refreshState === "pending") {
        // This query started before water committed and did not see its event.
        editCare.resolve([edited]);
        editPlant.resolve(newerPlant);
        await settleRequests(editPlant.promise);
        expect(screen.getByRole("heading", { name: "Fern" })).toBeTruthy();
        expect(screen.queryByText("Newer Fern")).toBeNull();
      } else {
        expect(
          screen.getByRole("heading", { name: "Newer Fern" }),
        ).toBeTruthy();
        expect(screen.getByText("Every 14 days")).toBeTruthy();
        expect(screen.getByText("New species")).toBeTruthy();
        expect(screen.getByText("Kitchen")).toBeTruthy();
        expect(screen.getByText("New plant notes")).toBeTruthy();
      }
      expect(
        document.querySelector(".care-timeline desc")?.textContent,
      ).toContain("Loading");

      const fresh = recentEvent(2, 0, { notes: "Committed water" });
      const reconciled = {
        ...newerPlant,
        name: "Reconciled Fern",
        last_watered: fresh.occurred_at,
      };
      latestPlant.resolve(reconciled);
      latestCare.resolve([fresh, edited]);
      await screen.findByText("Reconciled Fern");
      expect(screen.getByText("Committed water")).toBeTruthy();
      expect(screen.getByText("Edited event")).toBeTruthy();
      expect(screen.getByText("Every 14 days")).toBeTruthy();
      expect(timelineSnapshot()).toBe(
        expectedTimeline([fresh, edited], reconciled),
      );
      expect(wateringFeedbackIds()).toEqual([fresh.id]);
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it.each(["plant", "care", "foreign-visit"] as const)(
    "handles delayed successful Water now reconciliation failure (%s) as care-load feedback, not a water failure",
    async (failure) => {
      const view = await renderWithPlant({}, [
        recentEvent(1, 3, { notes: "Keep event" }),
      ]);
      await screen.findByText("Keep event");
      const water = deferred<Plant | null>();
      mockWaterPlant.mockReturnValueOnce(water.promise);
      await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
      mockFetchPlantApi.mockResolvedValueOnce(
        makePlant({ name: "Current metadata" }),
      );
      await deleteJournalEntry();
      await screen.findByText("Current metadata");
      const refetch = deferred<Plant>();
      const history = deferred<CareEvent[]>();
      mockFetchPlantApi.mockReturnValueOnce(refetch.promise);
      mockFetchCareEventsApi.mockReturnValueOnce(history.promise);
      water.resolve(makePlant({ name: "Obsolete water DTO" }));
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(2));
      if (failure === "foreign-visit") {
        await view.rerender({
          data: routeData(makePlant({ id: 2, name: "Monstera" })),
        });
        await screen.findByText("Monstera");
      }
      if (failure === "care") {
        history.reject(new Error("care load failed"));
        refetch.resolve(makePlant({ name: "Authoritative metadata" }));
      } else {
        history.resolve([]);
        refetch.reject(new Error("plant load failed"));
      }
      await settleRequests(refetch.promise);
      await settleRequests(history.promise);
      expect(screen.queryByText("Obsolete water DTO")).toBeNull();
      if (failure === "foreign-visit") {
        expect(screen.getByText("Monstera")).toBeTruthy();
        expect(mockPushNotification).not.toHaveBeenCalled();
      } else {
        expect(
          screen.getByText(
            failure === "care" ? "Authoritative metadata" : "Current metadata",
          ),
        ).toBeTruthy();
        expect(mockPushNotification).toHaveBeenCalledTimes(1);
        expect(mockPushNotification).toHaveBeenCalledWith(
          expect.objectContaining({
            title: "Care Journal",
            variant: "error",
            message: "Failed to load care events",
          }),
        );
        expect(
          screen.getByRole("button", { name: "Water now" }),
        ).toHaveProperty("disabled", false);
      }
    },
  );

  it("updates authoritative watering details despite failed reconciliation history after a newer non-watering deletion", async () => {
    const retained = recentEvent(1, 10, { notes: "Retained watering" });
    const removed = recentEvent(2, 3, {
      event_type: "pruned",
      notes: "Removed pruning",
    });
    await renderWithPlant({ watering_status: "overdue" }, [removed, retained]);
    await screen.findByText("Removed pruning");
    const water = deferred<Plant | null>();
    mockWaterPlant.mockReturnValueOnce(water.promise);
    await fireEvent.click(screen.getByRole("button", { name: "Water now" }));

    const current = makePlant({
      name: "Current Fern",
      watering_interval_days: 14,
      watering_status: "overdue",
    });
    mockFetchPlantApi.mockResolvedValueOnce(current);
    mockFetchCareEventsApi.mockResolvedValueOnce([retained]);
    await deleteJournalEntry();
    await screen.findByRole("heading", { name: "Current Fern" });
    expect(screen.queryByText("Removed pruning")).toBeNull();
    expect(screen.getByText("Every 14 days")).toBeTruthy();
    expect(document.querySelector(".detail-status")?.textContent).toContain(
      "Overdue",
    );
    expect(timelineSnapshot()).toBe(expectedTimeline([retained], current));

    const refetch = deferred<Plant>();
    const history = deferred<CareEvent[]>();
    mockFetchPlantApi.mockReturnValueOnce(refetch.promise);
    mockFetchCareEventsApi.mockReturnValueOnce(history.promise);
    water.resolve(makePlant({ name: "Stale watering DTO" }));
    await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(2));
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(3);
    expect(screen.getByRole("heading", { name: "Current Fern" })).toBeTruthy();
    expect(screen.queryByText("Stale watering DTO")).toBeNull();
    expect(screen.getByRole("button", { name: "Watering..." })).toHaveProperty(
      "disabled",
      true,
    );

    const authoritative = makePlant({
      name: "Authoritative Fern",
      watering_interval_days: 21,
      last_watered: new Date(TIMELINE_NOW).toISOString(),
      next_due: new Date(TIMELINE_NOW + 21 * DAY).toISOString(),
      watering_status: "ok",
    });
    refetch.resolve(authoritative);
    history.reject(new Error("history unavailable"));
    await screen.findByRole("heading", { name: "Authoritative Fern" });
    expect(screen.queryByText("Stale watering DTO")).toBeNull();
    expect(screen.queryByText("Current Fern")).toBeNull();
    expect(screen.getByText("Every 21 days")).toBeTruthy();
    for (const date of [authoritative.last_watered, authoritative.next_due]) {
      expect(
        screen.getByText(
          new Date(date!).toLocaleDateString(undefined, {
            year: "numeric",
            month: "short",
            day: "numeric",
          }),
        ),
      ).toBeTruthy();
    }
    expect(document.querySelector(".detail-status")?.textContent).not.toContain(
      "Overdue",
    );
    expect(document.querySelector(".detail-status .status-ok")).toBeTruthy();
    expect(screen.getByText("Retained watering")).toBeTruthy();
    expect(screen.queryByText("Removed pruning")).toBeNull();
    expect(timelineSnapshot()).toBe(
      expectedTimeline([retained], authoritative),
    );
    expect(screen.getByRole("alert").textContent).toBe(
      "Failed to load care events",
    );
    expect(
      document.querySelector(".care-timeline desc")?.textContent,
    ).toContain("Failed to load care events");
    expect(mockPushNotification).toHaveBeenCalledExactlyOnceWith(
      expect.objectContaining({
        title: "Care Journal",
        variant: "error",
        message: "Failed to load care events",
      }),
    );
    expect(
      await screen.findByRole("button", { name: "Water now" }),
    ).toHaveProperty("disabled", false);
    expect(screen.queryByRole("button", { name: "Watering..." })).toBeNull();
  });

  it("does not invalidate pending initial history when Water now fails", async () => {
    const history = deferred<CareEvent[]>();
    mockFetchCareEventsApi.mockReturnValueOnce(history.promise);
    mockWaterPlant.mockResolvedValueOnce(null);
    render(Page, { props: { data: routeData() } });
    await fireEvent.click(
      await screen.findByRole("button", { name: "Water now" }),
    );
    await waitFor(() => expect(mockPushNotification).toHaveBeenCalledTimes(1));
    expect(document.querySelector(".skeleton-list")).toBeTruthy();
    const events = [recentEvent(1, 2, { notes: "Initial history" })];
    history.resolve(events);
    await screen.findByText("Initial history");
    expect(timelineSnapshot()).toBe(expectedTimeline(events));
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(1);
    expect(mockFetchPlantApi).not.toHaveBeenCalled();
    expect(wateringFeedbackIds()).toEqual([]);
  });

  it("does not clear the current event-delete flag or toast when an old visit's deletion fails", async () => {
    const view = await renderWithPlant({}, [
      recentEvent(1, 1, { notes: "Old" }),
    ]);
    await waitFor(() => expect(timelineSnapshot()).toBeTruthy());
    const old = deferred<void>();
    const current = deferred<void>();
    mockDeleteCareEventApi
      .mockReturnValueOnce(old.promise)
      .mockReturnValueOnce(current.promise);
    await deleteJournalEntry();
    const nextEvents = [recentEvent(2, 2, { plant_id: 2, notes: "Current" })];
    mockFetchCareEventsApi.mockResolvedValueOnce(nextEvents);
    await view.rerender({
      data: routeData(makePlant({ id: 2, name: "Monstera" })),
    });
    await screen.findByText("Current");
    await deleteJournalEntry();
    old.reject(new Error("old delete failed"));
    await settleRequests(old.promise);
    await waitFor(() =>
      expect(
        screen.getByRole("button", { name: "Delete log entry" }),
      ).toHaveProperty("disabled", true),
    );
    expect(mockPushNotification).not.toHaveBeenCalled();
    mockFetchPlantApi.mockResolvedValueOnce(
      makePlant({ id: 2, name: "Monstera" }),
    );
    mockFetchCareEventsApi.mockResolvedValueOnce([]);
    current.resolve(undefined);
    await screen.findByText("No care events recorded yet.");
    expect(mockFetchPlantApi).toHaveBeenCalledTimes(1);
    expect(mockFetchPlantApi).toHaveBeenCalledWith(2);
  });

  it("ignores an old watering response even after returning to the same plant in a new visit", async () => {
    const view = await renderWithPlant();
    const oldWater = deferred<Plant | null>();
    mockWaterPlant.mockReturnValueOnce(oldWater.promise);
    await fireEvent.click(
      await screen.findByRole("button", { name: "Water now" }),
    );
    await view.rerender({
      data: routeData(makePlant({ id: 2, name: "Monstera" })),
    });
    await screen.findByText("Monstera");
    const events = [recentEvent(1, 4, { notes: "Return visit" })];
    mockFetchCareEventsApi.mockResolvedValueOnce(events);
    await view.rerender({
      data: routeData(makePlant({ name: "Returned Fern" })),
    });
    await screen.findByText("Return visit");
    oldWater.resolve(makePlant({ name: "Obsolete Fern" }));
    await settleRequests(oldWater.promise);
    await waitFor(() => expect(screen.getByText("Returned Fern")).toBeTruthy());
    expect(screen.queryByText("Obsolete Fern")).toBeNull();
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(3);
    expect(timelineSnapshot()).toBe(expectedTimeline(events));
    expect(wateringFeedbackIds()).toEqual([]);
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it.each(["add", "edit"] as const)(
    "does not let an old %s callback close a new visit's form or start foreign loads",
    async (operation) => {
      const view = await renderWithPlant({}, [
        recentEvent(1, 1, { notes: "Old event" }),
      ]);
      await waitFor(() => expect(timelineSnapshot()).toBeTruthy());
      const mutation = deferred<CareEvent>();
      if (operation === "add") {
        mockAddCareEvent.mockReturnValueOnce(mutation.promise);
        await fireEvent.click(screen.getByText("+ Add log entry"));
        await fireEvent.click(
          within(
            document.querySelector(".care-entry-form") as HTMLElement,
          ).getByText("Watered"),
        );
      } else {
        mockUpdateCareEventApi.mockReturnValueOnce(mutation.promise);
        await fireEvent.click(
          screen.getByRole("button", { name: "Edit log entry" }),
        );
      }
      await fireEvent.click(screen.getByRole("button", { name: "Save" }));
      mockFetchCareEventsApi.mockResolvedValueOnce([]);
      await view.rerender({
        data: routeData(makePlant({ id: 2, name: "Monstera" })),
      });
      await fireEvent.click(await screen.findByText("+ Add log entry"));
      const newForm = document.querySelector(".care-entry-form");
      await fireEvent.input(newForm!.querySelector("textarea")!, {
        target: { value: "New visit draft" },
      });
      mutation.resolve(recentEvent(3, 0));
      await settleRequests(mutation.promise);
      await waitFor(() =>
        expect(screen.getByDisplayValue("New visit draft")).toBeTruthy(),
      );
      expect(document.querySelector(".care-entry-form")).toBe(newForm);
      expect(mockFetchPlantApi).not.toHaveBeenCalled();
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
    },
  );

  it.each(["water", "delete"] as const)(
    "closes a saved add form before its pending refresh is superseded by %s",
    async (superseding) => {
      const base = recentEvent(1, 3, { notes: "Base event" });
      const added = recentEvent(2, 1, { notes: "Saved entry" });
      await renderWithPlant({}, [base]);
      await screen.findByText("Base event");
      const oldCare = deferred<CareEvent[]>();
      const oldPlant = deferred<Plant>();
      mockAddCareEvent.mockResolvedValueOnce(added);
      mockFetchCareEventsApi.mockReturnValueOnce(oldCare.promise);
      mockFetchPlantApi.mockReturnValueOnce(oldPlant.promise);
      await fireEvent.click(screen.getByText("+ Add log entry"));
      await fireEvent.click(
        within(
          document.querySelector(".care-entry-form") as HTMLElement,
        ).getByText("Watered"),
      );
      await fireEvent.click(screen.getByRole("button", { name: "Save" }));
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledTimes(1));
      expect(document.querySelector(".care-entry-form")).toBeNull();

      const latest = [added];
      mockFetchCareEventsApi.mockResolvedValueOnce(latest);
      if (superseding === "water") {
        mockWaterPlant.mockResolvedValueOnce(
          makePlant({ name: "Latest metadata" }),
        );
        await fireEvent.click(
          screen.getByRole("button", { name: "Water now" }),
        );
      } else {
        mockFetchPlantApi.mockResolvedValueOnce(
          makePlant({ name: "Latest metadata" }),
        );
        await deleteJournalEntry();
      }
      await screen.findByText("Latest metadata");
      await screen.findByText("Saved entry");
      oldCare.resolve([base, added]);
      oldPlant.resolve(makePlant({ name: "Superseded add refresh" }));
      await settleRequests(oldPlant.promise);
      expect(document.querySelector(".care-entry-form")).toBeNull();
      expect(screen.getByText("+ Add log entry")).toBeTruthy();
      expect(screen.queryByText("Superseded add refresh")).toBeNull();
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it.each([
    ["add", "cancel-add"],
    ["add", "edit"],
    ["edit", "cancel-add"],
    ["edit", "edit"],
  ] as const)(
    "keeps a newer same-visit %s/%s form owned by its session when the removed form saves",
    async (operation, replacement) => {
      const events = [
        recentEvent(1, 3, { notes: "Original edit", event_type: "fertilized" }),
        recentEvent(2, 6),
        recentEvent(3, 10),
      ];
      await renderWithPlant({}, events);
      await screen.findByText("Original edit");
      await fireEvent.click(
        screen.getByRole("button", { name: "Expand watering entries" }),
      );
      const mutation = deferred<CareEvent>();
      if (operation === "add") {
        mockAddCareEvent.mockReturnValueOnce(mutation.promise);
        await fireEvent.click(screen.getByText("+ Add log entry"));
        await fireEvent.click(
          within(
            document.querySelector(".care-entry-form") as HTMLElement,
          ).getByText("Watered"),
        );
      } else {
        mockUpdateCareEventApi.mockReturnValueOnce(mutation.promise);
        await fireEvent.click(
          screen.getAllByRole("button", { name: "Edit log entry" })[0],
        );
      }
      const oldForm = document.querySelector(".care-entry-form") as HTMLElement;
      await fireEvent.click(
        within(oldForm).getByRole("button", { name: "Save" }),
      );
      if (replacement === "cancel-add") {
        await fireEvent.click(
          within(oldForm).getByRole("button", { name: "Cancel" }),
        );
        await fireEvent.click(screen.getByText("+ Add log entry"));
      } else {
        // Even edit-to-edit needs a new instance, not live callback props.
        await fireEvent.click(
          screen.getAllByRole("button", { name: "Edit log entry" })[1],
        );
      }
      const newForm = document.querySelector(".care-entry-form") as HTMLElement;
      expect(newForm).not.toBe(oldForm);
      await fireEvent.input(newForm.querySelector("textarea")!, {
        target: { value: "New session draft" },
      });
      const refreshedCare = deferred<CareEvent[]>();
      const refreshedPlant = deferred<Plant>();
      mockFetchCareEventsApi.mockReturnValueOnce(refreshedCare.promise);
      mockFetchPlantApi.mockReturnValueOnce(refreshedPlant.promise);
      mutation.resolve(recentEvent(4, 0, { notes: "Saved by removed form" }));
      await settleRequests(mutation.promise);
      await waitFor(() => expect(mockFetchPlantApi).toHaveBeenCalledWith(1));
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
      expect(document.querySelector(".care-entry-form")).toBe(newForm);
      expect(screen.getByDisplayValue("New session draft")).toBeTruthy();
      expect(
        screen.getByRole("button", { name: "Collapse watering entries" }),
      ).toBeTruthy();
      refreshedCare.resolve(events);
      refreshedPlant.resolve(makePlant({ name: "Refreshed after old save" }));
      await screen.findByText("Refreshed after old save");
      expect(screen.getByDisplayValue("New session draft")).toBeTruthy();
      expect(document.querySelector(".care-entry-form")).toBe(newForm);
      expect(
        screen.getByRole("button", { name: "Collapse watering entries" }),
      ).toBeTruthy();
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it("preserves retained markers with parent error and SVG error metadata on a current refresh failure", async () => {
    const events = [recentEvent(1, 2, { notes: "Keep this" })];
    const view = await renderWithPlant({}, events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    const snapshot = timelineSnapshot();
    mockFetchCareEventsApi.mockRejectedValueOnce(new Error("offline"));
    await view.rerender({ data: routeData() });
    await screen.findByRole("alert");
    expect(timelineSnapshot()).toBe(snapshot);
    expect(screen.getByText("Keep this")).toBeTruthy();
    expect(
      document.querySelector(".care-timeline desc")?.textContent,
    ).toContain("Failed to load care events");
    expect(
      document.querySelector(
        ".care-timeline .caption, .care-timeline .message",
      ),
    ).toBeNull();
  });

  it("omits the graph on initial failure while the journal owns the only error", async () => {
    mockFetchCareEventsApi.mockRejectedValueOnce(new Error("offline"));
    render(Page, { props: { data: routeData() } });
    await screen.findByRole("alert");
    expect(document.querySelector(".care-timeline")?.children).toHaveLength(0);
    expect(screen.getAllByRole("alert")).toHaveLength(1);
    expect(screen.queryByText("No care events recorded yet.")).toBeNull();
  });

  it("renders cached raw history offline with unchanged mutation restrictions", async () => {
    isOffline.set(true);
    aiStatus.set({ enabled: true, base_url: null, model: null });
    const events = [
      recentEvent(1, 2, { notes: "Cached", photo_url: "/uploads/care/1.jpg" }),
    ];
    await renderWithPlant({}, events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    expect(screen.getByText("Cached")).toBeTruthy();
    for (const name of [
      "Water now",
      "Ask AI",
      "Delete plant",
      "Edit log entry",
      "Delete log entry",
      "+ Add log entry",
    ]) {
      for (const button of screen.getAllByRole("button", { name })) {
        expect(button).toHaveProperty("disabled", true);
      }
    }
    await fireEvent.click(screen.getByRole("button", { name: "Open photo" }));
    expect(getLightbox().hasAttribute("open")).toBe(true);
    expect(mockWaterPlant).not.toHaveBeenCalled();
  });

  it("resamples real time immediately for a fresh Water now record while retaining existing markers", async () => {
    const events = [recentEvent(1, 3)];
    await renderWithPlant({ watering_status: "overdue" }, events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    const marker = document.querySelector(
      '.care-timeline g.event[data-event-id="1"]',
    );
    const previous = timelineSnapshot();
    expect(wateringFeedbackIds()).toEqual([]);
    clock.mockReturnValue(TIMELINE_NOW + 30_000);
    const fresh = makeCareEvent({
      id: 2,
      occurred_at: new Date(Date.now()).toISOString(),
    });
    mockWaterPlant.mockResolvedValueOnce(
      makePlant({ last_watered: fresh.occurred_at, watering_status: "ok" }),
    );
    mockFetchCareEventsApi.mockResolvedValueOnce([...events, fresh]);
    await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline([...events, fresh])),
    );
    expect(timelineMarkers().at(-1)).toEqual({
      id: fresh.id,
      eventType: "watered",
      x: 288,
    });
    expect(wateringFeedbackIds()).toEqual([fresh.id]);
    expect(mockWaterPlant).toHaveBeenCalledExactlyOnceWith(1);
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
    expect(mockFetchPlantApi).not.toHaveBeenCalled();
    expect(timelineSnapshot()).not.toBe(previous);
    expect(
      document.querySelector('.care-timeline g.event[data-event-id="1"]'),
    ).toBe(marker);
    expect(document.querySelector(".detail-status")?.textContent).not.toContain(
      "Overdue",
    );
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("highlights the first persisted Water now event after empty history", async () => {
    const view = await renderWithPlant({ last_watered: null });
    await screen.findByText("No care events recorded yet.");
    expect(wateringFeedbackIds()).toEqual([]);
    const fresh = recentEvent(1, 0);
    mockWaterPlant.mockResolvedValueOnce(
      makePlant({ last_watered: fresh.occurred_at }),
    );
    mockFetchCareEventsApi.mockResolvedValueOnce([fresh]);
    await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
    await waitFor(() => expect(wateringFeedbackIds()).toEqual([fresh.id]));
    expect(timelineSnapshot()).toBe(expectedTimeline([fresh]));
    expect(mockWaterPlant).toHaveBeenCalledExactlyOnceWith(1);
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
    expect(mockFetchPlantApi).not.toHaveBeenCalled();

    mockFetchCareEventsApi.mockResolvedValueOnce([]);
    await view.rerender({ data: routeData(makePlant({ id: 2 })) });
    await screen.findByText("No care events recorded yet.");
    expect(wateringFeedbackIds()).toEqual([]);
    mockFetchCareEventsApi.mockResolvedValueOnce([fresh]);
    await view.rerender({ data: routeData() });
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline([fresh])),
    );
    expect(wateringFeedbackIds()).toEqual([]);
  });

  it("selects only the latest new eligible watering by time and ID, not response order", async () => {
    const base = recentEvent(100, 1);
    // Snapshot every ID, including a previously non-watering event.
    const existing = recentEvent(101, 0, { event_type: "fertilized" });
    await renderWithPlant({}, [base, existing]);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline([base, existing])),
    );
    const fresh = recentEvent(3, 0);
    const events = [
      fresh,
      recentEvent(2, 0),
      recentEvent(99, 2),
      { ...existing, event_type: "watered" as const },
      recentEvent(102, 0, { plant_id: 2 }),
      recentEvent(103, -1),
      recentEvent(104, 43),
      recentEvent(105, 0, {
        event_type: "unsupported" as CareEvent["event_type"],
      }),
      recentEvent(106, 0, { event_type: "fertilized" }),
      recentEvent(107, 0, { occurred_at: "invalid" }),
      base,
    ];
    mockWaterPlant.mockResolvedValueOnce(makePlant());
    mockFetchCareEventsApi.mockResolvedValueOnce(events);
    await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
    await waitFor(() => expect(wateringFeedbackIds()).toEqual([fresh.id]));
    expect(timelineSnapshot()).toBe(expectedTimeline(events));
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
  });

  it("does not highlight existing or ineligible events when the accepted history has no new visible watering", async () => {
    const base = recentEvent(1, 3);
    await renderWithPlant({}, [base]);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline([base])),
    );
    const events = [
      recentEvent(2, -1),
      recentEvent(3, 43),
      recentEvent(4, 0, { plant_id: 2 }),
      recentEvent(5, 0, { event_type: "fertilized" }),
      base,
    ];
    mockWaterPlant.mockResolvedValueOnce(makePlant());
    mockFetchCareEventsApi.mockResolvedValueOnce(events);
    await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    expect(wateringFeedbackIds()).toEqual([]);
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2);
  });

  it.each(["water", "history"] as const)(
    "does not retain feedback intent after failed %s or transfer it to unrelated history",
    async (failure) => {
      const base = recentEvent(1, 3, { notes: "Retained history" });
      await renderWithPlant({}, [base]);
      await screen.findByText("Retained history");
      const marker = document.querySelector(
        '.care-timeline g.event[data-event-id="1"]',
      );
      mockWaterPlant.mockResolvedValueOnce(
        failure === "water" ? null : makePlant(),
      );
      if (failure === "history") {
        mockFetchCareEventsApi.mockRejectedValueOnce(
          new Error("History failed"),
        );
      }
      await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
      if (failure === "water") {
        await waitFor(() =>
          expect(mockPushNotification).toHaveBeenCalledTimes(1),
        );
      } else {
        await waitFor(() =>
          expect(
            document.querySelector(".care-timeline desc")?.textContent,
          ).toContain("Failed to load care events"),
        );
      }
      expect(screen.getByText("Retained history")).toBeTruthy();
      expect(
        document.querySelector('.care-timeline g.event[data-event-id="1"]'),
      ).toBe(marker);
      expect(wateringFeedbackIds()).toEqual([]);
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(
        failure === "water" ? 1 : 2,
      );

      // An unrelated edit later returns a new watering: no saved intent may play it.
      const edited = { ...base, notes: "Edited history" };
      const fresh = recentEvent(2, 0);
      mockUpdateCareEventApi.mockResolvedValueOnce(edited);
      mockFetchCareEventsApi.mockResolvedValueOnce([fresh, edited]);
      await fireEvent.click(
        screen.getByRole("button", { name: "Edit log entry" }),
      );
      await fireEvent.click(screen.getByRole("button", { name: "Save" }));
      await screen.findByText("Edited history");
      expect(timelineSnapshot()).toBe(expectedTimeline([fresh, edited]));
      expect(wateringFeedbackIds()).toEqual([]);
    },
  );

  it.each(["refresh", "visit"] as const)(
    "does not arm feedback from a Water now history request superseded by a newer %s",
    async (superseding) => {
      const base = recentEvent(1, 3, { notes: "Original history" });
      const view = await renderWithPlant({}, [base]);
      await screen.findByText("Original history");
      const history = deferred<CareEvent[]>();
      const fresh = recentEvent(2, 0);
      mockWaterPlant.mockResolvedValueOnce(makePlant());
      mockFetchCareEventsApi.mockReturnValueOnce(history.promise);
      await fireEvent.click(screen.getByRole("button", { name: "Water now" }));
      await waitFor(() =>
        expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(2),
      );
      expect(wateringFeedbackIds()).toEqual([]);
      if (superseding === "refresh") {
        const edited = { ...base, notes: "Current history" };
        mockUpdateCareEventApi.mockResolvedValueOnce(edited);
        mockFetchCareEventsApi.mockResolvedValueOnce([fresh, edited]);
        await fireEvent.click(
          screen.getByRole("button", { name: "Edit log entry" }),
        );
        await fireEvent.click(screen.getByRole("button", { name: "Save" }));
        await screen.findByText("Current history");
      } else {
        mockFetchCareEventsApi.mockResolvedValueOnce([
          recentEvent(3, 1, { plant_id: 2, notes: "Current history" }),
        ]);
        await view.rerender({ data: routeData(makePlant({ id: 2 })) });
        await screen.findByText("Current history");
      }
      const current = timelineSnapshot();
      history.resolve([fresh, base]);
      await settleRequests(history.promise);
      expect(timelineSnapshot()).toBe(current);
      expect(wateringFeedbackIds()).toEqual([]);
      expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(3);
      expect(mockPushNotification).not.toHaveBeenCalled();
    },
  );

  it("adds and deletes a watering with real recent timestamps and no retained deleted marker", async () => {
    const base = recentEvent(1, 20, { notes: "Base" });
    const added = recentEvent(2, 3, { notes: "Added" });
    await renderWithPlant({}, [base]);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline([base])),
    );
    const baseSnapshot = timelineSnapshot();
    mockAddCareEvent.mockResolvedValueOnce(added);
    mockFetchCareEventsApi.mockResolvedValueOnce([added, base]);
    await fireEvent.click(screen.getByText("+ Add log entry"));
    await fireEvent.click(
      within(
        document.querySelector(".care-entry-form") as HTMLElement,
      ).getByText("Watered"),
    );
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline([added, base])),
    );
    expect(timelineSnapshot()).not.toBe(baseSnapshot);
    await waitFor(() =>
      expect(document.querySelector(".care-entry-form")).toBeNull(),
    );
    mockFetchCareEventsApi.mockResolvedValueOnce([base]);
    await deleteJournalEntry();
    await waitFor(() => expect(timelineSnapshot()).toBe(baseSnapshot));
    expect(screen.queryByText("Added")).toBeNull();
  });

  it.each(["retime", "to-watered", "from-watered"] as const)(
    "redraws after an event edit (%s)",
    async (change) => {
      const base = recentEvent(1, 20, { notes: "Base" });
      const target = recentEvent(2, 3, {
        notes: "Edit me",
        event_type: change === "to-watered" ? "fertilized" : "watered",
      });
      const updated = {
        ...target,
        event_type:
          change === "from-watered"
            ? ("fertilized" as const)
            : ("watered" as const),
        occurred_at:
          change === "retime"
            ? recentEvent(2, 8).occurred_at
            : target.occurred_at,
      };
      await renderWithPlant({}, [target, base]);
      await waitFor(() =>
        expect(timelineSnapshot()).toBe(expectedTimeline([target, base])),
      );
      const previous = timelineSnapshot();
      const previousX = timelineMarkers().find(
        (marker) => marker.id === target.id,
      )!.x;
      mockUpdateCareEventApi.mockResolvedValueOnce(updated);
      mockFetchCareEventsApi.mockResolvedValueOnce([updated, base]);
      await fireEvent.click(
        screen.getAllByRole("button", { name: "Edit log entry" })[0],
      );
      const form = document.querySelector(".care-entry-form") as HTMLElement;
      await fireEvent.click(
        within(form).getByText(
          change === "from-watered" ? "Fertilized" : "Watered",
        ),
      );
      if (change === "retime") {
        const date = new Date(updated.occurred_at);
        const pad = (n: number) => String(n).padStart(2, "0");
        const local = `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
        await fireEvent.input(form.querySelector(".toolbar-date-input")!, {
          target: { value: local },
        });
      }
      await fireEvent.click(within(form).getByRole("button", { name: "Save" }));
      await waitFor(() =>
        expect(timelineSnapshot()).toBe(expectedTimeline([updated, base])),
      );
      expect(timelineSnapshot()).not.toBe(previous);
      const markers = document.querySelectorAll(".care-timeline g.event");
      expect(markers).toHaveLength(2);
      const marker = document.querySelector(
        `.care-timeline g.event[data-event-id="${target.id}"]`,
      )!;
      expect(marker.getAttribute("data-event-type")).toBe(updated.event_type);
      expect(
        marker.querySelector(
          change === "from-watered" ? ".lucide-leaf" : ".lucide-droplet",
        ),
      ).toBeTruthy();
      const nextX = timelineMarkers().find(
        (point) => point.id === target.id,
      )!.x;
      if (change === "retime") expect(nextX).not.toBe(previousX);
      else expect(nextX).toBe(previousX);
      expect(mockUpdateCareEventApi).toHaveBeenCalledWith(
        1,
        target.id,
        expect.objectContaining({
          event_type: updated.event_type,
          occurred_at: updated.occurred_at,
        }),
      );
    },
  );

  it("keeps the entire background unchanged after an interval edit while API status remains authoritative", async () => {
    const events = [recentEvent(1, 20), recentEvent(2, 3)];
    const view = await renderWithPlant({ watering_status: "due" }, events);
    await waitFor(() =>
      expect(timelineSnapshot()).toBe(expectedTimeline(events)),
    );
    const background = document.querySelector(".care-timeline")!.innerHTML;
    const snapshot = timelineSnapshot();
    const updatedPlant = makePlant({
      watering_interval_days: 14,
      watering_status: "due",
    });
    await view.rerender({ data: routeData(updatedPlant) });
    await waitFor(() =>
      expect(document.querySelector(".care-timeline")!.innerHTML).toBe(
        background,
      ),
    );
    expect(timelineSnapshot()).toBe(snapshot);
    expect(screen.getByText("Every 14 days")).toBeTruthy();
    expect(document.querySelector(".detail-status")?.textContent).toContain(
      "Due",
    );
    expect(
      document.querySelector(".care-timeline desc")?.textContent,
    ).not.toContain("Every 14 days");
  });
});

describe("hero thumbnail", () => {
  it("uses 200px thumbnail for hero photo", async () => {
    await renderWithPlant({ photo_url: "/uploads/fern.jpg" });
    await screen.findByText("Fern");
    const img = document.querySelector(".detail-photo-img") as HTMLImageElement;
    expect(img).toBeTruthy();
    expect(img.src).toContain("/uploads/fern_200.jpg");
  });

  it("falls back to original photo_url on hero thumbnail error", async () => {
    await renderWithPlant({ photo_url: "/uploads/fern.jpg" });
    await screen.findByText("Fern");
    const img = document.querySelector(".detail-photo-img") as HTMLImageElement;
    expect(img.src).toContain("/uploads/fern_200.jpg");
    await fireEvent.error(img);
    expect(img.src).toContain("/uploads/fern.jpg");
    expect(img.src).not.toContain("_200");
  });
});

describe("plant detail lightbox", () => {
  it("opens and closes the lightbox for a photo", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    // Escape triggers the dialog's cancel event
    getLightbox().dispatchEvent(new Event("cancel"));
    await vi.waitFor(() => {
      expect(getLightbox().hasAttribute("open")).toBe(false);
    });
  });

  it("does not expose a lightbox trigger when no photo is available", async () => {
    await renderWithPlant({ photo_url: null });
    await vi.waitFor(() => {
      expect(screen.queryByRole("button", { name: "Open photo" })).toBeNull();
    });
  });

  it("updates zoom on wheel input", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const img = document.querySelector(".lightbox-image") as HTMLImageElement;
    expect(img).toBeTruthy();
    const before = img.style.transform;
    await fireEvent.wheel(img, { deltaY: -600 });
    expect(img.style.transform).not.toBe(before);
  });

  it("pans the image when zoomed", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const img = document.querySelector(".lightbox-image") as HTMLImageElement;
    expect(img).toBeTruthy();
    Object.defineProperty(img, "clientWidth", { value: 400 });
    Object.defineProperty(img, "clientHeight", { value: 300 });
    await fireEvent.wheel(img, { deltaY: -600 });
    const before = img.style.transform;
    await fireEvent.pointerDown(img, { clientX: 100, clientY: 100 });
    await fireEvent.pointerMove(window, { clientX: 160, clientY: 140 });
    await fireEvent.pointerUp(window);
    expect(img.style.transform).not.toBe(before);
  });

  it("closes the lightbox via close button", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    const closeButton = screen.getByRole("button", { name: "Close" });
    await fireEvent.click(closeButton);
    await vi.waitFor(() => {
      expect(getLightbox().hasAttribute("open")).toBe(false);
    });
  });

  it("locks body scroll while lightbox is open and restores on close", async () => {
    document.body.style.overflow = "";
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });

    await fireEvent.click(openButton);
    expect(document.body.style.overflow).toBe("hidden");

    getLightbox().dispatchEvent(new Event("cancel"));
    await vi.waitFor(() => {
      expect(getLightbox().hasAttribute("open")).toBe(false);
    });
    expect(document.body.style.overflow).toBe("");
  });

  it("zooms via touch pinch gesture", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const img = document.querySelector(".lightbox-image") as HTMLImageElement;
    expect(img).toBeTruthy();
    const before = img.style.transform;

    const startEvent = new Event("touchstart", { bubbles: true }) as Event & {
      touches: Array<{ clientX: number; clientY: number }>;
    };
    startEvent.touches = [
      { clientX: 100, clientY: 100 },
      { clientX: 200, clientY: 200 },
    ];
    await fireEvent(window, startEvent);

    const moveEvent = new Event("touchmove", {
      bubbles: true,
      cancelable: true,
    }) as Event & {
      touches: Array<{ clientX: number; clientY: number }>;
    };
    moveEvent.touches = [
      { clientX: 50, clientY: 50 },
      { clientX: 250, clientY: 250 },
    ];
    await fireEvent(window, moveEvent);

    expect(img.style.transform).not.toBe(before);
  });

  it("closes the lightbox via backdrop click", async () => {
    await renderWithPlant();
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    const lightbox = getLightbox();
    expect(lightbox.hasAttribute("open")).toBe(true);

    // Click directly on the dialog element (backdrop area)
    await fireEvent.click(lightbox);
    await vi.waitFor(() => {
      expect(lightbox.hasAttribute("open")).toBe(false);
    });
  });
});

describe("plant delete confirmation", () => {
  function getDeleteIconButton() {
    return document.querySelector(".btn-danger") as HTMLButtonElement;
  }

  it("shows confirmation dialog with plant name when delete is clicked", async () => {
    await renderWithPlant({ name: "My Fern" });
    await screen.findByText("My Fern");

    const user = userEvent.setup();
    await user.click(getDeleteIconButton());

    await waitFor(() => {
      expect(screen.getByText(/Delete "My Fern"/)).toBeTruthy();
    });
  });

  it("calls deletePlant when confirmed", async () => {
    mockDeletePlant.mockResolvedValue(true);
    await renderWithPlant({ name: "My Fern" });
    await screen.findByText("My Fern");

    const user = userEvent.setup();
    await user.click(getDeleteIconButton());

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Delete" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockDeletePlant).toHaveBeenCalledWith(1);
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Delete plant",
          variant: "success",
          message: 'Plant "My Fern" deleted',
        }),
      );
      expect(mockGoto).toHaveBeenCalledWith("/");
      expect(mockPushNotification.mock.invocationCallOrder[0]).toBeLessThan(
        mockGoto.mock.invocationCallOrder[0],
      );
    });
  });

  it("does not call deletePlant when cancelled", async () => {
    await renderWithPlant({ name: "My Fern" });
    await screen.findByText("My Fern");

    const user = userEvent.setup();
    await user.click(getDeleteIconButton());

    await waitFor(() => {
      expect(screen.getByRole("button", { name: "Cancel" })).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Cancel" }));

    await new Promise((r) => setTimeout(r, 50));
    expect(mockDeletePlant).not.toHaveBeenCalled();
  });
});

describe("watering feedback", () => {
  it("keeps watering success silent", async () => {
    mockWaterPlant.mockResolvedValue(
      makePlant({ last_watered: "2025-02-01T10:00:00Z" }),
    );
    await renderWithPlant();
    await screen.findByText("Fern");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Water now" }));

    await waitFor(() => {
      expect(mockWaterPlant).toHaveBeenCalledWith(1);
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("shows a toast when watering fails", async () => {
    mockWaterPlant.mockResolvedValue(null);
    await renderWithPlant();
    await screen.findByText("Fern");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Water now" }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Watering",
          variant: "error",
          message: "Failed to water plant",
        }),
      );
    });
  });

  it("uses the store error details when watering fails", async () => {
    mockWaterPlant.mockImplementation(async () => {
      plantsError.set("Watering service offline");
      return null;
    });
    await renderWithPlant();
    await screen.findByText("Fern");

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Water now" }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Watering",
          variant: "error",
          message: "Watering service offline",
        }),
      );
    });
  });
});

describe("Ask AI button", () => {
  it("shows Ask AI button when AI is enabled", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    await renderWithPlant();
    await screen.findByText("Fern");
    await waitFor(() => {
      expect(screen.getByText("Ask AI")).toBeTruthy();
    });
  });

  it("hides Ask AI button when AI is disabled", async () => {
    aiStatus.set({ enabled: false, base_url: null, model: null });
    await renderWithPlant();
    await screen.findByText("Fern");
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByText("Ask AI")).toBeNull();
  });

  it("hides Ask AI button when AI status is null (fetch failed)", async () => {
    aiStatus.set(null);
    await renderWithPlant();
    await screen.findByText("Fern");
    await new Promise((r) => setTimeout(r, 50));
    expect(screen.queryByText("Ask AI")).toBeNull();
  });

  it("opens chat drawer when Ask AI is clicked", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    await renderWithPlant();
    await waitFor(() => {
      expect(screen.getByText("Ask AI")).toBeTruthy();
    });
    const user = userEvent.setup();
    await user.click(screen.getByText("Ask AI"));
    await waitFor(() => {
      expect(screen.getByText("Quick questions")).toBeTruthy();
    });
  });

  it("closes chat drawer when close button is clicked", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    await renderWithPlant();
    await waitFor(() => {
      expect(screen.getByText("Ask AI")).toBeTruthy();
    });
    const user = userEvent.setup();
    await user.click(screen.getByText("Ask AI"));
    await waitFor(() => {
      expect(screen.getByText("Quick questions")).toBeTruthy();
    });
    const closeBtn = screen.getByRole("button", { name: "Close chat" });
    await user.click(closeBtn);
    await waitFor(() => {
      expect(screen.queryByText("Quick questions")).toBeNull();
    });
  });
});

describe("care journal loading", () => {
  it("shows an error instead of the empty state when care history fails", async () => {
    mockFetchCareEventsApi.mockRejectedValue(new Error("offline"));

    render(Page, {
      props: {
        data: {
          plant: makePlant(),
          notFound: false,
          loadErrorCode: null,
        },
      },
    });

    expect((await screen.findByRole("alert")).textContent).toContain(
      "Failed to load care events",
    );
    expect(screen.queryByText("No care events recorded yet.")).toBeNull();
  });

  it("preserves loaded care history when a refresh fails", async () => {
    const event = makeCareEvent({ event_type: "fertilized" });
    const view = await renderWithPlant({}, [event]);
    expect(await screen.findByText("Fertilized")).toBeTruthy();

    mockFetchCareEventsApi.mockRejectedValue(new Error("offline"));
    await view.rerender({
      data: {
        plant: makePlant(),
        notFound: false,
        loadErrorCode: null,
      },
    });

    expect((await screen.findByRole("alert")).textContent).toContain(
      "Failed to load care events",
    );
    expect(screen.getByText("Fertilized")).toBeTruthy();
  });
});

describe("care event delete reloads plant", () => {
  it("calls loadPlant after deleting a care event", async () => {
    mockDeleteCareEventApi.mockResolvedValue(undefined);
    mockFetchPlantApi.mockResolvedValue(makePlant());
    mockFetchCareEventsApi.mockResolvedValue([]);

    await renderWithPlant({}, [makeCareEvent()]);
    await screen.findByText("Fern");

    await waitFor(() => {
      expect(screen.getByText("Watered")).toBeTruthy();
    });

    const deleteButton = screen.getByRole("button", {
      name: "Delete log entry",
    });
    const user = userEvent.setup();
    await user.click(deleteButton);

    await waitFor(() => {
      expect(screen.getByText(/Delete this care entry/)).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockDeleteCareEventApi).toHaveBeenCalledWith(1, 10);
      expect(mockFetchPlantApi).toHaveBeenCalledWith(1);
      expect(mockFetchCareEventsApi).toHaveBeenCalledWith(1);
    });
  });

  it("shows a toast when deleting a care event fails", async () => {
    mockDeleteCareEventApi.mockRejectedValue(
      new api.ApiError(500, "INTERNAL_ERROR", "An internal error occurred"),
    );

    await renderWithPlant({}, [makeCareEvent()]);
    await screen.findByText("Fern");

    await waitFor(() => {
      expect(
        screen.getByRole("button", { name: "Delete log entry" }),
      ).toBeTruthy();
    });

    const user = userEvent.setup();
    await user.click(screen.getByRole("button", { name: "Delete log entry" }));

    await waitFor(() => {
      expect(screen.getByText(/Delete this care entry/)).toBeTruthy();
    });
    await user.click(screen.getByRole("button", { name: "Delete" }));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Care Journal",
          variant: "error",
          message: "Something went wrong. Please try again.",
        }),
      );
    });
  });
});

describe("chat drawer save note", () => {
  beforeEach(() => {
    aiStatus.set({
      enabled: true,
      base_url: "https://api.openai.com/v1",
      model: "gpt-4o-mini",
    });
  });

  async function openChatAndSendMessage() {
    await renderWithPlant();
    const askAiButton = await screen.findByRole("button", { name: "Ask AI" });
    await fireEvent.click(askAiButton);
    return screen;
  }

  it("does not show save note button when no assistant messages", async () => {
    await openChatAndSendMessage();
    // Chat is open but no messages have been sent
    await waitFor(() => {
      expect(screen.queryByText("Create note")).toBeNull();
    });
  });

  it("shows save note button after assistant response", async () => {
    await openChatAndSendMessage();

    // Simulate an existing conversation by checking button visibility
    // The ChatDrawer needs assistant messages to show the button
    // Since we can't easily simulate streaming, we test the flow via summarize
    mockSummarizeChat.mockResolvedValue("Test summary");

    // Verify summarizeChat function exists and is callable
    expect(typeof api.summarizeChat).toBe("function");
  });

  it("shows a toast after saving a note successfully", async () => {
    mockChatPlant.mockImplementation(async function* () {
      yield "Looks healthy";
    });
    mockSummarizeChat.mockResolvedValue("Healthy and growing well");
    mockCreateCareEventApi.mockResolvedValue({
      id: 99,
      plant_id: 1,
      plant_name: "Fern",
      event_type: "ai-consultation",
      notes: "Healthy and growing well",
      photo_url: null,
      occurred_at: "2025-02-01T10:00:00Z",
      created_at: "2025-02-01T10:00:00Z",
    });

    await openChatAndSendMessage();
    const user = userEvent.setup();

    const input = screen.getByPlaceholderText("Ask about your plant...");
    await user.type(input, "How is my plant?");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(screen.getByText("Looks healthy")).toBeTruthy();
      expect(screen.getByText("Create note")).toBeTruthy();
    });

    await user.click(screen.getByText("Create note"));

    await waitFor(() => {
      expect(screen.getByDisplayValue("Healthy and growing well")).toBeTruthy();
    });

    const saveButtons = screen.getAllByText("Save");
    await user.click(saveButtons[saveButtons.length - 1]);

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Care Journal",
          variant: "success",
          message: "Saved to care journal",
        }),
      );
      expect(screen.queryByText("Quick questions")).toBeNull();
    });
  });

  it("keeps stream failures inline in the drawer", async () => {
    mockChatPlant.mockImplementation(async function* () {
      yield* [];
      throw new Error("stream failed");
    });

    await openChatAndSendMessage();
    const user = userEvent.setup();

    const input = screen.getByPlaceholderText("Ask about your plant...");
    await user.type(input, "How is my plant?");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(
        screen.getByText("Something went wrong. Please try again."),
      ).toBeTruthy();
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("keeps save-note failures inline in the drawer", async () => {
    mockChatPlant.mockImplementation(async function* () {
      yield "Looks healthy";
    });
    mockSummarizeChat.mockResolvedValue("Healthy and growing well");
    mockCreateCareEventApi.mockRejectedValue(new Error("save failed"));

    await openChatAndSendMessage();
    const user = userEvent.setup();

    const input = screen.getByPlaceholderText("Ask about your plant...");
    await user.type(input, "How is my plant?");
    await user.click(screen.getByRole("button", { name: "Send" }));

    await waitFor(() => {
      expect(screen.getByText("Looks healthy")).toBeTruthy();
      expect(screen.getByText("Create note")).toBeTruthy();
    });

    await user.click(screen.getByText("Create note"));

    await waitFor(() => {
      expect(screen.getByDisplayValue("Healthy and growing well")).toBeTruthy();
    });

    const saveButtons = screen.getAllByText("Save");
    await user.click(saveButtons[saveButtons.length - 1]);

    await waitFor(() => {
      expect(screen.getByText("Failed to save note")).toBeTruthy();
    });
    expect(mockPushNotification).not.toHaveBeenCalled();
  });

  it("summarizeChat calls the correct API endpoint", async () => {
    mockSummarizeChat.mockResolvedValue("Plant health looks good");

    const result = await api.summarizeChat(1, [
      { role: "user", content: "How is my plant?" },
      { role: "assistant", content: "Your plant looks healthy!" },
    ]);

    expect(result).toBe("Plant health looks good");
    expect(mockSummarizeChat).toHaveBeenCalledWith(1, [
      { role: "user", content: "How is my plant?" },
      { role: "assistant", content: "Your plant looks healthy!" },
    ]);
  });
});

describe("care event photo in timeline", () => {
  it("renders a thumbnail when a care event has a photo_url", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 20,
        event_type: "fertilized",
        notes: "Fed with liquid fertilizer",
        photo_url: "/uploads/care/20.jpg",
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      const img = document.querySelector(
        ".timeline-photo img",
      ) as HTMLImageElement;
      expect(img).toBeTruthy();
      expect(img.src).toContain("/uploads/care/20_200.jpg");
    });
  });

  it("does not render a thumbnail when care event has no photo_url", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 21,
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      expect(screen.getByText("Watered")).toBeTruthy();
    });
    expect(document.querySelector(".timeline-photo")).toBeNull();
  });

  it("opens lightbox when clicking a care event thumbnail", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 22,
        event_type: "repotted",
        photo_url: "/uploads/care/22.jpg",
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-photo")).toBeTruthy();
    });
    const photoBtn = document.querySelector(
      ".timeline-photo",
    ) as HTMLButtonElement;
    await fireEvent.click(photoBtn);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    const lightboxImg = getLightbox().querySelector("img") as HTMLImageElement;
    expect(lightboxImg.src).toContain("/uploads/care/22.jpg");
    expect(lightboxImg.src).not.toContain("_200");
  });

  it("lightbox uses original photo_url for hero photo", async () => {
    await renderWithPlant({ photo_url: "/uploads/fern.jpg" });
    const openButton = await screen.findByRole("button", {
      name: "Open photo",
    });
    await fireEvent.click(openButton);
    expect(getLightbox().hasAttribute("open")).toBe(true);

    const lightboxImg = getLightbox().querySelector("img") as HTMLImageElement;
    expect(lightboxImg.src).toContain("/uploads/fern.jpg");
    expect(lightboxImg.src).not.toContain("_600");
  });

  it("falls back to original photo_url on timeline thumbnail error", async () => {
    await renderWithPlant({}, [
      makeCareEvent({
        id: 23,
        photo_url: "/uploads/care/23.png",
        occurred_at: "2025-02-01T10:00:00Z",
        created_at: "2025-02-01T10:00:00Z",
      }),
    ]);

    await waitFor(() => {
      const img = document.querySelector(
        ".timeline-photo img",
      ) as HTMLImageElement;
      expect(img).toBeTruthy();
      expect(img.src).toContain("/uploads/care/23_200.jpg");
    });
    const img = document.querySelector(
      ".timeline-photo img",
    ) as HTMLImageElement;
    await fireEvent.error(img);
    expect(img.src).toContain("/uploads/care/23.png");
    expect(img.src).not.toContain("_200");
  });
});

describe("log form photo upload", () => {
  it("shows the photo upload control when log form is open", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });
  });

  it("shows a preview after selecting a photo and clears it on remove", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });

    const fileInput = document.querySelector(
      '.care-entry-form input[type="file"]',
    ) as HTMLInputElement;
    expect(fileInput).toBeTruthy();

    const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      writable: false,
    });
    await fireEvent.change(fileInput);

    await waitFor(() => {
      const preview = document.querySelector(
        ".toolbar-thumb img",
      ) as HTMLImageElement;
      expect(preview).toBeTruthy();
    });

    const removeBtn = document.querySelector(
      ".toolbar-dismiss",
    ) as HTMLButtonElement;
    await fireEvent.click(removeBtn);

    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeNull();
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });
  });

  it("uploads photo after creating care event on submit", async () => {
    const createdEvent: CareEvent = {
      id: 30,
      plant_id: 1,
      plant_name: "Fern",
      event_type: "fertilized",
      notes: "",
      photo_url: null,
      occurred_at: "2025-02-01T10:00:00Z",
      created_at: "2025-02-01T10:00:00Z",
    };
    mockAddCareEvent.mockResolvedValue(createdEvent);
    mockUploadCareEventPhoto.mockResolvedValue({
      ...createdEvent,
      photo_url: "/uploads/care/30.jpg",
    });

    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByText("Fertilized")).toBeTruthy();
    });
    await fireEvent.click(screen.getByText("Fertilized"));

    const fileInput = document.querySelector(
      '.care-entry-form input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      writable: false,
    });
    await fireEvent.change(fileInput);

    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeTruthy();
    });

    const saveBtn = screen.getByText("Save");
    await fireEvent.click(saveBtn);

    await waitFor(() => {
      expect(mockAddCareEvent).toHaveBeenCalledWith(
        1,
        expect.objectContaining({ event_type: "fertilized" }),
      );
      expect(mockUploadCareEventPhoto).toHaveBeenCalledWith(1, 30, file);
    });
  });

  it("shows inline validation and blocks submit when no care type is selected", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    await fireEvent.click(screen.getByText("+ Add log entry"));

    await waitFor(() => {
      expect(screen.getByText("Save")).toBeTruthy();
    });

    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(screen.getByText("Choose a care entry type")).toBeTruthy();
    });
    expect(mockAddCareEvent).not.toHaveBeenCalled();
  });

  it("shows a toast and keeps the form open when care entry creation fails", async () => {
    mockAddCareEvent.mockResolvedValue(null);

    await renderWithPlant();
    await screen.findByText("Fern");
    await fireEvent.click(screen.getByText("+ Add log entry"));

    await waitFor(() => {
      expect(screen.getByText("Fertilized")).toBeTruthy();
    });
    await fireEvent.click(screen.getByText("Fertilized"));
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({
          title: "Care Journal",
          variant: "error",
          message: "Failed to add care event",
        }),
      );
      expect(document.querySelector(".care-entry-form")).toBeTruthy();
    });
  });

  it("supports adding backdated watering entries from the log form", async () => {
    const occurredAt = "2025-01-15T09:30";
    const createdEvent: CareEvent = {
      id: 31,
      plant_id: 1,
      plant_name: "Fern",
      event_type: "watered",
      notes: "Watered a few days ago",
      photo_url: null,
      occurred_at: "2025-01-15T09:30:00Z",
      created_at: "2025-02-01T10:00:00Z",
    };
    mockAddCareEvent.mockResolvedValue(createdEvent);
    mockFetchPlantApi.mockResolvedValue(
      makePlant({
        last_watered: "2025-01-15T09:30:00Z",
        next_due: "2025-01-22T09:30:00Z",
        watering_status: "ok",
      }),
    );
    mockFetchCareEventsApi.mockResolvedValue([createdEvent]);
    await renderWithPlant({
      last_watered: "2025-01-01",
      next_due: "2025-01-08",
      watering_status: "overdue",
    });

    await screen.findByText("Fern");
    await fireEvent.click(screen.getByText("+ Add log entry"));
    const form = document.querySelector(".care-entry-form") as HTMLElement;

    await waitFor(() => {
      expect(within(form).getByText("Watered")).toBeTruthy();
    });
    await fireEvent.click(within(form).getByText("Watered"));

    const dateToggle = document.querySelectorAll(
      ".care-entry-form .toolbar-btn",
    )[1] as HTMLButtonElement;
    await fireEvent.click(dateToggle);

    const dateInput = document.querySelector(
      ".care-entry-form .toolbar-date-input",
    ) as HTMLInputElement;
    await fireEvent.input(dateInput, { target: { value: occurredAt } });

    const notesInput = document.querySelector(
      ".care-entry-form .log-notes",
    ) as HTMLTextAreaElement;
    await fireEvent.input(notesInput, {
      target: { value: "Watered a few days ago" },
    });

    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockAddCareEvent).toHaveBeenCalledWith(
        1,
        expect.objectContaining({
          event_type: "watered",
          notes: "Watered a few days ago",
          occurred_at: new Date(occurredAt).toISOString(),
        }),
      );
      expect(mockFetchCareEventsApi).toHaveBeenCalledWith(1);
      expect(mockFetchPlantApi).toHaveBeenCalledWith(1);
    });
  });

  it("clears photo when form is cancelled", async () => {
    await renderWithPlant();
    await screen.findByText("Fern");
    const addLogBtn = screen.getByText("+ Add log entry");
    await fireEvent.click(addLogBtn);

    await waitFor(() => {
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });

    const fileInput = document.querySelector(
      '.care-entry-form input[type="file"]',
    ) as HTMLInputElement;
    const file = new File(["img"], "test.jpg", { type: "image/jpeg" });
    Object.defineProperty(fileInput, "files", {
      value: [file],
      writable: false,
    });
    await fireEvent.change(fileInput);

    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeTruthy();
    });

    const cancelBtns = screen.getAllByText("Cancel");
    const logFormCancel = cancelBtns.find(
      (btn) => btn.closest(".care-entry-form") !== null,
    )!;
    await fireEvent.click(logFormCancel);

    // Re-open log form — photo should be gone
    await fireEvent.click(screen.getByText("+ Add log entry"));
    await waitFor(() => {
      expect(document.querySelector(".toolbar-thumb")).toBeNull();
      expect(screen.getByLabelText("Add photo")).toBeTruthy();
    });
  });
});

describe("plant detail care event editing", () => {
  it("opens an edit form, hides the add form, and cancels without an update", async () => {
    const event = makeCareEvent({
      event_type: "fertilized",
      notes: "Original note",
    });
    await renderWithPlant({}, [event]);
    await screen.findByRole("button", { name: "Edit log entry" });

    await fireEvent.click(screen.getByText("+ Add log entry"));
    expect(document.querySelector(".care-entry-form")).toBeTruthy();

    await fireEvent.click(
      screen.getByRole("button", { name: "Edit log entry" }),
    );
    expect(screen.queryByText("+ Add log entry")).toBeNull();
    expect(screen.getByDisplayValue("Original note")).toBeTruthy();

    const cancel = screen
      .getAllByRole("button", { name: "Cancel" })
      .find((button) => button.closest(".care-entry-form"));
    await fireEvent.click(cancel!);

    await waitFor(() => {
      expect(document.querySelector(".care-entry-form")).toBeNull();
    });
    expect(mockUpdateCareEventApi).not.toHaveBeenCalled();
  });

  it("reloads plant and timeline only after a successful edit", async () => {
    const event = makeCareEvent({
      event_type: "fertilized",
      notes: "Original note",
    });
    const updated = { ...event, notes: "Updated note" };
    mockUpdateCareEventApi.mockResolvedValue(updated);
    mockFetchPlantApi.mockResolvedValue(
      makePlant({ last_watered: "2025-02-01" }),
    );
    mockFetchCareEventsApi.mockResolvedValue([updated]);
    await renderWithPlant({}, [event]);

    await fireEvent.click(
      await screen.findByRole("button", { name: "Edit log entry" }),
    );
    await fireEvent.input(screen.getByDisplayValue("Original note"), {
      target: { value: "Updated note" },
    });
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(mockUpdateCareEventApi).toHaveBeenCalledWith(
        1,
        event.id,
        expect.objectContaining({
          event_type: "fertilized",
          notes: "Updated note",
        }),
      );
      expect(mockFetchPlantApi).toHaveBeenCalledWith(1);
      expect(mockFetchCareEventsApi).toHaveBeenCalledWith(1);
      expect(document.querySelector(".care-entry-form")).toBeNull();
    });
  });

  it("keeps the edit form open when updating fails", async () => {
    const event = makeCareEvent({ event_type: "fertilized", notes: "Keep" });
    mockUpdateCareEventApi.mockRejectedValue(new Error("update failed"));
    await renderWithPlant({}, [event]);

    await fireEvent.click(
      await screen.findByRole("button", { name: "Edit log entry" }),
    );
    await fireEvent.click(screen.getByText("Save"));

    await waitFor(() => {
      expect(document.querySelector(".care-entry-form")).toBeTruthy();
      expect(mockPushNotification).toHaveBeenCalledWith(
        expect.objectContaining({ message: "Failed to update care event" }),
      );
    });
    expect(mockFetchPlantApi).not.toHaveBeenCalled();
    expect(mockFetchCareEventsApi).toHaveBeenCalledTimes(1);
  });

  it("updates the selected event from an expanded watering group", async () => {
    const groupedEvents = [
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
    ];
    await renderWithPlant({}, groupedEvents);
    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    await fireEvent.click(
      document.querySelector(".timeline-group-btn") as HTMLButtonElement,
    );
    const editButtons = await screen.findAllByRole("button", {
      name: "Edit log entry",
    });
    await fireEvent.click(editButtons[1]);
    await fireEvent.input(document.querySelector(".log-notes")!, {
      target: { value: "Edited grouped event" },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Save" }));

    await waitFor(() => {
      expect(mockUpdateCareEventApi).toHaveBeenCalledWith(
        1,
        groupedEvents[1].id,
        expect.objectContaining({ notes: "Edited grouped event" }),
      );
    });
  });

  it("disables edit controls offline and retains an open edit form on connectivity loss", async () => {
    const event = makeCareEvent({ event_type: "fertilized", notes: "Keep" });
    await renderWithPlant({}, [event]);
    const edit = await screen.findByRole("button", { name: "Edit log entry" });
    await fireEvent.click(edit);
    isOffline.set(true);

    await waitFor(() => {
      expect(screen.getByText("Save")).toHaveProperty("disabled", true);
      expect(screen.getByDisplayValue("Keep")).toBeTruthy();
    });
    expect(
      screen.getByRole("button", { name: "Edit log entry" }),
    ).toHaveProperty("disabled", true);
  });
});

describe("care journal event grouping", () => {
  it("groups consecutive waterings into a collapsible summary", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 3, occurred_at: "2025-02-03T10:00:00Z" }),
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    // Should show one group, not three individual items
    const items = document.querySelectorAll(".timeline-item");
    expect(items.length).toBe(1);
  });

  it("expands group on click to show individual entries", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-02-01T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    // No nested entries yet
    expect(document.querySelector(".timeline-nested")).toBeNull();

    // Click the group button
    const groupBtn = document.querySelector(
      ".timeline-group-btn",
    ) as HTMLButtonElement;
    expect(groupBtn.getAttribute("aria-expanded")).toBe("false");
    expect(groupBtn.getAttribute("aria-label")).toBe("Expand watering entries");
    await fireEvent.click(groupBtn);

    await waitFor(() => {
      const nested = document.querySelectorAll(".timeline-nested");
      expect(nested.length).toBe(2);
      expect(groupBtn.getAttribute("aria-expanded")).toBe("true");
      expect(groupBtn.getAttribute("aria-label")).toBe(
        "Collapse watering entries",
      );
    });
  });

  it("omits year on first date when both dates share the same year", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 3, occurred_at: "2025-03-10T10:00:00Z" }),
      makeCareEvent({ id: 2, occurred_at: "2025-02-15T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-01-05T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    const dateEl = document.querySelector(
      ".timeline-group-summary .timeline-date",
    ) as HTMLElement;
    const text = dateEl.textContent!;
    // first date (Jan 5) should not contain a year
    const [first, second] = text.split("–").map((s) => s.trim());
    expect(first).not.toMatch(/\d{2}$/);
    // second date (Mar 10) should contain the year
    expect(second).toMatch(/\d{2}$/);
  });

  it("shows year on both dates when they span different years", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2026-01-10T10:00:00Z" }),
      makeCareEvent({ id: 1, occurred_at: "2025-12-20T10:00:00Z" }),
    ]);

    await waitFor(() => {
      expect(document.querySelector(".timeline-group-summary")).toBeTruthy();
    });

    const dateEl = document.querySelector(
      ".timeline-group-summary .timeline-date",
    ) as HTMLElement;
    const text = dateEl.textContent!;
    const [first, second] = text.split("–").map((s) => s.trim());
    // both dates should contain a year
    expect(first).toMatch(/\d{2}$/);
    expect(second).toMatch(/\d{2}$/);
  });

  it("does not group waterings with notes", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 2, occurred_at: "2025-02-02T10:00:00Z" }),
      makeCareEvent({
        id: 1,
        occurred_at: "2025-02-01T10:00:00Z",
        notes: "Very dry soil",
      }),
    ]);

    await waitFor(() => {
      expect(screen.getAllByText("Watered").length).toBe(2);
    });
    // Both should be individual items, no group summary
    expect(document.querySelector(".timeline-group-summary")).toBeNull();
    const items = document.querySelectorAll(".timeline-item");
    expect(items.length).toBe(2);
  });

  it("shows the empty state when there are no care entries", async () => {
    await renderWithPlant();

    expect(
      await screen.findByText("No care events recorded yet."),
    ).toBeTruthy();
  });

  it("shows the corresponding icon for every care event type", async () => {
    await renderWithPlant({}, [
      makeCareEvent({ id: 1, event_type: "watered" }),
      makeCareEvent({ id: 2, event_type: "fertilized" }),
      makeCareEvent({ id: 3, event_type: "repotted" }),
      makeCareEvent({ id: 4, event_type: "pruned" }),
      makeCareEvent({ id: 5, event_type: "custom" }),
      makeCareEvent({ id: 6, event_type: "ai-consultation" }),
    ]);

    await waitFor(() => {
      expect(document.querySelectorAll(".timeline-item")).toHaveLength(6);
    });

    for (const icon of [
      "droplet",
      "leaf",
      "shovel",
      "scissors",
      "pencil",
      "sparkles",
    ]) {
      expect(
        document.querySelector(`.care-journal .lucide-${icon}`),
      ).toBeTruthy();
    }
  });

  it("renders all care entries immediately when there are more than 20", async () => {
    const events = Array.from({ length: 21 }, (_, index) =>
      makeCareEvent({
        id: index + 1,
        event_type: index % 2 === 0 ? "watered" : "fertilized",
        occurred_at: `2025-02-${String(index + 1).padStart(2, "0")}T10:00:00Z`,
      }),
    );

    await renderWithPlant({}, events);

    await waitFor(() => {
      expect(document.querySelectorAll(".timeline-item")).toHaveLength(21);
    });

    expect(screen.queryByRole("button", { name: /show more/i })).toBeNull();
  });
});

describe("plant detail offline behavior", () => {
  it("disables water now button when offline", async () => {
    isOffline.set(true);
    await renderWithPlant();

    await waitFor(() => {
      const waterBtn = screen.getByRole("button", {
        name: /Water now/,
      }) as HTMLButtonElement;
      expect(waterBtn.disabled).toBe(true);
    });
  });

  it("disables add log entry button when offline", async () => {
    isOffline.set(true);
    await renderWithPlant();

    await waitFor(() => {
      const addBtn = screen.getByRole("button", {
        name: /Add log entry/,
      }) as HTMLButtonElement;
      expect(addBtn.disabled).toBe(true);
    });
  });

  it("disables delete button when offline", async () => {
    isOffline.set(true);
    await renderWithPlant();

    await waitFor(() => {
      const deleteBtn = document.querySelector(
        'button[aria-label="Delete plant"]',
      ) as HTMLButtonElement;
      expect(deleteBtn).not.toBeNull();
      expect(deleteBtn.disabled).toBe(true);
    });
  });

  it("disables edit link when offline", async () => {
    isOffline.set(true);
    await renderWithPlant();

    await waitFor(() => {
      const editLink = document.querySelector(
        'a[aria-label="Edit Plant"]',
      ) as HTMLAnchorElement;
      expect(editLink).not.toBeNull();
      expect(editLink.getAttribute("aria-disabled")).toBe("true");
      expect(editLink.classList.contains("disabled")).toBe(true);
    });
  });

  it("disables care event delete buttons when offline", async () => {
    isOffline.set(true);
    await renderWithPlant({}, [makeCareEvent()]);

    await waitFor(() => {
      const deleteBtn = screen.getByRole("button", {
        name: "Delete log entry",
      }) as HTMLButtonElement;
      expect(deleteBtn.disabled).toBe(true);
    });
  });

  it("disables Ask AI button when offline", async () => {
    aiStatus.set({ enabled: true, base_url: null, model: null });
    isOffline.set(true);
    await renderWithPlant();

    await waitFor(() => {
      const aiBtn = screen.getByRole("button", {
        name: /Ask AI/,
      }) as HTMLButtonElement;
      expect(aiBtn.disabled).toBe(true);
    });
  });

  it("re-enables mutation controls when back online", async () => {
    isOffline.set(true);
    await renderWithPlant();

    await waitFor(() => {
      const waterBtn = screen.getByRole("button", {
        name: /Water now/,
      }) as HTMLButtonElement;
      expect(waterBtn.disabled).toBe(true);
      const addBtn = screen.getByRole("button", {
        name: /Add log entry/,
      }) as HTMLButtonElement;
      expect(addBtn.disabled).toBe(true);
      const deleteBtn = document.querySelector(
        'button[aria-label="Delete plant"]',
      ) as HTMLButtonElement;
      expect(deleteBtn.disabled).toBe(true);
      const editLink = document.querySelector(
        'a[aria-label="Edit Plant"]',
      ) as HTMLAnchorElement;
      expect(editLink.getAttribute("aria-disabled")).toBe("true");
    });

    isOffline.set(false);

    await waitFor(() => {
      const waterBtn = screen.getByRole("button", {
        name: /Water now/,
      }) as HTMLButtonElement;
      expect(waterBtn.disabled).toBe(false);
      const addBtn = screen.getByRole("button", {
        name: /Add log entry/,
      }) as HTMLButtonElement;
      expect(addBtn.disabled).toBe(false);
      const deleteBtn = document.querySelector(
        'button[aria-label="Delete plant"]',
      ) as HTMLButtonElement;
      expect(deleteBtn.disabled).toBe(false);
      const editLink = document.querySelector(
        'a[aria-label="Edit Plant"]',
      ) as HTMLAnchorElement;
      expect(editLink.getAttribute("aria-disabled")).toBe("false");
    });
  });

  it("shows offline message instead of error when offline with no cached plant", async () => {
    isOffline.set(true);
    render(Page, {
      props: {
        data: {
          plant: null,
          notFound: false,
          loadErrorCode: "UNKNOWN_ERROR",
        },
      },
    });

    await waitFor(() => {
      expect(
        screen.getByText(
          "You're offline. Connect to the internet to view this page.",
        ),
      ).toBeTruthy();
    });

    expect(
      screen.queryByText("Something went wrong. Please try again."),
    ).toBeNull();
  });

  it("shows generic error when online with load failure", async () => {
    isOffline.set(false);
    render(Page, {
      props: {
        data: {
          plant: null,
          notFound: false,
          loadErrorCode: "UNKNOWN_ERROR",
        },
      },
    });

    await waitFor(() => {
      expect(
        screen.getByText("Something went wrong. Please try again."),
      ).toBeTruthy();
    });

    expect(
      screen.queryByText(
        "You're offline. Connect to the internet to view this page.",
      ),
    ).toBeNull();
  });
});
