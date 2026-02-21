export interface Location {
	id: number;
	name: string;
	plant_count: number;
}

export interface Plant {
	id: number;
	name: string;
	species: string | null;
	icon: string;
	photo_url: string | null;
	location_id: number | null;
	location_name: string | null;
	watering_interval_days: number;
	watering_status: string;
	last_watered: string | null;
	next_due: string | null;
	light_needs: string;
	difficulty: string | null;
	pet_safety: string | null;
	growth_speed: string | null;
	soil_type: string | null;
	soil_moisture: string | null;
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
	difficulty?: string | null;
	pet_safety?: string | null;
	growth_speed?: string | null;
	soil_type?: string | null;
	soil_moisture?: string | null;
	notes?: string;
}

export interface UpdatePlant {
	name?: string;
	species?: string;
	icon?: string;
	location_id?: number | null;
	watering_interval_days?: number;
	light_needs?: string;
	difficulty?: string | null;
	pet_safety?: string | null;
	growth_speed?: string | null;
	soil_type?: string | null;
	soil_moisture?: string | null;
	notes?: string;
}

export interface AppInfo {
	version: string;
	repository: string;
	license: string;
}

export interface Stats {
	plant_count: number;
	care_event_count: number;
}

export interface MqttStatus {
	status: 'connected' | 'disconnected' | 'disabled';
	broker: string | null;
	topic_prefix: string | null;
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

export function fetchAppInfo(): Promise<AppInfo> {
	return request('GET', '/api/info');
}

export function fetchStats(): Promise<Stats> {
	return request('GET', '/api/stats');
}

export function fetchMqttStatus(): Promise<MqttStatus> {
	return request('GET', '/api/mqtt/status');
}

export interface MqttRepairResult {
	cleared: number;
	published: number;
}

export function repairMqtt(): Promise<MqttRepairResult> {
	return request('POST', '/api/mqtt/repair');
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

export function waterPlant(id: number): Promise<Plant> {
	return request('POST', `/api/plants/${id}/water`);
}

export async function uploadPlantPhoto(plantId: number, file: File): Promise<Plant> {
	const formData = new FormData();
	formData.append('file', file);

	const resp = await fetch(`/api/plants/${plantId}/photo`, {
		method: 'POST',
		body: formData
	});

	if (!resp.ok) {
		const data = await resp.json().catch(() => ({ message: resp.statusText }));
		throw new ApiError(resp.status, data.message || resp.statusText);
	}

	return resp.json();
}

export async function deletePlantPhoto(plantId: number): Promise<void> {
	const resp = await fetch(`/api/plants/${plantId}/photo`, { method: 'DELETE' });

	if (!resp.ok) {
		const data = await resp.json().catch(() => ({ message: resp.statusText }));
		throw new ApiError(resp.status, data.message || resp.statusText);
	}
}

// --- Import/Export ---

export interface ImportResult {
	locations: number;
	plants: number;
	care_events: number;
	photos: number;
}

export async function importData(file: File): Promise<ImportResult> {
	const formData = new FormData();
	formData.append('file', file);

	const resp = await fetch('/api/data/import', {
		method: 'POST',
		body: formData
	});

	if (!resp.ok) {
		const data = await resp.json().catch(() => ({ message: resp.statusText }));
		throw new ApiError(resp.status, data.message || resp.statusText);
	}

	return resp.json();
}

// --- Care Events ---

export interface CareEvent {
	id: number;
	plant_id: number;
	plant_name: string;
	event_type: string;
	notes: string | null;
	occurred_at: string;
	created_at: string;
}

export interface CreateCareEvent {
	event_type: string;
	notes?: string;
	occurred_at?: string;
}

export interface CareEventsPage {
	events: CareEvent[];
	has_more: boolean;
}

export function fetchCareEvents(plantId: number): Promise<CareEvent[]> {
	return request('GET', `/api/plants/${plantId}/care`);
}

export function fetchAllCareEvents(
	limit?: number,
	before?: number,
	type?: string
): Promise<CareEventsPage> {
	const params = new URLSearchParams();
	if (limit !== undefined) params.set('limit', String(limit));
	if (before !== undefined) params.set('before', String(before));
	if (type !== undefined) params.set('type', type);
	const qs = params.toString();
	return request('GET', `/api/care${qs ? `?${qs}` : ''}`);
}

export function createCareEvent(plantId: number, data: CreateCareEvent): Promise<CareEvent> {
	return request('POST', `/api/plants/${plantId}/care`, data);
}

export function deleteCareEvent(plantId: number, eventId: number): Promise<void> {
	return request('DELETE', `/api/plants/${plantId}/care/${eventId}`);
}

// --- Locations ---

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
