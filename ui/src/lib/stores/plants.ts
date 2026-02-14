import { writable } from 'svelte/store';
import type { Plant, CreatePlant, UpdatePlant } from '$lib/api';
import * as api from '$lib/api';

export const plants = writable<Plant[]>([]);
export const currentPlant = writable<Plant | null>(null);
export const plantsError = writable<string | null>(null);

export async function loadPlants() {
	plantsError.set(null);
	try {
		const data = await api.fetchPlants();
		plants.set(data);
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : 'Failed to load plants');
	}
}

export async function loadPlant(id: number) {
	plantsError.set(null);
	try {
		const data = await api.fetchPlant(id);
		currentPlant.set(data);
		return data;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : 'Failed to load plant');
		currentPlant.set(null);
		return null;
	}
}

export async function createPlant(data: CreatePlant): Promise<Plant | null> {
	plantsError.set(null);
	try {
		const plant = await api.createPlant(data);
		plants.update((list) => [...list, plant]);
		return plant;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : 'Failed to create plant');
		return null;
	}
}

export async function updatePlant(id: number, data: UpdatePlant): Promise<Plant | null> {
	plantsError.set(null);
	try {
		const plant = await api.updatePlant(id, data);
		plants.update((list) => list.map((p) => (p.id === id ? plant : p)));
		currentPlant.set(plant);
		return plant;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : 'Failed to update plant');
		return null;
	}
}

export async function deletePlant(id: number): Promise<boolean> {
	plantsError.set(null);
	try {
		await api.deletePlant(id);
		plants.update((list) => list.filter((p) => p.id !== id));
		currentPlant.set(null);
		return true;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : 'Failed to delete plant');
		return false;
	}
}
