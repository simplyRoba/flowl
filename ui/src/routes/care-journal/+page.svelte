<script lang="ts">
	import { onMount } from 'svelte';
	import { Droplet, Leaf, Shovel, Scissors, Pencil, Sparkles } from 'lucide-svelte';
	import type { CareEvent } from '$lib/api';
	import { fetchAllCareEvents } from '$lib/api';
	import { translations } from '$lib/stores/locale';
	import PhotoLightbox from '$lib/components/PhotoLightbox.svelte';

	let lightboxOpen = $state(false);
	let lightboxSrc = $state('');

	const PAGE_SIZE = 20;

	let events = $state<CareEvent[]>([]);
	let hasMore = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let activeFilter = $state('');
	let sentinel: HTMLElement;

	const FILTER_VALUES = ['', 'watered', 'fertilized', 'repotted', 'pruned', 'custom', 'ai-consultation'] as const;

	async function loadPage(reset = false) {
		if (loading) return;
		loading = true;
		error = null;
		const before = reset || events.length === 0
			? undefined
			: events[events.length - 1].id;
		const type = activeFilter || undefined;
		try {
			const page = await fetchAllCareEvents(PAGE_SIZE, before, type);
			if (reset) {
				events = page.events;
			} else {
				events = [...events, ...page.events];
			}
			hasMore = page.has_more;
		} catch (e) {
			error = e instanceof Error ? e.message : $translations.care.failedToLoad;
		}
		loading = false;
	}

	function setFilter(value: string) {
		activeFilter = value;
		loadPage(true);
	}

	function parseEventDate(dateStr: string): Date {
		const hasTimezone = /Z|[+-]\d{2}:\d{2}$/.test(dateStr);
		return new Date(hasTimezone ? dateStr : `${dateStr}Z`);
	}

	function dayLabel(dateStr: string): string {
		const date = parseEventDate(dateStr);
		if (isNaN(date.getTime())) return dateStr;
		const fullDate = date.toLocaleDateString(undefined, {
			year: 'numeric',
			month: 'short',
			day: 'numeric'
		});
		const today = new Date();
		today.setHours(0, 0, 0, 0);
		const yesterday = new Date(today);
		yesterday.setDate(yesterday.getDate() - 1);
		const eventDate = new Date(date);
		eventDate.setHours(0, 0, 0, 0);
		if (eventDate.getTime() === today.getTime()) return `${$translations.care.today} — ${fullDate}`;
		if (eventDate.getTime() === yesterday.getTime()) return `${$translations.care.yesterday} — ${fullDate}`;
		return fullDate;
	}

	function eventTypeLabel(type: string): string {
		if (type === 'watered') return $translations.care.watered;
		if (type === 'fertilized') return $translations.care.fertilized;
		if (type === 'repotted') return $translations.care.repotted;
		if (type === 'pruned') return $translations.care.pruned;
		if (type === 'ai-consultation') return $translations.care.aiConsultation;
		return $translations.care.custom;
	}

	function filterLabel(value: string): string {
		if (value === '') return $translations.care.filterAll;
		return eventTypeLabel(value);
	}

	function formatTime(dateStr: string): string {
		const date = parseEventDate(dateStr);
		if (isNaN(date.getTime())) return '';
		return date.toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' });
	}

	interface DayGroup {
		label: string;
		events: CareEvent[];
	}

	let groupedEvents: DayGroup[] = $derived.by(() => {
		const groups: DayGroup[] = [];
		let currentLabel = '';
		for (const event of events) {
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

	onMount(() => {
		loadPage(true);
	});

	$effect(() => {
		void events.length;
		if (!hasMore || !sentinel) return;

		const observer = new IntersectionObserver((entries) => {
			if (entries[0].isIntersecting && hasMore && !loading) {
				loadPage();
			}
		}, { rootMargin: '200px' });

		observer.observe(sentinel);
		return () => observer.disconnect();
	});
</script>

<div class="log-page">
	<header class="page-header">
		<h1>{$translations.care.title}</h1>
	</header>

	<div class="log-filters">
		{#each FILTER_VALUES as value}
			<button
				class="chip chip-solid"
				class:active={activeFilter === value}
				onclick={() => setFilter(value)}
				aria-label={filterLabel(value)}
			>
				{#if value === 'watered'}
					<Droplet size={14} />
				{:else if value === 'fertilized'}
					<Leaf size={14} />
				{:else if value === 'repotted'}
					<Shovel size={14} />
				{:else if value === 'pruned'}
					<Scissors size={14} />
				{:else if value === 'custom'}
					<Pencil size={14} />
				{:else if value === 'ai-consultation'}
					<Sparkles size={14} />
				{/if}
				<span class="filter-label" class:icon-has-label={value !== ''}>{filterLabel(value)}</span>
			</button>
		{/each}
	</div>

	{#if error}
		<p class="error">{error}</p>
	{:else if events.length === 0 && !loading}
		<div class="empty-state">
			<p>{$translations.care.noCareEvents}</p>
		</div>
	{:else}
		<div class="log-timeline">
			{#each groupedEvents as group}
				<div class="log-day-group">
					<div class="log-day-header">{group.label}</div>
					{#each group.events as event}
						<a href="/plants/{event.plant_id}?from=/care-journal" class="log-entry">
							<div class="log-entry-left">
								<div
									class="log-entry-icon
										{event.event_type === 'watered' ? 'water-icon' : ''}
										{event.event_type === 'fertilized' ? 'fertilize-icon' : ''}
										{event.event_type === 'repotted' ? 'repot-icon' : ''}
										{event.event_type === 'pruned' ? 'prune-icon' : ''}
										{event.event_type === 'custom' ? 'custom-icon' : ''}
									{event.event_type === 'ai-consultation' ? 'ai-icon' : ''}"
								>
									{#if event.event_type === 'watered'}
										<Droplet size={14} />
									{:else if event.event_type === 'fertilized'}
										<Leaf size={14} />
									{:else if event.event_type === 'repotted'}
										<Shovel size={14} />
									{:else if event.event_type === 'pruned'}
										<Scissors size={14} />
									{:else if event.event_type === 'ai-consultation'}
										<Sparkles size={14} />
									{:else}
										<Pencil size={14} />
									{/if}
								</div>
								<span class="log-entry-time">{formatTime(event.occurred_at)}</span>
							</div>
							<div class="log-entry-content">
								<span class="log-entry-plant">{event.plant_name}</span>
								{#if event.photo_url}
									<!-- svelte-ignore a11y_no_static_element_interactions -->
									<span
										class="log-entry-photo"
										role="button"
										tabindex="-1"
										onclick={(e) => { e.preventDefault(); e.stopPropagation(); lightboxSrc = event.photo_url!; lightboxOpen = true; }}
										onkeydown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); e.stopPropagation(); lightboxSrc = event.photo_url!; lightboxOpen = true; } }}
									>
										<img src={event.photo_url} alt="" />
									</span>
								{/if}
								<div class="log-entry-action">{eventTypeLabel(event.event_type)}</div>
								{#if event.notes}
									<div class="log-entry-note">{event.notes}</div>
								{/if}
							</div>
						</a>
					{/each}
				</div>
			{/each}
		</div>
	{/if}

	{#if loading}
		<p class="loading-text">{$translations.common.loading}</p>
	{/if}

	<div bind:this={sentinel} class="sentinel"></div>
</div>

<PhotoLightbox
	open={lightboxOpen}
	src={lightboxSrc}
	alt=""
	onclose={() => { lightboxOpen = false; }}
/>

<style>
	.log-page {
		max-width: var(--content-width-default);
		margin: 0 auto;
	}

	.page-header {
		margin-bottom: 16px;
	}

	.page-header h1 {
		font-size: var(--fs-page-title);
		font-weight: 700;
		margin: 0;
	}

	.log-filters {
		display: flex;
		gap: 6px;
		margin-bottom: 16px;
		flex-wrap: wrap;
	}

	.log-timeline {
		background: transparent;
		border: none;
		border-radius: 0;
		padding: 0;
	}

	.log-day-group {
		margin-bottom: 20px;
	}

	.log-day-group:last-child {
		margin-bottom: 0;
	}

	.log-day-header {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 8px;
		padding-bottom: 6px;
		border-bottom: 1px solid var(--color-border);
	}

	.log-entry {
		display: flex;
		gap: 12px;
		padding: 10px 0;
		border-bottom: 1px solid var(--color-border);
		align-items: flex-start;
		text-decoration: none;
		color: inherit;
	}

	.log-entry:last-child {
		border-bottom: none;
	}

	.log-entry-left {
		flex-shrink: 0;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
	}

	.log-entry-icon {
		width: 36px;
		height: 36px;
		border-radius: 10px;
		display: flex;
		align-items: center;
		justify-content: center;
		font-size: 16px;
		flex-shrink: 0;
		background: var(--color-surface-muted);
		color: var(--color-text-muted);
	}

	.log-entry-icon.water-icon { background: color-mix(in srgb, var(--color-water) 15%, transparent); }
	.log-entry-icon.fertilize-icon { background: color-mix(in srgb, var(--color-secondary) 15%, transparent); }
	.log-entry-icon.repot-icon { background: color-mix(in srgb, var(--color-success) 15%, transparent); }
	.log-entry-icon.prune-icon { background: color-mix(in srgb, var(--color-text-muted) 15%, transparent); }
	.log-entry-icon.custom-icon { background: color-mix(in srgb, var(--color-warning) 15%, transparent); }
	.log-entry-icon.ai-icon { background: color-mix(in srgb, var(--color-ai) 15%, transparent); }

	.log-entry-content {
		flex: 1;
		min-width: 0;
	}

	.log-entry-photo {
		float: right;
		display: block;
		width: 80px;
		height: 80px;
		border-radius: 8px;
		overflow: hidden;
		cursor: zoom-in;
		margin-left: 10px;
		margin-bottom: 4px;
	}

	.log-entry-photo img {
		width: 100%;
		height: 100%;
		object-fit: cover;
		display: block;
	}

	.log-entry-plant {
		font-size: 14px;
		font-weight: 600;
		color: var(--color-text);
	}

	.log-entry-time {
		font-size: 11px;
		color: var(--color-text-muted);
		white-space: nowrap;
	}

	.log-entry-action {
		font-size: 13px;
		color: var(--color-text-muted);
	}

	.log-entry-note {
		font-size: 13px;
		color: var(--color-text);
		margin-top: 4px;
		line-height: 1.4;
	}

	.empty-state {
		text-align: center;
		padding: 64px 24px;
		color: var(--color-text-muted);
	}

	.error {
		color: var(--color-danger);
		padding: 16px;
	}

	.loading-text {
		text-align: center;
		color: var(--color-text-muted);
		padding: 16px;
		font-size: 14px;
	}

	.sentinel {
		height: 1px;
	}

	@media (max-width: 768px) {
		.page-header h1 {
			font-size: 18px;
		}

		.filter-label.icon-has-label {
			display: none;
		}
	}
</style>
