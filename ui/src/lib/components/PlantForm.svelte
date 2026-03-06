<script lang="ts">
	import { onMount } from 'svelte';
	import { Sun, CloudSun, Cloud, Camera, X, Gauge, PawPrint, TrendingUp, Layers, Droplets, Sparkles, Check, TriangleAlert, ChevronLeft, ChevronRight } from 'lucide-svelte';
	import type { Plant, CreatePlant, Location, IdentifyResult } from '$lib/api';
	import { identifyPlant } from '$lib/api';
	import { aiStatus, loadAiStatus } from '$lib/stores/ai';
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

	// AI Identify state
	let aiEnabled = $derived($aiStatus?.enabled ?? false);
	let identifyState = $state<'idle' | 'loading' | 'result' | 'applied' | 'error'>('idle');
	let identifyResults = $state<IdentifyResult[]>([]);
	let currentSuggestion = $state(0);
	let identifyError = $state('');
	let appliedCount = $state(0);
	let previousValues = $state<Record<string, unknown> | null>(null);

	let activeSuggestion = $derived(identifyResults[currentSuggestion] ?? null);
	let suggestionCount = $derived(identifyResults.length);

	// Extra photo slots
	let extraPhoto1 = $state<File | null>(null);
	let extraPhoto2 = $state<File | null>(null);
	let extraPreview1 = $state<string | null>(null);
	let extraPreview2 = $state<string | null>(null);
	let extraInput1: HTMLInputElement = $state() as HTMLInputElement;
	let extraInput2: HTMLInputElement = $state() as HTMLInputElement;

	$effect(() => {
		if (!mediaTouched) {
			mediaMode = hasPhoto ? 'photo' : 'both';
		}
	});

	onMount(() => {
		// Initialize from initial prop once (for edit form)
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

		loadLocations();
		loadAiStatus();
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

	const VALID_IMAGE_TYPES = ['image/jpeg', 'image/png', 'image/webp'];

	function handlePhotoDrop(e: DragEvent) {
		e.preventDefault();
		isDraggingPhoto = false;
		const file = e.dataTransfer?.files?.[0];
		if (!file || !VALID_IMAGE_TYPES.includes(file.type)) return;
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

	// --- Identify handlers ---

	function setExtraPhoto(slot: 1 | 2, file: File) {
		const url = URL.createObjectURL(file);
		if (slot === 1) {
			if (extraPreview1) URL.revokeObjectURL(extraPreview1);
			extraPhoto1 = file;
			extraPreview1 = url;
		} else {
			if (extraPreview2) URL.revokeObjectURL(extraPreview2);
			extraPhoto2 = file;
			extraPreview2 = url;
		}
	}

	function removeExtraPhoto(slot: 1 | 2) {
		if (slot === 1) {
			if (extraPreview1) URL.revokeObjectURL(extraPreview1);
			extraPhoto1 = null;
			extraPreview1 = null;
		} else {
			if (extraPreview2) URL.revokeObjectURL(extraPreview2);
			extraPhoto2 = null;
			extraPreview2 = null;
		}
	}

	function handleExtraSelect(slot: 1 | 2, e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (file) setExtraPhoto(slot, file);
		input.value = '';
	}

	const VALID_LIGHT = ['direct', 'indirect', 'low'];
	const VALID_DIFFICULTY = ['easy', 'moderate', 'demanding'];
	const VALID_PET_SAFETY = ['safe', 'caution', 'toxic'];
	const VALID_GROWTH = ['slow', 'moderate', 'fast'];
	const VALID_SOIL_TYPE = ['standard', 'cactus-mix', 'orchid-bark', 'peat-moss'];
	const VALID_SOIL_MOISTURE = ['dry', 'moderate', 'moist'];

	type FillChip = { label: string; value: string };

	let willFillChips = $derived.by((): FillChip[] => {
		if (!activeSuggestion) return [];
		const t = $translations;
		const chips: FillChip[] = [];
		chips.push({ label: t.form.speciesLabel.replace(' (optional)', '').replace(' (opcional)', ''), value: activeSuggestion.scientific_name });
		const cp = activeSuggestion.care_profile;
		if (cp) {
			if (cp.watering_interval_days != null) chips.push({ label: t.form.watering, value: `${cp.watering_interval_days}d` });
			if (cp.light_needs && VALID_LIGHT.includes(cp.light_needs)) chips.push({ label: t.form.lightNeeds, value: cp.light_needs });
			if (cp.difficulty && VALID_DIFFICULTY.includes(cp.difficulty)) chips.push({ label: t.form.difficulty, value: cp.difficulty });
			if (cp.pet_safety && VALID_PET_SAFETY.includes(cp.pet_safety)) chips.push({ label: t.form.petSafety, value: cp.pet_safety });
			if (cp.growth_speed && VALID_GROWTH.includes(cp.growth_speed)) chips.push({ label: t.form.growthSpeed, value: cp.growth_speed });
			if (cp.soil_type && VALID_SOIL_TYPE.includes(cp.soil_type)) chips.push({ label: t.form.soilType, value: cp.soil_type });
			if (cp.soil_moisture && VALID_SOIL_MOISTURE.includes(cp.soil_moisture)) chips.push({ label: t.form.soilMoisture, value: cp.soil_moisture });
		}
		return chips;
	});

	async function handleIdentify() {
		identifyState = 'loading';
		identifyError = '';

		try {
			const photos: File[] = [];

			// Main photo
			if (photoFile) {
				photos.push(photoFile);
			} else if (initial?.photo_url) {
				const resp = await fetch(initial.photo_url);
				const blob = await resp.blob();
				photos.push(new File([blob], 'photo.jpg', { type: blob.type }));
			}

			if (extraPhoto1) photos.push(extraPhoto1);
			if (extraPhoto2) photos.push(extraPhoto2);

			if (photos.length === 0) {
				identifyState = 'idle';
				return;
			}

			const response = await identifyPlant(photos);
			identifyResults = response.suggestions;
			currentSuggestion = 0;
			identifyState = 'result';
		} catch (e: unknown) {
			identifyError = e instanceof Error ? e.message : $translations.identify.errorMessage;
			identifyState = 'error';
		}
	}

	function handleApply() {
		if (!activeSuggestion) return;

		// Snapshot current values
		previousValues = {
			name, species, notes, wateringDays, lightNeeds,
			difficulty, petSafety, growthSpeed, soilType, soilMoisture
		};

		let count = 0;
		const r = activeSuggestion;
		const cp = r.care_profile;

		species = r.scientific_name;
		count++;

		if (!name.trim() && r.common_name) {
			name = r.common_name;
			count++;
		}
		if (!notes.trim() && r.summary) {
			notes = r.summary;
			count++;
		}

		if (cp) {
			if (cp.watering_interval_days != null) { wateringDays = cp.watering_interval_days; count++; }
			if (cp.light_needs && VALID_LIGHT.includes(cp.light_needs)) { lightNeeds = cp.light_needs; count++; }
			if (cp.difficulty && VALID_DIFFICULTY.includes(cp.difficulty)) { difficulty = cp.difficulty; count++; }
			if (cp.pet_safety && VALID_PET_SAFETY.includes(cp.pet_safety)) { petSafety = cp.pet_safety; count++; }
			if (cp.growth_speed && VALID_GROWTH.includes(cp.growth_speed)) { growthSpeed = cp.growth_speed; count++; }
			if (cp.soil_type && VALID_SOIL_TYPE.includes(cp.soil_type)) { soilType = cp.soil_type; count++; }
			if (cp.soil_moisture && VALID_SOIL_MOISTURE.includes(cp.soil_moisture)) { soilMoisture = cp.soil_moisture; count++; }
		}

		appliedCount = count;
		identifyState = 'applied';
	}

	function handleUndo() {
		if (previousValues) {
			name = previousValues.name as string;
			species = previousValues.species as string;
			notes = previousValues.notes as string;
			wateringDays = previousValues.wateringDays as number;
			lightNeeds = previousValues.lightNeeds as string;
			difficulty = previousValues.difficulty as string | null;
			petSafety = previousValues.petSafety as string | null;
			growthSpeed = previousValues.growthSpeed as string | null;
			soilType = previousValues.soilType as string | null;
			soilMoisture = previousValues.soilMoisture as string | null;
			previousValues = null;
		}
		identifyState = 'idle';
		identifyResults = [];
		currentSuggestion = 0;
	}

	function handleDismiss() {
		identifyState = 'idle';
		identifyResults = [];
		currentSuggestion = 0;
	}

	// Carousel navigation (task 5.7)
	function prevSuggestion() {
		if (suggestionCount <= 1) return;
		currentSuggestion = (currentSuggestion - 1 + suggestionCount) % suggestionCount;
	}

	function nextSuggestion() {
		if (suggestionCount <= 1) return;
		currentSuggestion = (currentSuggestion + 1) % suggestionCount;
	}

	// Touch swipe state
	let swipeStartX = 0;
	let swiping = false;

	function handleSwipeStart(e: PointerEvent) {
		swipeStartX = e.clientX;
		swiping = true;
	}

	function handleSwipeEnd(e: PointerEvent) {
		if (!swiping) return;
		swiping = false;
		const dx = e.clientX - swipeStartX;
		if (Math.abs(dx) > 50) {
			if (dx < 0) nextSuggestion();
			else prevSuggestion();
		}
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
							{#if aiEnabled}
								<span class="upload-hint upload-hint-ai"><Sparkles size={12} /> {$translations.form.addPhotoHint}</span>
							{/if}
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

		<!-- Identify section -->
		{#if hasPhoto && aiEnabled}
			<div class="identify-section">
				{#if identifyState === 'idle'}
					<button type="button" class="identify-btn" onclick={handleIdentify}>
						<Sparkles size={18} />
						{$translations.identify.identifyPlant}
					</button>
					<div class="extra-photos-label">{$translations.identify.extraPhotosHint}</div>
					<div class="extra-photos">
						<div class="extra-photo-slot extra-photo-filled extra-photo-main">
							{#if photoPreview}
								<img src={photoPreview} alt={$translations.form.photoPreview} />
							{:else if initial?.photo_url}
								<img src={initial.photo_url} alt={initial.name} />
							{/if}
						</div>
						{#if extraPreview1}
							<div class="extra-photo-slot extra-photo-filled">
								<img src={extraPreview1} alt={$translations.identify.closeUp} />
								<button type="button" class="extra-photo-remove" onclick={() => removeExtraPhoto(1)}>
									<X size={12} />
								</button>
							</div>
						{:else}
							<label class="extra-photo-slot">
								<Camera size={18} />
								<span>{$translations.identify.closeUp}</span>
								<input
									type="file"
									accept="image/jpeg,image/png,image/webp"
									class="file-input"
									bind:this={extraInput1}
									onchange={(e) => handleExtraSelect(1, e)}
								/>
							</label>
						{/if}
						{#if extraPreview2}
							<div class="extra-photo-slot extra-photo-filled">
								<img src={extraPreview2} alt={$translations.identify.stemPot} />
								<button type="button" class="extra-photo-remove" onclick={() => removeExtraPhoto(2)}>
									<X size={12} />
								</button>
							</div>
						{:else}
							<label class="extra-photo-slot">
								<Camera size={18} />
								<span>{$translations.identify.stemPot}</span>
								<input
									type="file"
									accept="image/jpeg,image/png,image/webp"
									class="file-input"
									bind:this={extraInput2}
									onchange={(e) => handleExtraSelect(2, e)}
								/>
							</label>
						{/if}
					</div>

				{:else if identifyState === 'loading'}
					<div class="identify-loading-header">
						<span class="spinner"></span>
						<Sparkles size={16} />
						{$translations.identify.identifying}
					</div>
					<div class="loading-photos">
						{#if photoPreview}
							<img src={photoPreview} alt="" class="loading-thumb" />
						{:else if initial?.photo_url}
							<img src={initial.photo_url} alt="" class="loading-thumb" />
						{/if}
						{#if extraPreview1}
							<img src={extraPreview1} alt="" class="loading-thumb" />
						{/if}
						{#if extraPreview2}
							<img src={extraPreview2} alt="" class="loading-thumb" />
						{/if}
					</div>
					<div class="shimmer-lines">
						<div class="shimmer"></div>
						<div class="shimmer"></div>
						<div class="shimmer"></div>
					</div>

				{:else if identifyState === 'result' && activeSuggestion}
					<div class="suggestion-header">
						<Sparkles size={14} />
						{$translations.identify.aiSuggestion}
							{#if suggestionCount > 1}
								<span class="suggestion-counter">{$translations.identify.suggestionCount.replace('{current}', String(currentSuggestion + 1)).replace('{total}', String(suggestionCount))}</span>
							{/if}
					</div>
					<!-- svelte-ignore a11y_no_static_element_interactions -->
					<div
						class="suggestion-body"
						onpointerdown={suggestionCount > 1 ? handleSwipeStart : undefined}
						onpointerup={suggestionCount > 1 ? handleSwipeEnd : undefined}
					>
					<div class="suggestion-name">
						<span class="suggestion-scientific">{activeSuggestion.scientific_name}</span>
						{#if activeSuggestion.confidence != null}
							<span class="suggestion-confidence">{$translations.identify.confidence.replace('{n}', String(Math.round(activeSuggestion.confidence * 100)))}</span>
						{/if}
					</div>
					{#if activeSuggestion.common_name}
						<div class="suggestion-common">"{activeSuggestion.common_name}"</div>
					{/if}
					{#if activeSuggestion.summary}
						<div class="suggestion-summary">{activeSuggestion.summary}</div>
					{/if}
					{#if willFillChips.length > 0}
						<div class="will-fill">
							<div class="will-fill-label">{$translations.identify.willFill}</div>
							<div class="fill-chips">
								{#each willFillChips as chip}
									<span class="fill-chip"><Check size={11} /> {chip.label} ({chip.value})</span>
								{/each}
							</div>
						</div>
					{/if}
					</div>
					{#if suggestionCount > 1}
						<div class="suggestion-nav">
							<button type="button" class="nav-btn" onclick={prevSuggestion} aria-label={$translations.identify.prevSuggestion}>
								<ChevronLeft size={18} />
							</button>
							<div class="nav-dots">
								{#each identifyResults as _, i}
									<button
										type="button"
										class="nav-dot"
										class:active={i === currentSuggestion}
										onclick={() => { currentSuggestion = i; }}
										aria-label={$translations.identify.suggestionCount.replace('{current}', String(i + 1)).replace('{total}', String(suggestionCount))}
									></button>
								{/each}
							</div>
							<button type="button" class="nav-btn" onclick={nextSuggestion} aria-label={$translations.identify.nextSuggestion}>
								<ChevronRight size={18} />
							</button>
						</div>
					{/if}
					<div class="suggestion-actions">
						<button type="button" class="btn btn-ai" onclick={handleApply}>{$translations.identify.applyToForm}</button>
						<button type="button" class="btn btn-outline" onclick={handleDismiss}>{$translations.identify.dismiss}</button>
					</div>

				{:else if identifyState === 'applied'}
					<div class="applied-banner">
						<Check size={18} />
						<span>{$translations.identify.applied.replace('{n}', String(appliedCount))}</span>
						<button type="button" class="applied-undo" onclick={handleUndo}>{$translations.identify.undo}</button>
					</div>

				{:else if identifyState === 'error'}
					<div class="identify-error">
						<TriangleAlert size={18} />
						<span>{identifyError || $translations.identify.errorMessage}</span>
						<button type="button" class="btn btn-outline btn-sm" onclick={handleIdentify}>{$translations.identify.retry}</button>
					</div>
				{/if}
			</div>
		{/if}

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
		transition: all var(--transition-speed);
	}

	.photo-remove-btn:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
		border-color: var(--color-danger);
		transform: scale(1.15);
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

	.upload-hint-ai {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: var(--color-ai);
		font-size: 11px;
	}

	.upload-hint-ai :global(svg) {
		color: var(--color-ai);
	}

	/* ---- Identify section ---- */
	.identify-section {
		border: 1px dashed var(--color-border);
		border-radius: var(--radius-card);
		padding: 16px;
		background: var(--color-ai-tint);
		margin-top: 4px;
		margin-bottom: 12px;
		width: 100%;
		box-sizing: border-box;
	}

	.identify-btn {
		display: flex;
		align-items: center;
		gap: 8px;
		width: 100%;
		padding: 12px 16px;
		border: 1px solid color-mix(in srgb, var(--color-ai) 40%, var(--color-border));
		border-radius: var(--radius-btn);
		background: var(--color-ai-soft);
		color: var(--color-text);
		font-size: 15px;
		font-weight: 600;
		font-family: inherit;
		cursor: pointer;
		transition: all var(--transition-speed);
	}

	.identify-btn:hover {
		border-color: var(--color-ai);
		background: color-mix(in srgb, var(--color-ai) 20%, transparent);
	}

	.identify-btn :global(svg) {
		color: var(--color-ai);
	}

	.extra-photos-label {
		font-size: var(--fs-chip);
		color: var(--color-text-muted);
		margin-top: 12px;
		margin-bottom: 8px;
	}

	.extra-photos {
		display: flex;
		gap: 10px;
		flex-wrap: wrap;
	}

	.extra-photo-slot {
		width: 88px;
		height: 88px;
		border: 2px dashed var(--color-border);
		border-radius: var(--radius-btn);
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 2px;
		color: var(--color-text-muted);
		font-size: 11px;
		cursor: pointer;
		transition: all var(--transition-speed);
		background: color-mix(in srgb, var(--color-surface) 60%, transparent);
		text-align: center;
		line-height: 1.2;
	}

	.extra-photo-slot:hover {
		border-color: var(--color-primary);
		color: var(--color-primary);
	}

	.extra-photo-filled {
		border-style: solid;
		position: relative;
		overflow: visible;
		padding: 0;
		cursor: default;
	}

	.extra-photo-filled:hover {
		border-color: var(--color-border);
		color: var(--color-text-muted);
	}

	.extra-photo-filled img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: 6px;
	}

	.extra-photo-remove {
		position: absolute;
		top: -6px;
		right: -6px;
		width: 20px;
		height: 20px;
		border-radius: 50%;
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-danger);
		display: flex;
		align-items: center;
		justify-content: center;
		cursor: pointer;
		padding: 0;
		transition: all var(--transition-speed);
		box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
	}

	.extra-photo-remove:hover {
		background: color-mix(in srgb, var(--color-danger) 10%, var(--color-surface));
		border-color: var(--color-danger);
		transform: scale(1.15);
	}

	/* ---- Loading state ---- */
	.identify-loading-header {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: 15px;
		font-weight: 600;
		margin-bottom: 16px;
		color: var(--color-text);
	}

	.identify-loading-header :global(svg) {
		color: var(--color-ai);
	}

	.spinner {
		display: inline-block;
		width: 16px;
		height: 16px;
		border: 2px solid var(--color-border);
		border-top-color: var(--color-ai);
		border-radius: 50%;
		animation: spin 0.8s linear infinite;
	}

	@keyframes spin {
		to { transform: rotate(360deg); }
	}

	.loading-photos {
		display: flex;
		gap: 8px;
		margin-bottom: 16px;
	}

	.loading-thumb {
		width: 56px;
		height: 56px;
		border-radius: 6px;
		border: 1px solid var(--color-border);
		object-fit: cover;
	}

	.shimmer-lines {
		display: flex;
		flex-direction: column;
		gap: 10px;
	}

	.shimmer {
		height: 14px;
		border-radius: 7px;
		background: linear-gradient(90deg,
			var(--color-surface-muted) 25%,
			color-mix(in srgb, var(--color-ai) 12%, var(--color-surface-muted)) 50%,
			var(--color-surface-muted) 75%
		);
		background-size: 200% 100%;
		animation: shimmer 1.8s ease-in-out infinite;
	}

	.shimmer:nth-child(1) { width: 75%; }
	.shimmer:nth-child(2) { width: 60%; }
	.shimmer:nth-child(3) { width: 45%; }

	@keyframes shimmer {
		0% { background-position: 200% 0; }
		100% { background-position: -200% 0; }
	}

	/* ---- Suggestion card ---- */
	.suggestion-header {
		display: flex;
		align-items: center;
		gap: 8px;
		font-size: var(--fs-section-label);
		font-weight: 600;
		color: var(--color-ai);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 14px;
	}

	.suggestion-name {
		display: flex;
		align-items: baseline;
		justify-content: space-between;
		gap: 12px;
		margin-bottom: 2px;
	}

	.suggestion-scientific {
		font-size: 18px;
		font-weight: 700;
		color: var(--color-text);
	}

	.suggestion-confidence {
		font-size: var(--fs-chip);
		font-weight: 600;
		color: var(--color-success);
		background: var(--color-success-soft);
		padding: 2px 10px;
		border-radius: var(--radius-pill);
		white-space: nowrap;
	}

	.suggestion-common {
		font-size: 14px;
		color: var(--color-text-muted);
		margin-bottom: 10px;
		font-style: italic;
	}

	.suggestion-summary {
		font-size: 14px;
		color: var(--color-text-muted);
		line-height: 1.5;
		margin-bottom: 16px;
	}

	.will-fill {
		margin-bottom: 16px;
	}

	.will-fill-label {
		font-size: var(--fs-chip);
		font-weight: 600;
		color: var(--color-text-muted);
		margin-bottom: 8px;
	}

	.fill-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
	}

	.fill-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 4px 10px;
		border-radius: var(--radius-pill);
		font-size: 12px;
		font-weight: 500;
		background: var(--color-primary-tint);
		color: var(--color-primary);
		border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
	}

	.suggestion-body {
		touch-action: pan-y;
		user-select: none;
	}

	.suggestion-counter {
		margin-left: auto;
		font-size: var(--fs-chip);
		font-weight: 500;
		color: var(--color-text-muted);
		text-transform: none;
		letter-spacing: 0;
	}

	.suggestion-nav {
		display: flex;
		align-items: center;
		justify-content: center;
		gap: 8px;
		margin: 14px 0 16px;
	}

	.nav-btn {
		display: flex;
		align-items: center;
		justify-content: center;
		width: 32px;
		height: 32px;
		border-radius: 50%;
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		padding: 0;
		transition: all var(--transition-speed);
	}

	.nav-btn:hover {
		border-color: var(--color-ai);
		color: var(--color-ai);
		background: var(--color-ai-tint);
	}

	.nav-dots {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.nav-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		border: 1.5px solid var(--color-ai);
		background: transparent;
		padding: 0;
		cursor: pointer;
		transition: all var(--transition-speed);
	}

	.nav-dot.active {
		background: var(--color-ai);
	}

	.nav-dot:hover:not(.active) {
		background: color-mix(in srgb, var(--color-ai) 40%, transparent);
	}

	.suggestion-actions {
		display: flex;
		gap: 10px;
	}

	.suggestion-actions .btn {
		flex: 1;
	}

	:global(.btn-ai) {
		background: var(--color-ai);
		color: #fff;
	}

	:global(.btn-ai:hover) {
		background: color-mix(in srgb, var(--color-ai) 85%, #000);
	}

	/* ---- Applied state ---- */
	.applied-banner {
		display: flex;
		align-items: center;
		gap: 8px;
		padding: 10px 16px;
		background: color-mix(in srgb, var(--color-success) 12%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent);
		border-radius: var(--radius-btn);
		font-size: 14px;
		font-weight: 500;
		color: var(--color-text);
	}

	.applied-banner :global(svg) {
		color: var(--color-success);
		flex-shrink: 0;
	}

	.applied-undo {
		margin-left: auto;
		font-size: var(--fs-chip);
		color: var(--color-text-muted);
		cursor: pointer;
		text-decoration: underline;
		background: none;
		border: none;
		font-family: inherit;
		white-space: nowrap;
	}

	.applied-undo:hover {
		color: var(--color-text);
	}

	/* ---- Error state ---- */
	.identify-error {
		padding: 12px 16px;
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
		border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
		border-radius: var(--radius-btn);
		display: flex;
		align-items: center;
		gap: 10px;
		font-size: 14px;
		color: var(--color-text);
	}

	.identify-error :global(svg) {
		color: var(--color-danger);
		flex-shrink: 0;
	}

	.identify-error .btn {
		margin-left: auto;
		flex-shrink: 0;
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

		.extra-photo-slot {
			width: 80px;
			height: 80px;
		}

		.suggestion-actions {
			flex-direction: column;
		}

		.suggestion-actions .btn {
			min-height: 44px;
		}

		.identify-error {
			flex-wrap: wrap;
		}

		.identify-error .btn {
			margin-left: 0;
			width: 100%;
		}

		.applied-banner {
			flex-wrap: wrap;
		}
	}

</style>
