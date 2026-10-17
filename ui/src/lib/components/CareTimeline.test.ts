import { cleanup, render, screen } from "@testing-library/svelte";
import { tick } from "svelte";
import { get } from "svelte/store";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { CareEvent, EventType } from "#lib/api.js";
import {
  normalizeCareTimelineEvents,
  createCareTimelineGround,
  sampleCareTimelineGround,
} from "#lib/careTimeline.js";
import { en } from "#lib/i18n/en.js";
import { locale, translations, type Locale } from "#lib/stores/locale.js";
import CareTimeline, {
  type Props,
  type WateringFeedback,
} from "./CareTimeline.svelte";
import source from "./CareTimeline.svelte?raw";

vi.mock("#lib/careTimeline.js", async (importOriginal) => {
  const actual = await importOriginal<typeof import("#lib/careTimeline.js")>();
  return {
    ...actual,
    normalizeCareTimelineEvents: vi.fn(actual.normalizeCareTimelineEvents),
  };
});

const NOW = Date.parse("2026-04-01T12:00:00Z");
const DAY = 86_400_000;
function event(
  id: number,
  days: number,
  overrides: Partial<CareEvent> = {},
): CareEvent {
  return {
    id,
    plant_id: 1,
    plant_name: "Not rendered",
    event_type: "watered",
    notes: null,
    photo_url: null,
    occurred_at: new Date(NOW - days * DAY).toISOString(),
    created_at: new Date(NOW).toISOString(),
    ...overrides,
  };
}
const events = [
  event(1, 21),
  event(2, 14, { notes: "Keep", photo_url: "/keep.jpg" }),
  event(3, 7),
];
const props = { plantId: 1, events } satisfies Props;
class SizeObserver {
  static instances: SizeObserver[] = [];
  observe = vi.fn();
  disconnect = vi.fn();
  constructor(private callback: ResizeObserverCallback) {
    SizeObserver.instances.push(this);
  }
  resize(
    width: number,
    height: number,
    target = this.observe.mock.calls[0][0] as Element,
  ) {
    this.callback(
      [{ target, contentRect: { width, height } } as ResizeObserverEntry],
      this as unknown as ResizeObserver,
    );
  }
}
let hidden: ReturnType<typeof vi.spyOn>;
let fetchMock: ReturnType<typeof vi.fn>;
let media: MediaQueryList;
let mediaListeners: Set<(event: MediaQueryListEvent) => void>;
function motionPreference(reduced: boolean) {
  Object.defineProperty(media, "matches", {
    value: reduced,
    configurable: true,
  });
  for (const listener of mediaListeners)
    listener({ matches: reduced } as MediaQueryListEvent);
}
function isRevealing(container: HTMLElement) {
  return container
    .querySelector(".care-timeline")!
    .classList.contains("revealing");
}
const fixtures: HTMLElement[] = [];
function foreground(x: number, y: number, width: number, height: number) {
  const element = document.createElement("span");
  document.body.append(element);
  fixtures.push(element);
  const measure = vi.fn(() => new DOMRect(x, y, width, height));
  element.getBoundingClientRect = measure;
  return { element, measure };
}
function hostBounds(x: number, y: number, width: number, height: number) {
  const bounds = { current: new DOMRect(x, y, width, height) };
  vi.spyOn(HTMLElement.prototype, "getBoundingClientRect").mockImplementation(
    function (this: HTMLElement) {
      return this.classList.contains("care-timeline")
        ? bounds.current
        : new DOMRect();
    },
  );
  return bounds;
}
function positions(container: HTMLElement) {
  return [...container.querySelectorAll(".event")].map((element) => ({
    id: Number(element.getAttribute("data-event-id")),
    xy: element.getAttribute("transform")!.slice(10, -1).split(" ").map(Number),
  }));
}
function expectRenderedClearance(container: HTMLElement) {
  const height = Number(
    container
      .querySelector(".care-timeline > svg")!
      .getAttribute("viewBox")!
      .split(" ")[3],
  );
  const points = positions(container);
  for (const [index, { xy }] of points.entries()) {
    const stem = container.querySelectorAll(".stem")[index];
    const foot = Number(stem.getAttribute("y2"));
    expect(xy[1] - 7).toBeGreaterThanOrEqual(0);
    expect(xy[1] + 7).toBeLessThan(height);
    expect(foot).toBeGreaterThanOrEqual(18 - 1e-9);
    expect(xy[1] + foot + 4).toBeLessThan(height);
    for (const other of points.slice(index + 1)) {
      if (Math.abs(xy[0] - other.xy[0]) < 18)
        expect(Math.abs(xy[1] - other.xy[1])).toBeGreaterThanOrEqual(18 - 1e-9);
    }
  }
}
function groundPath(container: HTMLElement) {
  return container.querySelector(".ground")?.getAttribute("d");
}
function shields(container: HTMLElement) {
  return [...container.querySelectorAll("mask g rect")].map((rect) =>
    Object.fromEntries(
      ["x", "y", "width", "height"].map((key) => [
        key,
        Number(rect.getAttribute(key)),
      ]),
    ),
  );
}
async function setLanguage(language: Locale) {
  locale.set(language);
  await vi.dynamicImportSettled();
  await tick();
}
async function advance(ms: number) {
  await vi.advanceTimersByTimeAsync(ms);
  await tick();
}
async function visibility(value: boolean) {
  hidden.mockReturnValue(value);
  document.dispatchEvent(new Event("visibilitychange"));
  await tick();
}

beforeEach(() => {
  vi.useFakeTimers();
  vi.setSystemTime(NOW);
  SizeObserver.instances = [];
  vi.stubGlobal("ResizeObserver", SizeObserver);
  mediaListeners = new Set();
  media = {
    matches: false,
    addEventListener: vi.fn((_type, listener) => mediaListeners.add(listener)),
    removeEventListener: vi.fn((_type, listener) =>
      mediaListeners.delete(listener),
    ),
  } as unknown as MediaQueryList;
  vi.stubGlobal(
    "matchMedia",
    vi.fn(() => media),
  );
  fetchMock = vi.fn();
  vi.stubGlobal("fetch", fetchMock);
  hidden = vi.spyOn(document, "hidden", "get").mockReturnValue(false);
  vi.mocked(normalizeCareTimelineEvents).mockClear();
  locale.set("en");
});
afterEach(() => {
  cleanup();
  for (const element of fixtures.splice(0)) element.remove();
  locale.set("en");
  vi.clearAllTimers();
  vi.useRealTimers();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});

describe("CareTimeline presentation", () => {
  it("renders the approved floating icons, dotted stems, fixed ground and contact shadows", () => {
    const { container } = render(CareTimeline, { props });
    const ground = createCareTimelineGround(1, 320, 140);
    expect(positions(container).map(({ xy }) => xy[0])).toEqual([
      160,
      32 + (28 / 42) * 256,
      32 + (35 / 42) * 256,
    ]);
    expect(groundPath(container)).toBe(ground.path);
    expect(ground.path.match(/C/g)).toHaveLength(4);
    expect(container.querySelector(".ground-shadow")?.getAttribute("d")).toBe(
      ground.path,
    );
    expect(
      container.querySelector(".ground-shadow")?.getAttribute("transform"),
    ).toBe("translate(0 3)");
    for (const [index, node] of [
      ...container.querySelectorAll(".event"),
    ].entries()) {
      const { xy } = positions(container)[index];
      const foot = Number(node.querySelector(".stem")?.getAttribute("y2"));
      expect(xy[1] + foot).toBeCloseTo(
        sampleCareTimelineGround(ground, xy[0]),
        10,
      );
      expect(foot).toBeGreaterThanOrEqual(18);
      expect(Number(node.querySelector(".contact")?.getAttribute("cy"))).toBe(
        foot,
      );
      expect(
        Number(node.querySelector(".contact-shadow")?.getAttribute("cy")),
      ).toBe(foot + 2);
    }
    expect(container.querySelector(".thread, .rail, .period, text")).toBeNull();
    expect(source).toContain("stroke-dasharray: 1 5");
    expect(source).not.toMatch(/variant|windowDays|intervalDays|preview/i);
  });

  it("keeps ground unchanged and collision reflow deterministic as density, types and order change", async () => {
    const view = render(CareTimeline, { props });
    const path = groundPath(view.container);
    const original = positions(view.container);
    const dense = [
      ...events,
      ...Array.from({ length: 24 }, (_, index) =>
        event(100 + index, index + 0.5),
      ),
    ];
    await view.rerender({ ...props, events: dense.reverse() });
    expect(view.container.querySelectorAll(".event")).toHaveLength(27);
    expect(groundPath(view.container)).toBe(path);
    const crowded = positions(view.container);
    expect(crowded.filter(({ id }) => id < 100).map(({ xy }) => xy[0])).toEqual(
      original.map(({ xy }) => xy[0]),
    );
    expectRenderedClearance(view.container);
    await view.rerender({ ...props, events: [...dense].reverse() });
    expect(positions(view.container)).toEqual(crowded);
    await view.rerender({
      ...props,
      events: events.map((record) => ({
        ...record,
        event_type: "pruned" as const,
      })),
    });
    expect(positions(view.container)).toEqual(original);
    await view.rerender({ ...props, events: events.slice(0, 1) });
    expect(groundPath(view.container)).toBe(path);
  });

  it.each([2, 4, 12, 30])(
    "reports and honors the minimum for %s simultaneous mixed-type events without height feedback",
    async (count) => {
      const host = hostBounds(0, 0, 288, 20);
      const report = vi.fn();
      const types: EventType[] = [
        "watered",
        "fertilized",
        "repotted",
        "pruned",
        "custom",
        "ai-consultation",
      ];
      const data = Array.from({ length: count }, (_, index) =>
        event(index + 1, 21, { event_type: types[index % types.length] }),
      );
      const view = render(CareTimeline, {
        props: { ...props, events: data, onMinimumHeight: report },
      });
      const fixedHeight = 20 + 18 * (Math.max(4, count) - 1) + 32;
      const minimum = Math.max(
        140,
        Math.ceil(Math.min(fixedHeight / 0.82, fixedHeight + 36)),
      );
      expect(report).toHaveBeenLastCalledWith(minimum);
      expect(positions(view.container)).toHaveLength(count);
      expect(
        view.container.querySelector<HTMLElement>(".care-timeline")!.style
          .minHeight,
      ).toBe(`${minimum}px`);
      expect(positions(view.container).every(({ xy }) => xy[0] === 144)).toBe(
        true,
      );
      expectRenderedClearance(view.container);
      for (const width of [288, 640, 1024]) {
        host.current = new DOMRect(0, 0, width, Math.max(300, minimum));
        SizeObserver.instances.at(-1)!.resize(width, Math.max(300, minimum));
        await tick();
        expect(report).toHaveBeenCalledTimes(1);
        expect(
          positions(view.container).every(({ xy }) => xy[0] === width / 2),
        ).toBe(true);
        expectRenderedClearance(view.container);
      }
      const original = positions(view.container);
      const grownGround = groundPath(view.container);
      await view.rerender({
        ...props,
        events: [...data].reverse(),
        onMinimumHeight: report,
      });
      expect(positions(view.container)).toEqual(original);
      expect(report).toHaveBeenCalledTimes(1);
      await view.rerender({
        ...props,
        events: data.slice(0, 2),
        onMinimumHeight: report,
      });
      expect(report).toHaveBeenLastCalledWith(140);
      // Event removal alone cannot bend the ground at the same measured size.
      expect(groundPath(view.container)).toBe(grownGround);
      SizeObserver.instances.at(-1)!.resize(1024, 140);
      await tick();
      expect(
        view.container
          .querySelector(".care-timeline > svg")
          ?.getAttribute("viewBox"),
      ).toBe("0 0 1024 140");
      expectRenderedClearance(view.container);
      await view.rerender({ ...props, events: data, onMinimumHeight: report });
      await view.rerender({
        ...props,
        events: data,
        plantId: 2,
        onMinimumHeight: report,
      });
      expect(report).toHaveBeenLastCalledWith(0);
      expect(view.container.querySelector("svg")).toBeNull();
      expect(fetchMock).not.toHaveBeenCalled();
    },
  );

  it("replans nearby/neighboring-day collisions on width changes and shrinks without preserving grown height", async () => {
    hostBounds(0, 0, 100, 140);
    const report = vi.fn();
    const data = Array.from({ length: 30 }, (_, index) =>
      event(index + 1, 14 + index / 4),
    );
    const view = render(CareTimeline, {
      props: { ...props, events: data, onMinimumHeight: report },
    });
    const grown = report.mock.lastCall![0] as number;
    expect(grown).toBeGreaterThan(300);
    expectRenderedClearance(view.container);
    const original = positions(view.container);
    SizeObserver.instances.at(-1)!.resize(2048, grown);
    await tick();
    expect(report).toHaveBeenLastCalledWith(140);
    expectRenderedClearance(view.container);
    SizeObserver.instances.at(-1)!.resize(100, 140);
    await tick();
    expect(report).toHaveBeenLastCalledWith(grown);
    expect(positions(view.container)).toEqual(original);
    await advance(60_000);
    expect(positions(view.container).map(({ xy }) => xy[1])).toEqual(
      original.map(({ xy }) => xy[1]),
    );
    expectRenderedClearance(view.container);
    await view.rerender({ ...props, events: [], onMinimumHeight: report });
    expect(report).toHaveBeenLastCalledWith(0);
    expect(
      view.container.querySelector<HTMLElement>(".care-timeline")!.style
        .minHeight,
    ).toBe("0px");
    view.unmount();
    expect(SizeObserver.instances.at(-1)!.disconnect).toHaveBeenCalledOnce();
    expect(vi.getTimerCount()).toBe(0);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("adapts to measured sizes and reorders/localizes without random vertical repositioning", async () => {
    const host = hostBounds(0, 0, 320, 140);
    const view = render(CareTimeline, { props });
    const original = positions(view.container);
    await view.rerender({ ...props, events: [...events].reverse() });
    await setLanguage("de");
    expect(positions(view.container)).toEqual(original);
    host.current = new DOMRect(0, 0, 640, 140);
    SizeObserver.instances.at(-1)!.resize(640, 140);
    await tick();
    expect(positions(view.container)).toEqual(
      original.map(({ id, xy }) => ({ id, xy: [xy[0] * 2, xy[1]] })),
    );
    expect(view.container.querySelector("svg")?.getAttribute("viewBox")).toBe(
      "0 0 640 140",
    );
    expect(groundPath(view.container)).toBe(
      createCareTimelineGround(1, 640, 140).path,
    );
    SizeObserver.instances.at(-1)!.resize(0, 0);
    await tick();
    expect(view.container.querySelector("svg")?.getAttribute("viewBox")).toBe(
      "0 0 320 140",
    );
  });

  it("renders every raw type using the existing 14px icons and theme colors", () => {
    const types: EventType[] = [
      "watered",
      "fertilized",
      "repotted",
      "pruned",
      "custom",
      "ai-consultation",
    ];
    const { container } = render(CareTimeline, {
      props: {
        ...props,
        events: types.map((event_type, index) =>
          event(index + 1, 20 - index, { event_type }),
        ),
      },
    });
    const nodes = [...container.querySelectorAll(".event")];
    expect(nodes.map((node) => node.getAttribute("data-event-type"))).toEqual(
      types,
    );
    expect(nodes.map((node) => node.getAttribute("color"))).toEqual([
      "var(--color-water)",
      "var(--color-primary)",
      "var(--color-secondary)",
      "var(--color-text-muted)",
      "var(--color-text-muted)",
      "var(--color-ai)",
    ]);
    for (const node of nodes) {
      expect(node.querySelector("svg")?.getAttribute("width")).toBe("14");
      expect(node.querySelector(".lucide")?.getAttribute("opacity")).toBe(
        "0.46",
      );
    }
  });

  it("keeps separate equal-instant records, rejecting foreign, invalid, old and future records", () => {
    const { container } = render(CareTimeline, {
      props: {
        ...props,
        events: [
          event(1, 14, { occurred_at: "2026-03-18 12:00:00" }),
          event(2, 14, { occurred_at: "2026-03-18T14:00:00+02:00" }),
          event(3, 14, { occurred_at: "2026-03-18T12:00:00.000000" }),
          event(4, 14, { plant_id: 2 }),
          event(5, 43),
          event(6, -1),
          event(7, 0, { occurred_at: "invalid" }),
          event(8, 0, { occurred_at: "2026-02-30T12:00:00Z" }),
          event(9, -1 / DAY),
        ],
      },
    });
    expect(positions(container).map(({ id, xy }) => [id, xy[0]])).toEqual([
      [1, 32 + (28 / 42) * 256],
      [2, 32 + (28 / 42) * 256],
      [3, 32 + (28 / 42) * 256],
    ]);
  });

  it("renders the inclusive 42-day boundary and day-35 history at true positions without requests", () => {
    const { container } = render(CareTimeline, {
      props: {
        ...props,
        events: [
          event(1, 42),
          event(2, 35),
          event(3, 42, {
            occurred_at: new Date(NOW - 42 * DAY - 1).toISOString(),
          }),
          event(4, -1 / DAY),
        ],
      },
    });
    expect(positions(container).map(({ id, xy }) => [id, xy[0]])).toEqual([
      [1, 32],
      [2, 32 + (7 / 42) * 256],
    ]);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it.each([240, 288, 320, 640, 1024])(
    "keeps complete boundary icons inside the opaque fade at %spx with precise time gaps and full-width ground contact",
    (width) => {
      hostBounds(0, 0, width, 140);
      const data = [
        event(1, 42),
        event(2, 35),
        event(3, 35 - 1 / 24),
        event(4, 0),
      ];
      const { container } = render(CareTimeline, {
        props: { ...props, events: data },
      });
      const points = positions(container);
      const span = width * 0.8;
      expect(points[0].xy[0]).toBeCloseTo(width * 0.1, 10);
      expect(points[1].xy[0]).toBeCloseTo(width * 0.1 + span / 6, 10);
      expect(points[3].xy[0]).toBeCloseTo(width * 0.9, 10);
      expect(points[1].xy[0] - points[0].xy[0]).toBeCloseTo(
        (7 / 42) * span,
        10,
      );
      expect(points[2].xy[0] - points[1].xy[0]).toBeCloseTo(
        span / (42 * 24),
        10,
      );
      const stops = container.querySelectorAll("linearGradient stop");
      expect(stops[1].getAttribute("offset")).toBe("0.06");
      expect(stops[2].getAttribute("offset")).toBe("0.94");
      expect(stops[1].getAttribute("stop-opacity")).toBe("1");
      expect(stops[2].getAttribute("stop-opacity")).toBe("1");
      const ground = createCareTimelineGround(1, width, 140);
      expect(groundPath(container)).toBe(ground.path);
      for (const [index, { xy }] of points.entries()) {
        const node = container.querySelectorAll(".event")[index];
        expect(node.querySelector(".event-icon")?.getAttribute("width")).toBe(
          "14",
        );
        expect(xy[0] - 7).toBeGreaterThanOrEqual(width * 0.06);
        expect(xy[0] + 7).toBeLessThanOrEqual(width * 0.94);
        const foot = Number(node.querySelector(".stem")?.getAttribute("y2"));
        expect(xy[1] + foot).toBeCloseTo(
          sampleCareTimelineGround(ground, xy[0]),
          10,
        );
        expect(Number(node.querySelector(".contact")?.getAttribute("cy"))).toBe(
          foot,
        );
      }
      expectRenderedClearance(container);
    },
  );

  it("is noninteractive and transparent with soft outer edges and achromatic paint confined to masks", () => {
    const { container } = render(CareTimeline, { props });
    const graph = screen.getByRole("img", {
      name: en.plant.careJournalSection,
    });
    expect(graph.getAttribute("focusable")).toBe("false");
    expect(
      container.querySelectorAll(
        "button, a, input, [tabindex], text, .caption, .message, .legend",
      ),
    ).toHaveLength(0);
    expect(container.querySelectorAll("div")).toHaveLength(1);
    expect(container.textContent).not.toContain("Not rendered");
    expect(source).toMatch(/pointer-events:\s*none/);
    expect(source).toMatch(/position:\s*absolute/);
    expect(source).toMatch(/inset:\s*0/);
    expect(source).not.toMatch(
      /background(?:-color)?\s*:|padding|margin|#[\da-f]{3,8}\b/i,
    );
    const stops = container.querySelectorAll("linearGradient stop");
    expect(stops[0].getAttribute("stop-opacity")).toBe("0");
    expect(stops[stops.length - 1].getAttribute("stop-opacity")).toBe("0");
    expect(
      container.querySelector("mask")?.style.getPropertyValue("mask-type"),
    ).toBe("luminance");
    expect(
      container.querySelector("mask")?.getAttribute("maskContentUnits"),
    ).toBe("userSpaceOnUse");
    expect(
      container.querySelector("feGaussianBlur")?.getAttribute("stdDeviation"),
    ).toBe("12");
    expect(
      container.querySelectorAll(
        'svg > :not(defs) [fill="black"], svg > :not(defs) [fill="white"]',
      ),
    ).toHaveLength(0);
  });

  it("uses unique referenced metadata, gradient, mask and filter IDs for each instance", () => {
    render(CareTimeline, { props });
    render(CareTimeline, { props });
    const ids = [...document.querySelectorAll("[id]")].map((node) => node.id);
    expect(new Set(ids).size).toBe(ids.length);
    for (const svg of document.querySelectorAll(".care-timeline > svg")) {
      expect(svg.getAttribute("aria-labelledby")).toBe(
        svg.querySelector("title")?.id,
      );
      expect(svg.getAttribute("aria-describedby")).toBe(
        svg.querySelector("desc")?.id,
      );
      expect(svg.querySelector("mask rect")?.getAttribute("fill")).toBe(
        `url(#${svg.querySelector("linearGradient")?.id})`,
      );
      expect(svg.querySelector("mask g")?.getAttribute("filter")).toBe(
        `url(#${svg.querySelector("filter")?.id})`,
      );
      expect(
        svg
          .querySelector(".ground")
          ?.parentElement?.parentElement?.getAttribute("mask"),
      ).toBe(`url(#${svg.querySelector("mask")?.id})`);
    }
  });

  it.each(["en", "de", "es"] as const)(
    "localizes journal title and period/status metadata to %s without visible copy",
    async (language) => {
      const { container } = render(CareTimeline, {
        props: { ...props, historyError: true },
      });
      const original = positions(container);
      await setLanguage(language);
      const dictionary = get(translations);
      expect(
        screen.getByRole("img", { name: dictionary.plant.careJournalSection }),
      ).toBeTruthy();
      const format = new Intl.DateTimeFormat(language, {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
      expect(container.querySelector("desc")?.textContent).toBe(
        `${format.format(NOW - 42 * DAY)} – ${format.format(NOW)} · ${dictionary.error.loadCareEvents}`,
      );
      expect(positions(container)).toEqual(original);
      expect(
        container.querySelector("text, .period, .legend, .caption"),
      ).toBeNull();
      expect(fetchMock).not.toHaveBeenCalled();
    },
  );

  it("reveals the ground and shadow together behind fixed edge/content masks", () => {
    const { container } = render(CareTimeline, { props });
    const reveal = container.querySelector(".ground-reveal")!;
    expect(reveal.querySelector(".ground")).not.toBeNull();
    expect(reveal.querySelector(".ground-shadow")).not.toBeNull();
    expect(reveal.getAttribute("clip-path")).toBe(
      `url(#${container.querySelector("clipPath")!.id})`,
    );
    expect(reveal.parentElement!.getAttribute("mask")).toBe(
      `url(#${container.querySelector("mask")!.id})`,
    );
    expect(container.querySelector("mask .ground-reveal-clip")).toBeNull();
    expect(container.querySelector(".event")!.parentElement).toBe(
      reveal.parentElement,
    );
    expect(source).not.toMatch(
      /Math.random|requestAnimationFrame|animateTransform/,
    );
  });
});

describe("CareTimeline progressive entrance", () => {
  it("starts with eligible history, reveals for 2200ms and finishes once at 2600ms", async () => {
    const { container } = render(CareTimeline, { props });
    const original = positions(container);
    expect(matchMedia).toHaveBeenCalledWith("(prefers-reduced-motion: reduce)");
    expect(isRevealing(container)).toBe(true);
    expect(container.querySelector(".entrance-pending")).toBeNull();
    expect(vi.getTimerCount()).toBe(2);
    await advance(600);
    expect(isRevealing(container)).toBe(true);
    expect(positions(container)).toEqual(original);
    await advance(1600);
    expect(isRevealing(container)).toBe(true);
    await advance(399);
    expect(isRevealing(container)).toBe(true);
    await advance(1);
    expect(isRevealing(container)).toBe(false);
    expect(positions(container)).toEqual(original);
    expect(vi.getTimerCount()).toBe(1);
    expect(source).toMatch(/ground-entrance 2200ms linear both/);
  });

  it("waits for the first eligible history, including a late arrival", async () => {
    const view = render(CareTimeline, {
      props: { ...props, events: [], loading: true },
    });
    expect(isRevealing(view.container)).toBe(false);
    expect(vi.getTimerCount()).toBe(1);
    await advance(5000);
    await view.rerender({ ...props, events: [event(5, -1)] });
    expect(isRevealing(view.container)).toBe(false);
    await view.rerender(props);
    expect(isRevealing(view.container)).toBe(true);
    expect(vi.getTimerCount()).toBe(2);
    await advance(2599);
    expect(isRevealing(view.container)).toBe(true);
    await advance(1);
    expect(isRevealing(view.container)).toBe(false);
  });

  it("uses x-ordered stagger, opacity-only event fades and bounded icon-only scaling", async () => {
    const data = [event(1, 42), event(2, 31.5), event(3, 21), event(4, 0)];
    const view = render(CareTimeline, { props: { ...props, events: data } });
    expect(
      [...view.container.querySelectorAll<SVGElement>(".event")].map((node) =>
        node.style.getPropertyValue("--event-delay"),
      ),
    ).toEqual(["220ms", "660ms", "1100ms", "1980ms"]);
    expect(source).toContain(
      "Math.max(0, Math.min(1, point.x / size.width)) * 2200",
    );
    expect(source).toMatch(/event-entrance 220ms ease-out both/);
    expect(source).toMatch(/stem-entrance 360ms ease-out both/);
    expect(source).toMatch(
      /\.stem\s*\{[^}]*transform-box: fill-box;[^}]*transform-origin: center bottom;/,
    );
    const stem = source
      .split("@keyframes stem-entrance")[1]
      .split("@keyframes")[0];
    expect(stem.match(/scaleY\([\d.]+\)/g)).toEqual(["scaleY(0)", "scaleY(1)"]);
    expect(source).toMatch(/icon-entrance 300ms ease-out both/);
    expect(source).toMatch(/var\(--event-delay\) \+ 80ms/);
    const fade = source
      .split("@keyframes event-entrance")[1]
      .split("@keyframes")[0];
    expect(fade).toMatch(/opacity: 0/);
    expect(fade).toMatch(/opacity: 1/);
    expect(fade).not.toMatch(/transform|translate|scale/);
    const icon = source
      .split("@keyframes icon-entrance")[1]
      .split(".dotted")[0];
    expect(icon.match(/scale\([\d.]+\)/g)).toEqual(["scale(0.65)", "scale(1)"]);
    expect(source).toMatch(/transform-box: fill-box/);
    expect(source).toMatch(/transform-origin: center/);
    for (const node of view.container.querySelectorAll(".event")) {
      expect(
        node.querySelector(":scope > svg.event-icon")?.getAttribute("opacity"),
      ).toBe("0.46");
      expect(node.querySelector(".contact")?.getAttribute("opacity")).toBe(
        "0.3",
      );
      expect(
        node.querySelector(".contact-shadow")?.getAttribute("opacity"),
      ).toBe("0.1");
    }
    const original = positions(view.container);
    await advance(2600);
    expect(positions(view.container)).toEqual(original);
  });

  it("keeps simultaneous icons collision-free at their maximum entrance scale without moving time positions", async () => {
    const data = Array.from({ length: 12 }, (_, index) => event(index + 1, 21));
    const view = render(CareTimeline, { props: { ...props, events: data } });
    const original = positions(view.container);
    expect(original.every(({ xy }) => xy[0] === 160)).toBe(true);
    // The final 14px footprint is the maximum at every animation frame.
    for (const ms of [0, 600, 1100, 80, 300, 520]) {
      await advance(ms);
      expect(positions(view.container)).toEqual(original);
      expectRenderedClearance(view.container);
    }
    expect(isRevealing(view.container)).toBe(false);
  });

  it("does not restart for redraws or temporarily empty history, even during the reveal", async () => {
    const view = render(CareTimeline, { props });
    const ground = view.container.querySelector(".ground-reveal-clip");
    await advance(1000);
    SizeObserver.instances[0].resize(640, 140);
    await setLanguage("de");
    document.documentElement.classList.add("dark");
    await view.rerender({
      ...props,
      events: [...events].reverse(),
      historyError: true,
    });
    expect(view.container.querySelector(".ground-reveal-clip")).toBe(ground);
    expect(vi.getTimerCount()).toBe(2);
    await view.rerender({ ...props, events: [] });
    await view.rerender(props);
    expect(
      view.container
        .querySelector<SVGElement>(".ground-reveal-clip")!
        .style.getPropertyValue("--entrance-elapsed"),
    ).toBe("1000ms");
    expect(
      view.container
        .querySelector<SVGElement>(".event")!
        .style.getPropertyValue("--entrance-elapsed"),
    ).toBe("1000ms");
    await view.rerender({
      ...props,
      events: events.map((record) => ({
        ...record,
        event_type: "pruned" as const,
      })),
    });
    expect(
      view.container
        .querySelector<SVGElement>(".event-icon")!
        .style.getPropertyValue("--entrance-elapsed"),
    ).toBe("1000ms");
    await advance(1600);
    expect(isRevealing(view.container)).toBe(false);
    await advance(60_000);
    await view.rerender({ ...props, events: [] });
    await view.rerender({ ...props, events: [...events, event(9, 1)] });
    expect(isRevealing(view.container)).toBe(false);
    expect(vi.getTimerCount()).toBe(1);
    document.documentElement.classList.remove("dark");
  });

  it.each([
    ["forward", DAY],
    ["backward", -DAY],
  ] as const)(
    "keeps recreated nodes on the monotonic entrance clock when wall time jumps %s",
    async (_direction, jump) => {
      const view = render(CareTimeline, { props });
      const retained = [
        ...view.container.querySelectorAll<SVGElement>(
          ".ground-reveal-clip, .event, .event-icon",
        ),
      ];
      const initialElapsed = retained.map((node) =>
        node.style.getPropertyValue("--entrance-elapsed"),
      );
      await advance(1000);
      vi.setSystemTime(NOW + 1000 + jump);
      await view.rerender({ ...props, events: [...events] });
      expect([
        ...view.container.querySelectorAll(
          ".ground-reveal-clip, .event, .event-icon",
        ),
      ]).toEqual(retained);
      expect(
        retained.map((node) =>
          node.style.getPropertyValue("--entrance-elapsed"),
        ),
      ).toEqual(initialElapsed);

      await view.rerender({
        ...props,
        events: events.map((record) => ({
          ...record,
          event_type: "pruned" as const,
        })),
      });
      const icons = [
        ...view.container.querySelectorAll<SVGElement>(".event-icon"),
      ];
      expect(icons).toHaveLength(events.length);
      for (const icon of icons) {
        expect(retained).not.toContain(icon);
        expect(icon.style.getPropertyValue("--entrance-elapsed")).toBe(
          "1000ms",
        );
      }
      for (const node of retained.filter(
        (node) => !node.classList.contains("event-icon"),
      )) {
        expect(node.isConnected).toBe(true);
        expect(node.style.getPropertyValue("--entrance-elapsed")).toBe("0ms");
      }

      await view.rerender({ ...props, events: [] });
      await view.rerender(props);
      const recreated = [
        ...view.container.querySelectorAll<SVGElement>(
          ".ground-reveal-clip, .event, .event-icon",
        ),
      ];
      expect(recreated).toHaveLength(1 + events.length * 2);
      for (const node of recreated) {
        expect(retained).not.toContain(node);
        expect(node.style.getPropertyValue("--entrance-elapsed")).toBe(
          "1000ms",
        );
      }
      expect(isRevealing(view.container)).toBe(true);
      expect(vi.getTimerCount()).toBe(2);
      await advance(1599);
      expect(isRevealing(view.container)).toBe(true);
      await advance(1);
      expect(isRevealing(view.container)).toBe(false);
      expect(vi.getTimerCount()).toBe(1);
    },
  );

  it("resets for another plant and for a remounted visit", async () => {
    const view = render(CareTimeline, { props });
    await advance(1000);
    const originalGround = view.container.querySelector(".ground-reveal-clip");
    await view.rerender({
      ...props,
      plantId: 2,
      events: [event(20, 2, { plant_id: 2 })],
    });
    expect(view.container.querySelector(".ground-reveal-clip")).not.toBe(
      originalGround,
    );
    const newNodes = [
      ...view.container.querySelectorAll<SVGElement>(
        ".ground-reveal-clip, .event, .event-icon",
      ),
    ];
    expect(newNodes).toHaveLength(3);
    for (const node of newNodes)
      expect(node.style.getPropertyValue("--entrance-elapsed")).toBe("0ms");
    expect(isRevealing(view.container)).toBe(true);
    expect(vi.getTimerCount()).toBe(2);
    await advance(2599);
    expect(isRevealing(view.container)).toBe(true);
    await advance(1);
    expect(isRevealing(view.container)).toBe(false);
    view.unmount();
    const nextVisit = render(CareTimeline, { props });
    expect(isRevealing(nextVisit.container)).toBe(true);
    expect(vi.getTimerCount()).toBe(2);
  });

  it("skips initial reduced motion and never starts later in the same visit", async () => {
    motionPreference(true);
    const view = render(CareTimeline, { props });
    expect(isRevealing(view.container)).toBe(false);
    expect(vi.getTimerCount()).toBe(1);
    motionPreference(false);
    await view.rerender({ ...props, events: [] });
    await view.rerender(props);
    expect(isRevealing(view.container)).toBe(false);
    expect(vi.getTimerCount()).toBe(1);
  });

  it("finishes immediately when reduced motion is enabled and removes its timer/listener on teardown", async () => {
    const view = render(CareTimeline, { props });
    const original = positions(view.container);
    await advance(800);
    motionPreference(true);
    await tick();
    expect(isRevealing(view.container)).toBe(false);
    expect(positions(view.container)).toEqual(original);
    expect(vi.getTimerCount()).toBe(1);
    motionPreference(false);
    await tick();
    expect(isRevealing(view.container)).toBe(false);
    view.unmount();
    expect(vi.getTimerCount()).toBe(0);
    expect(media.removeEventListener).toHaveBeenCalledWith(
      "change",
      expect.any(Function),
    );
    expect(mediaListeners.size).toBe(0);
  });
});

describe("CareTimeline confirmed watering feedback", () => {
  const feedback: WateringFeedback = { plantId: 1, eventId: 9, request: 1 };
  const watered = event(9, 0);

  function controlAnimations() {
    const animations: {
      cancel: ReturnType<typeof vi.fn>;
      onfinish: Animation["onfinish"];
    }[] = [];
    const animate = vi
      .spyOn(Element.prototype, "animate")
      .mockImplementation(() => {
        const animation = {
          cancel: vi.fn(),
          onfinish: null as Animation["onfinish"],
        };
        animations.push(animation);
        return animation as unknown as Animation;
      });
    return { animate, animations };
  }

  function expectFeedback(
    container: HTMLElement,
    controlled: ReturnType<typeof controlAnimations>,
    eventId = 9,
    callOffset = 0,
  ) {
    const marker = container.querySelector<SVGElement>(
      `[data-event-id="${eventId}"]`,
    )!;
    const icon = marker.querySelector<SVGElement>(".event-icon")!;
    const stem = marker.querySelector<SVGElement>(".stem")!;
    expect(stem.style.animation).toBe("none");
    expect(marker.dataset.waterFeedback).toBe("true");
    expect(marker.style.animation).toBe("none");
    expect(icon.style.animation).toBe("none");
    expect(icon.getAttribute("opacity")).toBe("0.46");
    expect(controlled.animate).toHaveBeenCalledTimes(callOffset + 3);
    expect(controlled.animate.mock.contexts.slice(callOffset)).toEqual([
      marker,
      stem,
      icon,
    ]);
    expect(controlled.animate.mock.calls.slice(callOffset)).toEqual([
      [
        [{ opacity: 0.4 }, { opacity: 1 }],
        { duration: 180, easing: "ease-out" },
      ],
      [
        [{ transform: "scaleY(0)" }, { transform: "scaleY(1)" }],
        { duration: 360, easing: "ease-out" },
      ],
      [
        [
          { opacity: 1, transform: "scale(0.55)" },
          { opacity: 1, transform: "scale(1)", offset: 0.45 },
          { opacity: 0.46, transform: "scale(1)" },
        ],
        { duration: 900, easing: "cubic-bezier(0.16, 1, 0.3, 1)" },
      ],
    ]);
    // Exact frames keep x positions fixed and bound stem/icon growth to their final size.
    expect(
      container.querySelectorAll('[data-water-feedback="true"]'),
    ).toHaveLength(1);
    for (const other of container.querySelectorAll<SVGElement>(
      `.event:not([data-event-id="${eventId}"])`,
    )) {
      expect(other.style.animation).toBe("");
      expect(
        other.querySelector<SVGElement>(".event-icon")!.style.animation,
      ).toBe("");
    }
    return marker;
  }

  it.each([600, 2600])(
    "immediately animates only the confirmed marker after %sms without restarting the ground",
    async (elapsed) => {
      const controlled = controlAnimations();
      const view = render(CareTimeline, { props });
      const ground = view.container.querySelector(".ground-reveal-clip");
      const path = groundPath(view.container);
      await advance(elapsed);
      const previous = positions(view.container);
      await view.rerender({
        ...props,
        events: [...events, watered],
        wateringFeedback: feedback,
      });
      expectFeedback(view.container, controlled);
      const retained = positions(view.container).filter(({ id }) => id !== 9);
      expect(retained.map(({ id }) => id)).toEqual(
        previous.map(({ id }) => id),
      );
      for (const [index, { xy }] of retained.entries()) {
        // The history refresh samples real time between minute ticks.
        expect(xy[0]).toBeCloseTo(
          previous[index].xy[0] - (elapsed / (42 * DAY)) * 256,
          10,
        );
        expect(xy[1]).toBe(previous[index].xy[1]);
      }
      expect(view.container.querySelector(".ground-reveal-clip")).toBe(ground);
      expect(groundPath(view.container)).toBe(path);
      expect(isRevealing(view.container)).toBe(elapsed < 2600);
      expect(
        controlled.animations.every((animation) => animation.onfinish !== null),
      ).toBe(true);
      if (elapsed < 2600) {
        await advance(2599 - elapsed);
        expect(isRevealing(view.container)).toBe(true);
        await advance(1);
      }
      expect(isRevealing(view.container)).toBe(false);
      expect(controlled.animate).toHaveBeenCalledTimes(3);
    },
  );

  it("does not replay a request for history, theme, locale, resize, clock, visibility or recreated DOM updates", async () => {
    const controlled = controlAnimations();
    const view = render(CareTimeline, { props });
    await advance(2600);
    const data = [...events, watered];
    await view.rerender({ ...props, events: data, wateringFeedback: feedback });
    const marker = expectFeedback(view.container, controlled);
    await view.rerender({
      ...props,
      events: [...data].reverse(),
      wateringFeedback: { ...feedback },
    });
    document.documentElement.classList.add("dark");
    try {
      await setLanguage("de");
      SizeObserver.instances.at(-1)!.resize(640, 140);
      await tick();
      await advance(60_000);
      await visibility(true);
      await advance(60_000);
      await visibility(false);
      await view.rerender({ ...props, events: [], wateringFeedback: feedback });
      expect(marker.isConnected).toBe(false);
      expect(
        controlled.animations.every(
          (animation) => animation.cancel.mock.calls.length === 1,
        ),
      ).toBe(true);
      await view.rerender({
        ...props,
        events: data,
        wateringFeedback: { ...feedback },
      });
      const recreated = view.container.querySelector<SVGElement>(
        '[data-event-id="9"]',
      )!;
      expect(recreated).not.toBe(marker);
      expect(recreated.dataset.waterFeedback).toBe("true");
      expect(recreated.style.animation).toBe("none");
      expect(
        recreated.querySelector<SVGElement>(".event-icon")!.style.animation,
      ).toBe("none");
      expect(controlled.animate).toHaveBeenCalledTimes(3);
    } finally {
      document.documentElement.classList.remove("dark");
    }
  });

  it("plays a new request for the same marker, canceling its previous native animations", async () => {
    const controlled = controlAnimations();
    const view = render(CareTimeline, {
      props: { ...props, events: [...events, watered] },
    });
    await view.rerender({
      ...props,
      events: [...events, watered],
      wateringFeedback: feedback,
    });
    expectFeedback(view.container, controlled);
    await view.rerender({
      ...props,
      events: [...events, watered],
      wateringFeedback: { ...feedback, request: 2 },
    });
    expectFeedback(view.container, controlled, 9, 3);
    expect(
      controlled.animations
        .slice(0, 3)
        .every((animation) => animation.cancel.mock.calls.length === 1),
    ).toBe(true);
    expect(
      controlled.animations
        .slice(3)
        .every((animation) => animation.cancel.mock.calls.length === 0),
    ).toBe(true);
  });

  it("animates first watering from empty history while the first ground entrance runs", async () => {
    const controlled = controlAnimations();
    const view = render(CareTimeline, { props: { ...props, events: [] } });
    expect(view.container.querySelector("svg")).toBeNull();
    await view.rerender({
      ...props,
      events: [watered],
      wateringFeedback: feedback,
    });
    expectFeedback(view.container, controlled);
    expect(isRevealing(view.container)).toBe(true);
    await advance(2600);
    expect(isRevealing(view.container)).toBe(false);
    expect(controlled.animate).toHaveBeenCalledTimes(3);
  });

  it("consumes initial mount feedback without playing, including after DOM recreation", async () => {
    const controlled = controlAnimations();
    const initial = {
      ...props,
      events: [...events, watered],
      wateringFeedback: feedback,
    };
    const view = render(CareTimeline, { props: initial });
    const marker = view.container.querySelector<SVGElement>(
      '[data-event-id="9"]',
    )!;
    expect(marker.dataset.waterFeedback).toBe("true");
    expect(marker.style.animation).toBe("none");
    expect(
      marker.querySelector<SVGElement>(".event-icon")!.style.animation,
    ).toBe("none");
    await advance(2600);
    await view.rerender({ ...initial, events: [] });
    await view.rerender({ ...initial, wateringFeedback: { ...feedback } });
    expect(controlled.animate).not.toHaveBeenCalled();
    await view.rerender({
      ...initial,
      wateringFeedback: { ...feedback, request: 2 },
    });
    expectFeedback(view.container, controlled);
  });

  it("shows final visible feedback without animation under initial reduced motion and consumes its request", async () => {
    motionPreference(true);
    const controlled = controlAnimations();
    const view = render(CareTimeline, { props });
    const next = {
      ...props,
      events: [...events, watered],
      wateringFeedback: feedback,
    };
    await view.rerender(next);
    const marker = view.container.querySelector<SVGElement>(
      '[data-event-id="9"]',
    )!;
    expect(marker.dataset.waterFeedback).toBe("true");
    expect(marker.style.animation).toBe("none");
    expect(
      marker.querySelector<SVGElement>(".event-icon")!.style.animation,
    ).toBe("none");
    expect(marker.querySelector(".event-icon")!.getAttribute("opacity")).toBe(
      "0.46",
    );
    expect(isRevealing(view.container)).toBe(false);
    motionPreference(false);
    await tick();
    await view.rerender({ ...next, wateringFeedback: { ...feedback } });
    expect(controlled.animate).not.toHaveBeenCalled();
  });

  it("cancels in-flight feedback on live reduced motion without replaying or hiding the marker", async () => {
    const controlled = controlAnimations();
    const view = render(CareTimeline, { props });
    await view.rerender({
      ...props,
      events: [...events, watered],
      wateringFeedback: feedback,
    });
    const marker = expectFeedback(view.container, controlled);
    motionPreference(true);
    await tick();
    expect(
      controlled.animations.every(
        (animation) => animation.cancel.mock.calls.length === 1,
      ),
    ).toBe(true);
    expect(marker.isConnected).toBe(true);
    expect(marker.style.animation).toBe("none");
    expect(
      marker.querySelector<SVGElement>(".event-icon")!.style.animation,
    ).toBe("none");
    expect(marker.querySelector(".event-icon")!.getAttribute("opacity")).toBe(
      "0.46",
    );
    expect(isRevealing(view.container)).toBe(false);
    motionPreference(false);
    await tick();
    await view.rerender({
      ...props,
      events: [...events, watered],
      wateringFeedback: { ...feedback },
    });
    expect(controlled.animate).toHaveBeenCalledTimes(3);
    view.unmount();
    expect(
      controlled.animations.every(
        (animation) => animation.cancel.mock.calls.length === 1,
      ),
    ).toBe(true);
  });

  it.each(["remove", "unmount"] as const)(
    "cancels all unfinished native animations on %s",
    async (action) => {
      const controlled = controlAnimations();
      const view = render(CareTimeline, { props });
      await view.rerender({
        ...props,
        events: [...events, watered],
        wateringFeedback: feedback,
      });
      expectFeedback(view.container, controlled);
      if (action === "remove") {
        await view.rerender({ ...props, wateringFeedback: feedback });
        expect(view.container.querySelector('[data-event-id="9"]')).toBeNull();
      }
      view.unmount();
      expect(
        controlled.animations.every(
          (animation) => animation.cancel.mock.calls.length === 1,
        ),
      ).toBe(true);
      expect(vi.getTimerCount()).toBe(0);
    },
  );

  it.each([null, { ...feedback, plantId: 2 }, { ...feedback, eventId: 999 }])(
    "does not animate unrelated history without a matching confirmed marker (%j)",
    async (wateringFeedback) => {
      const controlled = controlAnimations();
      const view = render(CareTimeline, { props });
      await advance(2600);
      await view.rerender({
        ...props,
        events: [...events, watered],
        wateringFeedback,
      });
      expect(controlled.animate).not.toHaveBeenCalled();
      expect(
        view.container.querySelector('[data-water-feedback="true"]'),
      ).toBeNull();
    },
  );
});

describe("CareTimeline history and clock lifecycle", () => {
  it.each([
    ["empty", [], false, false],
    ["loading", [], true, false],
    ["initial failure", [], false, true],
    ["malformed", [event(1, 0, { occurred_at: "bad" })], false, false],
    ["other plant", [event(1, 1, { plant_id: 2 })], false, false],
    ["old", [event(1, 43)], false, false],
    ["future", [event(1, -1)], false, false],
  ] as const)(
    "omits the SVG and all placeholder UI for %s history",
    (_scenario, data, loading, historyError) => {
      const { container } = render(CareTimeline, {
        props: { ...props, events: data, loading, historyError },
      });
      expect(container.querySelector("svg")).toBeNull();
      expect(container.textContent?.trim()).toBe("");
      expect(container.querySelector(".care-timeline")?.children).toHaveLength(
        0,
      );
    },
  );

  it("retains markers through loading/error props and adds status only in accessible metadata", async () => {
    const view = render(CareTimeline, { props });
    const original = positions(view.container);
    const description = view.container.querySelector("desc")?.textContent;
    await view.rerender({ ...props, loading: true });
    expect(positions(view.container)).toEqual(original);
    expect(view.container.querySelector("desc")?.textContent).toBe(
      `${description} · ${en.common.loading}`,
    );
    await view.rerender({ ...props, loading: true, historyError: true });
    expect(positions(view.container)).toEqual(original);
    expect(view.container.querySelector("desc")?.textContent).toBe(
      `${description} · ${en.error.loadCareEvents}`,
    );
    await view.rerender({ ...props, loading: false, historyError: false });
    expect(view.container.querySelector("desc")?.textContent).toBe(description);
    expect(normalizeCareTimelineEvents).toHaveBeenCalledTimes(1);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("never carries markers across plant identity changes and responds to event edits/deletes", async () => {
    const view = render(CareTimeline, { props });
    await view.rerender({ ...props, plantId: 2, loading: true });
    expect(view.container.querySelector("svg")).toBeNull();
    const next = [event(20, 3, { plant_id: 2 })];
    await view.rerender({ ...props, plantId: 2, events: next });
    expect(positions(view.container).map(({ id }) => id)).toEqual([20]);
    const path = groundPath(view.container);
    await view.rerender({
      ...props,
      plantId: 2,
      events: [
        {
          ...next[0],
          event_type: "custom",
          occurred_at: new Date(NOW - 21 * DAY).toISOString(),
        },
      ],
    });
    expect(positions(view.container)[0].xy[0]).toBe(160);
    expect(
      view.container.querySelector(".event")?.getAttribute("data-event-type"),
    ).toBe("custom");
    expect(groundPath(view.container)).toBe(path);
    await view.rerender({ ...props, plantId: 2, events: [] });
    expect(view.container.querySelector("svg")).toBeNull();
  });

  it("ticks cached/offline history each visible minute, pauses hidden, then immediately catches up", async () => {
    vi.spyOn(navigator, "onLine", "get").mockReturnValue(false);
    const view = render(CareTimeline, { props });
    const original = positions(view.container);
    await advance(59_999);
    expect(positions(view.container)).toEqual(original);
    await advance(1);
    const minute = positions(view.container);
    expect(minute[0].xy[0]).toBeCloseTo(
      32 + ((21 * DAY - 60_000) / (42 * DAY)) * 256,
    );
    expect(minute).not.toEqual(original);
    await visibility(true);
    expect(vi.getTimerCount()).toBe(0);
    await advance(6 * 60_000);
    expect(positions(view.container)).toEqual(minute);
    await visibility(false);
    expect(positions(view.container)[0].xy[0]).toBeCloseTo(
      32 + ((21 * DAY - 7 * 60_000) / (42 * DAY)) * 256,
    );
    expect(vi.getTimerCount()).toBe(1);
    expect(normalizeCareTimelineEvents).toHaveBeenCalledTimes(1);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("starts no timer while hidden and makes future history eligible only when real time reaches it", async () => {
    hidden.mockReturnValue(true);
    const view = render(CareTimeline, {
      props: { ...props, events: [event(1, -60_000 / DAY)] },
    });
    expect(vi.getTimerCount()).toBe(0);
    expect(view.container.querySelector("svg")).toBeNull();
    await advance(60_000);
    expect(view.container.querySelector("svg")).toBeNull();
    await visibility(false);
    expect(positions(view.container)[0].xy[0]).toBe(288);
    expect(normalizeCareTimelineEvents).toHaveBeenCalledTimes(1);
  });

  it("updates future and aged-out eligibility on minute ticks without reparsing on size/locale/status changes", async () => {
    const future = [event(1, 42), event(2, -60_000 / DAY)];
    const view = render(CareTimeline, { props: { ...props, events: future } });
    expect(positions(view.container).map(({ id }) => id)).toEqual([1]);
    await advance(60_000);
    expect(positions(view.container)).toHaveLength(1);
    expect(positions(view.container)[0]).toMatchObject({
      id: 2,
      xy: [288, expect.any(Number)],
    });
    SizeObserver.instances[0].resize(500, 200);
    await setLanguage("de");
    await view.rerender({ ...props, events: future, historyError: true });
    expect(normalizeCareTimelineEvents).toHaveBeenCalledTimes(1);
    await view.rerender({ ...props, events: [...future] });
    expect(normalizeCareTimelineEvents).toHaveBeenCalledTimes(2);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("resamples actual Date.now on source changes between ticks, never advancing to a future import", async () => {
    const view = render(CareTimeline, { props });
    vi.setSystemTime(NOW + 30_000);
    const fresh = [...events, event(4, -30_000 / DAY)];
    await view.rerender({ ...props, events: fresh });
    expect(positions(view.container).at(-1)?.xy[0]).toBe(288);
    const current = positions(view.container);
    await view.rerender({ ...props, events: [...fresh, event(5, -1)] });
    expect(positions(view.container)).toEqual(current);
    expect(normalizeCareTimelineEvents).toHaveBeenCalledTimes(3);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("clears and regrows the reported minimum as dense clusters age out/arrive on the real source/visibility clock", async () => {
    const report = vi.fn();
    const old = Array.from({ length: 30 }, (_, index) => event(index + 1, 42));
    const future = Array.from({ length: 30 }, (_, index) =>
      event(index + 101, -120_000 / DAY),
    );
    const data = [...old, ...future];
    const view = render(CareTimeline, {
      props: { ...props, events: data, onMinimumHeight: report },
    });
    expect(report).toHaveBeenLastCalledWith(610);
    expect(positions(view.container).map(({ id }) => id)).toEqual(
      old.map(({ id }) => id),
    );
    // A source refresh between ticks samples real time, never the future cluster.
    vi.setSystemTime(NOW + 30_000);
    await view.rerender({
      ...props,
      events: [...data],
      onMinimumHeight: report,
    });
    expect(report).toHaveBeenLastCalledWith(0);
    expect(view.container.querySelector("svg")).toBeNull();
    await visibility(true);
    await advance(90_000);
    expect(report).toHaveBeenLastCalledWith(0);
    await visibility(false);
    expect(report).toHaveBeenLastCalledWith(610);
    expect(positions(view.container).map(({ id }) => id)).toEqual(
      future.map(({ id }) => id),
    );
    expectRenderedClearance(view.container);
    await advance(60_000);
    expect(report).toHaveBeenCalledTimes(3);
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("cleans timer, observer and resize/visibility listeners on teardown", async () => {
    const removeVisibility = vi.spyOn(document, "removeEventListener");
    const removeResize = vi.spyOn(window, "removeEventListener");
    const view = render(CareTimeline, { props });
    expect(vi.getTimerCount()).toBe(2);
    view.unmount();
    await tick();
    expect(vi.getTimerCount()).toBe(0);
    expect(SizeObserver.instances[0].disconnect).toHaveBeenCalledOnce();
    expect(removeVisibility).toHaveBeenCalledWith(
      "visibilitychange",
      expect.any(Function),
    );
    expect(removeResize).toHaveBeenCalledWith("resize", expect.any(Function));
    expect(media.removeEventListener).toHaveBeenCalledWith(
      "change",
      expect.any(Function),
    );
    expect(mediaListeners.size).toBe(0);
    await visibility(false);
    await advance(120_000);
    expect(vi.getTimerCount()).toBe(0);
    expect(fetchMock).not.toHaveBeenCalled();
  });
});

describe("CareTimeline foreground protection", () => {
  it("masks tight actual foreground bounds with 12px padding/blur and observes parents only for layout", () => {
    hostBounds(100, 200, 640, 300);
    const occupied = [
      foreground(116, 206, 198, 40),
      foreground(322, 226, 104, 20),
      foreground(116, 252, 128, 20),
      foreground(116, 282, 92, 18),
      foreground(116, 318, 144, 42),
    ];
    const invisible = foreground(0, 0, 0, 0);
    const detached = foreground(100, 200, 640, 300);
    detached.element.remove();
    const view = render(CareTimeline, {
      props: {
        ...props,
        foregroundElements: [...occupied, invisible, detached].map(
          ({ element }) => element,
        ),
      },
    });
    expect(shields(view.container)).toEqual([
      { x: 4, y: -6, width: 222, height: 64 },
      { x: 210, y: 14, width: 128, height: 44 },
      { x: 4, y: 40, width: 152, height: 44 },
      { x: 4, y: 70, width: 116, height: 42 },
      { x: 4, y: 106, width: 168, height: 66 },
    ]);
    const observer = SizeObserver.instances[0];
    expect(observer.observe).toHaveBeenCalledWith(
      view.container.querySelector(".care-timeline"),
    );
    for (const { element } of occupied)
      expect(observer.observe).toHaveBeenCalledWith(element);
    expect(observer.observe).toHaveBeenCalledWith(document.body);
    expect(
      observer.observe.mock.calls.some(
        ([target]) => target === detached.element,
      ),
    ).toBe(false);
    expect(detached.measure).not.toHaveBeenCalled();
    expect(view.container.querySelector("filter")?.getAttribute("width")).toBe(
      "712",
    );
    expect(view.container.querySelector("filter")?.getAttribute("x")).toBe(
      "-36",
    );
    expect(fetchMock).not.toHaveBeenCalled();
  });

  it("remeasures wrapping, parent-driven movement and locale changes, omitting disconnected bindings", async () => {
    hostBounds(100, 200, 640, 300);
    const name = foreground(116, 206, 198, 40);
    const status = foreground(116, 282, 92, 18);
    const view = render(CareTimeline, {
      props: { ...props, foregroundElements: [name.element, status.element] },
    });
    name.measure.mockReturnValue(new DOMRect(116, 206, 298, 80));
    status.measure.mockReturnValue(new DOMRect(116, 322, 92, 18));
    SizeObserver.instances[0].resize(298, 80, name.element);
    await tick();
    expect(shields(view.container)).toEqual([
      { x: 4, y: -6, width: 322, height: 104 },
      { x: 4, y: 110, width: 116, height: 42 },
    ]);
    status.measure.mockReturnValue(new DOMRect(120, 342, 92, 18));
    SizeObserver.instances[0].resize(640, 320, document.body);
    await tick();
    expect(shields(view.container)[1]).toMatchObject({ x: 8, y: 130 });
    status.measure.mockReturnValue(new DOMRect(124, 352, 92, 18));
    await setLanguage("de");
    expect(shields(view.container)[1]).toMatchObject({ x: 12, y: 140 });
    status.element.remove();
    const calls = status.measure.mock.calls.length;
    SizeObserver.instances.at(-1)!.resize(640, 300);
    await tick();
    expect(shields(view.container)).toHaveLength(1);
    expect(status.measure).toHaveBeenCalledTimes(calls);
  });

  it("replaces optional foreground bindings and cleans obsolete observers/cutouts", async () => {
    const name = foreground(0, 0, 100, 40);
    const ai = foreground(0, 120, 100, 40);
    const view = render(CareTimeline, {
      props: { ...props, foregroundElements: [name.element, ai.element] },
    });
    expect(shields(view.container)).toHaveLength(2);
    const original = SizeObserver.instances[0];
    await view.rerender({ ...props, foregroundElements: [name.element] });
    expect(original.disconnect).toHaveBeenCalledOnce();
    expect(shields(view.container)).toHaveLength(1);
    const replacement = SizeObserver.instances[1];
    expect(
      replacement.observe.mock.calls.some(([target]) => target === ai.element),
    ).toBe(false);
    await view.rerender({ ...props, foregroundElements: [] });
    expect(replacement.disconnect).toHaveBeenCalledOnce();
    expect(shields(view.container)).toHaveLength(0);
    const final = SizeObserver.instances[2];
    view.unmount();
    expect(final.disconnect).toHaveBeenCalledOnce();
    expect(vi.getTimerCount()).toBe(0);
  });

  it("remeasures responsive host/content bounds on window resize without ResizeObserver", async () => {
    vi.stubGlobal("ResizeObserver", undefined);
    const host = hostBounds(0, 0, 320, 140);
    const name = foreground(10, 10, 100, 40);
    const view = render(CareTimeline, {
      props: { ...props, foregroundElements: [name.element] },
    });
    expect(shields(view.container)).toEqual([
      { x: -2, y: -2, width: 124, height: 64 },
    ]);
    host.current = new DOMRect(16, 420, 288, 180);
    name.measure.mockReturnValue(new DOMRect(16, 420, 250, 80));
    window.dispatchEvent(new Event("resize"));
    await tick();
    expect(shields(view.container)).toEqual([
      { x: -12, y: -12, width: 274, height: 104 },
    ]);
    expect(view.container.querySelector("svg")?.getAttribute("viewBox")).toBe(
      "0 0 288 180",
    );
  });
});
