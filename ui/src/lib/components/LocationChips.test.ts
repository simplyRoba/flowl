import { cleanup, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import type { Location } from '$lib/api';
import LocationChips from './LocationChips.svelte';

afterEach(() => {
	cleanup();
});

const mockLocations: Location[] = [
	{ id: 1, name: 'Bedroom', plant_count: 2 },
	{ id: 2, name: 'Kitchen', plant_count: 1 }
];

describe('LocationChips', () => {
	it('renders location chips', () => {
		const onchange = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange }
		});
		expect(screen.getByText('Bedroom')).toBeTruthy();
		expect(screen.getByText('Kitchen')).toBeTruthy();
	});

	it('renders None chip when showNone is true (default)', () => {
		const onchange = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange }
		});
		expect(screen.getByText('None')).toBeTruthy();
	});

	it('hides None chip when showNone is false', () => {
		const onchange = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange, showNone: false }
		});
		expect(screen.queryByText('None')).toBeNull();
	});

	it('marks the selected location as active', () => {
		const onchange = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: 1, onchange }
		});
		const bedroomBtn = screen.getByText('Bedroom').closest('button')!;
		expect(bedroomBtn.classList.contains('active')).toBe(true);

		const kitchenBtn = screen.getByText('Kitchen').closest('button')!;
		expect(kitchenBtn.classList.contains('active')).toBe(false);
	});

	it('calls onchange with location id when chip is clicked', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange }
		});

		await user.click(screen.getByText('Kitchen'));
		expect(onchange).toHaveBeenCalledWith(2);
	});

	it('calls onchange with null when None chip is clicked', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: 1, onchange }
		});

		await user.click(screen.getByText('None'));
		expect(onchange).toHaveBeenCalledWith(null);
	});

	it('shows new location form when "+ New" is clicked', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		const oncreate = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange, oncreate }
		});

		await user.click(screen.getByText('+ New'));
		expect(screen.getByPlaceholderText('Location name')).toBeTruthy();
		expect(screen.getByText('Add')).toBeTruthy();
		expect(screen.getByText('Cancel')).toBeTruthy();
	});

	it('hides new location form on Cancel', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		const oncreate = vi.fn();
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange, oncreate }
		});

		await user.click(screen.getByText('+ New'));
		await user.click(screen.getByText('Cancel'));
		expect(screen.queryByPlaceholderText('Location name')).toBeNull();
	});

	it('calls oncreate and onchange when new location is submitted', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		const newLoc: Location = { id: 3, name: 'Balcony', plant_count: 0 };
		const oncreate = vi.fn().mockResolvedValue(newLoc);
		render(LocationChips, {
			props: { locations: mockLocations, value: null, onchange, oncreate }
		});

		await user.click(screen.getByText('+ New'));
		const input = screen.getByPlaceholderText('Location name');
		await user.type(input, 'Balcony');
		await user.click(screen.getByText('Add'));

		expect(oncreate).toHaveBeenCalledWith('Balcony');
		expect(onchange).toHaveBeenCalledWith(3);
	});
});
