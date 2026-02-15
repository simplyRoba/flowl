<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { ArrowLeft, Pencil, Trash2, Droplet, MapPin, Sun, CloudSun, Cloud, Leaf, Shovel, Scissors, X } from 'lucide-svelte';
	import { currentPlant, plantsError, loadPlant, deletePlant, waterPlant } from '$lib/stores/plants';
	import { careEvents, loadCareEvents, addCareEvent, removeCareEvent } from '$lib/stores/care';
	import { emojiToSvgPath } from '$lib/emoji';
	import type { CareEvent } from '$lib/api';

	let notFound = $state(false);
	let deleting = $state(false);
	let watering = $state(false);
	let showLogForm = $state(false);
	let logEventType = $state('');
	let logNotes = $state('');
	let logSubmitting = $state(false);
	let showAllEvents = $state(false);

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

	let LightIcon = $derived(
		$currentPlant?.light_needs === 'direct' ? Sun :
		$currentPlant?.light_needs === 'low' ? Cloud :
		CloudSun
	);

	function lightLabel(needs: string) {
		if (needs === 'direct') return 'Direct sunlight';
		if (needs === 'low') return 'Low light';
		return 'Indirect light';
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

	function statusLabel(status: string): string {
		if (status === 'overdue') return 'Overdue';
		if (status === 'due') return 'Due';
		return 'OK';
	}

	function eventTypeLabel(type: string): string {
		if (type === 'watered') return 'Watered';
		if (type === 'fertilized') return 'Fertilized';
		if (type === 'repotted') return 'Repotted';
		if (type === 'pruned') return 'Pruned';
		return 'Custom';
	}

	function dayLabel(dateStr: string): string {
		const date = new Date(dateStr);
		const today = new Date();
		today.setHours(0, 0, 0, 0);
		const yesterday = new Date(today);
		yesterday.setDate(yesterday.getDate() - 1);
		const eventDate = new Date(date);
		eventDate.setHours(0, 0, 0, 0);
		if (eventDate.getTime() === today.getTime()) return 'Today';
		if (eventDate.getTime() === yesterday.getTime()) return 'Yesterday';
		return date.toLocaleDateString(undefined, { year: 'numeric', month: 'short', day: 'numeric' });
	}

	function formatTime(dateStr: string): string {
		const date = new Date(dateStr);
		if (isNaN(date.getTime())) return '';
		return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
	}

	async function handleLogSubmit() {
		if (!$currentPlant || !logEventType || logSubmitting) return;
		logSubmitting = true;
		await addCareEvent($currentPlant.id, {
			event_type: logEventType,
			notes: logNotes.trim() || undefined
		});
		logEventType = '';
		logNotes = '';
		showLogForm = false;
		logSubmitting = false;
	}

	function handleLogCancel() {
		showLogForm = false;
		logEventType = '';
		logNotes = '';
	}

	async function handleDeleteEvent(eventId: number) {
		if (!$currentPlant) return;
		await removeCareEvent($currentPlant.id, eventId);
	}

	let displayEvents = $derived(
		showAllEvents ? $careEvents : $careEvents.slice(0, EVENT_LIMIT)
	);

	let hasMoreEvents = $derived($careEvents.length > EVENT_LIMIT);

	interface DayGroup {
		label: string;
		events: CareEvent[];
	}

	let groupedEvents: DayGroup[] = $derived.by(() => {
		const groups: DayGroup[] = [];
		let currentLabel = '';
		for (const event of displayEvents) {
			const label = dayLabel(event.occurred_at);
			if (label !== currentLabel) {
				groups.push({ label, events: [event] });
				currentLabel = label;
			} else {
				groups[groups.length - 1].events.push(event);
			}
		}
		return groups;
	});
</script>

{#if notFound}
	<div class="not-found">
		<h2>Plant not found</h2>
		<p>This plant doesn't exist or may have been deleted.</p>
		<a href="/" class="back-link"><ArrowLeft size={16} /> Back to plants</a>
	</div>
{:else if $currentPlant}
	<div class="detail">
		<header class="detail-header">
			<a href="/" class="back-link"><ArrowLeft size={18} /> Back</a>
			<div class="actions">
				<a href="/plants/{$currentPlant.id}/edit" class="action-btn edit-btn">
					<Pencil size={16} />
				</a>
				<button class="action-btn delete-btn" onclick={handleDelete} disabled={deleting}>
					<Trash2 size={16} />
				</button>
			</div>
		</header>

		<div class="plant-hero">
			{#if $currentPlant.photo_url}
				<img
					src={$currentPlant.photo_url}
					alt={$currentPlant.name}
					class="hero-photo"
				/>
			{:else}
				<img
					src={emojiToSvgPath($currentPlant.icon)}
					alt={$currentPlant.name}
					class="hero-icon"
				/>
			{/if}
			<div>
				<h1>{$currentPlant.name}</h1>
				{#if $currentPlant.species}
					<p class="species">{$currentPlant.species}</p>
				{/if}
				{#if $currentPlant.location_name}
					<p class="location"><MapPin size={14} /> {$currentPlant.location_name}</p>
				{/if}
			</div>
		</div>

		<div class="watering-card info-card">
			<div class="watering-header">
				<h3><Droplet size={16} /> Watering</h3>
				<span class="watering-status watering-{$currentPlant.watering_status}">
					{statusLabel($currentPlant.watering_status)}
				</span>
			</div>
			<div class="watering-details">
				<div class="watering-detail">
					<span class="watering-label">Interval</span>
					<span>Every {$currentPlant.watering_interval_days} days</span>
				</div>
				<div class="watering-detail">
					<span class="watering-label">Last watered</span>
					<span>{formatDate($currentPlant.last_watered)}</span>
				</div>
				<div class="watering-detail">
					<span class="watering-label">Next due</span>
					<span>{$currentPlant.next_due ? formatDate($currentPlant.next_due) : 'N/A'}</span>
				</div>
			</div>
			<button class="water-btn" onclick={handleWater} disabled={watering}>
				<Droplet size={16} />
				{watering ? 'Watering...' : 'Water now'}
			</button>
		</div>

		<div class="info-cards">
			<div class="info-card">
				<h3>
					<LightIcon size={16} />
					Light
				</h3>
				<p>{lightLabel($currentPlant.light_needs)}</p>
			</div>
		</div>

		{#if $currentPlant.notes}
			<div class="info-card notes-card">
				<h3>Notes</h3>
				<p>{$currentPlant.notes}</p>
			</div>
		{/if}

		<div class="care-journal info-card">
			<h3><Leaf size={16} /> Care Journal</h3>

			{#if $careEvents.length === 0}
				<p class="journal-empty">No care events recorded yet.</p>
			{:else}
				<div class="timeline">
					{#each groupedEvents as group}
						<div class="day-group">
							<div class="day-label">{group.label}</div>
							{#each group.events as event}
								<div class="timeline-event">
									<div class="event-icon">
										{#if event.event_type === 'watered'}
											<Droplet size={14} />
										{:else if event.event_type === 'fertilized'}
											<Leaf size={14} />
										{:else if event.event_type === 'repotted'}
											<Shovel size={14} />
										{:else if event.event_type === 'pruned'}
											<Scissors size={14} />
										{:else}
											<Pencil size={14} />
										{/if}
									</div>
									<div class="event-content">
										<span class="event-label">{eventTypeLabel(event.event_type)}</span>
										{#if event.notes}
											<p class="event-notes">{event.notes}</p>
										{/if}
									</div>
									<span class="event-time">{formatTime(event.occurred_at)}</span>
									<button class="event-delete" onclick={() => handleDeleteEvent(event.id)} title="Delete event">
										<X size={14} />
									</button>
								</div>
							{/each}
						</div>
					{/each}
				</div>
				{#if hasMoreEvents && !showAllEvents}
					<button class="show-more" onclick={() => showAllEvents = true}>
						Show more events
					</button>
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
					<div class="log-actions">
						<button class="log-save" onclick={handleLogSubmit} disabled={!logEventType || logSubmitting}>
							{logSubmitting ? 'Saving...' : 'Save'}
						</button>
						<button class="log-cancel" onclick={handleLogCancel}>Cancel</button>
					</div>
				</div>
			{:else}
				<button class="add-log-btn" onclick={() => showLogForm = true}>
					+ Add log entry
				</button>
			{/if}
		</div>
	</div>
{:else if $plantsError}
	<p class="error">{$plantsError}</p>
{:else}
	<p class="loading">Loading...</p>
{/if}

<style>
	.detail {
		max-width: 800px;
		margin: 0 auto;
	}

	.detail-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
	}

	.back-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: #6B8F71;
		text-decoration: none;
		font-size: 15px;
		font-weight: 500;
	}

	.back-link:hover {
		color: #4A6B4F;
	}

	.actions {
		display: flex;
		gap: 8px;
	}

	.action-btn {
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		border: 1px solid #E5DDD3;
		background: #FFFFFF;
		color: #8C7E6E;
		cursor: pointer;
		text-decoration: none;
		transition: background 0.15s, color 0.15s;
	}

	.action-btn:hover {
		background: #FAF6F1;
		color: #2C2418;
	}

	.delete-btn:hover {
		color: #C45B5B;
		border-color: #C45B5B;
	}

	.plant-hero {
		display: flex;
		align-items: center;
		gap: 24px;
		margin-bottom: 32px;
	}

	.hero-icon {
		width: 80px;
		height: 80px;
		flex-shrink: 0;
	}

	.hero-photo {
		width: 80px;
		height: 80px;
		flex-shrink: 0;
		border-radius: 8px;
		object-fit: cover;
	}

	.plant-hero h1 {
		font-size: 28px;
		font-weight: 700;
		margin: 0 0 4px;
	}

	.species {
		color: #8C7E6E;
		font-size: 15px;
		margin: 0 0 4px;
		font-style: italic;
	}

	.location {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: #8C7E6E;
		font-size: 13px;
		margin: 0;
	}

	.info-cards {
		display: grid;
		grid-template-columns: 1fr 1fr;
		gap: 16px;
		margin-bottom: 16px;
	}

	.info-card {
		background: #FFFFFF;
		border: 1px solid #E5DDD3;
		border-radius: 12px;
		padding: 16px;
	}

	.info-card h3 {
		display: flex;
		align-items: center;
		gap: 6px;
		font-size: 13px;
		font-weight: 600;
		color: #8C7E6E;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin: 0 0 8px;
	}

	.info-card p {
		font-size: 15px;
		margin: 0;
	}

	.watering-card {
		margin-bottom: 16px;
	}

	.watering-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 12px;
	}

	.watering-header h3 {
		margin: 0;
	}

	.watering-status {
		font-size: 12px;
		font-weight: 600;
		padding: 3px 10px;
		border-radius: 10px;
		text-transform: uppercase;
		letter-spacing: 0.3px;
	}

	.watering-ok {
		background: #E8F5E9;
		color: #4A6B4F;
	}

	.watering-due {
		background: #FFF4E5;
		color: #C48B3B;
	}

	.watering-overdue {
		background: #FDECEA;
		color: #C45B5B;
	}

	.watering-details {
		display: flex;
		flex-direction: column;
		gap: 8px;
		margin-bottom: 16px;
	}

	.watering-detail {
		display: flex;
		justify-content: space-between;
		font-size: 14px;
	}

	.watering-label {
		color: #8C7E6E;
	}

	.water-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 20px;
		background: #4A90D9;
		color: #fff;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
		width: 100%;
		justify-content: center;
	}

	.water-btn:hover:not(:disabled) {
		background: #3A7BC8;
	}

	.water-btn:disabled {
		opacity: 0.6;
		cursor: not-allowed;
	}

	.notes-card {
		margin-bottom: 16px;
	}

	.notes-card p {
		white-space: pre-wrap;
	}

	.not-found {
		text-align: center;
		padding: 64px 24px;
	}

	.not-found h2 {
		font-size: 22px;
		font-weight: 600;
		margin: 0 0 8px;
	}

	.not-found p {
		color: #8C7E6E;
		margin: 0 0 24px;
	}

	.error {
		color: #C45B5B;
		padding: 16px;
	}

	.loading {
		color: #8C7E6E;
		padding: 16px;
	}

	.care-journal {
		margin-bottom: 16px;
	}

	.journal-empty {
		color: #8C7E6E;
		font-size: 14px;
		margin: 8px 0 0;
	}

	.timeline {
		margin-top: 12px;
	}

	.day-group {
		margin-bottom: 16px;
	}

	.day-group:last-child {
		margin-bottom: 0;
	}

	.day-label {
		font-size: 12px;
		font-weight: 600;
		color: #8C7E6E;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 8px;
	}

	.timeline-event {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 8px 0;
		border-bottom: 1px solid #F0EBE4;
	}

	.timeline-event:last-child {
		border-bottom: none;
	}

	.event-icon {
		width: 28px;
		height: 28px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 50%;
		background: #F5F0EA;
		color: #8C7E6E;
		flex-shrink: 0;
	}

	.event-content {
		flex: 1;
		min-width: 0;
	}

	.event-label {
		font-size: 14px;
		font-weight: 500;
	}

	.event-time {
		font-size: 12px;
		color: #8C7E6E;
		flex-shrink: 0;
		padding-top: 2px;
	}

	.event-notes {
		font-size: 13px;
		color: #8C7E6E;
		margin: 2px 0 0;
	}

	.event-delete {
		background: none;
		border: none;
		color: #C4B5A5;
		cursor: pointer;
		padding: 4px;
		border-radius: 4px;
		flex-shrink: 0;
		opacity: 0;
		transition: opacity 0.15s, color 0.15s;
	}

	.timeline-event:hover .event-delete {
		opacity: 1;
	}

	.event-delete:hover {
		color: #C45B5B;
	}

	.show-more {
		display: block;
		width: 100%;
		padding: 8px;
		background: none;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		color: #6B8F71;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		margin-top: 8px;
		transition: background 0.15s;
	}

	.show-more:hover {
		background: #FAF6F1;
	}

	.add-log-btn {
		display: block;
		width: 100%;
		padding: 10px;
		background: none;
		border: 1px dashed #D5CCC2;
		border-radius: 8px;
		color: #6B8F71;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		margin-top: 12px;
		transition: background 0.15s, border-color 0.15s;
	}

	.add-log-btn:hover {
		background: #FAF6F1;
		border-color: #6B8F71;
	}

	.log-form {
		margin-top: 12px;
		padding-top: 12px;
		border-top: 1px solid #F0EBE4;
	}

	.type-chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-bottom: 10px;
	}

	.type-chip {
		padding: 6px 14px;
		border: 1px solid #E5DDD3;
		border-radius: 16px;
		background: #FFFFFF;
		color: #8C7E6E;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s, border-color 0.15s, color 0.15s;
	}

	.type-chip:hover {
		background: #FAF6F1;
	}

	.type-chip.selected {
		background: #6B8F71;
		border-color: #6B8F71;
		color: #FFFFFF;
	}

	.log-notes {
		width: 100%;
		padding: 8px 10px;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		font-size: 14px;
		font-family: inherit;
		resize: vertical;
		margin-bottom: 10px;
		box-sizing: border-box;
	}

	.log-notes:focus {
		outline: none;
		border-color: #6B8F71;
	}

	.log-actions {
		display: flex;
		gap: 8px;
	}

	.log-save {
		padding: 8px 20px;
		background: #6B8F71;
		color: #FFFFFF;
		border: none;
		border-radius: 8px;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
	}

	.log-save:hover:not(:disabled) {
		background: #4A6B4F;
	}

	.log-save:disabled {
		opacity: 0.5;
		cursor: not-allowed;
	}

	.log-cancel {
		padding: 8px 20px;
		background: none;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		color: #8C7E6E;
		font-size: 14px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
	}

	.log-cancel:hover {
		background: #FAF6F1;
	}

	@media (min-width: 1280px) {
		.detail {
			max-width: 960px;
		}

		.hero-icon {
			width: 100px;
			height: 100px;
		}

		.hero-photo {
			width: 100px;
			height: 100px;
		}
	}

	@media (max-width: 768px) {
		.plant-hero h1 {
			font-size: 22px;
		}

		.hero-icon {
			width: 60px;
			height: 60px;
		}

		.hero-photo {
			width: 60px;
			height: 60px;
		}

		.info-cards {
			grid-template-columns: 1fr;
		}

		.plant-hero {
			flex-direction: column;
			text-align: center;
			gap: 16px;
			margin-bottom: 24px;
		}

		.event-delete {
			opacity: 1;
		}
	}
</style>
