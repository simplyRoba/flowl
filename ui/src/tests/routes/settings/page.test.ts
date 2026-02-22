import { cleanup, render, screen, waitFor } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../../../routes/settings/+page.svelte';
import { setThemePreference, THEME_STORAGE_KEY } from '$lib/stores/theme';
import { locations, locationsError } from '$lib/stores/locations';
import * as api from '$lib/api';

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
	this.setAttribute('open', '');
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
	this.removeAttribute('open');
});

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

describe('settings data section export/import', () => {
	beforeEach(() => {
		vi.spyOn(api, 'fetchStats').mockResolvedValue({ plant_count: 5, care_event_count: 10 });
		vi.spyOn(api, 'fetchAppInfo').mockRejectedValue(new Error('skip'));
		vi.spyOn(api, 'fetchMqttStatus').mockRejectedValue(new Error('skip'));
	});

	it('shows Export and Import buttons on same row when stats load', async () => {
		render(Page);
		await waitFor(() => {
			expect(screen.getByText('Data')).toBeTruthy();
		});
		expect(screen.getByText('Backup')).toBeTruthy();
		expect(screen.getByRole('button', { name: /Export/ })).toBeTruthy();
		expect(screen.getByRole('button', { name: /Import/ })).toBeTruthy();
	});

	it('export button navigates to export URL', async () => {
		// Mock window.location.href setter
		const hrefSpy = vi.fn();
		Object.defineProperty(window, 'location', {
			value: { ...window.location, href: '' },
			writable: true,
			configurable: true
		});
		Object.defineProperty(window.location, 'href', {
			set: hrefSpy,
			configurable: true
		});

		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Export/ })).toBeTruthy();
		});

		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: /Export/ }));
		expect(hrefSpy).toHaveBeenCalledWith('/api/data/export');
	});

	it('import button opens file picker', async () => {
		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Import/ })).toBeTruthy();
		});

		const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
		expect(fileInput).toBeTruthy();
		expect(fileInput.accept).toBe('.zip');
	});

	it('shows import confirmation dialog with file name', async () => {
		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Import/ })).toBeTruthy();
		});

		const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
		const file = new File(['zip'], 'test.zip', { type: 'application/zip' });
		Object.defineProperty(fileInput, 'files', { value: [file], configurable: true });
		fileInput.dispatchEvent(new Event('change', { bubbles: true }));

		await waitFor(() => {
			expect(screen.getByText(/test\.zip/)).toBeTruthy();
			expect(screen.getByText(/replaced/)).toBeTruthy();
		});
	});

	it('shows import error on failure', async () => {
		vi.spyOn(api, 'importData').mockRejectedValue(new Error('Version mismatch'));

		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Import/ })).toBeTruthy();
		});

		const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
		const file = new File(['zip'], 'test.zip', { type: 'application/zip' });
		Object.defineProperty(fileInput, 'files', { value: [file], configurable: true });
		fileInput.dispatchEvent(new Event('change', { bubbles: true }));

		// Confirm in dialog
		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Import' })).toBeTruthy();
		});
		const user = userEvent.setup();
		// There are two "Import" buttons (the toolbar one and the dialog one) — click the dialog one
		const importButtons = screen.getAllByRole('button', { name: 'Import' });
		await user.click(importButtons[importButtons.length - 1]);

		await waitFor(() => {
			expect(screen.getByText('Version mismatch')).toBeTruthy();
		});
	});

	it('shows success message after import', async () => {
		vi.spyOn(api, 'importData').mockResolvedValue({
			locations: 1,
			plants: 3,
			care_events: 5,
			photos: 2
		});

		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Import/ })).toBeTruthy();
		});

		const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
		const file = new File(['zip'], 'test.zip', { type: 'application/zip' });
		Object.defineProperty(fileInput, 'files', { value: [file], configurable: true });
		fileInput.dispatchEvent(new Event('change', { bubbles: true }));

		// Confirm in dialog
		await waitFor(() => {
			expect(screen.getByText(/test\.zip/)).toBeTruthy();
		});
		const user = userEvent.setup();
		const importButtons = screen.getAllByRole('button', { name: 'Import' });
		await user.click(importButtons[importButtons.length - 1]);

		await waitFor(() => {
			expect(screen.getByText(/Imported 3 plants/)).toBeTruthy();
		});
	});

	it('does not import when dialog is cancelled', async () => {
		const importSpy = vi.spyOn(api, 'importData');

		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Import/ })).toBeTruthy();
		});

		const fileInput = document.querySelector('input[type="file"]') as HTMLInputElement;
		const file = new File(['zip'], 'test.zip', { type: 'application/zip' });
		Object.defineProperty(fileInput, 'files', { value: [file], configurable: true });
		fileInput.dispatchEvent(new Event('change', { bubbles: true }));

		// Cancel in dialog
		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
		});
		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: 'Cancel' }));

		await new Promise((r) => setTimeout(r, 50));
		expect(importSpy).not.toHaveBeenCalled();
	});
});

describe('settings delete location confirmation', () => {
	function getDeleteButtons() {
		return document.querySelectorAll('.btn-danger') as NodeListOf<HTMLButtonElement>;
	}

	it('deletes immediately when location has no plants', async () => {
		locations.set([{ id: 1, name: 'Bedroom', plant_count: 0 }]);
		render(Page);

		const user = userEvent.setup();
		await user.click(getDeleteButtons()[0]);

		await waitFor(() => {
			expect(mockDeleteLocation).toHaveBeenCalledWith(1);
		});
	});

	it('shows confirmation dialog with location name when location has plants', async () => {
		locations.set([{ id: 1, name: 'Bedroom', plant_count: 3 }]);
		render(Page);

		const user = userEvent.setup();
		await user.click(getDeleteButtons()[0]);

		await waitFor(() => {
			expect(screen.getByText(/Delete "Bedroom"/)).toBeTruthy();
			expect(screen.getByText(/3 plants will lose their location/)).toBeTruthy();
		});
	});

	it('calls deleteLocation when confirmed', async () => {
		locations.set([{ id: 1, name: 'Bedroom', plant_count: 2 }]);
		render(Page);

		const user = userEvent.setup();
		await user.click(getDeleteButtons()[0]);

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Delete' })).toBeTruthy();
		});
		await user.click(screen.getByRole('button', { name: 'Delete' }));

		await waitFor(() => {
			expect(mockDeleteLocation).toHaveBeenCalledWith(1);
		});
	});

	it('does not delete when dialog is cancelled', async () => {
		locations.set([{ id: 1, name: 'Bedroom', plant_count: 1 }]);
		render(Page);

		const user = userEvent.setup();
		await user.click(getDeleteButtons()[0]);

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
		});
		await user.click(screen.getByRole('button', { name: 'Cancel' }));

		await new Promise((r) => setTimeout(r, 50));
		expect(mockDeleteLocation).not.toHaveBeenCalled();
	});
});

describe('settings MQTT repair confirmation', () => {
	beforeEach(() => {
		vi.spyOn(api, 'fetchStats').mockRejectedValue(new Error('skip'));
		vi.spyOn(api, 'fetchAppInfo').mockRejectedValue(new Error('skip'));
		vi.spyOn(api, 'fetchMqttStatus').mockResolvedValue({
			status: 'connected',
			broker: 'mqtt://localhost',
			topic_prefix: 'flowl'
		});
	});

	it('shows confirmation dialog when repair is clicked', async () => {
		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Repair/ })).toBeTruthy();
		});

		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: /Repair/ }));

		await waitFor(() => {
			expect(screen.getByText(/Clear all retained MQTT topics/)).toBeTruthy();
		});
	});

	it('calls repairMqtt when confirmed', async () => {
		vi.spyOn(api, 'repairMqtt').mockResolvedValue({ cleared: 5, published: 3 });

		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Repair/ })).toBeTruthy();
		});

		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: /Repair/ }));

		await waitFor(() => {
			expect(screen.getByText(/Clear all retained MQTT topics/)).toBeTruthy();
		});
		// Two "Repair" buttons: toolbar and dialog confirm — click the dialog one
		const repairButtons = screen.getAllByRole('button', { name: 'Repair' });
		await user.click(repairButtons[repairButtons.length - 1]);

		await waitFor(() => {
			expect(screen.getByText(/Cleared 5, published 3/)).toBeTruthy();
		});
	});

	it('does not repair when dialog is cancelled', async () => {
		const repairSpy = vi.spyOn(api, 'repairMqtt');

		render(Page);
		await waitFor(() => {
			expect(screen.getByRole('button', { name: /Repair/ })).toBeTruthy();
		});

		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: /Repair/ }));

		await waitFor(() => {
			expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
		});
		await user.click(screen.getByRole('button', { name: 'Cancel' }));

		await new Promise((r) => setTimeout(r, 50));
		expect(repairSpy).not.toHaveBeenCalled();
	});
});
