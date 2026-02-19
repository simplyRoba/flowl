import { cleanup, render, screen } from '@testing-library/svelte';
import { afterEach, describe, expect, it, vi } from 'vitest';
import StatusBadge from './StatusBadge.svelte';

afterEach(() => {
	cleanup();
});

describe('StatusBadge', () => {
	it('renders "Ok" for ok status', () => {
		render(StatusBadge, { props: { status: 'ok', nextDue: null } });
		expect(screen.getByText(/Ok/)).toBeTruthy();
	});

	it('renders "Due" for due status', () => {
		render(StatusBadge, { props: { status: 'due', nextDue: null } });
		expect(screen.getByText(/Due/)).toBeTruthy();
	});

	it('renders "Overdue" for overdue status', () => {
		render(StatusBadge, { props: { status: 'overdue', nextDue: null } });
		expect(screen.getByText(/Overdue/)).toBeTruthy();
	});

	it('shows "today" suffix when next due is today', () => {
		const today = new Date();
		const iso = today.toISOString();
		render(StatusBadge, { props: { status: 'due', nextDue: iso } });
		expect(screen.getByText(/today/)).toBeTruthy();
	});

	it('shows "in N days" for future dates', () => {
		const future = new Date();
		future.setDate(future.getDate() + 3);
		const iso = future.toISOString();
		render(StatusBadge, { props: { status: 'ok', nextDue: iso } });
		expect(screen.getByText(/in 3 days/)).toBeTruthy();
	});

	it('shows "in 1 day" for tomorrow', () => {
		const tomorrow = new Date();
		tomorrow.setDate(tomorrow.getDate() + 1);
		const iso = tomorrow.toISOString();
		render(StatusBadge, { props: { status: 'ok', nextDue: iso } });
		expect(screen.getByText(/in 1 day$/)).toBeTruthy();
	});

	it('shows "N days" for past dates', () => {
		const past = new Date();
		past.setDate(past.getDate() - 2);
		const iso = past.toISOString();
		render(StatusBadge, { props: { status: 'overdue', nextDue: iso } });
		expect(screen.getByText(/2 days/)).toBeTruthy();
	});

	it('shows "1 day" for yesterday', () => {
		const yesterday = new Date();
		yesterday.setDate(yesterday.getDate() - 1);
		const iso = yesterday.toISOString();
		render(StatusBadge, { props: { status: 'overdue', nextDue: iso } });
		expect(screen.getByText(/1 day/)).toBeTruthy();
	});

	it('shows no suffix when nextDue is null', () => {
		render(StatusBadge, { props: { status: 'ok', nextDue: null } });
		const badge = screen.getByText(/Ok/);
		expect(badge.textContent).not.toContain('day');
		expect(badge.textContent).not.toContain('today');
	});

	it('shows no suffix for invalid date', () => {
		render(StatusBadge, { props: { status: 'ok', nextDue: 'not-a-date' } });
		const badge = screen.getByText(/Ok/);
		expect(badge.textContent).not.toContain('day');
	});
});
