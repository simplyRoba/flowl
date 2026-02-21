import { cleanup, fireEvent, render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../../../../routes/plants/[id]/+page.svelte';

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
	this.setAttribute('open', '');
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
	this.removeAttribute('open');
});

const mockLoadPlant = vi.fn();
const mockDeletePlant = vi.fn();
const mockWaterPlant = vi.fn();
const mockLoadCareEvents = vi.fn();
const mockAddCareEvent = vi.fn();
const mockRemoveCareEvent = vi.fn();

vi.mock('$app/stores', async () => {
	const { readable } = await import('svelte/store');
	return {
		page: readable({
			params: { id: '1' },
			url: new URL('http://localhost/plants/1')
		})
	};
});

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

vi.mock('$lib/stores/plants', async () => {
	const { writable } = await import('svelte/store');
	const currentPlant = writable<any | null>(null);
	const plantsError = writable<string | null>(null);
	return {
		currentPlant,
		plantsError,
		loadPlant: (...args: any[]) => mockLoadPlant(...args),
		deletePlant: (...args: any[]) => mockDeletePlant(...args),
		waterPlant: (...args: any[]) => mockWaterPlant(...args)
	};
});

vi.mock('$lib/stores/care', async () => {
	const { writable } = await import('svelte/store');
	const careEvents = writable<any[]>([]);
	return {
		careEvents,
		loadCareEvents: (...args: any[]) => mockLoadCareEvents(...args),
		addCareEvent: (...args: any[]) => mockAddCareEvent(...args),
		removeCareEvent: (...args: any[]) => mockRemoveCareEvent(...args)
	};
});

vi.mock('$lib/emoji', () => ({
	emojiToSvgPath: (emoji: string) => `/emoji/${emoji}.svg`
}));

import { currentPlant } from '$lib/stores/plants';

function makePlant(overrides: Partial<any> = {}) {
	return {
		id: 1,
		name: 'Fern',
		species: 'Boston Fern',
		icon: '🌿',
		photo_url: '/uploads/fern.jpg',
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

async function renderWithPlant(plantOverrides: Partial<any> = {}) {
	const plant = makePlant(plantOverrides);
	mockLoadPlant.mockImplementationOnce(async () => {
		currentPlant.set(plant);
		return plant;
	});
	return render(Page);
}

beforeEach(() => {
	currentPlant.set(null);
	vi.clearAllMocks();
});

afterEach(() => {
	cleanup();
});

function getLightbox() {
	return document.querySelector('dialog.lightbox') as HTMLDialogElement;
}

describe('plant detail lightbox', () => {
	it('opens and closes the lightbox for a photo', async () => {
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });
		await fireEvent.click(openButton);
		expect(getLightbox().hasAttribute('open')).toBe(true);

		// Escape triggers the dialog's cancel event
		getLightbox().dispatchEvent(new Event('cancel'));
		await vi.waitFor(() => {
			expect(getLightbox().hasAttribute('open')).toBe(false);
		});
	});

	it('does not expose a lightbox trigger when no photo is available', async () => {
		await renderWithPlant({ photo_url: null });
		await vi.waitFor(() => {
			expect(screen.queryByRole('button', { name: 'Open photo' })).toBeNull();
		});
	});

	it('updates zoom on wheel input', async () => {
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });
		await fireEvent.click(openButton);
		const img = document.querySelector('.lightbox-image') as HTMLImageElement;
		expect(img).toBeTruthy();
		const before = img.style.transform;
		await fireEvent.wheel(img, { deltaY: -600 });
		expect(img.style.transform).not.toBe(before);
	});

	it('pans the image when zoomed', async () => {
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });
		await fireEvent.click(openButton);
		const img = document.querySelector('.lightbox-image') as HTMLImageElement;
		expect(img).toBeTruthy();
		Object.defineProperty(img, 'clientWidth', { value: 400 });
		Object.defineProperty(img, 'clientHeight', { value: 300 });
		await fireEvent.wheel(img, { deltaY: -600 });
		const before = img.style.transform;
		await fireEvent.pointerDown(img, { clientX: 100, clientY: 100 });
		await fireEvent.pointerMove(window, { clientX: 160, clientY: 140 });
		await fireEvent.pointerUp(window);
		expect(img.style.transform).not.toBe(before);
	});

	it('closes the lightbox via close button', async () => {
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });
		await fireEvent.click(openButton);
		expect(getLightbox().hasAttribute('open')).toBe(true);

		const closeButton = screen.getByRole('button', { name: 'Close' });
		await fireEvent.click(closeButton);
		await vi.waitFor(() => {
			expect(getLightbox().hasAttribute('open')).toBe(false);
		});
	});

	it('locks body scroll while lightbox is open and restores on close', async () => {
		document.body.style.overflow = '';
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });

		await fireEvent.click(openButton);
		expect(document.body.style.overflow).toBe('hidden');

		getLightbox().dispatchEvent(new Event('cancel'));
		await vi.waitFor(() => {
			expect(getLightbox().hasAttribute('open')).toBe(false);
		});
		expect(document.body.style.overflow).toBe('');
	});

	it('zooms via touch pinch gesture', async () => {
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });
		await fireEvent.click(openButton);
		const img = document.querySelector('.lightbox-image') as HTMLImageElement;
		expect(img).toBeTruthy();
		const before = img.style.transform;

		const startEvent = new Event('touchstart', { bubbles: true }) as any;
		startEvent.touches = [
			{ clientX: 100, clientY: 100 },
			{ clientX: 200, clientY: 200 }
		];
		await fireEvent(window, startEvent);

		const moveEvent = new Event('touchmove', { bubbles: true, cancelable: true }) as any;
		moveEvent.touches = [
			{ clientX: 50, clientY: 50 },
			{ clientX: 250, clientY: 250 }
		];
		await fireEvent(window, moveEvent);

		expect(img.style.transform).not.toBe(before);
	});

	it('closes the lightbox via backdrop click', async () => {
		await renderWithPlant();
		const openButton = await screen.findByRole('button', { name: 'Open photo' });
		await fireEvent.click(openButton);
		const lightbox = getLightbox();
		expect(lightbox.hasAttribute('open')).toBe(true);

		// Click directly on the dialog element (backdrop area)
		await fireEvent.click(lightbox);
		await vi.waitFor(() => {
			expect(lightbox.hasAttribute('open')).toBe(false);
		});
	});
});

describe('plant delete confirmation', () => {
	function getDeleteIconButton() {
		return document.querySelector('.btn-danger') as HTMLButtonElement;
	}

	it('shows confirmation dialog with plant name when delete is clicked', async () => {
		await renderWithPlant({ name: 'My Fern' });
		await screen.findByText('My Fern');

		const user = userEvent.setup();
		await user.click(getDeleteIconButton());

		await waitFor(() => {
			expect(screen.getByText(/Delete "My Fern"/)).toBeTruthy();
		});
	});

	it('calls deletePlant when confirmed', async () => {
		mockDeletePlant.mockResolvedValue(true);
		await renderWithPlant({ name: 'My Fern' });
		await screen.findByText('My Fern');

		const user = userEvent.setup();
		await user.click(getDeleteIconButton());

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Delete' })).toBeTruthy();
		});
		await user.click(screen.getByRole('button', { name: 'Delete' }));

		await waitFor(() => {
			expect(mockDeletePlant).toHaveBeenCalledWith(1);
		});
	});

	it('does not call deletePlant when cancelled', async () => {
		await renderWithPlant({ name: 'My Fern' });
		await screen.findByText('My Fern');

		const user = userEvent.setup();
		await user.click(getDeleteIconButton());

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
		});
		await user.click(screen.getByRole('button', { name: 'Cancel' }));

		await new Promise((r) => setTimeout(r, 50));
		expect(mockDeletePlant).not.toHaveBeenCalled();
	});
});
