<script lang="ts">
	import { onMount } from 'svelte';
	import { Droplet, Leaf, Shovel, Scissors, Pencil } from 'lucide-svelte';
	import type { CareEvent } from '$lib/api';
	import { fetchAllCareEvents } from '$lib/api';

	const PAGE_SIZE = 20;

	let events = $state<CareEvent[]>([]);
	let hasMore = $state(false);
	let loading = $state(false);
	let error = $state<string | null>(null);
	let activeFilter = $state('');
	let sentinel: HTMLElement;

	const FILTERS = [
		{ value: '', label: 'All' },
		{ value: 'watered', label: 'Watered' },
		{ value: 'fertilized', label: 'Fertilized' },
		{ value: 'repotted', label: 'Repotted' },
		{ value: 'pruned', label: 'Pruned' },
		{ value: 'custom', label: 'Custom' },
	];

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
			error = e instanceof Error ? e.message : 'Failed to load events';
		}
		loading = false;
	}

	function setFilter(value: string) {
		activeFilter = value;
		loadPage(true);
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

	function eventTypeLabel(type: string): string {
		if (type === 'watered') return 'Watered';
		if (type === 'fertilized') return 'Fertilized';
		if (type === 'repotted') return 'Repotted';
		if (type === 'pruned') return 'Pruned';
		return 'Custom';
	}

	function formatTime(dateStr: string): string {
		const date = new Date(dateStr);
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
		<h1>Care Journal</h1>
	</header>

	<div class="filter-chips">
		{#each FILTERS as filter}
			<button
				class="filter-chip"
				class:active={activeFilter === filter.value}
				onclick={() => setFilter(filter.value)}
			>
				{filter.label}
			</button>
		{/each}
	</div>

	{#if error}
		<p class="error">{error}</p>
	{:else if events.length === 0 && !loading}
		<div class="empty-state">
			<p>No care events recorded yet.</p>
		</div>
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
								<a href="/plants/{event.plant_id}" class="event-plant">{event.plant_name}</a>
								<span class="event-type">{eventTypeLabel(event.event_type)}</span>
								{#if event.notes}
									<p class="event-notes">{event.notes}</p>
								{/if}
							</div>
							<span class="event-time">{formatTime(event.occurred_at)}</span>
						</div>
					{/each}
				</div>
			{/each}
		</div>
	{/if}

	{#if loading}
		<p class="loading-text">Loading...</p>
	{/if}

	<div bind:this={sentinel} class="sentinel"></div>
</div>

<style>
	.log-page {
		max-width: 800px;
		margin: 0 auto;
	}

	.page-header {
		margin-bottom: 20px;
	}

	.page-header h1 {
		font-size: 28px;
		font-weight: 700;
		margin: 0;
	}

	.filter-chips {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
		margin-bottom: 24px;
	}

	.filter-chip {
		padding: 6px 16px;
		border: 1px solid #E5DDD3;
		border-radius: 16px;
		background: #FFFFFF;
		color: #8C7E6E;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s, border-color 0.15s, color 0.15s;
	}

	.filter-chip:hover {
		background: #FAF6F1;
	}

	.filter-chip.active {
		background: #6B8F71;
		border-color: #6B8F71;
		color: #FFFFFF;
	}

	.timeline {
		background: #FFFFFF;
		border: 1px solid #E5DDD3;
		border-radius: 12px;
		padding: 16px;
	}

	.day-group {
		margin-bottom: 20px;
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
		margin-bottom: 10px;
	}

	.timeline-event {
		display: flex;
		align-items: flex-start;
		gap: 10px;
		padding: 10px 0;
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

	.event-plant {
		font-size: 14px;
		font-weight: 600;
		color: #6B8F71;
		text-decoration: none;
	}

	.event-plant:hover {
		text-decoration: underline;
	}

	.event-type {
		font-size: 13px;
		color: #8C7E6E;
		margin-left: 8px;
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

	.empty-state {
		text-align: center;
		padding: 64px 24px;
		color: #8C7E6E;
	}

	.error {
		color: #C45B5B;
		padding: 16px;
	}

	.loading-text {
		text-align: center;
		color: #8C7E6E;
		padding: 16px;
		font-size: 14px;
	}

	.sentinel {
		height: 1px;
	}

	@media (min-width: 1280px) {
		.log-page {
			max-width: 960px;
		}
	}

	@media (max-width: 768px) {
		.page-header h1 {
			font-size: 22px;
		}

		.filter-chips {
			gap: 6px;
		}

		.filter-chip {
			padding: 5px 12px;
			font-size: 12px;
		}
	}
</style>
