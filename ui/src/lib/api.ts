export interface Location {
	id: number;
	name: string;
}

export interface Plant {
	id: number;
	name: string;
	species: string | null;
	icon: string;
	location_id: number | null;
	location_name: string | null;
	watering_interval_days: number;
	light_needs: string;
	notes: string | null;
	created_at: string;
	updated_at: string;
}

export interface CreatePlant {
	name: string;
	species?: string;
	icon?: string;
	location_id?: number | null;
	watering_interval_days?: number;
	light_needs?: string;
	notes?: string;
}

export interface UpdatePlant {
	name?: string;
	species?: string;
	icon?: string;
	location_id?: number | null;
	watering_interval_days?: number;
	light_needs?: string;
	notes?: string;
}

class ApiError extends Error {
	status: number;

	constructor(status: number, message: string) {
		super(message);
		this.status = status;
	}
}

async function request<T>(method: string, url: string, body?: unknown): Promise<T> {
	const init: RequestInit = { method };
	if (body !== undefined) {
		init.headers = { 'Content-Type': 'application/json' };
		init.body = JSON.stringify(body);
	}

	const resp = await fetch(url, init);

	if (!resp.ok) {
		const data = await resp.json().catch(() => ({ message: resp.statusText }));
		throw new ApiError(resp.status, data.message || resp.statusText);
	}

	if (resp.status === 204) {
		return undefined as T;
	}

	return resp.json();
}

export function fetchPlants(): Promise<Plant[]> {
	return request('GET', '/api/plants');
}

export function fetchPlant(id: number): Promise<Plant> {
	return request('GET', `/api/plants/${id}`);
}

export function createPlant(data: CreatePlant): Promise<Plant> {
	return request('POST', '/api/plants', data);
}

export function updatePlant(id: number, data: UpdatePlant): Promise<Plant> {
	return request('PUT', `/api/plants/${id}`, data);
}

export function deletePlant(id: number): Promise<void> {
	return request('DELETE', `/api/plants/${id}`);
}

export function fetchLocations(): Promise<Location[]> {
	return request('GET', '/api/locations');
}

export function createLocation(name: string): Promise<Location> {
	return request('POST', '/api/locations', { name });
}

export function updateLocation(id: number, name: string): Promise<Location> {
	return request('PUT', `/api/locations/${id}`, { name });
}

export function deleteLocation(id: number): Promise<void> {
	return request('DELETE', `/api/locations/${id}`);
}
