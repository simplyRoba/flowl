import { writable } from 'svelte/store';
import type { CareEvent, CreateCareEvent } from '$lib/api';
import * as api from '$lib/api';

export const careEvents = writable<CareEvent[]>([]);
export const careError = writable<string | null>(null);

export async function loadCareEvents(plantId: number) {
	careError.set(null);
	try {
		const data = await api.fetchCareEvents(plantId);
		careEvents.set(data);
	} catch (e) {
		careError.set(e instanceof Error ? e.message : 'Failed to load care events');
	}
}

export async function addCareEvent(plantId: number, data: CreateCareEvent): Promise<CareEvent | null> {
	careError.set(null);
	try {
		const event = await api.createCareEvent(plantId, data);
		careEvents.update((list) => [event, ...list]);
		return event;
	} catch (e) {
		careError.set(e instanceof Error ? e.message : 'Failed to add care event');
		return null;
	}
}

export async function removeCareEvent(plantId: number, eventId: number): Promise<boolean> {
	careError.set(null);
	try {
		await api.deleteCareEvent(plantId, eventId);
		careEvents.update((list) => list.filter((e) => e.id !== eventId));
		return true;
	} catch (e) {
		careError.set(e instanceof Error ? e.message : 'Failed to delete care event');
		return false;
	}
}
