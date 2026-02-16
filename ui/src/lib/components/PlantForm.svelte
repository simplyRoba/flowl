<script lang="ts">
	import { onMount } from 'svelte';
	import { Sun, CloudSun, Cloud, Camera, X } from 'lucide-svelte';
	import type { Plant, CreatePlant, Location } from '$lib/api';
	import { locations, loadLocations, createLocation } from '$lib/stores/locations';
	import IconPicker from './IconPicker.svelte';
	import LocationChips from './LocationChips.svelte';
	import WateringInterval from './WateringInterval.svelte';

	let {
		initial = null,
		onsave,
		onremovephoto,
		saving = false,
		formId = 'plant-form',
		showFooterActions = true,
		showLocationNone = true
	}: {
		initial?: Plant | null;
		onsave: (data: CreatePlant, photo?: File) => void;
		onremovephoto?: () => void;
		saving?: boolean;
		formId?: string;
		showFooterActions?: boolean;
		showLocationNone?: boolean;
	} = $props();

	let name = $state('');
	let species = $state('');
	let icon = $state('');
	let locationId = $state<number | null>(null);
	let wateringDays = $state(7);
	let lightNeeds = $state('indirect');
	let notes = $state('');
	let nameError = $state('');

	let photoFile = $state<File | null>(null);
	let photoPreview = $state<string | null>(null);
	let isDraggingPhoto = $state(false);

	let mediaMode = $state<'both' | 'icon' | 'photo'>('both');
	let mediaTouched = $state(false);

	let hasPhoto = $derived(photoPreview !== null || (initial?.photo_url != null && photoFile === null));

	$effect(() => {
		if (!mediaTouched) {
			mediaMode = hasPhoto ? 'photo' : 'both';
		}
	});

	// Initialize/re-initialize from initial prop (for edit form)
	$effect(() => {
		if (initial) {
			name = initial.name;
			species = initial.species ?? '';
			icon = initial.icon ?? '';
			locationId = initial.location_id;
			wateringDays = initial.watering_interval_days;
			lightNeeds = initial.light_needs;
			notes = initial.notes ?? '';
		}
	});

	onMount(() => {
		loadLocations();
	});

	function handlePhotoSelect(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;
		setPhotoFile(file);
		mediaMode = 'photo';
		mediaTouched = true;
	}

	function setPhotoFile(file: File) {
		if (photoPreview) {
			URL.revokeObjectURL(photoPreview);
		}
		photoFile = file;
		photoPreview = URL.createObjectURL(file);
	}

	function handlePhotoDrop(e: DragEvent) {
		e.preventDefault();
		isDraggingPhoto = false;
		const file = e.dataTransfer?.files?.[0];
		if (!file) return;
		setPhotoFile(file);
		mediaMode = 'photo';
		mediaTouched = true;
	}

	function handleDragEnter(e: DragEvent) {
		e.preventDefault();
		isDraggingPhoto = true;
	}

	function handleDragLeave(e: DragEvent) {
		if (e.currentTarget === e.target) {
			isDraggingPhoto = false;
		}
	}

	function handleRemoveNewPhoto() {
		if (photoPreview) {
			URL.revokeObjectURL(photoPreview);
		}
		photoFile = null;
		photoPreview = null;
	}

	function handleRemoveExistingPhoto() {
		if (onremovephoto) {
			onremovephoto();
		}
	}

	function handleIconSelect(nextIcon: string) {
		icon = nextIcon;
		mediaMode = 'icon';
		mediaTouched = true;
		if (photoPreview) {
			URL.revokeObjectURL(photoPreview);
		}
		photoFile = null;
		photoPreview = null;
	}

	function switchToIcon() {
		mediaMode = 'icon';
		mediaTouched = true;
		if (photoPreview) {
			URL.revokeObjectURL(photoPreview);
		}
		photoFile = null;
		photoPreview = null;
	}

	function switchToPhoto() {
		mediaMode = 'photo';
		mediaTouched = true;
	}

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!name.trim()) {
			nameError = 'Name is required';
			return;
		}
		nameError = '';

		const data: CreatePlant = {
			name: name.trim()
		};
		if (mediaMode !== 'photo' && icon.trim()) data.icon = icon;
		if (species.trim()) data.species = species.trim();
		data.location_id = locationId;
		data.watering_interval_days = wateringDays;
		data.light_needs = lightNeeds;
		if (notes.trim()) data.notes = notes.trim();

		onsave(data, photoFile ?? undefined);
	}

	async function handleCreateLocation(locationName: string): Promise<Location | null> {
		return await createLocation(locationName);
	}
</script>

<form class="plant-form" id={formId} onsubmit={handleSubmit}>
	<section class="form-section media-section">
		<div class="media-header">
			<div class="form-section-title">Media</div>
		</div>
		<div class="media-stack">
			{#if mediaMode !== 'icon'}
				<div class="media-photo">
					{#if photoPreview}
						<div class="photo-preview">
							<img src={photoPreview} alt="Preview" class="preview-img" />
							<button type="button" class="photo-remove-btn" onclick={handleRemoveNewPhoto}>
								<X size={16} />
							</button>
						</div>
					{:else if initial?.photo_url}
						<div class="photo-preview">
							<img src={initial.photo_url} alt={initial.name} class="preview-img" />
							{#if onremovephoto}
								<button type="button" class="photo-remove-btn" onclick={handleRemoveExistingPhoto}>
									<X size={16} />
								</button>
							{/if}
						</div>
					{:else}
						<label
							class="photo-upload-refined"
							class:dragging={isDraggingPhoto}
							ondragenter={handleDragEnter}
							ondragover={handleDragEnter}
							ondragleave={handleDragLeave}
							ondrop={handlePhotoDrop}
						>
							<div class="upload-icon"><Camera size={24} /></div>
							<span>Add a photo</span>
							<span class="upload-hint">Click to select or drag & drop</span>
							<input
								type="file"
								accept="image/jpeg,image/png,image/webp"
								onchange={handlePhotoSelect}
								class="file-input"
							/>
						</label>
					{/if}
					<div class="media-actions">
						{#if !hasPhoto}
							<!-- No replace link when empty -->
						{:else if !photoPreview && initial?.photo_url}
							<label class="photo-replace">
								<Camera size={16} />
								<span>Replace photo</span>
								<input
									type="file"
									accept="image/jpeg,image/png,image/webp"
									onchange={handlePhotoSelect}
									class="file-input"
								/>
							</label>
						{/if}
						{#if mediaMode === 'photo'}
							<button type="button" class="media-switch" onclick={switchToIcon}>
								Use icon instead
							</button>
						{/if}
					</div>
				</div>
			{/if}
			{#if mediaMode === 'both'}
				<div class="media-divider"><span>or</span></div>
			{/if}
			{#if mediaMode !== 'photo'}
				<div class="media-icon">
					<IconPicker value={icon} onchange={handleIconSelect} />
					{#if mediaMode === 'icon'}
						<div class="media-actions">
							<button type="button" class="media-switch" onclick={switchToPhoto}>
								Use photo instead
							</button>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</section>

	<section class="form-section">
		<div class="form-section-title">Identity</div>
		<div class="form-group">
			<label class="form-label">Name *</label>
			<input
				type="text"
				bind:value={name}
				placeholder="e.g. Monstera Deliciosa"
				class="form-input"
				class:error={nameError}
				oninput={() => { nameError = ''; }}
			/>
			{#if nameError}
				<span class="field-error">{nameError}</span>
			{/if}
		</div>

		<div class="form-group">
			<label class="form-label">Species (optional)</label>
			<input
				type="text"
				bind:value={species}
				placeholder="e.g. Monstera"
				class="form-input"
			/>
		</div>

		<!-- Icon picker moved to Media section -->
	</section>

	<section class="form-section">
		<div class="form-section-title">Location</div>
		<LocationChips
			locations={$locations}
			value={locationId}
			onchange={(v) => { locationId = v; }}
			oncreate={handleCreateLocation}
			showNone={showLocationNone}
		/>
	</section>

	<section class="form-section">
		<div class="form-section-title">Watering</div>
		<WateringInterval value={wateringDays} onchange={(v) => { wateringDays = v; }} />
	</section>

	<section class="form-section">
		<div class="form-section-title">Light needs</div>
		<div class="light-selector">
			<button
				type="button"
				class="light-option"
				class:active={lightNeeds === 'direct'}
				onclick={() => { lightNeeds = 'direct'; }}
			>
				<span class="light-icon"><Sun size={20} /></span>
				<span>Direct</span>
				<span class="light-label">Full sun</span>
			</button>
			<button
				type="button"
				class="light-option"
				class:active={lightNeeds === 'indirect'}
				onclick={() => { lightNeeds = 'indirect'; }}
			>
				<span class="light-icon"><CloudSun size={20} /></span>
				<span>Indirect</span>
				<span class="light-label">Bright, filtered</span>
			</button>
			<button
				type="button"
				class="light-option"
				class:active={lightNeeds === 'low'}
				onclick={() => { lightNeeds = 'low'; }}
			>
				<span class="light-icon"><Cloud size={20} /></span>
				<span>Low</span>
				<span class="light-label">Shade tolerant</span>
			</button>
		</div>
	</section>

	<section class="form-section">
		<div class="form-section-title">Notes</div>
		<textarea
			bind:value={notes}
			placeholder="Care tips, observations, anything useful..."
			class="form-input textarea"
			rows="4"
		></textarea>
	</section>

	{#if showFooterActions}
		<button type="submit" class="save-btn" disabled={saving}>
			{saving ? 'Saving...' : 'Save'}
		</button>
	{/if}
</form>

<style>
	.plant-form {
		display: flex;
		flex-direction: column;
		gap: 16px;
		max-width: 640px;
		margin: 0 auto;
	}

	.form-section {
		background: #FFFFFF;
		border: 1px solid #E5DDD3;
		border-radius: 12px;
		padding: 16px;
	}

	.media-section {
		padding: 16px;
	}

	.media-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 12px;
	}

	.media-stack {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.media-icon,
	.media-photo {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 10px;
		width: 100%;
	}

	.media-actions {
		display: flex;
		flex-wrap: wrap;
		gap: 12px;
		width: 100%;
		max-width: 100%;
		margin-top: 12px;
		box-sizing: border-box;
		align-self: stretch;
	}

	.media-icon .form-label {
		margin-bottom: 4px;
	}

	.media-divider {
		display: flex;
		align-items: center;
		gap: 12px;
		color: #8C7E6E;
		font-size: 12px;
		text-transform: uppercase;
		letter-spacing: 0.6px;
	}

	.media-divider::before,
	.media-divider::after {
		content: '';
		height: 1px;
		background: #E5DDD3;
		flex: 1;
	}

	.media-switch {
		margin: 0;
		padding: 8px 12px;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		background: #FFFFFF;
		color: #8C7E6E;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
		white-space: nowrap;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		flex: 1 1 0;
		box-sizing: border-box;
		line-height: 1.2;
	}

	.media-switch:hover {
		border-color: #6B8F71;
		color: #6B8F71;
	}


	.form-section-title {
		font-size: 13px;
		font-weight: 600;
		color: #8C7E6E;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin: 0 0 16px;
	}

	.form-group {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 18px;
	}

	.form-group:last-child {
		margin-bottom: 0;
	}

	.form-label {
		font-size: 13px;
		font-weight: 600;
		color: #8C7E6E;
	}

	.required {
		color: #C45B5B;
	}

	.form-input {
		padding: 10px 12px;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		font-size: 15px;
		font-family: inherit;
		outline: none;
		transition: border-color 0.15s;
		background: #FFFFFF;
		color: #2C2418;
	}

	.form-input:focus {
		border-color: #6B8F71;
	}

	.form-input.error {
		border-color: #C45B5B;
	}

	.textarea {
		width: 100%;
		resize: vertical;
		min-height: 80px;
		box-sizing: border-box;
	}

	.field-error {
		font-size: 13px;
		color: #C45B5B;
	}

	.photo-upload-refined {
		width: 100%;
		height: 160px;
		border: 2px dashed #E5DDD3;
		border-radius: 12px;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		color: #8C7E6E;
		font-size: 14px;
		cursor: pointer;
		transition: all 0.15s;
		background: color-mix(in srgb, #6B8F71 3%, transparent);
		position: relative;
		overflow: hidden;
	}

	.photo-upload-refined.dragging {
		border-color: #6B8F71;
		background: color-mix(in srgb, #6B8F71 10%, transparent);
		color: #4A6B4F;
	}

	.media-photo .photo-upload-refined {
		max-width: 100%;
	}

	.photo-upload-refined:hover {
		border-color: #6B8F71;
		background: color-mix(in srgb, #6B8F71 8%, transparent);
	}

	.photo-upload-refined .upload-icon {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		background: color-mix(in srgb, #6B8F71 12%, transparent);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
	}

	.photo-upload-refined .upload-hint {
		font-size: 12px;
		color: #8C7E6E;
	}

	.file-input {
		display: none;
	}

	.photo-preview {
		position: relative;
		display: inline-flex;
		justify-content: center;
		align-items: center;
	}

	.preview-img {
		width: 180px;
		height: 180px;
		object-fit: cover;
		border-radius: 12px;
		border: 1px solid #E5DDD3;
	}

	.photo-remove-btn {
		position: absolute;
		top: -8px;
		right: -8px;
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		border: 1px solid #E5DDD3;
		background: #FFFFFF;
		color: #C45B5B;
		cursor: pointer;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.photo-remove-btn:hover {
		background: #fef2f2;
	}

	.photo-replace {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 8px 12px;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		cursor: pointer;
		color: #8C7E6E;
		font-size: 13px;
		font-weight: 500;
		transition: border-color 0.15s, color 0.15s;
		justify-content: center;
		flex: 1 1 0;
		box-sizing: border-box;
		line-height: 1.2;
	}

	.photo-replace:hover {
		border-color: #6B8F71;
		color: #6B8F71;
	}

	.light-selector {
		display: flex;
		gap: 8px;
	}

	.light-option {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 10px 8px;
		border: 1px solid #E5DDD3;
		border-radius: 10px;
		background: #FFFFFF;
		cursor: pointer;
		transition: all 0.15s;
		color: #2C2418;
		font-size: 13px;
	}

	.light-option:hover {
		border-color: #6B8F71;
	}

	.light-option.active {
		border-color: #6B8F71;
		background: color-mix(in srgb, #6B8F71 10%, transparent);
		color: #6B8F71;
	}

	.light-option .light-icon {
		font-size: 20px;
	}

	.light-option .light-label {
		font-size: 11px;
		color: #8C7E6E;
	}

	.light-option.active .light-label {
		color: #6B8F71;
	}

	.save-btn {
		padding: 12px 24px;
		background: #6B8F71;
		color: #fff;
		border: none;
		border-radius: 8px;
		font-size: 15px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
		align-self: flex-start;
	}

	.save-btn:hover:not(:disabled) {
		background: #4A6B4F;
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: default;
	}

	@media (max-width: 768px) {
		.plant-form {
			gap: 16px;
		}

		.form-section {
			padding: 16px;
		}

		.photo-section {
			margin-bottom: 8px;
		}

		.photo-upload-refined {
			height: 120px;
		}

		.preview-img {
			width: 150px;
			height: 150px;
		}

		.media-divider {
			font-size: 11px;
			gap: 8px;
		}

		.media-actions {
			max-width: 100%;
			margin-top: 16px;
			flex-direction: column;
		}

		.media-switch,
		.photo-replace {
			width: 100%;
			justify-content: center;
			padding: 10px 16px;
			font-size: 14px;
			flex: 1 1 auto;
		}

		.photo-upload-refined .upload-hint {
			display: none;
		}

		.light-selector {
			gap: 6px;
		}

		.light-option {
			padding: 10px 8px;
		}

		.light-option .light-label {
			display: none;
		}

		.save-btn {
			align-self: stretch;
			text-align: center;
		}
	}

</style>
