import { writable, get } from 'svelte/store';
import type { Plant, CreatePlant, UpdatePlant } from '$lib/api';
import * as api from '$lib/api';
import { translations } from './locale';

export const plants = writable<Plant[]>([]);
export const currentPlant = writable<Plant | null>(null);
export const plantsError = writable<string | null>(null);

export async function loadPlants() {
	plantsError.set(null);
	try {
		const data = await api.fetchPlants();
		plants.set(data);
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : get(translations).error.loadPlants);
	}
}

export async function loadPlant(id: number) {
	plantsError.set(null);
	try {
		const data = await api.fetchPlant(id);
		currentPlant.set(data);
		return data;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : get(translations).error.loadPlant);
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
		plantsError.set(e instanceof Error ? e.message : get(translations).error.createPlant);
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
		plantsError.set(e instanceof Error ? e.message : get(translations).error.updatePlant);
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
		plantsError.set(e instanceof Error ? e.message : get(translations).error.deletePlant);
		return false;
	}
}

export async function uploadPhoto(plantId: number, file: File): Promise<Plant | null> {
	plantsError.set(null);
	try {
		const plant = await api.uploadPlantPhoto(plantId, file);
		plants.update((list) => list.map((p) => (p.id === plantId ? plant : p)));
		currentPlant.set(plant);
		return plant;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : get(translations).error.uploadPhoto);
		return null;
	}
}

export async function waterPlant(id: number): Promise<Plant | null> {
	plantsError.set(null);
	try {
		const plant = await api.waterPlant(id);
		plants.update((list) => list.map((p) => (p.id === id ? plant : p)));
		currentPlant.set(plant);
		return plant;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : get(translations).error.waterPlant);
		return null;
	}
}

export async function deletePhoto(plantId: number): Promise<boolean> {
	plantsError.set(null);
	try {
		await api.deletePlantPhoto(plantId);
		const updater = (p: Plant) =>
			p.id === plantId ? { ...p, photo_url: null } : p;
		plants.update((list) => list.map(updater));
		currentPlant.update((p) => (p && p.id === plantId ? { ...p, photo_url: null } : p));
		return true;
	} catch (e) {
		plantsError.set(e instanceof Error ? e.message : get(translations).error.deletePhoto);
		return false;
	}
}
