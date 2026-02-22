import { writable, get } from 'svelte/store';
import type { CareEvent, CreateCareEvent } from '$lib/api';
import * as api from '$lib/api';
import { translations } from './locale';

export const careEvents = writable<CareEvent[]>([]);
export const careError = writable<string | null>(null);

export async function loadCareEvents(plantId: number) {
	careError.set(null);
	try {
		const data = await api.fetchCareEvents(plantId);
		careEvents.set(data);
	} catch (e) {
		careError.set(e instanceof Error ? e.message : get(translations).error.loadCareEvents);
	}
}

function eventTime(event: CareEvent): number {
	const time = new Date(event.occurred_at).getTime();
	return Number.isNaN(time) ? 0 : time;
}

function sortEvents(events: CareEvent[]): CareEvent[] {
	return [...events].sort((a, b) => eventTime(b) - eventTime(a));
}

export async function addCareEvent(plantId: number, data: CreateCareEvent): Promise<CareEvent | null> {
	careError.set(null);
	try {
		const event = await api.createCareEvent(plantId, data);
		careEvents.update((list) => sortEvents([event, ...list]));
		return event;
	} catch (e) {
		careError.set(e instanceof Error ? e.message : get(translations).error.addCareEvent);
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
		careError.set(e instanceof Error ? e.message : get(translations).error.deleteCareEvent);
		return false;
	}
}
