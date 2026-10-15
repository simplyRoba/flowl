import type { CareEvent, EventType } from "#lib/api.js";

const DAY_MS = 86_400_000;
const GROUND_DEPTH_CAP = 36;
const GROUND_DEPTH_RATIO = 0.18;
const GROUND_FLOOR = 8;
const GROUND_OFFSET = 6;
const STEM_CLEARANCE = 18;
const EVENT_TYPES: ReadonlySet<string> = new Set<EventType>([
  "watered",
  "fertilized",
  "repotted",
  "pruned",
  "custom",
  "ai-consultation",
]);

export interface NormalizedCareEvent {
  id: number;
  eventType: EventType;
  occurredAt: number;
}

/**
 * Backend naive T/SQLite timestamps are UTC. Validate calendar fields before
 * applying offsets; truncate fractions to milliseconds. Chrono leap seconds
 * map into the next second, including their fractional milliseconds.
 */
function parseOccurredAt(timestamp: string): number | null {
  const match =
    /^(\d{4})-(\d{2})-(\d{2})[Tt ](\d{2}):(\d{2}):(\d{2})(?:\.(\d+))?([Zz]|[+-]\d{2}:\d{2})?$/.exec(
      timestamp,
    );
  if (!match || match[0] !== timestamp) return null;
  const [
    ,
    yearText,
    monthText,
    dayText,
    hourText,
    minuteText,
    secondText,
    fraction,
    zone,
  ] = match;
  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const hour = Number(hourText);
  const minute = Number(minuteText);
  const second = Number(secondText);
  if (
    month < 1 ||
    month > 12 ||
    day < 1 ||
    day > 31 ||
    hour > 23 ||
    minute > 59 ||
    second > 60
  )
    return null;

  const date = new Date(0);
  date.setUTCFullYear(year, month - 1, day);
  date.setUTCHours(
    hour,
    minute,
    Math.min(second, 59),
    Number((fraction ?? "").slice(0, 3).padEnd(3, "0")),
  );
  if (
    date.getUTCFullYear() !== year ||
    date.getUTCMonth() !== month - 1 ||
    date.getUTCDate() !== day
  )
    return null;

  let offsetMinutes = 0;
  if (zone && zone.toUpperCase() !== "Z") {
    const hours = Number(zone.slice(1, 3));
    const minutes = Number(zone.slice(4, 6));
    if (hours > 23 || minutes > 59) return null;
    offsetMinutes = (hours * 60 + minutes) * (zone[0] === "+" ? 1 : -1);
  }
  return date.getTime() + (second === 60 ? 1000 : 0) - offsetMinutes * 60_000;
}

/** Normalize only on source changes, independently of the clock or layout. */
export function normalizeCareTimelineEvents(
  events: readonly CareEvent[],
  plantId: number,
): NormalizedCareEvent[] {
  const history: NormalizedCareEvent[] = [];
  for (const event of events) {
    if (event.plant_id !== plantId || !EVENT_TYPES.has(event.event_type))
      continue;
    const occurredAt = parseOccurredAt(event.occurred_at);
    if (occurredAt !== null)
      history.push({ id: event.id, eventType: event.event_type, occurredAt });
  }
  return history.sort((a, b) => a.occurredAt - b.occurredAt || a.id - b.id);
}

function windowStart(now: number, windowDays: number): number {
  const duration = windowDays * DAY_MS;
  const start = now - duration;
  if (
    !Number.isFinite(now) ||
    !Number.isFinite(start) ||
    !Number.isFinite(duration) ||
    duration <= 0
  ) {
    throw new RangeError(
      "Timeline requires a finite clock and positive finite window",
    );
  }
  return start;
}

/** Include both boundaries, never an old seed or a future forecast. */
export function selectCareTimelineEvents(
  history: readonly NormalizedCareEvent[],
  now: number,
  windowDays = 28,
): NormalizedCareEvent[] {
  const start = windowStart(now, windowDays);
  return history.filter(
    (event) => event.occurredAt >= start && event.occurredAt <= now,
  );
}

export interface TimelinePosition {
  x: number;
  y: number;
}

export interface CareTimelineGround {
  width: number;
  height: number;
  anchors: readonly TimelinePosition[];
  path: string;
}

export interface CareTimelinePoint extends TimelinePosition {
  event: NormalizedCareEvent;
  groundY: number;
}

function identitySeed(id: number): number {
  let hash = id | 0;
  hash = Math.imul(hash ^ (hash >>> 16), 0x45d9f3b);
  hash = Math.imul(hash ^ (hash >>> 16), 0x45d9f3b);
  return ((hash ^ (hash >>> 16)) >>> 0) / 0xffffffff;
}

/** Five plant-seeded anchors: history density cannot add ground bends. */
export function createCareTimelineGround(
  plantId: number,
  width: number,
  height: number,
): CareTimelineGround {
  if (
    !Number.isFinite(width) ||
    width <= 0 ||
    !Number.isFinite(height) ||
    height <= 0
  ) {
    throw new RangeError("Timeline dimensions must be positive and finite");
  }
  const baseline = height - GROUND_FLOOR;
  const anchors = Array.from({ length: 5 }, (_, index) => ({
    x: (index / 4) * width,
    y:
      baseline -
      GROUND_OFFSET -
      identitySeed(plantId ^ Math.imul(index + 1, 0x6d2b79f5)) *
        Math.min(GROUND_DEPTH_CAP, height * GROUND_DEPTH_RATIO),
  }));
  const path = anchors
    .map((point, index) => {
      if (!index) return `M ${point.x} ${point.y}`;
      const previous = anchors[index - 1];
      const third = (point.x - previous.x) / 3;
      return `C ${previous.x + third} ${previous.y} ${point.x - third} ${point.y} ${point.x} ${point.y}`;
    })
    .join(" ");
  return { width, height, anchors, path };
}

/** The cubic's x controls are thirds, so its parameter is the true x fraction. */
export function sampleCareTimelineGround(
  ground: CareTimelineGround,
  x: number,
): number {
  const position = Math.max(0, Math.min(4, (x / ground.width) * 4));
  const index = Math.min(3, Math.floor(position));
  const u = position - index;
  const start = ground.anchors[index];
  const end = ground.anchors[index + 1];
  return start.y + (end.y - start.y) * u * u * (3 - 2 * u);
}

const ICON_SEPARATION = 18; // 14px icon and 4px clearance on either axis.
const BASE_LANES = 4;
const TOP = 20;

export interface CareTimelinePlan {
  placements: readonly {
    event: NormalizedCareEvent;
    x: number;
    lane: number;
  }[];
  laneCount: number;
  minimumHeight: number;
}

/**
 * Color horizontal conflict intervals in chronological/ID order. Prefer the
 * nearest free identity-derived base lane, allocating extra lanes only when
 * every existing lane conflicts. Neither lanes nor minimum height depend on
 * the current host height, so reporting the minimum cannot cause resize loops.
 */
export function planCareTimelineEvents(
  events: readonly NormalizedCareEvent[],
  now: number,
  width: number,
  windowDays = 28,
): CareTimelinePlan {
  const start = windowStart(now, windowDays);
  if (!Number.isFinite(width) || width <= 0)
    throw new RangeError("Timeline width must be positive and finite");
  const lastX: number[] = Array<number>(BASE_LANES).fill(-Infinity);
  const placements = [...events]
    .sort((a, b) => a.occurredAt - b.occurredAt || a.id - b.id)
    .map((event) => {
      const x = ((event.occurredAt - start) / (now - start)) * width;
      const preferred = Math.min(
        BASE_LANES - 1,
        Math.floor(identitySeed(event.id) * BASE_LANES),
      );
      const candidates = lastX
        .map((_, lane) => lane)
        .sort((a, b) => {
          if (a >= BASE_LANES || b >= BASE_LANES) return a - b;
          return Math.abs(a - preferred) - Math.abs(b - preferred) || a - b;
        });
      const lane =
        candidates.find((lane) => x - lastX[lane] >= ICON_SEPARATION) ??
        lastX.length;
      lastX[lane] = x;
      return { event, x, lane };
    });
  const laneCount = placements.length ? lastX.length : 0;
  const fixedHeight =
    TOP +
    ICON_SEPARATION * (laneCount - 1) +
    GROUND_FLOOR +
    GROUND_OFFSET +
    STEM_CLEARANCE;
  // Solve H >= fixedHeight + min(cap, ratio * H), independently of host size.
  const proportionalHeight = fixedHeight / (1 - GROUND_DEPTH_RATIO);
  const requiredHeight =
    proportionalHeight <= GROUND_DEPTH_CAP / GROUND_DEPTH_RATIO
      ? proportionalHeight
      : fixedHeight + GROUND_DEPTH_CAP;
  return {
    placements,
    laneCount,
    minimumHeight: laneCount ? Math.max(140, Math.ceil(requiredHeight)) : 0,
  };
}

/** Project a height-independent plan into a host that honors its minimum. */
export function layoutCareTimelinePlan(
  plan: CareTimelinePlan,
  ground: CareTimelineGround,
): CareTimelinePoint[] {
  if (plan.placements.length && ground.height < plan.minimumHeight)
    throw new RangeError("Timeline host must honor its planned minimum height");
  const lowestGround = Math.min(...ground.anchors.map(({ y }) => y));
  const step =
    (lowestGround - STEM_CLEARANCE - TOP) /
    Math.max(BASE_LANES - 1, plan.laneCount - 1);
  // Use spare lane space for organic, ID-stable heights while preserving
  // the full icon clearance even when neighboring lanes jitter toward each other.
  const jitterRadius = Math.max(0, (step - ICON_SEPARATION) / 2);
  const highestCenter = lowestGround - STEM_CLEARANCE;
  return plan.placements.map(({ event, x, lane }) => ({
    event,
    x,
    y: Math.max(
      TOP,
      Math.min(
        highestCenter,
        TOP +
          lane * step +
          (identitySeed(event.id ^ 0x6d2b79f5) * 2 - 1) * jitterRadius,
      ),
    ),
    groundY: sampleCareTimelineGround(ground, x),
  }));
}

/** Convenience projection for callers already honoring the planned minimum. */
export function layoutCareTimelineEvents(
  events: readonly NormalizedCareEvent[],
  now: number,
  ground: CareTimelineGround,
  windowDays = 28,
): CareTimelinePoint[] {
  return layoutCareTimelinePlan(
    planCareTimelineEvents(events, now, ground.width, windowDays),
    ground,
  );
}
