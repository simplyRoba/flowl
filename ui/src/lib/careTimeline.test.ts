import { describe, expect, it } from "vitest";
import type { CareEvent, EventType } from "#lib/api.js";
import {
  createCareTimelineGround,
  layoutCareTimelineEvents,
  layoutCareTimelinePlan,
  normalizeCareTimelineEvents,
  planCareTimelineEvents,
  type CareTimelinePoint,
  sampleCareTimelineGround,
  selectCareTimelineEvents,
} from "./careTimeline";

const DAY = 86_400_000;
const NOW = Date.UTC(2026, 3, 1, 12);
const ORIGIN = Date.UTC(2026, 0, 1);
function event(
  id: number,
  occurred_at: string,
  overrides: Partial<CareEvent> = {},
): CareEvent {
  return {
    id,
    occurred_at,
    plant_id: 1,
    plant_name: "Fern",
    event_type: "watered",
    notes: null,
    photo_url: null,
    created_at: "2026-01-01T00:00:00Z",
    ...overrides,
  };
}
function history(...days: number[]) {
  return normalizeCareTimelineEvents(
    days.map((day, index) =>
      event(index + 1, new Date(NOW - day * DAY).toISOString()),
    ),
    1,
  );
}

function expectClearance(points: CareTimelinePoint[], height: number) {
  for (const [index, point] of points.entries()) {
    expect(point.y - 7).toBeGreaterThanOrEqual(0);
    expect(point.y + 7).toBeLessThan(height);
    expect(point.groundY - point.y).toBeGreaterThanOrEqual(18 - 1e-9);
    expect(point.groundY + 4).toBeLessThan(height);
    for (const other of points.slice(index + 1)) {
      if (Math.abs(point.x - other.x) < 18)
        expect(Math.abs(point.y - other.y)).toBeGreaterThanOrEqual(18 - 1e-9);
    }
  }
}

describe("normalizeCareTimelineEvents", () => {
  it("includes all six types, notes and photos but excludes foreign and unsupported records", () => {
    const types: EventType[] = [
      "watered",
      "fertilized",
      "repotted",
      "pruned",
      "custom",
      "ai-consultation",
    ];
    const records = types.map((event_type, index) =>
      event(index + 1, "2026-01-01T00:00:00Z", {
        event_type,
        notes: "Keep",
        photo_url: "/keep.jpg",
      }),
    );
    const normalized = normalizeCareTimelineEvents(
      [
        ...records,
        event(7, "2026-01-01T00:00:00Z", { plant_id: 2 }),
        event(8, "2026-01-01T00:00:00Z", {
          event_type: "unknown" as EventType,
        }),
      ],
      1,
    );
    expect(normalized.map((record) => record.eventType)).toEqual(types);
    expect(normalized.map((record) => record.id)).toEqual([1, 2, 3, 4, 5, 6]);
  });

  it("sorts actual instants, retaining equal instants with distinct IDs", () => {
    const records = normalizeCareTimelineEvents(
      [
        event(10, "2026-01-01T00:30:00-02:00"),
        event(90, "2026-01-01T01:00:00+02:00"),
        event(9, "2026-01-01T03:30:00+03:30"),
        event(1, "2025-12-31T19:00:00-05:00"),
        event(5, "2026-01-01T00:00:00Z"),
      ],
      1,
    );
    expect(records.map(({ id }) => id)).toEqual([90, 1, 5, 9, 10]);
    expect(records.slice(1, 4).map(({ occurredAt }) => occurredAt)).toEqual([
      ORIGIN,
      ORIGIN,
      ORIGIN,
    ]);
  });

  it.each([
    ["2026-01-01T00:00:00", 0],
    ["2026-01-01 00:00:00", 0],
    ["2026-01-01t00:00:00z", 0],
    ["2026-01-01T00:00:00.1", 100],
    ["2026-01-01 00:00:00.123456", 123],
    ["2026-01-01T00:00:00.123456789Z", 123],
    ["2026-01-01T01:00:00.999+01:00", 999],
  ])("interprets %s as UTC milliseconds", (timestamp, milliseconds) => {
    expect(normalizeCareTimelineEvents([event(1, timestamp)], 1)).toEqual([
      { id: 1, eventType: "watered", occurredAt: ORIGIN + milliseconds },
    ]);
  });

  it.each([
    "",
    "not a date",
    "2026-01-01",
    "01/01/2026 00:00:00",
    "2026-02-29T00:00:00Z",
    "2026-04-31T00:00:00Z",
    "2026-00-01T00:00:00Z",
    "2026-13-01T00:00:00Z",
    "2026-01-00T00:00:00Z",
    "2026-01-32T00:00:00Z",
    "2026-01-01T24:00:00Z",
    "2026-01-01T00:60:00Z",
    "2026-01-01T00:00:61Z",
    "2026-01-01T00:00:00+24:00",
    "2026-01-01T00:00:00+00:60",
    "2026-01-01T00:00:00+0100",
    "2026-01-01T00:00:00.Z",
    " 2026-01-01T00:00:00Z",
    "2026-01-01T00:00:00Z trailing",
    "2026-01-01T00:00:00Z\n",
  ])("rejects malformed or impossible timestamp %s", (timestamp) => {
    expect(
      normalizeCareTimelineEvents(
        [event(1, timestamp), event(2, "2026-01-01T00:00:00Z")],
        1,
      ).map(({ id }) => id),
    ).toEqual([2]);
  });

  it.each([
    ["2016-12-31T23:59:60Z", 0],
    ["2016-12-31T23:59:60", 0],
    ["2016-12-31 23:59:60.123456", 123],
    ["2016-12-31T23:59:60.999Z", 999],
    ["2017-01-01T00:59:60+01:00", 0],
    ["2016-12-31T18:59:60-05:00", 0],
  ])(
    "normalizes backend-supported leap second %s",
    (timestamp, milliseconds) => {
      expect(
        normalizeCareTimelineEvents([event(1, timestamp)], 1)[0].occurredAt,
      ).toBe(Date.UTC(2017, 0, 1) + milliseconds);
    },
  );

  it("accepts leap dates and does not reinterpret years below 100", () => {
    const records = normalizeCareTimelineEvents(
      [event(1, "2024-02-29T00:00:00Z"), event(2, "0099-01-01T00:00:00Z")],
      1,
    );
    expect(new Date(records[0].occurredAt).getUTCFullYear()).toBe(99);
    expect(records[1].occurredAt).toBe(Date.UTC(2024, 1, 29));
  });

  it("does not mutate journal data and retains future records independently of clock/size", () => {
    const records = Object.freeze([
      Object.freeze(
        event(2, "9999-01-01T00:00:00Z", {
          notes: "Keep",
          photo_url: "/keep.jpg",
        }),
      ),
      Object.freeze(event(1, "2026-01-01T00:00:00Z")),
    ]);
    const snapshot = JSON.stringify(records);
    const normalized = normalizeCareTimelineEvents(records, 1);
    expect(normalized.map(({ id }) => id)).toEqual([1, 2]);
    expect(normalized[0]).not.toBe(records[1]);
    expect(JSON.stringify(records)).toBe(snapshot);
  });
});

describe("selectCareTimelineEvents", () => {
  it("uses exactly the trailing 28 days, including boundaries but no old seed or future record", () => {
    const normalized = history(29, 28 + 1 / DAY, 28, 14, 0, -1 / DAY, -1);
    expect(
      selectCareTimelineEvents(normalized, NOW).map(({ id }) => id),
    ).toEqual([3, 4, 5]);
    expect(
      selectCareTimelineEvents(normalized, NOW, 7).map(({ id }) => id),
    ).toEqual([5]);
  });

  it("rechecks future and aging eligibility without changing or reparsing the normalized history", () => {
    const normalized = Object.freeze(
      history(28, 1, -1).map((record) => Object.freeze(record)),
    );
    const snapshot = JSON.stringify(normalized);
    expect(selectCareTimelineEvents(normalized, NOW)).toEqual(
      normalized.slice(0, 2),
    );
    expect(selectCareTimelineEvents(normalized, NOW + DAY)).toEqual(
      normalized.slice(1),
    );
    expect(selectCareTimelineEvents(normalized, NOW + 30 * DAY)).toEqual([]);
    expect(JSON.stringify(normalized)).toBe(snapshot);
  });

  it.each([0, -1, NaN, Infinity, Number.MAX_VALUE])(
    "rejects invalid window %s",
    (days) => {
      expect(() => selectCareTimelineEvents([], NOW, days)).toThrow(RangeError);
    },
  );
  it.each([NaN, Infinity, -Infinity])("rejects nonfinite clock %s", (now) => {
    expect(() => selectCareTimelineEvents([], now)).toThrow(RangeError);
  });
});

describe("care timeline layout", () => {
  it("projects precise true-time spacing, preserving equal instants and identities", () => {
    const ground = createCareTimelineGround(1, 320, 140);
    const data = history(28, 21, 14, 14, 7, 0);
    const points = layoutCareTimelineEvents(data, NOW, ground);
    expect(points.map(({ x }) => x)).toEqual([0, 80, 160, 160, 240, 320]);
    expect(points.map(({ event }) => event.id)).toEqual([1, 2, 3, 4, 5, 6]);
    const precise = layoutCareTimelineEvents(
      history(13.123456),
      NOW,
      ground,
    )[0];
    expect(precise.x).toBeCloseTo((1 - 13.123456 / 28) * 320, 6);
  });

  it("keeps four fixed cubic segments with five plant/size-only anchors and no legend allowance", () => {
    const ground = createCareTimelineGround(1, 320, 140);
    expect(ground.anchors.map(({ x }) => x)).toEqual([0, 80, 160, 240, 320]);
    expect(ground.path.match(/C/g)).toHaveLength(4);
    for (const { y } of ground.anchors) {
      expect(y).toBeGreaterThanOrEqual(140 - 8 - 6 - 140 * 0.18);
      expect(y).toBeLessThanOrEqual(140 - 8 - 6);
    }
    expect(createCareTimelineGround(1, 320, 140)).toEqual(ground);
    expect(createCareTimelineGround(2, 320, 140).path).not.toBe(ground.path);
  });

  it("scales seeded ground depth at 18% of height up to the 36px cap", () => {
    const capped = createCareTimelineGround(1, 320, 200);
    // Lock the seeded amplitude as well as its bounds and proportional scaling.
    expect(capped.anchors[0].y).toBeCloseTo(175.85519709760678, 10);
    for (const height of [140, 180, 200, 300, 610]) {
      const ground = createCareTimelineGround(1, 320, height);
      const depth = Math.min(36, height * 0.18);
      for (const [index, { y }] of ground.anchors.entries()) {
        expect(y).toBeGreaterThanOrEqual(height - 8 - 6 - depth);
        expect(y).toBeLessThanOrEqual(height - 8 - 6);
        expect(height - 14 - y).toBeCloseTo(
          ((200 - 14 - capped.anchors[index].y) / 36) * depth,
          10,
        );
      }
    }
  });

  it("adds stems, never bends, and deterministically reflows only conflicting neighborhoods", () => {
    const ground = createCareTimelineGround(1, 640, 240);
    const sparse = history(21, 14, 7);
    const original = layoutCareTimelineEvents(sparse, NOW, ground);
    const dense = [
      ...sparse,
      ...history(...Array.from({ length: 200 }, (_, i) => i * 0.13)).map(
        (record) => ({ ...record, id: record.id + 100 }),
      ),
    ];
    const more = layoutCareTimelineEvents(dense, NOW, ground);
    expect(layoutCareTimelineEvents(dense.reverse(), NOW, ground)).toEqual(
      more,
    );
    expect(more).toHaveLength(dense.length);
    expectClearance(more, 240);
    // An isolated addition does not displace existing nonconflicting events.
    const isolated = [
      ...sparse,
      { id: 99, eventType: "custom" as const, occurredAt: NOW },
    ];
    expect(layoutCareTimelineEvents(isolated, NOW, ground).slice(0, 3)).toEqual(
      original,
    );
    expect(layoutCareTimelineEvents(sparse, NOW, ground)).toEqual(original);
    expect(new Set(original.map(({ y }) => y)).size).toBeGreaterThan(1);
    expect(ground).toEqual(createCareTimelineGround(1, 640, 240));
  });

  it("lands every stem on the actual rendered cubic between anchors, with at least 18px clearance", () => {
    const ground = createCareTimelineGround(3, 517, 213);
    const commands = ground.path.match(/[MC][^MC]*/g)!;
    const data = history(
      ...Array.from({ length: 101 }, (_, index) => index * 0.28),
    );
    for (const point of layoutCareTimelineEvents(data, NOW, ground)) {
      const index = Math.min(3, Math.floor((point.x / ground.width) * 4));
      const start = ground.anchors[index];
      const [cx1, cy1, cx2, cy2, endX, endY] = commands[index + 1]
        .slice(1)
        .trim()
        .split(/\s+/)
        .map(Number);
      const t = (point.x - start.x) / (endX - start.x);
      const cubic = (a: number, b: number, c: number, d: number) =>
        (1 - t) ** 3 * a +
        3 * (1 - t) ** 2 * t * b +
        3 * (1 - t) * t * t * c +
        t ** 3 * d;
      expect(cubic(start.x, cx1, cx2, endX)).toBeCloseTo(point.x, 10);
      expect(cubic(start.y, cy1, cy2, endY)).toBeCloseTo(point.groundY, 10);
      expect(sampleCareTimelineGround(ground, point.x)).toBe(point.groundY);
      expect(point.groundY - point.y).toBeGreaterThanOrEqual(18);
      expect(point.y - 7).toBeGreaterThanOrEqual(0);
      expect(point.groundY + 4).toBeLessThan(ground.height);
    }
  });

  it("preserves normalized ground shape and time fractions on width resize, adapting height safely", () => {
    const small = createCareTimelineGround(1, 320, 140);
    const wide = createCareTimelineGround(1, 640, 140);
    expect(wide.anchors).toEqual(
      small.anchors.map(({ x, y }) => ({ x: x * 2, y })),
    );
    const data = history(21, 14, 7);
    expect(layoutCareTimelineEvents(data, NOW, wide)).toEqual(
      layoutCareTimelineEvents(data, NOW, small).map((point) => ({
        ...point,
        x: point.x * 2,
      })),
    );
    const plan = planCareTimelineEvents(data, NOW, 320);
    for (const hostHeight of [20, 60, 100, 240, 400]) {
      const height = Math.max(hostHeight, plan.minimumHeight);
      const ground = createCareTimelineGround(1, 320, height);
      for (const point of layoutCareTimelineEvents(data, NOW, ground)) {
        expect(point.groundY - point.y).toBeGreaterThanOrEqual(18);
        expect(point.y).toBeGreaterThanOrEqual(7);
        expect(point.groundY + 4).toBeLessThan(height);
      }
    }
    // Pure projection refuses undersized geometry; production grows the host
    // before projection instead of dropping eligible records.
    expect(() =>
      layoutCareTimelineEvents(data, NOW, createCareTimelineGround(1, 320, 20)),
    ).toThrow(RangeError);
  });

  it.each([288, 320, 640, 1024])(
    "avoids all-type equal-instant, same/neighboring-day conflicts at width %s",
    (width) => {
      const types: EventType[] = [
        "watered",
        "fertilized",
        "repotted",
        "pruned",
        "custom",
        "ai-consultation",
      ];
      for (const count of [2, 4, 12, 30]) {
        for (const spacing of [0, 1 / 48, 0.5]) {
          const data = normalizeCareTimelineEvents(
            Array.from({ length: count }, (_, index) =>
              event(
                index + 1,
                new Date(
                  NOW - (14 + (index - count / 2) * spacing) * DAY,
                ).toISOString(),
                {
                  event_type: types[index % types.length],
                },
              ),
            ),
            1,
          );
          const plan = planCareTimelineEvents(data, NOW, width);
          expect(
            planCareTimelineEvents([...data].reverse(), NOW, width),
          ).toEqual(plan);
          if (!spacing) {
            expect(plan.laneCount).toBe(Math.max(4, count));
            const fixedHeight = 20 + 18 * (Math.max(4, count) - 1) + 32;
            const proportionalHeight = fixedHeight / (1 - 0.18);
            expect(plan.minimumHeight).toBe(
              Math.max(
                140,
                Math.ceil(
                  proportionalHeight <= 36 / 0.18
                    ? proportionalHeight
                    : fixedHeight + 36,
                ),
              ),
            );
          }
          for (const baseline of [140, 260, 300]) {
            const height = Math.max(baseline, plan.minimumHeight);
            const ground = createCareTimelineGround(1, width, height);
            const points = layoutCareTimelinePlan(plan, ground);
            expect(points).toHaveLength(count);
            expectClearance(points, height);
            for (const point of points) {
              expect(point.x).toBe(
                ((point.event.occurredAt - (NOW - 28 * DAY)) / (28 * DAY)) *
                  width,
              );
              expect(point.groundY).toBe(
                sampleCareTimelineGround(ground, point.x),
              );
            }
            expect(
              layoutCareTimelineEvents([...data].reverse(), NOW, ground),
            ).toEqual(points);
            expect(ground).toEqual(createCareTimelineGround(1, width, height));
          }
        }
      }
    },
  );

  it.each([
    [4, 140],
    [7, 196],
    [8, 214],
    [12, 286],
    [30, 610],
  ])(
    "plans the minimum safe height for %s simultaneous events as %spx",
    (count, minimumHeight) => {
      const data = history(...Array<number>(count).fill(14));
      const plan = planCareTimelineEvents(data, NOW, 320);
      expect(plan.minimumHeight).toBe(minimumHeight);
      const fixedHeight = 20 + 18 * (count - 1) + 32;
      expect(minimumHeight).toBeGreaterThanOrEqual(
        fixedHeight + Math.min(36, minimumHeight * 0.18),
      );
      if (minimumHeight > 140) {
        expect(minimumHeight - 1).toBeLessThan(
          fixedHeight + Math.min(36, (minimumHeight - 1) * 0.18),
        );
      }
      for (const plantId of [1, 2, 3, 42, 1000]) {
        for (const height of [minimumHeight, minimumHeight + 100]) {
          const ground = createCareTimelineGround(plantId, 320, height);
          expectClearance(layoutCareTimelinePlan(plan, ground), height);
        }
      }
      expect(planCareTimelineEvents(data, NOW, 320)).toEqual(plan);
    },
  );

  it("reuses lanes at the 18px boundary, growing only for actual conflicts and shrinking on removal/width changes", () => {
    const data = Array.from({ length: 30 }, (_, index) => ({
      id: index + 1,
      eventType: "watered" as const,
      occurredAt: NOW - 14 * DAY + (index * DAY) / 4,
    }));
    const narrow = planCareTimelineEvents(data, NOW, 100);
    expect(narrow.minimumHeight).toBeGreaterThan(300);
    const wide = planCareTimelineEvents(data, NOW, 2048);
    expect(wide.minimumHeight).toBe(140);
    expect(planCareTimelineEvents(data, NOW, 100)).toEqual(narrow);
    expect(
      planCareTimelineEvents(data.slice(0, 2), NOW, 100).minimumHeight,
    ).toBe(140);
    expect(planCareTimelineEvents([], NOW, 100).minimumHeight).toBe(0);
    const boundary = [0, 18, 36, 54].map((x, index) => ({
      id: index + 1,
      eventType: "custom" as const,
      occurredAt: NOW - 28 * DAY + x * DAY,
    }));
    // Use a 56-day window: all four events are eligible and exactly 18px apart.
    const separated = planCareTimelineEvents(boundary, NOW + 28 * DAY, 56, 56);
    const preferred = boundary.map(
      (record) =>
        planCareTimelineEvents([record], NOW + 28 * DAY, 56, 56).placements[0]
          .lane,
    );
    expect(separated.placements.map(({ lane }) => lane)).toEqual(preferred);
    expect(separated.minimumHeight).toBe(140);
  });

  it.each([0, -1, NaN, Infinity])(
    "rejects invalid layout dimension %s",
    (dimension) => {
      expect(() => createCareTimelineGround(1, dimension, 140)).toThrow(
        RangeError,
      );
      expect(() => createCareTimelineGround(1, 320, dimension)).toThrow(
        RangeError,
      );
    },
  );
});
