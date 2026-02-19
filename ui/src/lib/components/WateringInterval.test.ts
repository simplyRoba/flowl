import { cleanup, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, describe, expect, it, vi } from 'vitest';
import WateringInterval from './WateringInterval.svelte';

afterEach(() => {
	cleanup();
});

describe('WateringInterval', () => {
	it('renders preset buttons', () => {
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 7, onchange } });
		expect(screen.getByText('Thirsty')).toBeTruthy();
		expect(screen.getByText('Weekly')).toBeTruthy();
		expect(screen.getByText('Biweekly')).toBeTruthy();
		expect(screen.getByText('Monthly')).toBeTruthy();
	});

	it('calls onchange when preset is clicked', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 7, onchange } });

		const biweekly = screen.getByText('Biweekly').closest('button')!;
		await user.click(biweekly);
		expect(onchange).toHaveBeenCalledWith(14);
	});

	it('marks active preset', () => {
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 7, onchange } });
		const weekly = screen.getByText('Weekly').closest('button')!;
		expect(weekly.classList.contains('active')).toBe(true);
	});

	it('decrements value on minus button click', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 5, onchange } });

		const stepperBtns = Array.from(document.querySelectorAll('.stepper-btn'));
		await user.click(stepperBtns[0] as HTMLElement);
		expect(onchange).toHaveBeenCalledWith(4);
	});

	it('increments value on plus button click', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 5, onchange } });

		const stepperBtns = Array.from(document.querySelectorAll('.stepper-btn'));
		await user.click(stepperBtns[1] as HTMLElement);
		expect(onchange).toHaveBeenCalledWith(6);
	});

	it('disables decrement at minimum value of 1', () => {
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 1, onchange } });
		const stepperBtns = Array.from(document.querySelectorAll('.stepper-btn'));
		expect((stepperBtns[0] as HTMLButtonElement).disabled).toBe(true);
	});

	it('does not decrement below 1', async () => {
		const user = userEvent.setup();
		const onchange = vi.fn();
		render(WateringInterval, { props: { value: 1, onchange } });

		const stepperBtns = Array.from(document.querySelectorAll('.stepper-btn'));
		await user.click(stepperBtns[0] as HTMLElement);
		expect(onchange).not.toHaveBeenCalled();
	});
});
