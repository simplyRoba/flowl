import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { get } from 'svelte/store';
import Page from '../../routes/+page.svelte';

const mockLoadPlants = vi.fn();

vi.mock('$lib/stores/plants', async () => {
	const { writable } = await import('svelte/store');
	const plants = writable<any[]>([]);
	const plantsError = writable<string | null>(null);
	return {
		plants,
		plantsError,
		loadPlants: (...args: any[]) => mockLoadPlants(...args)
	};
});

vi.mock('$lib/emoji', () => ({
	emojiToSvgPath: (emoji: string) => `/emoji/${emoji}.svg`
}));

import { plants, plantsError } from '$lib/stores/plants';

beforeEach(() => {
	plants.set([]);
	plantsError.set(null);
	vi.clearAllMocks();
});

afterEach(() => {
	cleanup();
});

describe('dashboard page', () => {
	it('calls loadPlants on mount', () => {
		render(Page);
		expect(mockLoadPlants).toHaveBeenCalled();
	});

	it('shows empty state when no plants', () => {
		render(Page);
		expect(screen.getByText('No plants yet')).toBeTruthy();
		expect(screen.getByText('Add your first plant to get started.')).toBeTruthy();
	});

	it('shows Add Plant link in empty state', () => {
		render(Page);
		const addLink = screen.getByText('Add Plant').closest('a');
		expect(addLink?.getAttribute('href')).toBe('/plants/new');
	});

	it('shows error message when plantsError is set', () => {
		plantsError.set('Server error');
		render(Page);
		expect(screen.getByText('Server error')).toBeTruthy();
	});

	it('renders plant cards with mocked data', () => {
		plants.set([
			{
				id: 1,
				name: 'Fern',
				species: 'Boston Fern',
				icon: '🌿',
				photo_url: null,
				location_id: 1,
				location_name: 'Bedroom',
				watering_interval_days: 7,
				watering_status: 'ok',
				last_watered: '2025-01-01',
				next_due: '2025-01-08',
				light_needs: 'indirect',
				notes: null,
				created_at: '2025-01-01T00:00:00Z',
				updated_at: '2025-01-01T00:00:00Z'
			},
			{
				id: 2,
				name: 'Cactus',
				species: 'Saguaro',
				icon: '🌵',
				photo_url: null,
				location_id: null,
				location_name: null,
				watering_interval_days: 30,
				watering_status: 'due',
				last_watered: '2024-12-15',
				next_due: '2025-01-14',
				light_needs: 'direct',
				notes: null,
				created_at: '2024-12-01T00:00:00Z',
				updated_at: '2024-12-01T00:00:00Z'
			}
		]);
		render(Page);
		expect(screen.getByText('Fern')).toBeTruthy();
		expect(screen.getByText('Cactus')).toBeTruthy();
		expect(screen.getByText('Bedroom')).toBeTruthy();
	});

	it('links plant cards to plant detail page', () => {
		plants.set([
			{
				id: 42,
				name: 'Fern',
				species: null,
				icon: '🌿',
				photo_url: null,
				location_id: null,
				location_name: null,
				watering_interval_days: 7,
				watering_status: 'ok',
				last_watered: null,
				next_due: null,
				light_needs: 'indirect',
				notes: null,
				created_at: '2025-01-01T00:00:00Z',
				updated_at: '2025-01-01T00:00:00Z'
			}
		]);
		render(Page);
		const link = screen.getByText('Fern').closest('a');
		expect(link?.getAttribute('href')).toBe('/plants/42');
	});

	it('shows "My Plants" header', () => {
		render(Page);
		expect(screen.getByText('My Plants')).toBeTruthy();
	});

	it('shows greeting text', () => {
		render(Page);
		// The greeting is random but there should be an h2 in the greeting div
		const headings = screen.getAllByRole('heading', { level: 2 });
		expect(headings.length).toBeGreaterThanOrEqual(1);
	});
});
