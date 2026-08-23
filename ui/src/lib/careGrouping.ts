import type { CareEvent } from "$lib/api";

export interface WateringGroup {
  kind: "group";
  /** Stable identity anchored to the newest event in the streak. */
  key: string;
  /** Whether the streak may continue into unloaded older history. */
  partial: boolean;
  plantId: number;
  plantName: string;
  count: number;
  firstAt: string;
  lastAt: string;
  events: CareEvent[];
}

export type TimelineItem = CareEvent | WateringGroup;

export function isGroup(item: TimelineItem): item is WateringGroup {
  return "kind" in item && item.kind === "group";
}

/**
 * Groups consecutive watering events per plant into summaries.
 * Events are expected newest-first. Only watering events with no notes
 * and no photo are eligible for grouping. Events from other plants do
 * NOT break a plant's streak — streaks are tracked independently per plant.
 * A streak of 2+ becomes a WateringGroup; an exact streak of 1 stays
 * individual. When older history remains, every streak active at the oldest
 * loaded boundary is unresolved and emitted as a partial group.
 */
export function groupCareEvents(
  events: CareEvent[],
  hasMore = false,
): TimelineItem[] {
  type Annotated = {
    event: CareEvent;
    groupable: boolean; // watered, no notes, no photo
  };

  const annotated: Annotated[] = events.map((e) => ({
    event: e,
    groupable: e.event_type === "watered" && !e.notes && !e.photo_url,
  }));

  // For each plant, find the streak segments.
  // A streak for plant P is a maximal run of groupable events for P,
  // uninterrupted by any non-groupable event for the SAME plant.
  // Events for OTHER plants between two groupable events of P do NOT
  // break P's streak.

  // To handle interleaving: we walk the array and track per-plant state.
  // When we see a non-groupable event for plant P (or a groupable event
  // for P that is watered-with-notes/photo, or a non-watering event),
  // we flush P's streak.

  // Assign a streak ID to each groupable event.
  const streakIds = new Array<number | null>(annotated.length).fill(null);
  let nextStreakId = 0;
  // plantId -> current streak id
  const activeStreak = new Map<number, number>();
  // streakId -> list of indices
  const streakMembers = new Map<number, number[]>();

  for (let i = 0; i < annotated.length; i++) {
    const { event, groupable } = annotated[i];
    const pid = event.plant_id;

    if (groupable) {
      if (!activeStreak.has(pid)) {
        const sid = nextStreakId++;
        activeStreak.set(pid, sid);
        streakMembers.set(sid, []);
      }
      const sid = activeStreak.get(pid)!;
      streakIds[i] = sid;
      streakMembers.get(sid)!.push(i);
    } else {
      // This event breaks plant pid's streak (if any).
      activeStreak.delete(pid);
    }
  }

  // Any streak still active after the oldest loaded event could continue
  // into an older page. A same-plant breaker removes its streak from this map,
  // so only truly unresolved boundary streaks are partial.
  const partialStreaks = hasMore
    ? new Set(activeStreak.values())
    : new Set<number>();

  // Now build the result. Walk events in order. Exact streaks need at least
  // two members to group; unresolved boundary streaks group even with one.

  const emittedStreaks = new Set<number>();
  const result: TimelineItem[] = [];

  for (let i = 0; i < annotated.length; i++) {
    const sid = streakIds[i];
    if (sid === null) {
      result.push(annotated[i].event);
      continue;
    }

    const members = streakMembers.get(sid)!;
    const partial = partialStreaks.has(sid);
    if (members.length < 2 && !partial) {
      result.push(annotated[i].event);
      continue;
    }

    if (emittedStreaks.has(sid)) {
      // Already emitted this streak's group — skip this member.
      continue;
    }

    emittedStreaks.add(sid);

    // Members are in timeline order (newest-first), so first member
    // is newest, last member is oldest.
    const memberEvents = members.map((idx) => annotated[idx].event);
    const lastAt = memberEvents[0].occurred_at; // newest
    const firstAt = memberEvents[memberEvents.length - 1].occurred_at; // oldest

    result.push({
      kind: "group",
      key: `watering-${memberEvents[0].id}`,
      partial,
      plantId: memberEvents[0].plant_id,
      plantName: memberEvents[0].plant_name,
      count: memberEvents.length,
      firstAt,
      lastAt,
      events: memberEvents,
    });
  }

  return result;
}
