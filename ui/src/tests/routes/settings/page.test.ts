import { cleanup, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../../../routes/settings/+page.svelte';
import { setThemePreference, THEME_STORAGE_KEY } from '$lib/stores/theme';
import { locations, locationsError } from '$lib/stores/locations';

const mockDeleteLocation = vi.fn();
const mockUpdateLocation = vi.fn();

vi.mock('$lib/stores/locations', async () => {
	const { writable } = await import('svelte/store');
	return {
		locations: writable([]),
		locationsError: writable(null),
		loadLocations: vi.fn(),
		deleteLocation: (...args: any[]) => mockDeleteLocation(...args),
		updateLocation: (...args: any[]) => mockUpdateLocation(...args)
	};
});

beforeEach(() => {
	localStorage.clear();
	setThemePreference('system');
	locations.set([]);
	locationsError.set(null);
	vi.clearAllMocks();
});

afterEach(() => {
	cleanup();
});

describe('settings appearance theme selector', () => {
	it('shows appearance section with light, dark, and system options', () => {
		render(Page);

		expect(screen.getByText('Appearance')).toBeTruthy();
		expect(screen.getByRole('radiogroup', { name: 'Theme' })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Light' })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Dark' })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'System' })).toBeTruthy();
	});

	it('persists selection and reflects active state', async () => {
		const user = userEvent.setup();
		render(Page);

		const darkButton = screen.getByRole('button', { name: 'Dark' });
		await user.click(darkButton);

		expect(localStorage.getItem(THEME_STORAGE_KEY)).toBe('dark');
		expect(darkButton.classList.contains('active')).toBe(true);
	});
});

describe('settings locations section', () => {
	it('shows Locations heading', () => {
		render(Page);
		expect(screen.getByText('Locations')).toBeTruthy();
	});

	it('shows empty state when no locations', () => {
		render(Page);
		expect(screen.getByText('No locations yet. Create locations when adding plants.')).toBeTruthy();
	});

	it('renders location list', () => {
		locations.set([
			{ id: 1, name: 'Bedroom', plant_count: 2 },
			{ id: 2, name: 'Kitchen', plant_count: 0 }
		]);
		render(Page);
		expect(screen.getByText('Bedroom')).toBeTruthy();
		expect(screen.getByText('Kitchen')).toBeTruthy();
	});

	it('shows plant count badge for locations with plants', () => {
		locations.set([
			{ id: 1, name: 'Bedroom', plant_count: 3 }
		]);
		render(Page);
		expect(screen.getByText('3 plants')).toBeTruthy();
	});

	it('shows singular plant count', () => {
		locations.set([
			{ id: 1, name: 'Bedroom', plant_count: 1 }
		]);
		render(Page);
		expect(screen.getByText('1 plant')).toBeTruthy();
	});

	it('shows error when locationsError is set', () => {
		locationsError.set('Failed to load');
		render(Page);
		expect(screen.getByText('Failed to load')).toBeTruthy();
	});
});
