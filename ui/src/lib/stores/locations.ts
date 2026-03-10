import { writable } from "svelte/store";
import type { Location } from "$lib/api";
import * as api from "$lib/api";
import { resolveError } from "./errors";

export const locations = writable<Location[]>([]);
export const locationsError = writable<string | null>(null);

export type CreateLocationResult = { location: Location } | { error: string };

export async function loadLocations() {
  locationsError.set(null);
  try {
    const data = await api.fetchLocations();
    locations.set(data);
  } catch (e) {
    locationsError.set(resolveError(e, "loadLocations"));
  }
}

export async function createLocation(
  name: string,
): Promise<CreateLocationResult> {
  locationsError.set(null);
  try {
    const location = await api.createLocation(name);
    locations.update((list) =>
      [...list, location].sort((a, b) => a.name.localeCompare(b.name)),
    );
    return { location };
  } catch (e) {
    const message = resolveError(e, "createLocation");
    locationsError.set(message);
    return { error: message };
  }
}

export async function updateLocation(
  id: number,
  name: string,
): Promise<{ location: Location } | { error: string }> {
  locationsError.set(null);
  try {
    const location = await api.updateLocation(id, name);
    locations.update((list) =>
      list
        .map((l) => (l.id === id ? location : l))
        .sort((a, b) => a.name.localeCompare(b.name)),
    );
    return { location };
  } catch (e) {
    const message = resolveError(e, "updateLocation");
    locationsError.set(message);
    return { error: message };
  }
}

export async function deleteLocation(id: number): Promise<boolean> {
  locationsError.set(null);
  try {
    await api.deleteLocation(id);
    locations.update((list) => list.filter((l) => l.id !== id));
    return true;
  } catch (e) {
    locationsError.set(resolveError(e, "deleteLocation"));
    return false;
  }
}
