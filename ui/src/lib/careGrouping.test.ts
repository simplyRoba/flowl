import { describe, expect, it } from "vitest";
import type { CareEvent } from "$lib/api";
import { groupCareEvents, isGroup } from "./careGrouping";
import type { WateringGroup } from "./careGrouping";

function makeEvent(overrides: Partial<CareEvent> & { id: number }): CareEvent {
  return {
    plant_id: 1,
    plant_name: "Monstera",
    event_type: "watered",
    notes: null,
    photo_url: null,
    occurred_at: "2026-03-14T10:00:00Z",
    created_at: "2026-03-14T10:00:00Z",
    ...overrides,
  };
}

describe("groupCareEvents", () => {
  it("groups consecutive waterings without notes or photos", () => {
    const events = [
      makeEvent({ id: 3, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({ id: 2, occurred_at: "2026-03-07T10:00:00Z" }),
      makeEvent({ id: 1, occurred_at: "2026-02-28T10:00:00Z" }),
    ];
    const result = groupCareEvents(events);
    expect(result).toHaveLength(1);
    expect(isGroup(result[0])).toBe(true);
    const group = result[0] as WateringGroup;
    expect(group.count).toBe(3);
    expect(group.firstAt).toBe("2026-02-28T10:00:00Z");
    expect(group.lastAt).toBe("2026-03-14T10:00:00Z");
    expect(group.plantName).toBe("Monstera");
    expect(group.key).toBe("watering-3");
    expect(group.partial).toBe(false);
  });

  it("breaks streak when watering has notes", () => {
    const events = [
      makeEvent({ id: 3, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({
        id: 2,
        occurred_at: "2026-03-07T10:00:00Z",
        notes: "Extra dry",
      }),
      makeEvent({ id: 1, occurred_at: "2026-02-28T10:00:00Z" }),
    ];
    const result = groupCareEvents(events);
    // Event with notes breaks streak: event 3 alone, event 2 individual, event 1 alone
    expect(result).toHaveLength(3);
    expect(result.every((r) => !isGroup(r))).toBe(true);
  });

  it("breaks streak when watering has photo", () => {
    const events = [
      makeEvent({ id: 3, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({
        id: 2,
        occurred_at: "2026-03-07T10:00:00Z",
        photo_url: "/uploads/photo.jpg",
      }),
      makeEvent({ id: 1, occurred_at: "2026-02-28T10:00:00Z" }),
    ];
    const result = groupCareEvents(events);
    expect(result).toHaveLength(3);
    expect(result.every((r) => !isGroup(r))).toBe(true);
  });

  it("does not group a streak of one", () => {
    const events = [
      makeEvent({ id: 2, event_type: "fertilized" }),
      makeEvent({ id: 1 }),
    ];
    const result = groupCareEvents(events);
    expect(result).toHaveLength(2);
    expect(result.every((r) => !isGroup(r))).toBe(true);
  });

  it("groups a streak of two", () => {
    const events = [
      makeEvent({ id: 2, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({ id: 1, occurred_at: "2026-03-07T10:00:00Z" }),
    ];
    const result = groupCareEvents(events);
    expect(result).toHaveLength(1);
    expect(isGroup(result[0])).toBe(true);
    expect((result[0] as WateringGroup).count).toBe(2);
  });

  it("tracks interleaved plants independently", () => {
    // Monstera waterings interleaved with Ficus waterings
    const events = [
      makeEvent({
        id: 6,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-14T10:00:00Z",
      }),
      makeEvent({
        id: 5,
        plant_id: 2,
        plant_name: "Ficus",
        occurred_at: "2026-03-13T10:00:00Z",
      }),
      makeEvent({
        id: 4,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-12T10:00:00Z",
      }),
      makeEvent({
        id: 3,
        plant_id: 2,
        plant_name: "Ficus",
        occurred_at: "2026-03-11T10:00:00Z",
      }),
      makeEvent({
        id: 2,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-10T10:00:00Z",
      }),
      makeEvent({
        id: 1,
        plant_id: 2,
        plant_name: "Ficus",
        occurred_at: "2026-03-09T10:00:00Z",
      }),
    ];
    const result = groupCareEvents(events);
    // Both plants should have their own group
    expect(result).toHaveLength(2);
    expect(result.every((r) => isGroup(r))).toBe(true);
    const monstera = result[0] as WateringGroup;
    const ficus = result[1] as WateringGroup;
    expect(monstera.plantName).toBe("Monstera");
    expect(monstera.count).toBe(3);
    expect(monstera.key).toBe("watering-6");
    expect(monstera.partial).toBe(false);
    expect(ficus.plantName).toBe("Ficus");
    expect(ficus.count).toBe(3);
    expect(ficus.key).toBe("watering-5");
    expect(ficus.partial).toBe(false);
  });

  it("breaks streak when same plant has non-watering event", () => {
    const events = [
      makeEvent({ id: 3, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({
        id: 2,
        event_type: "fertilized",
        occurred_at: "2026-03-10T10:00:00Z",
      }),
      makeEvent({ id: 1, occurred_at: "2026-03-07T10:00:00Z" }),
    ];
    const result = groupCareEvents(events);
    // Fertilize breaks the watering streak for plant 1
    expect(result).toHaveLength(3);
    expect(result.every((r) => !isGroup(r))).toBe(true);
  });

  it("handles mixed scenario end-to-end", () => {
    const events = [
      // Monstera: 3 waterings (group)
      makeEvent({
        id: 10,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-14T10:00:00Z",
      }),
      makeEvent({
        id: 9,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-12T10:00:00Z",
      }),
      // Ficus fertilized (individual)
      makeEvent({
        id: 8,
        plant_id: 2,
        plant_name: "Ficus",
        event_type: "fertilized",
        occurred_at: "2026-03-11T10:00:00Z",
      }),
      makeEvent({
        id: 7,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-10T10:00:00Z",
      }),
      // Monstera pruned (breaks Monstera streak, individual)
      makeEvent({
        id: 6,
        plant_id: 1,
        plant_name: "Monstera",
        event_type: "pruned",
        occurred_at: "2026-03-08T10:00:00Z",
      }),
      // Monstera watered with notes (individual, starts new streak but notes break it)
      makeEvent({
        id: 5,
        plant_id: 1,
        plant_name: "Monstera",
        notes: "Soil was very dry",
        occurred_at: "2026-03-05T10:00:00Z",
      }),
      // Monstera watered plain (solo, no streak partner)
      makeEvent({
        id: 4,
        plant_id: 1,
        plant_name: "Monstera",
        occurred_at: "2026-03-01T10:00:00Z",
      }),
    ];
    const result = groupCareEvents(events);

    // Expected order:
    // 1. Monstera group (3 waterings: ids 10, 9, 7)
    // 2. Ficus fertilized (individual)
    // 3. Monstera pruned (individual)
    // 4. Monstera watered with notes (individual)
    // 5. Monstera watered plain (individual, streak of 1)
    expect(result).toHaveLength(5);

    expect(isGroup(result[0])).toBe(true);
    const group = result[0] as WateringGroup;
    expect(group.count).toBe(3);
    expect(group.plantName).toBe("Monstera");

    expect(isGroup(result[1])).toBe(false);
    expect((result[1] as CareEvent).event_type).toBe("fertilized");

    expect(isGroup(result[2])).toBe(false);
    expect((result[2] as CareEvent).event_type).toBe("pruned");

    expect(isGroup(result[3])).toBe(false);
    expect((result[3] as CareEvent).notes).toBe("Soil was very dry");

    expect(isGroup(result[4])).toBe(false);
    expect((result[4] as CareEvent).id).toBe(4);
  });

  it("emits an unresolved multi-member oldest-boundary streak as partial", () => {
    const events = [
      makeEvent({ id: 3, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({ id: 2, occurred_at: "2026-03-07T10:00:00Z" }),
    ];

    const result = groupCareEvents(events, true);

    expect(result).toHaveLength(1);
    expect(isGroup(result[0])).toBe(true);
    const group = result[0] as WateringGroup;
    expect(group).toMatchObject({
      key: "watering-3",
      partial: true,
      count: 2,
      firstAt: "2026-03-07T10:00:00Z",
      lastAt: "2026-03-14T10:00:00Z",
    });
    expect(group.events.map((event) => event.id)).toEqual([3, 2]);
  });

  it("emits an unresolved one-member oldest-boundary streak as partial", () => {
    const event = makeEvent({ id: 1 });

    const result = groupCareEvents([event], true);

    expect(result).toHaveLength(1);
    expect(isGroup(result[0])).toBe(true);
    expect(result[0]).toMatchObject({
      key: "watering-1",
      partial: true,
      count: 1,
      events: [event],
    });
  });

  it("keeps a partial group's newest-event key while older members append", () => {
    const newestPage = [
      makeEvent({ id: 4, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({ id: 3, occurred_at: "2026-03-07T10:00:00Z" }),
    ];
    const firstGroup = groupCareEvents(newestPage, true)[0] as WateringGroup;

    const result = groupCareEvents(
      [
        ...newestPage,
        makeEvent({ id: 2, occurred_at: "2026-02-28T10:00:00Z" }),
      ],
      true,
    );

    expect(result).toHaveLength(1);
    expect(isGroup(result[0])).toBe(true);
    const expandedGroup = result[0] as WateringGroup;
    expect(expandedGroup.key).toBe("watering-4");
    expect(expandedGroup.key).toBe(firstGroup.key);
    expect(expandedGroup.partial).toBe(true);
    expect(expandedGroup.count).toBe(3);
    expect(expandedGroup.events.map((event) => event.id)).toEqual([4, 3, 2]);
  });

  it("completes a partial group when an older same-plant breaker arrives", () => {
    const newestPage = [
      makeEvent({ id: 3, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({ id: 2, occurred_at: "2026-03-07T10:00:00Z" }),
    ];
    const partial = groupCareEvents(newestPage, true)[0] as WateringGroup;

    const result = groupCareEvents(
      [
        ...newestPage,
        makeEvent({
          id: 1,
          event_type: "fertilized",
          occurred_at: "2026-02-28T10:00:00Z",
        }),
      ],
      true,
    );

    expect(result).toHaveLength(2);
    expect(isGroup(result[0])).toBe(true);
    const complete = result[0] as WateringGroup;
    expect(complete.key).toBe(partial.key);
    expect(complete.partial).toBe(false);
    expect(complete.count).toBe(2);
    expect((result[1] as CareEvent).id).toBe(1);
  });

  it("completes a partial group when loaded history reaches its end", () => {
    const events = [
      makeEvent({ id: 2, occurred_at: "2026-03-14T10:00:00Z" }),
      makeEvent({ id: 1, occurred_at: "2026-03-07T10:00:00Z" }),
    ];
    const partial = groupCareEvents(events, true)[0] as WateringGroup;

    const result = groupCareEvents(events, false);

    expect(result).toHaveLength(1);
    expect(isGroup(result[0])).toBe(true);
    const complete = result[0] as WateringGroup;
    expect(complete.key).toBe(partial.key);
    expect(complete.partial).toBe(false);
    expect(complete.count).toBe(2);
  });

  it("returns an exact one-member boundary streak as an individual event", () => {
    const event = makeEvent({ id: 1 });

    expect(groupCareEvents([event], false)).toEqual([event]);
  });

  it("returns empty array for empty input", () => {
    expect(groupCareEvents([])).toEqual([]);
  });

  it("passes through non-watering events untouched", () => {
    const events = [
      makeEvent({ id: 2, event_type: "repotted" }),
      makeEvent({ id: 1, event_type: "pruned" }),
    ];
    const result = groupCareEvents(events);
    expect(result).toHaveLength(2);
    expect(result.every((r) => !isGroup(r))).toBe(true);
  });
});
