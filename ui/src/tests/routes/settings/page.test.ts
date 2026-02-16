import { cleanup, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../../../routes/settings/+page.svelte';
import { setThemePreference, THEME_STORAGE_KEY } from '$lib/stores/theme';

vi.mock('$lib/stores/locations', async () => {
	const { writable } = await import('svelte/store');
	return {
		locations: writable([]),
		locationsError: writable(null),
		loadLocations: vi.fn(),
		deleteLocation: vi.fn()
	};
});

beforeEach(() => {
	localStorage.clear();
	setThemePreference('system');
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
