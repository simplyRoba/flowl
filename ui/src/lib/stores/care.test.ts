import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { CareEvent, CreateCareEvent } from '$lib/api';
import {
	careEvents,
	careError,
	loadCareEvents,
	addCareEvent,
	removeCareEvent
} from './care';

vi.mock('$lib/api', () => ({
	fetchCareEvents: vi.fn(),
	createCareEvent: vi.fn(),
	deleteCareEvent: vi.fn()
}));

import * as api from '$lib/api';

const mockEvent: CareEvent = {
	id: 1,
	plant_id: 10,
	plant_name: 'Fern',
	event_type: 'watering',
	notes: null,
	occurred_at: '2025-01-10T10:00:00Z',
	created_at: '2025-01-10T10:00:00Z'
};

const mockEvent2: CareEvent = {
	...mockEvent,
	id: 2,
	occurred_at: '2025-01-09T10:00:00Z',
	created_at: '2025-01-09T10:00:00Z'
};

beforeEach(() => {
	careEvents.set([]);
	careError.set(null);
	vi.clearAllMocks();
});

describe('loadCareEvents', () => {
	it('sets care events on success', async () => {
		vi.mocked(api.fetchCareEvents).mockResolvedValue([mockEvent, mockEvent2]);
		await loadCareEvents(10);
		expect(get(careEvents)).toEqual([mockEvent, mockEvent2]);
		expect(get(careError)).toBeNull();
	});

	it('sets error on failure', async () => {
		vi.mocked(api.fetchCareEvents).mockRejectedValue(new Error('Network error'));
		await loadCareEvents(10);
		expect(get(careEvents)).toEqual([]);
		expect(get(careError)).toBe('Network error');
	});

	it('uses fallback message for non-Error throws', async () => {
		vi.mocked(api.fetchCareEvents).mockRejectedValue(null);
		await loadCareEvents(10);
		expect(get(careError)).toBe('Failed to load care events');
	});
});

describe('addCareEvent', () => {
	it('adds event and sorts by occurred_at descending', async () => {
		careEvents.set([mockEvent]); // Jan 10
		const newEvent: CareEvent = {
			...mockEvent,
			id: 3,
			occurred_at: '2025-01-11T10:00:00Z',
			created_at: '2025-01-11T10:00:00Z'
		};
		vi.mocked(api.createCareEvent).mockResolvedValue(newEvent);
		const result = await addCareEvent(10, { event_type: 'watering' } as CreateCareEvent);
		expect(result).toEqual(newEvent);
		const events = get(careEvents);
		expect(events[0].id).toBe(3); // newest first
		expect(events[1].id).toBe(1);
	});

	it('inserts older event at correct position', async () => {
		careEvents.set([mockEvent]); // Jan 10
		const olderEvent: CareEvent = {
			...mockEvent,
			id: 4,
			occurred_at: '2025-01-05T10:00:00Z',
			created_at: '2025-01-05T10:00:00Z'
		};
		vi.mocked(api.createCareEvent).mockResolvedValue(olderEvent);
		const result = await addCareEvent(10, { event_type: 'watering' } as CreateCareEvent);
		expect(result).toEqual(olderEvent);
		const events = get(careEvents);
		expect(events[0].id).toBe(1); // Jan 10 first
		expect(events[1].id).toBe(4); // Jan 5 second
	});

	it('sets error on failure', async () => {
		vi.mocked(api.createCareEvent).mockRejectedValue(new Error('Add failed'));
		const result = await addCareEvent(10, { event_type: 'watering' } as CreateCareEvent);
		expect(result).toBeNull();
		expect(get(careError)).toBe('Add failed');
	});
});

describe('removeCareEvent', () => {
	it('removes event from list', async () => {
		careEvents.set([mockEvent, mockEvent2]);
		vi.mocked(api.deleteCareEvent).mockResolvedValue(undefined);
		const result = await removeCareEvent(10, 1);
		expect(result).toBe(true);
		expect(get(careEvents)).toEqual([mockEvent2]);
	});

	it('sets error on failure', async () => {
		vi.mocked(api.deleteCareEvent).mockRejectedValue(new Error('Delete failed'));
		const result = await removeCareEvent(10, 1);
		expect(result).toBe(false);
		expect(get(careError)).toBe('Delete failed');
	});
});
