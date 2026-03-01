<script lang="ts">
	import { Leaf, Shovel, Scissors, Pencil as PencilIcon, Camera, CalendarClock, X as XIcon } from 'lucide-svelte';
	import { addCareEvent } from '$lib/stores/care';
	import { uploadCareEventPhoto } from '$lib/api';
	import { translations } from '$lib/stores/locale';

	let { plantId, onsubmit, oncancel }: {
		plantId: number;
		onsubmit: () => void;
		oncancel: () => void;
	} = $props();

	let eventType = $state('');
	let notes = $state('');
	let photo = $state<File | null>(null);
	let photoPreview = $state<string | null>(null);
	let occurredAt = $state('');
	let showOccurredAt = $state(false);
	let submitting = $state(false);

	function nowLocalInputValue(): string {
		const now = new Date();
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
	}

	function stagePhoto(file: File) {
		const valid = ['image/jpeg', 'image/png', 'image/webp'];
		if (!valid.includes(file.type)) return;
		if (photoPreview) URL.revokeObjectURL(photoPreview);
		photo = file;
		photoPreview = URL.createObjectURL(file);
	}

	function clearPhoto() {
		if (photoPreview) URL.revokeObjectURL(photoPreview);
		photo = null;
		photoPreview = null;
	}

	function handlePhotoSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (file) stagePhoto(file);
		input.value = '';
	}

	function resetForm() {
		clearPhoto();
		eventType = '';
		notes = '';
		occurredAt = '';
		showOccurredAt = false;
	}

	function handleCancel() {
		resetForm();
		oncancel();
	}

	async function handleSubmit() {
		if (!eventType || submitting) return;
		submitting = true;
		const occ = showOccurredAt ? occurredAt.trim() : '';
		const occDate = occ ? new Date(occ) : null;
		const occIso = occDate && !isNaN(occDate.getTime()) ? occDate.toISOString() : undefined;
		const photoFile = photo;
		const event = await addCareEvent(plantId, {
			event_type: eventType,
			notes: notes.trim() || undefined,
			occurred_at: occIso
		});
		if (event && photoFile) {
			await uploadCareEventPhoto(plantId, event.id, photoFile);
		}
		resetForm();
		submitting = false;
		onsubmit();
	}
</script>

<div class="care-entry-form">
	<div class="type-chips">
		{#each [
			{ value: 'fertilized', label: $translations.care.fertilized, icon: Leaf },
			{ value: 'repotted', label: $translations.care.repotted, icon: Shovel },
			{ value: 'pruned', label: $translations.care.pruned, icon: Scissors },
			{ value: 'custom', label: $translations.care.custom, icon: PencilIcon }
		] as chip}
			<button
				class="chip chip-solid"
				class:active={eventType === chip.value}
				onclick={() => eventType = chip.value}
			>
				<chip.icon size={14} />
				{chip.label}
			</button>
		{/each}
	</div>

	<textarea
		class="input log-notes"
		placeholder={$translations.plant.notesOptional}
		bind:value={notes}
		rows="2"
	></textarea>

	<div class="toolbar">
		<div class="toolbar-left">
			{#if photoPreview}
				<div class="toolbar-compound">
					<div class="toolbar-thumb">
						<img src={photoPreview} alt="" />
					</div>
					<button class="toolbar-dismiss" onclick={clearPhoto} aria-label={$translations.chat.removePhoto}>
						<XIcon size={12} />
					</button>
				</div>
			{:else}
				<label class="toolbar-btn" aria-label={$translations.plant.addLogPhoto}>
					<Camera size={16} />
					<input
						type="file"
						accept="image/jpeg,image/png,image/webp"
						onchange={handlePhotoSelect}
						class="file-input-hidden"
					/>
				</label>
			{/if}

			{#if showOccurredAt}
				<div class="toolbar-compound">
					<input
						class="toolbar-date-input"
						type="datetime-local"
						max={nowLocalInputValue()}
						bind:value={occurredAt}
					/>
					<button class="toolbar-dismiss" onclick={() => { showOccurredAt = false; occurredAt = ''; }} aria-label={$translations.common.cancel}>
						<XIcon size={12} />
					</button>
				</div>
			{:else}
				<button
					class="toolbar-btn"
					onclick={() => { showOccurredAt = true; if (!occurredAt) occurredAt = nowLocalInputValue(); }}
				>
					<CalendarClock size={16} />
				</button>
			{/if}
		</div>

		<div class="toolbar-right">
			<button class="btn btn-outline" onclick={handleCancel}>{$translations.common.cancel}</button>
			<button class="btn btn-primary" onclick={handleSubmit} disabled={!eventType || submitting}>
				{submitting ? $translations.common.saving : $translations.common.save}
			</button>
		</div>
	</div>
</div>

<style>
	.care-entry-form {
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.type-chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}

	.log-notes {
		width: 100%;
		resize: vertical;
		margin-bottom: 10px;
	}

	.toolbar {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		align-items: center;
	}

	.toolbar-left {
		display: flex;
		gap: 6px;
		align-items: center;
		flex-shrink: 0;
	}

	.toolbar-right {
		display: flex;
		gap: 6px;
		align-items: center;
		margin-left: auto;
	}

	.toolbar-btn {
		box-sizing: border-box;
		width: 36px;
		height: 36px;
		padding: 0;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-btn);
		background: var(--color-surface);
		color: var(--color-text-muted);
		transition: background var(--transition-speed), border-color var(--transition-speed), color var(--transition-speed);
	}

	.toolbar-btn:hover {
		background: var(--color-primary-tint);
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.toolbar-compound {
		box-sizing: border-box;
		display: inline-flex;
		align-items: center;
		border: 1px solid var(--color-primary);
		border-radius: var(--radius-btn);
		overflow: hidden;
		height: 36px;
	}

	.toolbar-thumb {
		width: 36px;
		height: 34px;
		overflow: hidden;
	}

	.toolbar-thumb img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.toolbar-date-input {
		border: none;
		background: none;
		padding: 2px 8px;
		font-size: 13px;
		height: 34px;
		width: 165px;
		color: var(--color-text);
		font-family: inherit;
	}

	.toolbar-dismiss {
		width: 34px;
		height: 34px;
		border: none;
		border-left: 1px solid var(--color-border-subtle);
		background: none;
		color: var(--color-text-muted);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		padding: 0;
		transition: color var(--transition-speed), background var(--transition-speed);
	}

	.toolbar-dismiss:hover {
		color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
	}

	.file-input-hidden {
		display: none;
	}

	@media (max-width: 768px) {
		.toolbar-btn {
			width: 44px;
			height: 44px;
		}

		.toolbar-compound {
			height: 44px;
		}

		.toolbar-thumb {
			width: 44px;
			height: 42px;
		}

		.toolbar-date-input {
			height: 42px;
		}

		.toolbar-dismiss {
			width: 42px;
			height: 42px;
		}
	}
</style>
