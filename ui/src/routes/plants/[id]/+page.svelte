<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { ArrowLeft, Pencil, Trash2, Droplet, Droplets, MapPin, Sun, CloudSun, Cloud, Leaf, Shovel, Scissors, BookOpen, Pencil as PencilIcon, Info, Gauge, PawPrint, TrendingUp, Layers, Repeat, CalendarCheck, CalendarClock } from 'lucide-svelte';
	import { currentPlant, plantsError, loadPlant, deletePlant, waterPlant } from '$lib/stores/plants';
	import { careEvents, loadCareEvents, addCareEvent, removeCareEvent } from '$lib/stores/care';
	import { translations } from '$lib/stores/locale';
	import { emojiToSvgPath } from '$lib/emoji';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
	import PhotoLightbox from '$lib/components/PhotoLightbox.svelte';
	import ModalDialog from '$lib/components/ModalDialog.svelte';
	import type { CareEvent } from '$lib/api';

	let notFound = $state(false);
	let deleting = $state(false);
	let watering = $state(false);
	let showLogForm = $state(false);
	let logEventType = $state('');
	let logNotes = $state('');
	let logOccurredAt = $state('');
	let showLogOccurredAt = $state(false);
	let logSubmitting = $state(false);
	let showAllEvents = $state(false);
	let deletingEventId = $state<number | null>(null);
	let backHref = $state('/');
	let lightboxOpen = $state(false);
	let deleteDialogOpen = $state(false);
	const BACK_PATHS = new Set(['/', '/care-journal', '/plants', '/settings']);

	const EVENT_LIMIT = 20;

	onMount(async () => {
		const id = Number($page.params.id);
		const plant = await loadPlant(id);
		if (!plant) {
			notFound = true;
		} else {
			await loadCareEvents(id);
		}
	});

	$effect(() => {
		const from = $page.url.searchParams.get('from');
		backHref = from && BACK_PATHS.has(from) ? from : '/';
	});

	function handleDelete() {
		deleteDialogOpen = true;
	}

	async function handleDeleteConfirm() {
		deleteDialogOpen = false;
		if (!$currentPlant) return;
		deleting = true;
		const success = await deletePlant($currentPlant.id);
		if (success) {
			goto('/');
		}
		deleting = false;
	}

	function lightLabel(needs: string) {
		if (needs === 'direct') return $translations.plant.lightDirect;
		if (needs === 'low') return $translations.plant.lightLow;
		return $translations.plant.lightIndirect;
	}

	function lightIcon(needs: string) {
		if (needs === 'direct') return Sun;
		if (needs === 'low') return Cloud;
		return CloudSun;
	}

	async function handleWater() {
		if (!$currentPlant || watering) return;
		watering = true;
		await waterPlant($currentPlant.id);
		await loadCareEvents($currentPlant.id);
		watering = false;
	}

	function formatDate(dateStr: string | null): string {
		if (!dateStr) return $translations.plant.never;
		const date = new Date(dateStr);
		if (isNaN(date.getTime())) return dateStr;
		return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
	}

	function eventTypeLabel(type: string): string {
		if (type === 'watered') return $translations.care.watered;
		if (type === 'fertilized') return $translations.care.fertilized;
		if (type === 'repotted') return $translations.care.repotted;
		if (type === 'pruned') return $translations.care.pruned;
		return $translations.care.custom;
	}

	function parseEventDate(dateStr: string): Date {
		const hasTimezone = /Z|[+-]\d{2}:\d{2}$/.test(dateStr);
		return new Date(hasTimezone ? dateStr : `${dateStr}Z`);
	}

	function formatShortDate(dateStr: string): string {
		const date = parseEventDate(dateStr);
		if (isNaN(date.getTime())) return dateStr;
		return date.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: '2-digit' });
	}

	async function handleLogSubmit() {
		if (!$currentPlant || !logEventType || logSubmitting) return;
		logSubmitting = true;
		const occurredAt = showLogOccurredAt ? logOccurredAt.trim() : '';
		const occurredAtDate = occurredAt ? new Date(occurredAt) : null;
		const occurredAtIso = occurredAtDate && !isNaN(occurredAtDate.getTime())
			? occurredAtDate.toISOString()
			: undefined;
		await addCareEvent($currentPlant.id, {
			event_type: logEventType,
			notes: logNotes.trim() || undefined,
			occurred_at: occurredAtIso
		});
		logEventType = '';
		logNotes = '';
		logOccurredAt = '';
		showLogOccurredAt = false;
		showLogForm = false;
		logSubmitting = false;
	}

	async function handleEventDelete(event: CareEvent) {
		if (!$currentPlant || deletingEventId === event.id) return;
		deletingEventId = event.id;
		await removeCareEvent($currentPlant.id, event.id);
		await loadPlant($currentPlant.id);
		deletingEventId = null;
	}

	function handleLogCancel() {
		showLogForm = false;
		logEventType = '';
		logNotes = '';
		logOccurredAt = '';
		showLogOccurredAt = false;
	}

	function nowLocalInputValue(): string {
		const now = new Date();
		const pad = (n: number) => String(n).padStart(2, '0');
		return `${now.getFullYear()}-${pad(now.getMonth() + 1)}-${pad(now.getDate())}T${pad(now.getHours())}:${pad(now.getMinutes())}`;
	}

	let displayEvents = $derived(
		showAllEvents ? $careEvents : $careEvents.slice(0, EVENT_LIMIT)
	);

	let hasMoreEvents = $derived($careEvents.length > EVENT_LIMIT);

	let LightNeedsIcon = $derived(
		$currentPlant ? lightIcon($currentPlant.light_needs) : Sun
	);

	function difficultyLabel(val: string): string {
		if (val === 'easy') return $translations.plant.difficultyEasy;
		if (val === 'moderate') return $translations.plant.difficultyModerate;
		return $translations.plant.difficultyDemanding;
	}

	function petSafetyLabel(val: string): string {
		if (val === 'safe') return $translations.plant.petSafe;
		if (val === 'caution') return $translations.plant.petCaution;
		return $translations.plant.petToxic;
	}

	function growthSpeedLabel(val: string): string {
		if (val === 'slow') return $translations.plant.growthSlow;
		if (val === 'moderate') return $translations.plant.growthModerate;
		return $translations.plant.growthFast;
	}

	function soilMoistureLabel(val: string): string {
		if (val === 'dry') return $translations.plant.moistureDry;
		if (val === 'moderate') return $translations.plant.moistureModerate;
		return $translations.plant.moistureMoist;
	}

	function soilTypeLabel(val: string): string {
		if (val === 'standard') return $translations.plant.soilStandard;
		if (val === 'cactus-mix') return $translations.plant.soilCactus;
		if (val === 'orchid-bark') return $translations.plant.soilOrchid;
		return $translations.plant.soilPeat;
	}

	function openLightbox() {
		if (!$currentPlant?.photo_url) return;
		lightboxOpen = true;
	}

	function closeLightbox() {
		lightboxOpen = false;
	}

</script>

{#if notFound}
	<div class="not-found">
		<h2>{$translations.plant.notFound}</h2>
		<p>{$translations.plant.notFoundHint}</p>
		<a href="/" class="back-link"><ArrowLeft size={16} /> {$translations.plant.backToPlants}</a>
	</div>
{:else if $currentPlant}
	<div class="detail">
		<PageHeader {backHref} backLabel={$translations.common.back}>
			<a href="/plants/{$currentPlant.id}/edit" class="btn btn-icon">
				<Pencil size={16} />
			</a>
			<button class="btn btn-icon btn-danger" onclick={handleDelete} disabled={deleting}>
				<Trash2 size={16} />
			</button>
		</PageHeader>

		<div class="detail-hero">
			<div class="detail-photo">
				{#if $currentPlant.photo_url}
					<button
						type="button"
						class="detail-photo-button"
						aria-label={$translations.plant.openPhoto}
						onclick={openLightbox}
					>
						<img
							src={$currentPlant.photo_url}
							alt={$currentPlant.name}
							class="detail-photo-img"
						/>
					</button>
				{:else}
					<img
						src={emojiToSvgPath($currentPlant.icon)}
						alt={$currentPlant.name}
						class="detail-photo-icon"
					/>
				{/if}
			</div>
			<div class="detail-info">
				<div class="detail-name">
					<h2>{$currentPlant.name}</h2>
					{#if $currentPlant.species}
						<span class="detail-species">{$currentPlant.species}</span>
					{/if}
				</div>
				{#if $currentPlant.location_name}
					<p class="detail-location"><MapPin size={14} /> {$currentPlant.location_name}</p>
				{/if}
				<div class="detail-status">
					<StatusBadge status={$currentPlant.watering_status} nextDue={$currentPlant.next_due ?? null} />
				</div>
				<button class="btn btn-water btn-lg" onclick={handleWater} disabled={watering}>
					<Droplet size={16} />
					{watering ? $translations.dashboard.watering : $translations.plant.waterNow}
				</button>
			</div>
		</div>

		<div class="detail-sections">
			<div class="detail-grid">
				<div class="section">
					<div class="section-title"><Droplet size={14} /> {$translations.plant.wateringSection}</div>
					<div class="detail-row"><span class="detail-row-label">{$translations.plant.interval}</span><span class="detail-row-value">{$translations.plant.everyNDays.replace('{n}', String($currentPlant.watering_interval_days))} <Repeat size={14} /></span></div>
					<div class="detail-row"><span class="detail-row-label">{$translations.plant.lastWatered}</span><span class="detail-row-value">{formatDate($currentPlant.last_watered)} <CalendarCheck size={14} /></span></div>
					<div class="detail-row"><span class="detail-row-label">{$translations.plant.nextDue}</span><span class="detail-row-value">{$currentPlant.next_due ? formatDate($currentPlant.next_due) : $translations.plant.na} <CalendarClock size={14} /></span></div>
					{#if $currentPlant.soil_moisture}
						<div class="detail-row">
							<span class="detail-row-label">{$translations.plant.soilMoisture}</span>
							<span class="detail-row-value">{soilMoistureLabel($currentPlant.soil_moisture)} <Droplets size={14} /></span>
						</div>
					{/if}
				</div>
				<div class="section">
					<div class="section-title"><Info size={14} /> {$translations.plant.careInfoSection}</div>
					<div class="detail-row">
						<span class="detail-row-label">{$translations.plant.light}</span>
						<span class="detail-row-value">
							{lightLabel($currentPlant.light_needs)}
							<LightNeedsIcon size={14} />
						</span>
					</div>
					{#if $currentPlant.difficulty}
						<div class="detail-row">
							<span class="detail-row-label">{$translations.plant.difficulty}</span>
							<span class="detail-row-value">{difficultyLabel($currentPlant.difficulty)} <Gauge size={14} /></span>
						</div>
					{/if}
					{#if $currentPlant.pet_safety}
						<div class="detail-row">
							<span class="detail-row-label">{$translations.plant.petSafety}</span>
							<span class="detail-row-value">{petSafetyLabel($currentPlant.pet_safety)} <PawPrint size={14} /></span>
						</div>
					{/if}
					{#if $currentPlant.growth_speed}
						<div class="detail-row">
							<span class="detail-row-label">{$translations.plant.growth}</span>
							<span class="detail-row-value">{growthSpeedLabel($currentPlant.growth_speed)} <TrendingUp size={14} /></span>
						</div>
					{/if}
					{#if $currentPlant.soil_type}
						<div class="detail-row">
							<span class="detail-row-label">{$translations.plant.soil}</span>
							<span class="detail-row-value">{soilTypeLabel($currentPlant.soil_type)} <Layers size={14} /></span>
						</div>
					{/if}
				</div>
			</div>

			{#if $currentPlant.notes}
				<div class="section">
					<div class="section-title"><PencilIcon size={14} /> {$translations.plant.notesSection}</div>
					<div class="detail-notes">{$currentPlant.notes}</div>
				</div>
			{/if}

			<div class="section care-journal">
				<div class="section-title"><BookOpen size={14} /> {$translations.plant.careJournalSection}</div>

			{#if $careEvents.length === 0}
				<p class="journal-empty">{$translations.plant.noCareEvents}</p>
			{:else}
					<ul class="timeline">
						{#each displayEvents as event}
							<li class="timeline-item">
								<span class="timeline-date">{formatShortDate(event.occurred_at)}</span>
								<span class="timeline-icon">
									{#if event.event_type === 'watered'}
										<Droplet size={14} />
									{:else if event.event_type === 'fertilized'}
										<Leaf size={14} />
									{:else if event.event_type === 'repotted'}
										<Shovel size={14} />
									{:else if event.event_type === 'pruned'}
										<Scissors size={14} />
									{:else}
										<PencilIcon size={14} />
									{/if}
								</span>
								<span class="timeline-text">
									{eventTypeLabel(event.event_type)}
									{#if event.notes}
										<span class="timeline-sub">{event.notes}</span>
									{/if}
								</span>
								<span class="timeline-actions">
									<button
										class="btn btn-ghost event-delete"
										onclick={() => handleEventDelete(event)}
										disabled={deletingEventId === event.id}
										aria-label={$translations.plant.deleteLogEntry}
									>
										<Trash2 size={16} />
									</button>
								</span>
							</li>
						{/each}
					</ul>
				{#if hasMoreEvents && !showAllEvents}
					<button class="btn btn-ghost" onclick={() => showAllEvents = true}>{$translations.plant.showMore}</button>
				{/if}
			{/if}

			{#if showLogForm}
				<div class="log-form">
					<div class="type-chips">
						{#each [
							{ value: 'fertilized', label: $translations.care.fertilized },
							{ value: 'repotted', label: $translations.care.repotted },
							{ value: 'pruned', label: $translations.care.pruned },
							{ value: 'custom', label: $translations.care.custom }
						] as chip}
							<button
								class="chip chip-solid"
								class:active={logEventType === chip.value}
								onclick={() => logEventType = chip.value}
							>
								{chip.label}
							</button>
						{/each}
					</div>
					<textarea
						class="input log-notes"
						placeholder={$translations.plant.notesOptional}
						bind:value={logNotes}
						rows="2"
					></textarea>
					<div class="log-when">
						{#if showLogOccurredAt}
							<label class="log-label">
								{$translations.plant.when}
								<input
									class="input log-input"
									type="datetime-local"
									max={nowLocalInputValue()}
									bind:value={logOccurredAt}
								/>
							</label>
						{/if}
					</div>
					<div class="log-actions">
						<button class="btn btn-primary" onclick={handleLogSubmit} disabled={!logEventType || logSubmitting}>
							{logSubmitting ? $translations.common.saving : $translations.common.save}
						</button>
						<button
							type="button"
							class="btn btn-outline btn-sm"
							onclick={() => {
								showLogOccurredAt = !showLogOccurredAt;
								if (showLogOccurredAt && !logOccurredAt) {
									logOccurredAt = nowLocalInputValue();
								}
							}}
						>
							{$translations.plant.backdate}
						</button>
						<button class="btn btn-outline" onclick={handleLogCancel}>{$translations.common.cancel}</button>
					</div>
				</div>
			{:else}
				<button class="btn btn-ghost" onclick={() => showLogForm = true}>
					{$translations.plant.addLogEntry}
				</button>
			{/if}
			</div>
		</div>

		<PhotoLightbox
			open={lightboxOpen}
			src={$currentPlant.photo_url ?? ''}
			alt={$currentPlant.name}
			onclose={closeLightbox}
		/>
	</div>
{:else if $plantsError}
	<p class="error">{$plantsError}</p>
{:else}
	<p class="loading">{$translations.common.loading}</p>
{/if}

<ModalDialog
	open={deleteDialogOpen}
	title={$translations.plant.deletePlant}
	message={$currentPlant ? $translations.plant.deleteConfirm.replace('{name}', $currentPlant.name) : ''}
	mode="confirm"
	variant="danger"
	confirmLabel={$translations.common.delete}
	onconfirm={handleDeleteConfirm}
	oncancel={() => { deleteDialogOpen = false; }}
/>

<style>
	.detail {
		max-width: var(--content-width-default);
		margin: 0 auto;
	}


	.detail-hero {
		display: flex;
		align-items: flex-start;
		gap: 20px;
		margin-bottom: 24px;
	}

	.detail-photo {
		width: 260px;
		height: 260px;
		flex-shrink: 0;
		border-radius: var(--radius-card);
		overflow: hidden;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.detail-photo-button {
		width: 100%;
		height: 100%;
		border: none;
		background: transparent;
		padding: 0;
		cursor: zoom-in;
	}

	.detail-photo-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: var(--radius-card);
	}

	.detail-photo-icon {
		width: 110px;
		height: 110px;
	}

	.detail-name {
		display: flex;
		align-items: baseline;
		flex-wrap: wrap;
		gap: 6px;
		margin-bottom: 6px;
	}

	.detail-info h2 {
		font-size: var(--fs-page-title);
		font-weight: 700;
		margin: 0;
	}

	.detail-species {
		color: var(--color-text-muted);
		font-size: 14px;
		font-style: italic;
	}

	.detail-info {
		flex: 1;
	}

	.detail-location {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--color-text-muted);
		font-size: 14px;
		margin: 0 0 10px;
	}

	.detail-status {
		margin-bottom: 14px;
	}

	.detail-sections {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.detail-grid {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
	}

	.detail-row {
		display: flex;
		justify-content: space-between;
		padding: 6px 0;
		font-size: 14px;
	}

	.detail-row-label {
		color: var(--color-text-muted);
	}

	.detail-row-value {
		display: inline-flex;
		align-items: center;
		gap: 6px;
	}

	.detail-notes {
		font-size: 14px;
		line-height: 1.5;
		color: var(--color-text);
		white-space: pre-wrap;
	}

	.not-found {
		text-align: center;
		padding: 64px 24px;
	}

	.not-found h2 {
		font-size: var(--fs-page-title);
		font-weight: 600;
		margin: 0 0 8px;
	}

	.not-found p {
		color: var(--color-text-muted);
		margin: 0 0 24px;
	}

	.error {
		color: var(--color-danger);
		padding: 16px;
	}

	.loading {
		color: var(--color-text-muted);
		padding: 16px;
	}

	.care-journal {
		margin-bottom: 16px;
	}

	.journal-empty {
		color: var(--color-text-muted);
		font-size: 14px;
		margin: 8px 0 0;
	}

	.timeline {
		list-style: none;
		margin: 0;
		padding: 0;
	}

	.timeline-item {
		display: flex;
		gap: 12px;
		padding: 8px 0;
		font-size: 14px;
		border-bottom: 1px solid var(--color-border);
	}

	.timeline-item:last-child {
		border-bottom: none;
	}

	.timeline-date {
		color: var(--color-text-muted);
		font-size: 13px;
		min-width: 72px;
		flex-shrink: 0;
		text-align: end;
	}

	.timeline-icon {
		font-size: 16px;
		flex-shrink: 0;
	}

	.timeline-text {
		flex: 1;
		min-width: 0;
	}

	.timeline-actions {
		display: flex;
		align-items: flex-start;
	}	.event-delete {
		color: var(--color-text-muted);
	}

	.event-delete:hover:not(:disabled) {
		color: var(--color-danger);
		opacity: 1;
	}

	.timeline-sub {
		display: block;
		color: var(--color-text-muted);
		font-size: 13px;
		margin-top: 2px;
	}

	.btn-ghost {
		margin-top: 8px;
	}

	.log-form {
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid var(--color-border-subtle);
	}

	.log-when {
		margin: 8px 0 10px;
	}

	.log-label {
		display: flex;
		flex-direction: column;
		gap: 6px;
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-muted);
		margin-bottom: 10px;
	}


	.log-input {
		width: 100%;
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

	.log-actions {
		display: flex;
		gap: 8px;
	}


	@media (min-width: 1280px) {
		.detail-photo {
			width: 300px;
			height: 300px;
		}
	}

	@media (max-width: 768px) {
		.detail {
			padding-bottom: 64px;
		}

		.detail-hero {
			flex-direction: column;
			gap: 16px;
		}

		.detail-photo {
			width: 100%;
			height: 220px;
		}

		.detail-info h2 {
			font-size: var(--fs-page-title);
		}

		.btn-water {
			width: 100%;
		}

		.detail-grid {
			grid-template-columns: 1fr;
		}

	}
</style>
