import { beforeEach, describe, expect, it, vi } from 'vitest';
import {
	fetchPlants,
	fetchPlant,
	createPlant,
	deletePlant,
	fetchLocations,
	createLocation,
	fetchCareEvents,
	fetchAllCareEvents,
	importData
} from './api';

beforeEach(() => {
	vi.restoreAllMocks();
});

function mockFetch(response: Partial<Response>) {
	const fn = vi.fn().mockResolvedValue({
		ok: true,
		status: 200,
		json: vi.fn().mockResolvedValue({}),
		...response
	});
	globalThis.fetch = fn;
	return fn;
}

describe('request helper (via public API functions)', () => {
	it('returns parsed JSON on success', async () => {
		const data = [{ id: 1, name: 'Fern' }];
		mockFetch({ ok: true, status: 200, json: vi.fn().mockResolvedValue(data) });
		const result = await fetchPlants();
		expect(result).toEqual(data);
	});

	it('sends GET request with correct URL', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue([]) });
		await fetchPlants();
		expect(fn).toHaveBeenCalledWith('/api/plants', { method: 'GET' });
	});

	it('sends POST request with JSON body', async () => {
		const plant = { id: 1, name: 'Fern' };
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue(plant) });
		await createPlant({ name: 'Fern' });
		expect(fn).toHaveBeenCalledWith('/api/plants', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name: 'Fern' })
		});
	});

	it('throws ApiError on non-ok response', async () => {
		mockFetch({
			ok: false,
			status: 404,
			statusText: 'Not Found',
			json: vi.fn().mockResolvedValue({ message: 'Plant not found' })
		});
		await expect(fetchPlant(999)).rejects.toThrow('Plant not found');
	});

	it('uses statusText when error JSON has no message', async () => {
		mockFetch({
			ok: false,
			status: 500,
			statusText: 'Internal Server Error',
			json: vi.fn().mockResolvedValue({})
		});
		await expect(fetchPlant(1)).rejects.toThrow('Internal Server Error');
	});

	it('uses statusText when error JSON parsing fails', async () => {
		mockFetch({
			ok: false,
			status: 500,
			statusText: 'Internal Server Error',
			json: vi.fn().mockRejectedValue(new Error('parse error'))
		});
		await expect(fetchPlant(1)).rejects.toThrow('Internal Server Error');
	});

	it('returns undefined for 204 No Content', async () => {
		mockFetch({ ok: true, status: 204 });
		const result = await deletePlant(1);
		expect(result).toBeUndefined();
	});

	it('includes status on thrown error', async () => {
		mockFetch({
			ok: false,
			status: 422,
			statusText: 'Unprocessable Entity',
			json: vi.fn().mockResolvedValue({ message: 'Validation error' })
		});
		try {
			await createPlant({ name: '' });
		} catch (e: any) {
			expect(e.status).toBe(422);
			expect(e.message).toBe('Validation error');
		}
	});
});

describe('API endpoint functions', () => {
	it('fetchPlant calls correct URL', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue({}) });
		await fetchPlant(42);
		expect(fn).toHaveBeenCalledWith('/api/plants/42', { method: 'GET' });
	});

	it('fetchLocations calls correct URL', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue([]) });
		await fetchLocations();
		expect(fn).toHaveBeenCalledWith('/api/locations', { method: 'GET' });
	});

	it('createLocation sends name in body', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue({}) });
		await createLocation('Bedroom');
		expect(fn).toHaveBeenCalledWith('/api/locations', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ name: 'Bedroom' })
		});
	});

	it('fetchCareEvents calls correct URL', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue([]) });
		await fetchCareEvents(5);
		expect(fn).toHaveBeenCalledWith('/api/plants/5/care', { method: 'GET' });
	});

	it('fetchAllCareEvents builds query string', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue({ events: [], has_more: false }) });
		await fetchAllCareEvents(10, 5, 'watering');
		const url = fn.mock.calls[0][0] as string;
		expect(url).toContain('limit=10');
		expect(url).toContain('before=5');
		expect(url).toContain('type=watering');
	});

	it('fetchAllCareEvents with no params has no query string', async () => {
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue({ events: [], has_more: false }) });
		await fetchAllCareEvents();
		expect(fn).toHaveBeenCalledWith('/api/care', { method: 'GET' });
	});
});

describe('importData', () => {
	it('sends POST with FormData to /api/data/import', async () => {
		const result = { locations: 1, plants: 2, care_events: 3, photos: 0 };
		const fn = mockFetch({ ok: true, json: vi.fn().mockResolvedValue(result) });
		const file = new File(['zip content'], 'export.zip', { type: 'application/zip' });
		const response = await importData(file);
		expect(response).toEqual(result);
		expect(fn).toHaveBeenCalledTimes(1);
		const [url, init] = fn.mock.calls[0];
		expect(url).toBe('/api/data/import');
		expect(init.method).toBe('POST');
		expect(init.body).toBeInstanceOf(FormData);
	});

	it('throws ApiError on failure', async () => {
		mockFetch({
			ok: false,
			status: 400,
			statusText: 'Bad Request',
			json: vi.fn().mockResolvedValue({ message: 'Version mismatch' })
		});
		const file = new File(['bad'], 'bad.zip', { type: 'application/zip' });
		await expect(importData(file)).rejects.toThrow('Version mismatch');
	});
});
