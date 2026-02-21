import { cleanup, render, screen } from '@testing-library/svelte';
import userEvent from '@testing-library/user-event';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import ModalDialog from './ModalDialog.svelte';

// jsdom doesn't implement HTMLDialogElement.showModal/close — stub them
beforeEach(() => {
	HTMLDialogElement.prototype.showModal = vi.fn(function (this: HTMLDialogElement) {
		this.setAttribute('open', '');
	});
	HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
		this.removeAttribute('open');
	});
});

afterEach(() => {
	cleanup();
});

describe('ModalDialog confirm mode', () => {
	it('renders cancel and confirm buttons', () => {
		render(ModalDialog, {
			props: { open: true, title: 'Delete?', message: 'Are you sure?', mode: 'confirm', confirmLabel: 'Delete' }
		});
		expect(screen.getByText('Delete?')).toBeTruthy();
		expect(screen.getByText('Are you sure?')).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Cancel' })).toBeTruthy();
		expect(screen.getByRole('button', { name: 'Delete' })).toBeTruthy();
	});

	it('fires onconfirm when confirm button clicked', async () => {
		const onconfirm = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'confirm', confirmLabel: 'Yes', onconfirm }
		});
		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: 'Yes' }));
		expect(onconfirm).toHaveBeenCalledOnce();
	});

	it('fires oncancel when cancel button clicked', async () => {
		const oncancel = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'confirm', oncancel }
		});
		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: 'Cancel' }));
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('fires oncancel on Escape key (cancel event)', () => {
		const oncancel = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'confirm', oncancel }
		});
		const dialog = document.querySelector('dialog')!;
		dialog.dispatchEvent(new Event('cancel'));
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('fires oncancel on backdrop click', async () => {
		const oncancel = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'confirm', oncancel }
		});
		const dialog = document.querySelector('dialog')!;
		// Click directly on dialog element (backdrop area)
		await userEvent.setup().click(dialog);
		expect(oncancel).toHaveBeenCalledOnce();
	});

	it('uses danger styling when variant is danger', () => {
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'confirm', variant: 'danger', confirmLabel: 'Delete' }
		});
		const confirmBtn = screen.getByRole('button', { name: 'Delete' });
		expect(confirmBtn.className).toContain('btn-danger-fill');
	});

	it('uses primary styling when variant is warning', () => {
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'confirm', variant: 'warning', confirmLabel: 'OK' }
		});
		const confirmBtn = screen.getByRole('button', { name: 'OK' });
		expect(confirmBtn.className).toContain('btn-primary');
	});
});

describe('ModalDialog alert mode', () => {
	it('renders single OK button', () => {
		render(ModalDialog, {
			props: { open: true, title: 'Error', message: 'Something failed', mode: 'alert' }
		});
		expect(screen.getByText('Error')).toBeTruthy();
		expect(screen.getByText('Something failed')).toBeTruthy();
		const buttons = screen.getAllByRole('button');
		expect(buttons).toHaveLength(1);
		expect(buttons[0].textContent?.trim()).toBe('OK');
	});

	it('fires onclose when OK button clicked', async () => {
		const onclose = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'alert', onclose }
		});
		const user = userEvent.setup();
		await user.click(screen.getByRole('button', { name: 'OK' }));
		expect(onclose).toHaveBeenCalledOnce();
	});

	it('fires onclose on Escape key (cancel event)', () => {
		const onclose = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'alert', onclose }
		});
		const dialog = document.querySelector('dialog')!;
		dialog.dispatchEvent(new Event('cancel'));
		expect(onclose).toHaveBeenCalledOnce();
	});

	it('does not close on backdrop click', async () => {
		const onclose = vi.fn();
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'alert', onclose }
		});
		const dialog = document.querySelector('dialog')!;
		await userEvent.setup().click(dialog);
		expect(onclose).not.toHaveBeenCalled();
	});

	it('uses danger styling for alert with danger variant', () => {
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M', mode: 'alert', variant: 'danger' }
		});
		const okBtn = screen.getByRole('button', { name: 'OK' });
		expect(okBtn.className).toContain('btn-danger-fill');
	});
});

describe('ModalDialog open prop', () => {
	it('calls showModal when open becomes true', () => {
		render(ModalDialog, {
			props: { open: true, title: 'T', message: 'M' }
		});
		const dialog = document.querySelector('dialog')!;
		expect(dialog.showModal).toHaveBeenCalled();
	});

	it('does not call showModal when open is false', () => {
		render(ModalDialog, {
			props: { open: false, title: 'T', message: 'M' }
		});
		const dialog = document.querySelector('dialog')!;
		expect(dialog.showModal).not.toHaveBeenCalled();
	});
});
