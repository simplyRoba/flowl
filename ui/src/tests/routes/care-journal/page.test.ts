import { cleanup, fireEvent, render } from '@testing-library/svelte';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import Page from '../../../routes/care-journal/+page.svelte';

// jsdom doesn't implement HTMLDialogElement.showModal/close
HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
	this.setAttribute('open', '');
});
HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
	this.removeAttribute('open');
});

const mockFetchAllCareEvents = vi.fn();

vi.mock('$lib/api', () => ({
	fetchAllCareEvents: (...args: any[]) => mockFetchAllCareEvents(...args)
}));

vi.mock('$app/navigation', () => ({
	goto: vi.fn()
}));

function makeEvent(overrides: Partial<any> = {}) {
	return {
		id: 1,
		plant_id: 1,
		plant_name: 'Fern',
		event_type: 'watered',
		notes: null,
		photo_url: null,
		occurred_at: '2025-02-01T10:00:00Z',
		created_at: '2025-02-01T10:00:00Z',
		...overrides
	};
}

beforeEach(() => {
	vi.clearAllMocks();
	mockFetchAllCareEvents.mockResolvedValue({ events: [], has_more: false });
});

afterEach(() => {
	cleanup();
});

describe('care journal thumbnails', () => {
	it('uses 200px thumbnail for event photo', async () => {
		mockFetchAllCareEvents.mockResolvedValue({
			events: [makeEvent({ id: 1, photo_url: '/uploads/care/1.jpg' })],
			has_more: false
		});
		render(Page);

		await vi.waitFor(() => {
			const img = document.querySelector('.log-entry-photo img') as HTMLImageElement;
			expect(img).toBeTruthy();
			expect(img.src).toContain('/uploads/care/1_200.jpg');
		});
	});

	it('falls back to original photo_url on thumbnail error', async () => {
		mockFetchAllCareEvents.mockResolvedValue({
			events: [makeEvent({ id: 2, photo_url: '/uploads/care/2.png' })],
			has_more: false
		});
		render(Page);

		await vi.waitFor(() => {
			expect(document.querySelector('.log-entry-photo img')).toBeTruthy();
		});
		const img = document.querySelector('.log-entry-photo img') as HTMLImageElement;
		expect(img.src).toContain('/uploads/care/2_200.jpg');
		await fireEvent.error(img);
		expect(img.src).toContain('/uploads/care/2.png');
		expect(img.src).not.toContain('_200');
	});

	it('opens lightbox with original photo_url when clicking thumbnail', async () => {
		mockFetchAllCareEvents.mockResolvedValue({
			events: [makeEvent({ id: 3, photo_url: '/uploads/care/3.jpg' })],
			has_more: false
		});
		render(Page);

		await vi.waitFor(() => {
			expect(document.querySelector('.log-entry-photo')).toBeTruthy();
		});
		const photoBtn = document.querySelector('.log-entry-photo') as HTMLButtonElement;
		await fireEvent.click(photoBtn);

		const lightbox = document.querySelector('dialog.lightbox') as HTMLDialogElement;
		expect(lightbox.hasAttribute('open')).toBe(true);
		const lightboxImg = lightbox.querySelector('img') as HTMLImageElement;
		expect(lightboxImg.src).toContain('/uploads/care/3.jpg');
		expect(lightboxImg.src).not.toContain('_200');
	});

	it('does not render photo element when event has no photo_url', async () => {
		mockFetchAllCareEvents.mockResolvedValue({
			events: [makeEvent({ id: 4, photo_url: null })],
			has_more: false
		});
		render(Page);

		await vi.waitFor(() => {
			expect(document.querySelector('.log-entry')).toBeTruthy();
		});
		expect(document.querySelector('.log-entry-photo')).toBeNull();
	});
});
