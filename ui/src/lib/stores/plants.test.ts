import { get } from 'svelte/store';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import type { Plant, CreatePlant, UpdatePlant } from '$lib/api';
import {
	plants,
	currentPlant,
	plantsError,
	loadPlants,
	loadPlant,
	createPlant,
	updatePlant,
	deletePlant,
	uploadPhoto,
	waterPlant,
	deletePhoto
} from './plants';

vi.mock('$lib/api', () => ({
	fetchPlants: vi.fn(),
	fetchPlant: vi.fn(),
	createPlant: vi.fn(),
	updatePlant: vi.fn(),
	deletePlant: vi.fn(),
	waterPlant: vi.fn(),
	uploadPlantPhoto: vi.fn(),
	deletePlantPhoto: vi.fn()
}));

import * as api from '$lib/api';

const mockPlant: Plant = {
	id: 1,
	name: 'Fern',
	species: 'Boston Fern',
	icon: '🌿',
	photo_url: null,
	location_id: null,
	location_name: null,
	watering_interval_days: 7,
	watering_status: 'ok',
	last_watered: '2025-01-01',
	next_due: '2025-01-08',
	light_needs: 'indirect',
	notes: null,
	difficulty: null,
	pet_safety: null,
	growth_speed: null,
	soil_type: null,
	soil_moisture: null,
	created_at: '2025-01-01T00:00:00Z',
	updated_at: '2025-01-01T00:00:00Z'
};

const mockPlant2: Plant = {
	...mockPlant,
	id: 2,
	name: 'Cactus',
	species: 'Saguaro'
};

beforeEach(() => {
	plants.set([]);
	currentPlant.set(null);
	plantsError.set(null);
	vi.clearAllMocks();
});

describe('loadPlants', () => {
	it('sets plants on successful load', async () => {
		vi.mocked(api.fetchPlants).mockResolvedValue([mockPlant, mockPlant2]);
		await loadPlants();
		expect(get(plants)).toEqual([mockPlant, mockPlant2]);
		expect(get(plantsError)).toBeNull();
	});

	it('sets error on failure', async () => {
		vi.mocked(api.fetchPlants).mockRejectedValue(new Error('Network error'));
		await loadPlants();
		expect(get(plants)).toEqual([]);
		expect(get(plantsError)).toBe('Network error');
	});

	it('sets fallback error for non-Error throws', async () => {
		vi.mocked(api.fetchPlants).mockRejectedValue('unknown');
		await loadPlants();
		expect(get(plantsError)).toBe('Failed to load plants');
	});
});

describe('loadPlant', () => {
	it('sets currentPlant on success', async () => {
		vi.mocked(api.fetchPlant).mockResolvedValue(mockPlant);
		const result = await loadPlant(1);
		expect(result).toEqual(mockPlant);
		expect(get(currentPlant)).toEqual(mockPlant);
		expect(get(plantsError)).toBeNull();
	});

	it('sets error and clears currentPlant on failure', async () => {
		vi.mocked(api.fetchPlant).mockRejectedValue(new Error('Not found'));
		const result = await loadPlant(99);
		expect(result).toBeNull();
		expect(get(currentPlant)).toBeNull();
		expect(get(plantsError)).toBe('Not found');
	});
});

describe('createPlant', () => {
	it('appends new plant to list on success', async () => {
		plants.set([mockPlant]);
		vi.mocked(api.createPlant).mockResolvedValue(mockPlant2);
		const result = await createPlant({ name: 'Cactus' } as CreatePlant);
		expect(result).toEqual(mockPlant2);
		expect(get(plants)).toEqual([mockPlant, mockPlant2]);
		expect(get(plantsError)).toBeNull();
	});

	it('sets error on failure', async () => {
		vi.mocked(api.createPlant).mockRejectedValue(new Error('Validation failed'));
		const result = await createPlant({ name: '' } as CreatePlant);
		expect(result).toBeNull();
		expect(get(plantsError)).toBe('Validation failed');
	});
});

describe('updatePlant', () => {
	it('updates plant in list and sets currentPlant', async () => {
		plants.set([mockPlant, mockPlant2]);
		const updated = { ...mockPlant, name: 'Updated Fern' };
		vi.mocked(api.updatePlant).mockResolvedValue(updated);
		const result = await updatePlant(1, { name: 'Updated Fern' } as UpdatePlant);
		expect(result).toEqual(updated);
		expect(get(plants)[0].name).toBe('Updated Fern');
		expect(get(currentPlant)).toEqual(updated);
	});

	it('sets error on failure', async () => {
		vi.mocked(api.updatePlant).mockRejectedValue(new Error('Update failed'));
		const result = await updatePlant(1, { name: '' } as UpdatePlant);
		expect(result).toBeNull();
		expect(get(plantsError)).toBe('Update failed');
	});
});

describe('deletePlant', () => {
	it('removes plant from list and clears currentPlant', async () => {
		plants.set([mockPlant, mockPlant2]);
		currentPlant.set(mockPlant);
		vi.mocked(api.deletePlant).mockResolvedValue(undefined);
		const result = await deletePlant(1);
		expect(result).toBe(true);
		expect(get(plants)).toEqual([mockPlant2]);
		expect(get(currentPlant)).toBeNull();
	});

	it('sets error on failure', async () => {
		vi.mocked(api.deletePlant).mockRejectedValue(new Error('Delete failed'));
		const result = await deletePlant(1);
		expect(result).toBe(false);
		expect(get(plantsError)).toBe('Delete failed');
	});
});

describe('waterPlant', () => {
	it('updates plant in list on success', async () => {
		plants.set([mockPlant]);
		const watered = { ...mockPlant, last_watered: '2025-01-10', watering_status: 'ok' };
		vi.mocked(api.waterPlant).mockResolvedValue(watered);
		const result = await waterPlant(1);
		expect(result).toEqual(watered);
		expect(get(plants)[0].last_watered).toBe('2025-01-10');
		expect(get(currentPlant)).toEqual(watered);
	});

	it('sets error on failure', async () => {
		vi.mocked(api.waterPlant).mockRejectedValue(new Error('Water failed'));
		const result = await waterPlant(1);
		expect(result).toBeNull();
		expect(get(plantsError)).toBe('Water failed');
	});
});

describe('uploadPhoto', () => {
	it('updates plant with new photo on success', async () => {
		plants.set([mockPlant]);
		const withPhoto = { ...mockPlant, photo_url: '/uploads/photo.jpg' };
		vi.mocked(api.uploadPlantPhoto).mockResolvedValue(withPhoto);
		const file = new File(['test'], 'photo.jpg', { type: 'image/jpeg' });
		const result = await uploadPhoto(1, file);
		expect(result).toEqual(withPhoto);
		expect(get(plants)[0].photo_url).toBe('/uploads/photo.jpg');
		expect(get(currentPlant)).toEqual(withPhoto);
	});

	it('sets error on failure', async () => {
		vi.mocked(api.uploadPlantPhoto).mockRejectedValue(new Error('Upload failed'));
		const file = new File(['test'], 'photo.jpg', { type: 'image/jpeg' });
		const result = await uploadPhoto(1, file);
		expect(result).toBeNull();
		expect(get(plantsError)).toBe('Upload failed');
	});
});

describe('deletePhoto', () => {
	it('clears photo_url from plant in list and currentPlant', async () => {
		const plantWithPhoto = { ...mockPlant, photo_url: '/uploads/photo.jpg' };
		plants.set([plantWithPhoto, mockPlant2]);
		currentPlant.set(plantWithPhoto);
		vi.mocked(api.deletePlantPhoto).mockResolvedValue(undefined);
		const result = await deletePhoto(1);
		expect(result).toBe(true);
		expect(get(plants)[0].photo_url).toBeNull();
		expect(get(currentPlant)!.photo_url).toBeNull();
	});

	it('sets error on failure', async () => {
		vi.mocked(api.deletePlantPhoto).mockRejectedValue(new Error('Delete photo failed'));
		const result = await deletePhoto(1);
		expect(result).toBe(false);
		expect(get(plantsError)).toBe('Delete photo failed');
	});
});
