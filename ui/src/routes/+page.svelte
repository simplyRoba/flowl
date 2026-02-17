<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus } from 'lucide-svelte';
	import { plants, plantsError, loadPlants } from '$lib/stores/plants';
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
	const subtitle = pick(SUBTITLES[timeOfDay]);

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

	<header class="page-header">
		<h1>My Plants</h1>
		{#if $plants.length > 0}
			<a href="/plants/new" class="add-btn">
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
			<a href="/plants/new" class="add-btn">
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
		max-width: 1200px;
		margin: 0 auto;
	}

	.greeting {
		margin-bottom: 20px;
	}

	.greeting h2 {
		font-size: 22px;
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
		font-size: 22px;
		font-weight: 700;
		margin: 0;
	}

	.add-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		background: var(--color-primary);
		color: var(--color-text-on-primary);
		border: none;
		border-radius: 8px;
		font-size: 15px;
		font-weight: 500;
		text-decoration: none;
		cursor: pointer;
		transition: background 0.15s;
	}

	.add-btn:hover {
		background: var(--color-primary-dark);
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
		font-size: 22px;
		font-weight: 600;
		margin: 0 0 8px;
	}

	.empty-state p {
		color: var(--color-text-muted);
		margin: 0 0 24px;
	}

	.plant-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 16px;
	}

	.plant-card {
		position: relative;
		border: none;
		border-radius: 12px;
		overflow: hidden;
		text-decoration: none;
		color: inherit;
		cursor: pointer;
		transition: transform 0.15s, box-shadow 0.15s;
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
		border-radius: 0 0 12px 12px;
	}

	.plant-card-name {
		font-size: 14px;
		font-weight: 600;
		margin-bottom: 2px;
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

	@media (min-width: 1280px) {
		.dashboard {
			max-width: 1400px;
		}

		.plant-grid {
			grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
			gap: 20px;
		}

		.plant-card-photo {
			height: 240px;
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

		.add-btn {
			padding: 8px 14px;
			font-size: 14px;
		}

		.plant-grid {
			grid-template-columns: repeat(2, 1fr);
			gap: 12px;
		}
	}
</style>
