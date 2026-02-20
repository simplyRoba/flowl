<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus, TriangleAlert, Droplet } from 'lucide-svelte';
	import { plants, plantsError, loadPlants, waterPlant } from '$lib/stores/plants';
	import { emojiToSvgPath } from '$lib/emoji';
	import StatusBadge from '$lib/components/StatusBadge.svelte';

	const GREETINGS: Record<string, string[]> = {
		morning: [
			'Good morning!',
			'Rise and shine!',
			'Morning, plant parent!',
			'Wakey wakey, leaves and stems!',
			'Top of the morning!',
		],
		afternoon: [
			'Good afternoon!',
			'Afternoon check-in!',
			'Hope lunch was good!',
			'Post-lunch plant patrol!',
			'Sun\'s still up, so are your plants!',
		],
		evening: [
			'Good evening!',
			'Evening wind-down!',
			'Golden hour for greens!',
			'Almost bedtime for the ferns!',
			'Night shift plant check!',
		],
		night: [
			'Still up?',
			'Burning the midnight oil?',
			'The plants are sleeping, are you?',
			'Late night leaf gazing!',
			'Shh... the succulents are dreaming!',
		],
	};

	const SUBTITLES: Record<string, string[]> = {
		morning: [
			'Your plants had their morning dew.',
			'Time to check on the green crew.',
			'Coffee first, then watering.',
			'The early bird waters the plant.',
		],
		afternoon: [
			'How\'s the garden doing?',
			'Perfect time for a leaf inspection.',
			'Your plants missed you since morning.',
			'Sunshine report: looking leafy.',
		],
		evening: [
			'One last look before the day ends.',
			'Tuck your plants in for the night.',
			'Sunset vibes and happy leaves.',
			'The plants say goodnight soon.',
		],
		night: [
			'Your plants are on autopilot.',
			'Nothing to water at this hour... probably.',
			'Even owls check on their plants.',
			'Moonlight gardening, very cool.',
		],
	};

	const ATTENTION_SUBTITLES_PLURAL = [
		(n: number) => `${n} plants are thirsty today.`,
		(n: number) => `${n} plants could use a drink.`,
		(n: number) => `${n} plants are waiting for water.`,
		(n: number) => `Your plants are calling — ${n} need water.`,
		(n: number) => `Time to hydrate! ${n} plants are due.`,
	];

	const ATTENTION_SUBTITLES_SINGULAR = [
		'1 plant is thirsty today.',
		'1 plant could use a drink.',
		'1 plant is waiting for water.',
		'Your plant is calling — it needs water.',
		'Time to hydrate! 1 plant is due.',
	];

	function getTimeOfDay(): string {
		const hour = new Date().getHours();
		if (hour >= 5 && hour < 12) return 'morning';
		if (hour >= 12 && hour < 17) return 'afternoon';
		if (hour >= 17 && hour < 22) return 'evening';
		return 'night';
	}

	function pick<T>(arr: T[]): T {
		return arr[Math.floor(Math.random() * arr.length)];
	}

	const timeOfDay = getTimeOfDay();
	const greeting = pick(GREETINGS[timeOfDay]);
	const defaultSubtitle = pick(SUBTITLES[timeOfDay]);
	const attentionMsgIndex = Math.floor(Math.random() * ATTENTION_SUBTITLES_PLURAL.length);

	let attentionPlants = $derived(
		$plants
			.filter((p) => p.watering_status === 'overdue' || p.watering_status === 'due')
			.sort((a, b) => {
				if (a.watering_status === 'overdue' && b.watering_status !== 'overdue') return -1;
				if (a.watering_status !== 'overdue' && b.watering_status === 'overdue') return 1;
				return 0;
			})
	);

	let subtitle = $derived(
		attentionPlants.length === 0
			? defaultSubtitle
			: attentionPlants.length === 1
				? ATTENTION_SUBTITLES_SINGULAR[attentionMsgIndex]
				: ATTENTION_SUBTITLES_PLURAL[attentionMsgIndex](attentionPlants.length)
	);

	let wateringIds: Set<number> = $state(new Set());

	async function handleWater(plantId: number) {
		wateringIds = new Set([...wateringIds, plantId]);
		await waterPlant(plantId);
		wateringIds = new Set([...wateringIds].filter((id) => id !== plantId));
	}

	const BG_GRADIENTS = [
		'linear-gradient(135deg, color-mix(in srgb, var(--color-success) 35%, transparent), color-mix(in srgb, var(--color-success) 15%, transparent))',
		'linear-gradient(135deg, color-mix(in srgb, var(--color-water) 35%, transparent), color-mix(in srgb, var(--color-water) 15%, transparent))',
		'linear-gradient(135deg, color-mix(in srgb, var(--color-warning) 35%, transparent), color-mix(in srgb, var(--color-warning) 15%, transparent))',
		'linear-gradient(135deg, color-mix(in srgb, var(--color-secondary) 30%, transparent), color-mix(in srgb, var(--color-secondary) 12%, transparent))',
		'linear-gradient(135deg, color-mix(in srgb, var(--color-primary) 35%, transparent), color-mix(in srgb, var(--color-primary) 15%, transparent))',
		'linear-gradient(135deg, color-mix(in srgb, var(--color-text-muted) 25%, transparent), color-mix(in srgb, var(--color-text-muted) 10%, transparent))',
	];

	function cardBg(id: number): string {
		return BG_GRADIENTS[id % BG_GRADIENTS.length];
	}

	onMount(() => {
		loadPlants();
	});
</script>

<div class="dashboard">
	<div class="greeting">
		<h2>{greeting}</h2>
		<p>{subtitle}</p>
	</div>

	{#if attentionPlants.length > 0}
		<div class="attention-section">
			<div class="attention-title">
				<TriangleAlert size={16} />
				Needs Attention
			</div>
			<div class="attention-cards">
				{#each attentionPlants as plant (plant.id)}
					<a href="/plants/{plant.id}" class="attention-card">
						<div class="attention-card-accent" class:accent-overdue={plant.watering_status === 'overdue'} class:accent-due={plant.watering_status === 'due'}></div>
						{#if plant.photo_url}
							<div class="attention-card-photo">
								<img src={plant.photo_url} alt={plant.name} class="attention-photo-img" />
							</div>
						{:else}
							<div class="attention-card-photo attention-card-photo-emoji" style:background={cardBg(plant.id)}>
								<img src={emojiToSvgPath(plant.icon)} alt={plant.name} class="attention-icon" />
							</div>
						{/if}
						<div class="attention-card-body">
							<div class="attention-card-name">{plant.name}</div>
							{#if plant.location_name}
								<span class="attention-card-location">{plant.location_name}</span>
							{/if}
							<StatusBadge status={plant.watering_status} nextDue={plant.next_due ?? null} />
							<div class="attention-card-actions">
								<button
									class="btn btn-water btn-sm"
									disabled={wateringIds.has(plant.id)}
									onclick={(e) => { e.preventDefault(); handleWater(plant.id); }}
								>
									<Droplet size={16} />
									<span class="water-btn-label">
										{wateringIds.has(plant.id) ? 'Watering...' : 'Water'}
									</span>
								</button>
							</div>
						</div>
					</a>
				{/each}
			</div>
		</div>
	{/if}

	<header class="page-header">
		<h1>My Plants</h1>
		{#if $plants.length > 0}
			<a href="/plants/new" class="btn btn-primary btn-sm">
				<Plus size={18} />
				Add Plant
			</a>
		{/if}
	</header>

	{#if $plantsError}
		<p class="error">{$plantsError}</p>
	{:else if $plants.length === 0}
		<div class="empty-state">
			<img src={emojiToSvgPath('🪴')} alt="Plant" class="empty-icon" />
			<h2>No plants yet</h2>
			<p>Add your first plant to get started.</p>
			<a href="/plants/new" class="btn btn-primary btn-sm">
				<Plus size={18} />
				Add Plant
			</a>
		</div>
	{:else}
		<div class="plant-grid">
			{#each $plants as plant (plant.id)}
				<a href="/plants/{plant.id}" class="plant-card">
					{#if plant.photo_url}
						<div class="plant-card-photo">
							<img src={plant.photo_url} alt={plant.name} class="photo-img" />
						</div>
					{:else}
						<div class="plant-card-photo" style:background={cardBg(plant.id)}>
							<img src={emojiToSvgPath(plant.icon)} alt={plant.name} class="plant-icon" />
						</div>
					{/if}
					<div class="plant-card-body">
						<div class="plant-card-name">{plant.name}</div>
						{#if plant.location_name}
							<div class="plant-card-location">{plant.location_name}</div>
						{/if}
						<div class="plant-card-footer">
							<StatusBadge status={plant.watering_status} nextDue={plant.next_due ?? null} />
						</div>
					</div>
				</a>
			{/each}
		</div>
	{/if}
</div>

<style>
	.dashboard {
		max-width: var(--content-width-wide);
		margin: 0 auto;
	}

	.greeting {
		margin-bottom: 20px;
	}

	.greeting h2 {
		font-size: var(--fs-page-title);
		font-weight: 600;
		margin: 0 0 4px;
	}

	.greeting p {
		font-size: 14px;
		color: var(--color-text-muted);
		margin: 0;
	}

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
	}

	.page-header h1 {
		font-size: var(--fs-page-title);
		font-weight: 700;
		margin: 0;
	}


	.empty-state {
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		padding: 64px 24px;
		text-align: center;
	}

	.empty-icon {
		width: 64px;
		height: 64px;
		margin-bottom: 16px;
	}

	.empty-state h2 {
		font-size: var(--fs-page-title);
		font-weight: 600;
		margin: 0 0 8px;
	}

	.empty-state p {
		color: var(--color-text-muted);
		margin: 0 0 24px;
	}

	.plant-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(240px, 1fr));
		gap: 16px;
	}

	.plant-card {
		position: relative;
		border: none;
		border-radius: var(--radius-card);
		overflow: hidden;
		text-decoration: none;
		color: inherit;
		cursor: pointer;
		transition: transform var(--transition-speed), box-shadow var(--transition-speed);
	}

	.plant-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 8px 24px rgba(0, 0, 0, 0.12);
	}

	.plant-card-photo {
		height: 180px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.plant-icon {
		width: 64px;
		height: 64px;
	}

	.photo-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.plant-card-body {
		position: absolute;
		bottom: 0;
		left: 0;
		right: 0;
		padding: 28px 12px 10px;
		background: linear-gradient(to top, rgba(0, 0, 0, 0.55), transparent);
		border-radius: 0 0 var(--radius-card) var(--radius-card);
	}

	.plant-card-name {
		font-size: 14px;
		font-weight: 600;
		margin-bottom: 6px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
		color: #fff;
		text-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
	}

	.plant-card-location {
		font-size: 12px;
		color: rgba(255, 255, 255, 0.85);
		margin-bottom: 6px;
	}

	.plant-card-footer {
		display: flex;
		justify-content: space-between;
		align-items: center;
	}

	.error {
		color: var(--color-danger);
		padding: 16px;
	}

	/* Needs Attention section */
	.attention-section {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-card);
		padding: 16px;
		margin-bottom: 24px;
	}

	.attention-title {
		font-size: 13px;
		font-weight: 600;
		color: var(--color-secondary);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin-bottom: 12px;
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.attention-cards {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
		gap: 12px;
	}

	.attention-card {
		display: flex;
		align-items: stretch;
		border-radius: var(--radius-card);
		overflow: hidden;
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		text-decoration: none;
		color: inherit;
		cursor: pointer;
		transition: transform var(--transition-speed), box-shadow var(--transition-speed);
	}

	.attention-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
	}

	.attention-card-accent {
		width: 4px;
		flex-shrink: 0;
	}

	.accent-overdue {
		background: var(--color-danger);
	}

	.accent-due {
		background: var(--color-warning);
	}

	.attention-card-photo {
		width: 120px;
		flex-shrink: 0;
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
	}

	.attention-photo-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.attention-icon {
		width: 48px;
		height: 48px;
	}

	.attention-card-body {
		flex: 1;
		padding: 12px 14px;
		display: flex;
		flex-direction: column;
		align-items: flex-start;
		justify-content: center;
		gap: 4px;
		min-width: 0;
	}

	.attention-card-name {
		font-size: 15px;
		font-weight: 600;
		color: var(--color-text);
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.attention-card-location {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.attention-card-actions {
		display: flex;
		align-items: flex-end;
		align-self: stretch;
		justify-content: flex-end;
		margin-top: auto;
	}


	@media (min-width: 1280px) {
		.attention-cards {
			grid-template-columns: repeat(auto-fill, minmax(380px, 1fr));
		}

		.plant-grid {
			grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
			gap: 20px;
		}

		.plant-card-photo {
			height: 240px;
		}

		.plant-card-name {
			font-size: 16px;
		}

		.plant-card-location {
			font-size: 13px;
		}

		.plant-icon {
			width: 80px;
			height: 80px;
		}
	}

	@media (max-width: 768px) {
		.greeting h2 {
			font-size: 18px;
		}

		.water-btn-label {
			display: none;
		}

		.plant-grid {
			grid-template-columns: 1fr;
			gap: 12px;
		}

		.plant-card-name {
			font-size: 15px;
		}

		.plant-card-location {
			font-size: 13px;
		}
	}
</style>
