import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../../routes/+page.svelte';

const mockLoadPlants = vi.fn();
const mockWaterPlant = vi.fn();

vi.mock('$lib/stores/plants', async () => {
	const { writable } = await import('svelte/store');
	const plants = writable<any[]>([]);
	const plantsError = writable<string | null>(null);
	return {
		plants,
		plantsError,
		loadPlants: (...args: any[]) => mockLoadPlants(...args),
		waterPlant: (...args: any[]) => mockWaterPlant(...args)
	};
});

vi.mock('$lib/emoji', () => ({
	emojiToSvgPath: (emoji: string) => `/emoji/${emoji}.svg`
}));

import { plants, plantsError } from '$lib/stores/plants';

function makePlant(overrides: Partial<any> = {}) {
	return {
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
		difficulty: null,
		pet_safety: null,
		growth_speed: null,
		soil_type: null,
		soil_moisture: null,
		created_at: '2025-01-01T00:00:00Z',
		updated_at: '2025-01-01T00:00:00Z',
		...overrides
	};
}

beforeEach(() => {
	plants.set([]);
	plantsError.set(null);
	vi.clearAllMocks();
	mockWaterPlant.mockResolvedValue(null);
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
			makePlant({ id: 1, name: 'Fern', watering_status: 'ok', location_name: 'Bedroom' }),
			makePlant({ id: 2, name: 'Cactus', watering_status: 'due', location_name: null })
		]);
		render(Page);
		expect(screen.getAllByText('Fern').length).toBeGreaterThanOrEqual(1);
		expect(screen.getAllByText('Cactus').length).toBeGreaterThanOrEqual(1);
		expect(screen.getByText('Bedroom')).toBeTruthy();
	});

	it('links plant cards to plant detail page', () => {
		plants.set([makePlant({ id: 42, name: 'Fern' })]);
		render(Page);
		const link = screen.getByText('Fern').closest('a');
		expect(link?.getAttribute('href')).toBe('/plants/42?from=/');
	});

	it('shows "My Plants" header', () => {
		render(Page);
		expect(screen.getByText('My Plants')).toBeTruthy();
	});

	it('shows greeting text', () => {
		render(Page);
		const headings = screen.getAllByRole('heading', { level: 2 });
		expect(headings.length).toBeGreaterThanOrEqual(1);
	});
});

describe('dynamic greeting subtitle', () => {
	it('shows attention subtitle when plants need water', () => {
		plants.set([
			makePlant({ id: 1, name: 'Fern', watering_status: 'due' }),
			makePlant({ id: 2, name: 'Cactus', watering_status: 'overdue' })
		]);
		render(Page);
		const greeting = document.querySelector('.greeting p');
		expect(greeting?.textContent).toBeTruthy();
		// Should contain the count "2" in the subtitle
		expect(greeting?.textContent).toMatch(/2/);
	});

	it('shows singular attention subtitle for one plant', () => {
		plants.set([makePlant({ id: 1, name: 'Fern', watering_status: 'due' })]);
		render(Page);
		const greeting = document.querySelector('.greeting p');
		expect(greeting?.textContent).toBeTruthy();
		// Should contain attention keywords (singular variant)
		expect(greeting?.textContent).toMatch(/thirsty|drink|waiting for water|calling|hydrate/);
	});

	it('shows default time-of-day subtitle when all plants are ok', () => {
		plants.set([makePlant({ id: 1, name: 'Fern', watering_status: 'ok' })]);
		render(Page);
		const greeting = document.querySelector('.greeting p');
		expect(greeting?.textContent).toBeTruthy();
		// Should NOT contain attention keywords
		expect(greeting?.textContent).not.toMatch(/thirsty|drink|waiting for water|calling|hydrate/);
	});

	it('shows default subtitle when no plants exist', () => {
		render(Page);
		const greeting = document.querySelector('.greeting p');
		expect(greeting?.textContent).toBeTruthy();
		expect(greeting?.textContent).not.toMatch(/thirsty|drink|waiting for water|calling|hydrate/);
	});
});

describe('needs attention section', () => {
	it('renders when plants are due or overdue', () => {
		plants.set([
			makePlant({ id: 1, name: 'Fern', watering_status: 'overdue' }),
			makePlant({ id: 2, name: 'Cactus', watering_status: 'due' })
		]);
		render(Page);
		expect(screen.getByText('Needs Attention')).toBeTruthy();
	});

	it('is hidden when all plants are ok', () => {
		plants.set([makePlant({ id: 1, name: 'Fern', watering_status: 'ok' })]);
		render(Page);
		expect(screen.queryByText('Needs Attention')).toBeNull();
	});

	it('is hidden when no plants exist', () => {
		render(Page);
		expect(screen.queryByText('Needs Attention')).toBeNull();
	});

	it('shows overdue plants before due plants', () => {
		plants.set([
			makePlant({ id: 1, name: 'Due Plant', watering_status: 'due' }),
			makePlant({ id: 2, name: 'Overdue Plant', watering_status: 'overdue' }),
			makePlant({ id: 3, name: 'Ok Plant', watering_status: 'ok' })
		]);
		render(Page);
		const attentionSection = document.querySelector('.attention-section');
		expect(attentionSection).toBeTruthy();
		const names = attentionSection!.querySelectorAll('.attention-card-name');
		expect(names.length).toBe(2);
		expect(names[0].textContent).toBe('Overdue Plant');
		expect(names[1].textContent).toBe('Due Plant');
	});

	it('does not include ok plants in attention section', () => {
		plants.set([
			makePlant({ id: 1, name: 'Fern', watering_status: 'ok' }),
			makePlant({ id: 2, name: 'Cactus', watering_status: 'due' })
		]);
		render(Page);
		const attentionSection = document.querySelector('.attention-section');
		const names = attentionSection!.querySelectorAll('.attention-card-name');
		expect(names.length).toBe(1);
		expect(names[0].textContent).toBe('Cactus');
	});

	it('shows photo when plant has photo_url', () => {
		plants.set([
			makePlant({ id: 1, name: 'Fern', watering_status: 'due', photo_url: '/uploads/fern.jpg' })
		]);
		render(Page);
		const attentionSection = document.querySelector('.attention-section');
		const img = attentionSection!.querySelector('.attention-photo-img') as HTMLImageElement;
		expect(img).toBeTruthy();
		expect(img.src).toContain('/uploads/fern.jpg');
	});

	it('shows emoji icon fallback when no photo', () => {
		plants.set([
			makePlant({ id: 1, name: 'Fern', watering_status: 'due', photo_url: null, icon: '🌿' })
		]);
		render(Page);
		const attentionSection = document.querySelector('.attention-section');
		const icon = attentionSection!.querySelector('.attention-icon') as HTMLImageElement;
		expect(icon).toBeTruthy();
		expect(icon.src).toContain('/emoji/');
	});

	it('links attention cards to plant detail page', () => {
		plants.set([makePlant({ id: 42, name: 'Fern', watering_status: 'due' })]);
		render(Page);
		const attentionSection = document.querySelector('.attention-section');
		const link = attentionSection!.querySelector('a');
		expect(link?.getAttribute('href')).toBe('/plants/42?from=/');
	});
});

describe('attention card water action', () => {
	it('calls waterPlant when Water button is clicked', async () => {
		plants.set([makePlant({ id: 1, name: 'Fern', watering_status: 'due' })]);
		render(Page);
		const waterBtn = screen.getByRole('button', { name: /Water/ });
		await waterBtn.click();
		expect(mockWaterPlant).toHaveBeenCalledWith(1);
	});

	it('shows loading state while watering', async () => {
		let resolveWater: () => void;
		mockWaterPlant.mockImplementation(
			() => new Promise<void>((resolve) => { resolveWater = resolve; })
		);
		plants.set([makePlant({ id: 1, name: 'Fern', watering_status: 'due' })]);
		render(Page);
		const waterBtn = screen.getByRole('button', { name: /Water/ }) as HTMLButtonElement;
		waterBtn.click();

		// Wait for the click handler to run
		await vi.waitFor(() => {
			expect(waterBtn.disabled).toBe(true);
		});

		resolveWater!();
	});

	it('removes plant from attention section after watering to ok', async () => {
		mockWaterPlant.mockImplementation((id: number) => {
			plants.update((list) =>
				list.map((p) => (p.id === id ? { ...p, watering_status: 'ok' } : p))
			);
			return Promise.resolve(null);
		});
		plants.set([
			makePlant({ id: 1, name: 'Fern', watering_status: 'due' }),
			makePlant({ id: 2, name: 'Cactus', watering_status: 'overdue' })
		]);
		render(Page);

		expect(document.querySelectorAll('.attention-card-name').length).toBe(2);

		const waterBtns = screen.getAllByRole('button', { name: /Water/ });
		await waterBtns[0].click();

		await vi.waitFor(() => {
			expect(document.querySelectorAll('.attention-card-name').length).toBe(1);
		});
	});
});
