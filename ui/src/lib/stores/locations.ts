import { writable, get } from 'svelte/store';
import type { Location } from '$lib/api';
import * as api from '$lib/api';
import { translations } from './locale';

export const locations = writable<Location[]>([]);
export const locationsError = writable<string | null>(null);

export async function loadLocations() {
	locationsError.set(null);
	try {
		const data = await api.fetchLocations();
		locations.set(data);
	} catch (e) {
		locationsError.set(e instanceof Error ? e.message : get(translations).error.loadLocations);
	}
}

export async function createLocation(name: string): Promise<Location | null> {
	locationsError.set(null);
	try {
		const location = await api.createLocation(name);
		locations.update((list) => [...list, location].sort((a, b) => a.name.localeCompare(b.name)));
		return location;
	} catch (e) {
		locationsError.set(e instanceof Error ? e.message : get(translations).error.createLocation);
		return null;
	}
}

export async function updateLocation(id: number, name: string): Promise<Location | null> {
	locationsError.set(null);
	try {
		const location = await api.updateLocation(id, name);
		locations.update((list) =>
			list.map((l) => (l.id === id ? location : l)).sort((a, b) => a.name.localeCompare(b.name))
		);
		return location;
	} catch (e) {
		locationsError.set(e instanceof Error ? e.message : get(translations).error.updateLocation);
		return null;
	}
}

export async function deleteLocation(id: number): Promise<boolean> {
	locationsError.set(null);
	try {
		await api.deleteLocation(id);
		locations.update((list) => list.filter((l) => l.id !== id));
		return true;
	} catch (e) {
		locationsError.set(e instanceof Error ? e.message : get(translations).error.deleteLocation);
		return false;
	}
}
