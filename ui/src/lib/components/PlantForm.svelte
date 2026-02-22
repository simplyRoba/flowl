<script lang="ts">
	import { onMount } from 'svelte';
	import { Sun, CloudSun, Cloud, Camera, X, Gauge, PawPrint, TrendingUp, Layers, Droplets } from 'lucide-svelte';
	import type { Plant, CreatePlant, Location } from '$lib/api';
	import { locations, loadLocations, createLocation } from '$lib/stores/locations';
	import { translations } from '$lib/stores/locale';
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
	let difficulty = $state<string | null>(null);
	let petSafety = $state<string | null>(null);
	let growthSpeed = $state<string | null>(null);
	let soilType = $state<string | null>(null);
	let soilMoisture = $state<string | null>(null);
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
			difficulty = initial.difficulty;
			petSafety = initial.pet_safety;
			growthSpeed = initial.growth_speed;
			soilType = initial.soil_type;
			soilMoisture = initial.soil_moisture;
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
			nameError = $translations.form.nameRequired;
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
		data.difficulty = difficulty;
		data.pet_safety = petSafety;
		data.growth_speed = growthSpeed;
		data.soil_type = soilType;
		data.soil_moisture = soilMoisture;
		if (notes.trim()) data.notes = notes.trim();

		onsave(data, photoFile ?? undefined);
	}

	async function handleCreateLocation(locationName: string): Promise<Location | null> {
		return await createLocation(locationName);
	}
</script>

<form class="plant-form" id={formId} onsubmit={handleSubmit}>
	<section class="section media-section">
		<div class="media-header">
			<div class="section-title">{$translations.form.media}</div>
		</div>
		<div class="media-stack">
			{#if mediaMode !== 'icon'}
				<div class="media-photo">
					{#if photoPreview}
						<div class="photo-preview">
							<img src={photoPreview} alt={$translations.form.photoPreview} class="preview-img" />
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
							<span>{$translations.form.addPhoto}</span>
							<span class="upload-hint">{$translations.form.clickOrDrag}</span>
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
							<label class="btn btn-outline photo-replace">
								<Camera size={16} />
								<span>{$translations.form.replacePhoto}</span>
								<input
									type="file"
									accept="image/jpeg,image/png,image/webp"
									onchange={handlePhotoSelect}
									class="file-input"
								/>
							</label>
						{/if}
						{#if mediaMode === 'photo'}
							<button type="button" class="btn btn-outline media-switch" onclick={switchToIcon}>
								{$translations.form.useIcon}
							</button>
						{/if}
					</div>
				</div>
			{/if}
			{#if mediaMode === 'both'}
				<div class="media-divider"><span>{$translations.common.or}</span></div>
			{/if}
			{#if mediaMode !== 'photo'}
				<div class="media-icon">
					<IconPicker value={icon} onchange={handleIconSelect} />
					{#if mediaMode === 'icon'}
						<div class="media-actions">
							<button type="button" class="btn btn-outline media-switch" onclick={switchToPhoto}>
								{$translations.form.usePhoto}
							</button>
						</div>
					{/if}
				</div>
			{/if}
		</div>
	</section>

	<section class="section">
		<div class="section-title">{$translations.form.identity}</div>
		<div class="form-group">
			<label class="form-label" for="plant-name">{$translations.form.nameLabel}</label>
			<input
				id="plant-name"
				type="text"
				bind:value={name}
				placeholder={$translations.form.namePlaceholder}
				class="input"
				class:input-error={nameError}
				oninput={() => { nameError = ''; }}
			/>
			{#if nameError}
				<span class="field-error">{nameError}</span>
			{/if}
		</div>

		<div class="form-group">
			<label class="form-label" for="plant-species">{$translations.form.speciesLabel}</label>
			<input
				id="plant-species"
				type="text"
				bind:value={species}
				placeholder={$translations.form.speciesPlaceholder}
				class="input"
			/>
		</div>

		<!-- Icon picker moved to Media section -->
	</section>

	<section class="section">
		<div class="section-title">{$translations.form.location}</div>
		<LocationChips
			locations={$locations}
			value={locationId}
			onchange={(v) => { locationId = v; }}
			oncreate={handleCreateLocation}
			showNone={showLocationNone}
		/>
	</section>

	<section class="section">
		<div class="section-title">{$translations.form.watering}</div>
		<WateringInterval value={wateringDays} onchange={(v) => { wateringDays = v; }} />
	</section>

	<section class="section">
		<div class="section-title">{$translations.form.lightNeeds}</div>
		<div class="light-selector">
			<button
				type="button"
				class="light-option"
				class:active={lightNeeds === 'direct'}
				onclick={() => { lightNeeds = 'direct'; }}
			>
				<span class="light-icon"><Sun size={20} /></span>
				<span>{$translations.form.direct}</span>
				<span class="light-label">{$translations.form.fullSun}</span>
			</button>
			<button
				type="button"
				class="light-option"
				class:active={lightNeeds === 'indirect'}
				onclick={() => { lightNeeds = 'indirect'; }}
			>
				<span class="light-icon"><CloudSun size={20} /></span>
				<span>{$translations.form.indirect}</span>
				<span class="light-label">{$translations.form.brightFiltered}</span>
			</button>
			<button
				type="button"
				class="light-option"
				class:active={lightNeeds === 'low'}
				onclick={() => { lightNeeds = 'low'; }}
			>
				<span class="light-icon"><Cloud size={20} /></span>
				<span>{$translations.form.low}</span>
				<span class="light-label">{$translations.form.shadeTolerant}</span>
			</button>
		</div>
	</section>

	<section class="section">
		<div class="section-title">{$translations.form.careInfo} <span class="section-optional">{$translations.form.optional}</span></div>

		<div class="care-info-group">
			<span class="care-info-label"><Gauge size={14} /> {$translations.form.difficulty}</span>
			<div class="light-selector">
				{#each [
					{ value: 'easy', label: $translations.form.easy },
					{ value: 'moderate', label: $translations.form.moderate },
					{ value: 'demanding', label: $translations.form.demanding }
				] as opt}
					<button
						type="button"
						class="btn btn-outline care-option"
						class:active={difficulty === opt.value}
						onclick={() => { difficulty = difficulty === opt.value ? null : opt.value; }}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>

		<div class="care-info-group">
			<span class="care-info-label"><PawPrint size={14} /> {$translations.form.petSafety}</span>
			<div class="light-selector">
				{#each [
					{ value: 'safe', label: $translations.form.safe },
					{ value: 'caution', label: $translations.form.caution },
					{ value: 'toxic', label: $translations.form.toxic }
				] as opt}
					<button
						type="button"
						class="btn btn-outline care-option"
						class:active={petSafety === opt.value}
						onclick={() => { petSafety = petSafety === opt.value ? null : opt.value; }}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>

		<div class="care-info-group">
			<span class="care-info-label"><TrendingUp size={14} /> {$translations.form.growthSpeed}</span>
			<div class="light-selector">
				{#each [
					{ value: 'slow', label: $translations.form.slow },
					{ value: 'moderate', label: $translations.form.moderate },
					{ value: 'fast', label: $translations.form.fast }
				] as opt}
					<button
						type="button"
						class="btn btn-outline care-option"
						class:active={growthSpeed === opt.value}
						onclick={() => { growthSpeed = growthSpeed === opt.value ? null : opt.value; }}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>

		<div class="care-info-group">
			<span class="care-info-label"><Layers size={14} /> {$translations.form.soilType}</span>
			<div class="light-selector">
				{#each [
					{ value: 'standard', label: $translations.form.standard },
					{ value: 'cactus-mix', label: $translations.form.cactusMix },
					{ value: 'orchid-bark', label: $translations.form.orchidBark },
					{ value: 'peat-moss', label: $translations.form.peatMoss }
				] as opt}
					<button
						type="button"
						class="btn btn-outline care-option"
						class:active={soilType === opt.value}
						onclick={() => { soilType = soilType === opt.value ? null : opt.value; }}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>

		<div class="care-info-group">
			<span class="care-info-label"><Droplets size={14} /> {$translations.form.soilMoisture}</span>
			<div class="light-selector">
				{#each [
					{ value: 'dry', label: $translations.form.dry },
					{ value: 'moderate', label: $translations.form.moderate },
					{ value: 'moist', label: $translations.form.moist }
				] as opt}
					<button
						type="button"
						class="btn btn-outline care-option"
						class:active={soilMoisture === opt.value}
						onclick={() => { soilMoisture = soilMoisture === opt.value ? null : opt.value; }}
					>
						{opt.label}
					</button>
				{/each}
			</div>
		</div>
	</section>

	<section class="section">
		<div class="section-title">{$translations.form.notes}</div>
		<textarea
			bind:value={notes}
			placeholder={$translations.form.notesPlaceholder}
			class="input textarea"
			rows="4"
		></textarea>
	</section>

	{#if showFooterActions}
		<button type="submit" class="btn btn-primary save-btn" disabled={saving}>
			{saving ? $translations.common.saving : $translations.common.save}
		</button>
	{/if}
</form>

<style>
	.plant-form {
		display: flex;
		flex-direction: column;
		gap: 16px;
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

	.media-divider {
		display: flex;
		align-items: center;
		gap: 12px;
		color: var(--color-text-muted);
		font-size: 12px;
		text-transform: uppercase;
		letter-spacing: 0.6px;
	}

	.media-divider::before,
	.media-divider::after {
		content: '';
		height: 1px;
		background: var(--color-border);
		flex: 1;
	}

	.media-switch {
		flex: 1 1 0;
		white-space: nowrap;
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
		font-size: var(--fs-chip);
		font-weight: 600;
		color: var(--color-text-muted);
	}

	.textarea {
		width: 100%;
		resize: vertical;
		min-height: 80px;
		box-sizing: border-box;
	}

	.field-error {
		font-size: var(--fs-chip);
		color: var(--color-danger);
	}

	.photo-upload-refined {
		width: 100%;
		height: 220px;
		border: 2px dashed var(--color-border);
		border-radius: var(--radius-card);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 8px;
		color: var(--color-text-muted);
		font-size: 14px;
		cursor: pointer;
		transition: all var(--transition-speed);
		background: color-mix(in srgb, var(--color-primary) 3%, transparent);
		position: relative;
		overflow: hidden;
	}

	.photo-upload-refined.dragging {
		border-color: var(--color-primary);
		background: var(--color-primary-tint);
		color: var(--color-primary-dark);
	}

	.media-photo .photo-upload-refined {
		max-width: 100%;
	}

	.photo-upload-refined:hover {
		border-color: var(--color-primary);
		background: color-mix(in srgb, var(--color-primary) 8%, transparent);
	}

	.photo-upload-refined .upload-icon {
		width: 48px;
		height: 48px;
		border-radius: 50%;
		background: color-mix(in srgb, var(--color-primary) 12%, transparent);
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 22px;
	}

	.photo-upload-refined .upload-hint {
		font-size: 12px;
		color: var(--color-text-muted);
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
		width: 240px;
		height: 240px;
		object-fit: cover;
		border-radius: var(--radius-card);
		border: 1px solid var(--color-border);
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
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-danger);
		cursor: pointer;
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.photo-remove-btn:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	.photo-replace {
		flex: 1 1 0;
		white-space: nowrap;
	}

	.section-optional {
		font-weight: 400;
		color: var(--color-text-muted);
		font-size: 10px;
		text-transform: none;
		letter-spacing: 0;
	}

	.care-info-group {
		margin-bottom: 16px;
	}

	.care-info-group:last-child {
		margin-bottom: 0;
	}

	.care-info-label {
		display: inline-flex;
		align-items: center;
		gap: 5px;
		font-size: var(--fs-chip);
		font-weight: 600;
		color: var(--color-text-muted);
		margin-bottom: 8px;
	}

	.care-option {
		flex: 1;
		padding: 8px 10px;
		border-radius: 10px;
		font-size: var(--fs-chip);
	}

	.light-selector {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.light-option {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 10px 8px;
		border: 1px solid var(--color-border);
		border-radius: 10px;
		background: var(--color-surface);
		cursor: pointer;
		transition: all var(--transition-speed);
		color: var(--color-text);
		font-size: 13px;
	}

	.light-option:hover {
		border-color: var(--color-primary);
	}

	.light-option.active {
		border-color: var(--color-primary);
		background: var(--color-primary-tint);
		color: var(--color-primary);
	}

	.light-option .light-icon {
		font-size: 20px;
	}

	.light-option .light-label {
		font-size: 11px;
		color: var(--color-text-muted);
	}

	.light-option.active .light-label {
		color: var(--color-primary);
	}

	.save-btn {
		align-self: flex-start;
	}

	@media (max-width: 768px) {
		.plant-form {
			gap: 16px;
		}

		.photo-upload-refined {
			height: 180px;
		}

		.preview-img {
			width: 200px;
			height: 200px;
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
		}
	}

</style>
