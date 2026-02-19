<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { ArrowLeft, Pencil, Trash2, Droplet, Droplets, MapPin, Sun, CloudSun, Cloud, Leaf, Shovel, Scissors, BookOpen, Pencil as PencilIcon, X, Info, Gauge, PawPrint, TrendingUp, Layers, Repeat, CalendarCheck, CalendarClock } from 'lucide-svelte';
	import { currentPlant, plantsError, loadPlant, deletePlant, waterPlant } from '$lib/stores/plants';
	import { careEvents, loadCareEvents, addCareEvent, removeCareEvent } from '$lib/stores/care';
	import { emojiToSvgPath } from '$lib/emoji';
	import StatusBadge from '$lib/components/StatusBadge.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';
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

	async function handleDelete() {
		if (!$currentPlant) return;
		if (!confirm(`Delete "${$currentPlant.name}"? This cannot be undone.`)) return;
		deleting = true;
		const success = await deletePlant($currentPlant.id);
		if (success) {
			goto('/');
		}
		deleting = false;
	}

	function lightLabel(needs: string) {
		if (needs === 'direct') return 'Direct sunlight';
		if (needs === 'low') return 'Low light';
		return 'Indirect light';
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
		if (!dateStr) return 'Never';
		const date = new Date(dateStr);
		if (isNaN(date.getTime())) return dateStr;
		return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
	}

	function eventTypeLabel(type: string): string {
		if (type === 'watered') return 'Watered';
		if (type === 'fertilized') return 'Fertilized';
		if (type === 'repotted') return 'Repotted';
		if (type === 'pruned') return 'Pruned';
		return 'Custom';
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
		if (val === 'easy') return 'Easy';
		if (val === 'moderate') return 'Moderate';
		return 'Demanding';
	}

	function petSafetyLabel(val: string): string {
		if (val === 'safe') return 'Safe';
		if (val === 'caution') return 'Caution';
		return 'Toxic';
	}

	function growthSpeedLabel(val: string): string {
		if (val === 'slow') return 'Slow';
		if (val === 'moderate') return 'Moderate';
		return 'Fast';
	}

	function soilMoistureLabel(val: string): string {
		if (val === 'dry') return 'Prefers dry';
		if (val === 'moderate') return 'Moderate';
		return 'Keeps moist';
	}

	function soilTypeLabel(val: string): string {
		if (val === 'standard') return 'Standard';
		if (val === 'cactus-mix') return 'Cactus mix';
		if (val === 'orchid-bark') return 'Orchid bark';
		return 'Peat moss';
	}

</script>

{#if notFound}
	<div class="not-found">
		<h2>Plant not found</h2>
		<p>This plant doesn't exist or may have been deleted.</p>
		<a href="/" class="back-link"><ArrowLeft size={16} /> Back to plants</a>
	</div>
{:else if $currentPlant}
	<div class="detail">
		<PageHeader backHref="/" backLabel="Back">
			<a href="/plants/{$currentPlant.id}/edit" class="action-btn edit-btn">
				<Pencil size={16} />
			</a>
			<button class="action-btn delete-btn" onclick={handleDelete} disabled={deleting}>
				<Trash2 size={16} />
			</button>
		</PageHeader>

		<div class="detail-hero">
			<div class="detail-photo">
				{#if $currentPlant.photo_url}
					<img
						src={$currentPlant.photo_url}
						alt={$currentPlant.name}
						class="detail-photo-img"
					/>
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
				<button class="detail-water-btn" onclick={handleWater} disabled={watering}>
					<Droplet size={16} />
					{watering ? 'Watering...' : 'Water now'}
				</button>
			</div>
		</div>

		<div class="detail-sections">
			<div class="detail-grid">
				<div class="detail-card">
					<div class="detail-card-title"><Droplet size={14} /> Watering</div>
					<div class="detail-row"><span class="detail-row-label">Interval</span><span class="detail-row-value">Every {$currentPlant.watering_interval_days} days <Repeat size={14} /></span></div>
					<div class="detail-row"><span class="detail-row-label">Last watered</span><span class="detail-row-value">{formatDate($currentPlant.last_watered)} <CalendarCheck size={14} /></span></div>
					<div class="detail-row"><span class="detail-row-label">Next due</span><span class="detail-row-value">{$currentPlant.next_due ? formatDate($currentPlant.next_due) : 'N/A'} <CalendarClock size={14} /></span></div>
					{#if $currentPlant.soil_moisture}
						<div class="detail-row">
							<span class="detail-row-label">Soil moisture</span>
							<span class="detail-row-value">{soilMoistureLabel($currentPlant.soil_moisture)} <Droplets size={14} /></span>
						</div>
					{/if}
				</div>
				<div class="detail-card">
					<div class="detail-card-title"><Info size={14} /> Care Info</div>
					<div class="detail-row">
						<span class="detail-row-label">Light</span>
						<span class="detail-row-value">
							{lightLabel($currentPlant.light_needs)}
							<LightNeedsIcon size={14} />
						</span>
					</div>
					{#if $currentPlant.difficulty}
						<div class="detail-row">
							<span class="detail-row-label">Difficulty</span>
							<span class="detail-row-value">{difficultyLabel($currentPlant.difficulty)} <Gauge size={14} /></span>
						</div>
					{/if}
					{#if $currentPlant.pet_safety}
						<div class="detail-row">
							<span class="detail-row-label">Pet safety</span>
							<span class="detail-row-value">{petSafetyLabel($currentPlant.pet_safety)} <PawPrint size={14} /></span>
						</div>
					{/if}
					{#if $currentPlant.growth_speed}
						<div class="detail-row">
							<span class="detail-row-label">Growth</span>
							<span class="detail-row-value">{growthSpeedLabel($currentPlant.growth_speed)} <TrendingUp size={14} /></span>
						</div>
					{/if}
					{#if $currentPlant.soil_type}
						<div class="detail-row">
							<span class="detail-row-label">Soil</span>
							<span class="detail-row-value">{soilTypeLabel($currentPlant.soil_type)} <Layers size={14} /></span>
						</div>
					{/if}
				</div>
			</div>

			{#if $currentPlant.notes}
				<div class="detail-card">
					<div class="detail-card-title"><PencilIcon size={14} /> Notes</div>
					<div class="detail-notes">{$currentPlant.notes}</div>
				</div>
			{/if}

			<div class="detail-card care-journal">
				<div class="detail-card-title"><BookOpen size={14} /> Care Journal</div>

			{#if $careEvents.length === 0}
				<p class="journal-empty">No care events recorded yet.</p>
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
										class="event-delete"
										onclick={() => handleEventDelete(event)}
										disabled={deletingEventId === event.id}
										aria-label="Delete log entry"
									>
										<X size={14} />
									</button>
								</span>
							</li>
						{/each}
					</ul>
				{#if hasMoreEvents && !showAllEvents}
					<button class="add-log-link" onclick={() => showAllEvents = true}>Show more events</button>
				{/if}
			{/if}

			{#if showLogForm}
				<div class="log-form">
					<div class="type-chips">
						{#each [
							{ value: 'fertilized', label: 'Fertilized' },
							{ value: 'repotted', label: 'Repotted' },
							{ value: 'pruned', label: 'Pruned' },
							{ value: 'custom', label: 'Custom' }
						] as chip}
							<button
								class="type-chip"
								class:selected={logEventType === chip.value}
								onclick={() => logEventType = chip.value}
							>
								{chip.label}
							</button>
						{/each}
					</div>
					<textarea
						class="log-notes"
						placeholder="Notes (optional)"
						bind:value={logNotes}
						rows="2"
					></textarea>
					<div class="log-when">
						{#if showLogOccurredAt}
							<label class="log-label">
								When
								<input
									class="log-input"
									type="datetime-local"
									max={nowLocalInputValue()}
									bind:value={logOccurredAt}
								/>
							</label>
						{/if}
					</div>
					<div class="log-actions">
						<button class="log-save" onclick={handleLogSubmit} disabled={!logEventType || logSubmitting}>
							{logSubmitting ? 'Saving...' : 'Save'}
						</button>
						<button
							type="button"
							class="log-when-toggle"
							onclick={() => {
								showLogOccurredAt = !showLogOccurredAt;
								if (showLogOccurredAt && !logOccurredAt) {
									logOccurredAt = nowLocalInputValue();
								}
							}}
						>
							Backdate
						</button>
						<button class="log-cancel" onclick={handleLogCancel}>Cancel</button>
					</div>
				</div>
			{:else}
				<button class="add-log-link" onclick={() => showLogForm = true}>
					+ Add log entry
				</button>
			{/if}
			</div>
		</div>
	</div>
{:else if $plantsError}
	<p class="error">{$plantsError}</p>
{:else}
	<p class="loading">Loading...</p>
{/if}

<style>
	.detail {
		max-width: var(--content-width-default);
		margin: 0 auto;
	}

	.action-btn {
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-btn);
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		text-decoration: none;
		transition: background var(--transition-speed), color var(--transition-speed);
	}

	.action-btn:hover {
		background: var(--color-surface-muted);
		color: var(--color-text);
	}

	.delete-btn:hover {
		color: var(--color-danger);
		border-color: var(--color-danger);
	}

	.detail-hero {
		display: flex;
		align-items: flex-start;
		gap: 20px;
		margin-bottom: 24px;
	}

	.detail-photo {
		width: 200px;
		height: 200px;
		flex-shrink: 0;
		border-radius: var(--radius-card);
		overflow: hidden;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.detail-photo-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		border-radius: var(--radius-card);
	}

	.detail-photo-icon {
		width: 80px;
		height: 80px;
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

	.detail-card-title {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: var(--fs-section-label);
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 12px;
	}
	.detail-water-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 24px;
		background: var(--color-water);
		color: var(--color-text-on-water);
		border: none;
		border-radius: var(--radius-btn);
		font-size: var(--fs-btn);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--transition-speed);
	}

	.detail-water-btn:hover:not(:disabled) {
		background: var(--color-water-strong);
	}

	.detail-water-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
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

	.detail-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-card);
		padding: 16px;
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
	}

	.event-delete {
		width: 28px;
		height: 28px;
		display: inline-flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-btn);
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: background var(--transition-speed), color var(--transition-speed), border-color var(--transition-speed);
	}

	.event-delete:hover:not(:disabled) {
		background: var(--color-danger-soft);
		border-color: var(--color-danger);
		color: var(--color-danger);
	}

	.event-delete:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.timeline-sub {
		display: block;
		color: var(--color-text-muted);
		font-size: 13px;
		margin-top: 2px;
	}

	.add-log-link {
		color: var(--color-primary);
		font-size: 14px;
		font-weight: 500;
		margin-top: 8px;
		cursor: pointer;
		display: inline-block;
		background: none;
		border: none;
		padding: 0;
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

	.log-when-toggle {
		padding: 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-btn);
		background: none;
		color: var(--color-primary);
		font-size: var(--fs-chip);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--transition-speed), border-color var(--transition-speed);
	}

	.log-when-toggle:hover {
		background: var(--color-surface-muted);
		border-color: var(--color-primary);
	}

	.log-input {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-btn);
		font-size: var(--fs-btn);
		font-family: inherit;
		box-sizing: border-box;
	}

	.log-input:focus {
		outline: none;
		border-color: var(--color-primary);
	}

	.type-chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}

	.type-chip {
		padding: 6px 14px;
		border: 1px solid var(--color-border);
		border-radius: 16px;
		background: var(--color-surface);
		color: var(--color-text-muted);
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s, border-color 0.15s, color 0.15s;
	}

	.type-chip:hover {
		background: var(--color-surface-muted);
	}

	.type-chip.selected {
		background: var(--color-primary);
		border-color: var(--color-primary);
		color: var(--color-text-on-primary);
	}

	.log-notes {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-btn);
		font-size: var(--fs-btn);
		font-family: inherit;
		resize: vertical;
		margin-bottom: 10px;
		box-sizing: border-box;
	}

	.log-notes:focus {
		outline: none;
		border-color: var(--color-primary);
	}

	.log-actions {
		display: flex;
		gap: 8px;
	}

	.log-save {
		padding: 8px 20px;
		background: var(--color-primary);
		color: var(--color-text-on-primary);
		border: none;
		border-radius: var(--radius-btn);
		font-size: var(--fs-btn);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--transition-speed);
	}

	.log-save:hover:not(:disabled) {
		background: var(--color-primary-dark);
	}

	.log-save:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.log-cancel {
		padding: 8px 20px;
		background: none;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-btn);
		color: var(--color-text-muted);
		font-size: var(--fs-btn);
		font-weight: 500;
		cursor: pointer;
		transition: background var(--transition-speed);
	}

	.log-cancel:hover {
		background: var(--color-surface-muted);
	}

	@media (min-width: 1280px) {
		.detail-photo {
			width: 220px;
			height: 220px;
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
			height: 180px;
		}

		.detail-info h2 {
			font-size: var(--fs-page-title);
		}

		.detail-water-btn {
			width: 100%;
			justify-content: center;
		}

		.detail-grid {
			grid-template-columns: 1fr;
		}

	}
</style>
