import { get } from 'svelte/store';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import type { Location } from '$lib/api';
import {
	locations,
	locationsError,
	loadLocations,
	createLocation,
	updateLocation,
	deleteLocation
} from './locations';

vi.mock('$lib/api', () => ({
	fetchLocations: vi.fn(),
	createLocation: vi.fn(),
	updateLocation: vi.fn(),
	deleteLocation: vi.fn()
}));

import * as api from '$lib/api';

const mockLocation: Location = { id: 1, name: 'Bedroom', plant_count: 2 };
const mockLocation2: Location = { id: 2, name: 'Kitchen', plant_count: 1 };
const mockLocation3: Location = { id: 3, name: 'Balcony', plant_count: 0 };

beforeEach(() => {
	locations.set([]);
	locationsError.set(null);
	vi.clearAllMocks();
});

describe('loadLocations', () => {
	it('sets locations on success', async () => {
		vi.mocked(api.fetchLocations).mockResolvedValue([mockLocation, mockLocation2]);
		await loadLocations();
		expect(get(locations)).toEqual([mockLocation, mockLocation2]);
		expect(get(locationsError)).toBeNull();
	});

	it('sets error on failure', async () => {
		vi.mocked(api.fetchLocations).mockRejectedValue(new Error('Network error'));
		await loadLocations();
		expect(get(locations)).toEqual([]);
		expect(get(locationsError)).toBe('Network error');
	});

	it('uses fallback message for non-Error throws', async () => {
		vi.mocked(api.fetchLocations).mockRejectedValue(42);
		await loadLocations();
		expect(get(locationsError)).toBe('Failed to load locations');
	});
});

describe('createLocation', () => {
	it('inserts location in sorted order', async () => {
		locations.set([mockLocation2]); // Kitchen
		vi.mocked(api.createLocation).mockResolvedValue(mockLocation3); // Balcony
		const result = await createLocation('Balcony');
		expect(result).toEqual(mockLocation3);
		const list = get(locations);
		expect(list[0].name).toBe('Balcony');
		expect(list[1].name).toBe('Kitchen');
	});

	it('sets error on failure', async () => {
		vi.mocked(api.createLocation).mockRejectedValue(new Error('Duplicate'));
		const result = await createLocation('Bedroom');
		expect(result).toBeNull();
		expect(get(locationsError)).toBe('Duplicate');
	});
});

describe('updateLocation', () => {
	it('updates location and re-sorts', async () => {
		locations.set([mockLocation, mockLocation2]); // Bedroom, Kitchen
		const updated = { ...mockLocation, name: 'Patio' };
		vi.mocked(api.updateLocation).mockResolvedValue(updated);
		const result = await updateLocation(1, 'Patio');
		expect(result).toEqual(updated);
		const list = get(locations);
		expect(list[0].name).toBe('Kitchen');
		expect(list[1].name).toBe('Patio');
	});

	it('sets error on failure', async () => {
		vi.mocked(api.updateLocation).mockRejectedValue(new Error('Update failed'));
		const result = await updateLocation(1, 'New Name');
		expect(result).toBeNull();
		expect(get(locationsError)).toBe('Update failed');
	});
});

describe('deleteLocation', () => {
	it('removes location from list', async () => {
		locations.set([mockLocation, mockLocation2]);
		vi.mocked(api.deleteLocation).mockResolvedValue(undefined);
		const result = await deleteLocation(1);
		expect(result).toBe(true);
		expect(get(locations)).toEqual([mockLocation2]);
	});

	it('sets error on failure', async () => {
		vi.mocked(api.deleteLocation).mockRejectedValue(new Error('Delete failed'));
		const result = await deleteLocation(1);
		expect(result).toBe(false);
		expect(get(locationsError)).toBe('Delete failed');
	});
});
