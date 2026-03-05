import { writable } from 'svelte/store';
import type { AiStatus } from '$lib/api';
import * as api from '$lib/api';

export const aiStatus = writable<AiStatus | null>(null);

let loaded = false;

export async function loadAiStatus() {
	if (loaded) return;
	try {
		const status = await api.fetchAiStatus();
		aiStatus.set(status);
		loaded = true;
	} catch {
		aiStatus.set(null);
	}
}

export function resetAiStatus() {
	loaded = false;
	aiStatus.set(null);
}
