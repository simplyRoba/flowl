<script lang="ts">
	import { translations } from '$lib/stores/locale';

	let {
		open = false,
		title = '',
		message = '',
		mode = 'confirm',
		variant = 'warning',
		confirmLabel = '',
		onconfirm,
		oncancel,
		onclose
	}: {
		open?: boolean;
		title?: string;
		message?: string;
		mode?: 'confirm' | 'alert';
		variant?: 'danger' | 'warning';
		confirmLabel?: string;
		onconfirm?: () => void;
		oncancel?: () => void;
		onclose?: () => void;
	} = $props();

	let dialogEl: HTMLDialogElement | undefined = $state();

	$effect(() => {
		if (!dialogEl) return;
		if (open && !dialogEl.open) {
			dialogEl.showModal();
		} else if (!open && dialogEl.open) {
			dialogEl.close();
		}
	});

	function handleCancel(e: Event) {
		e.preventDefault();
		if (mode === 'confirm') {
			oncancel?.();
		} else {
			onclose?.();
		}
	}

	function handleClick(e: MouseEvent) {
		if (mode === 'alert') return;
		if (e.target === dialogEl) {
			oncancel?.();
		}
	}

	function handleConfirm() {
		onconfirm?.();
	}

	function handleAlertClose() {
		onclose?.();
	}
</script>

<dialog
	bind:this={dialogEl}
	class="modal-dialog"
	oncancel={handleCancel}
	onclick={handleClick}
>
	<div class="modal-content">
		<h3 class="modal-title">{title}</h3>
		<p class="modal-message">{message}</p>
		<div class="modal-actions">
			{#if mode === 'confirm'}
				<button type="button" class="btn btn-outline" onclick={oncancel}>{$translations.common.cancel}</button>
				<button
					type="button"
					class="btn {variant === 'danger' ? 'btn-danger-fill' : 'btn-primary'}"
					onclick={handleConfirm}
				>
					{confirmLabel || $translations.common.confirm}
				</button>
			{:else}
				<button
					type="button"
					class="btn {variant === 'danger' ? 'btn-danger-fill' : 'btn-primary'}"
					onclick={handleAlertClose}
				>
					{$translations.common.ok}
				</button>
			{/if}
		</div>
	</div>
</dialog>

<style>
	.modal-dialog {
		border: none;
		border-radius: var(--radius-card);
		background: var(--color-surface);
		color: var(--color-text);
		padding: 0;
		max-width: 400px;
		width: calc(100vw - 32px);
		box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
	}

	.modal-dialog::backdrop {
		background: color-mix(in srgb, var(--color-background) 70%, transparent);
	}

	.modal-content {
		padding: 24px;
	}

	.modal-title {
		margin: 0 0 8px;
		font-size: 17px;
		font-weight: 600;
	}

	.modal-message {
		margin: 0 0 20px;
		font-size: 15px;
		color: var(--color-text-muted);
		line-height: 1.5;
	}

	.modal-actions {
		display: flex;
		justify-content: flex-end;
		gap: 8px;
	}

	.btn-danger-fill {
		background: var(--color-danger);
		color: #fff;
	}

	.btn-danger-fill:hover:not(:disabled) {
		background: color-mix(in srgb, var(--color-danger) 85%, black);
	}
</style>
