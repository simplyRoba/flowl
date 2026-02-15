<script lang="ts">
	import { onMount } from 'svelte';
	import { Plus } from 'lucide-svelte';
	import { plants, plantsError, loadPlants } from '$lib/stores/plants';
	import { emojiToSvgPath } from '$lib/emoji';

	const BG_GRADIENTS = [
		'linear-gradient(135deg, #e8f5e9, #c8e6c9)',
		'linear-gradient(135deg, #e3f2fd, #bbdefb)',
		'linear-gradient(135deg, #fff8e1, #ffecb3)',
		'linear-gradient(135deg, #fce4ec, #f8bbd0)',
		'linear-gradient(135deg, #f3e5f5, #e1bee7)',
		'linear-gradient(135deg, #e0f2f1, #b2dfdb)',
	];

	function cardBg(id: number): string {
		return BG_GRADIENTS[id % BG_GRADIENTS.length];
	}

	onMount(() => {
		loadPlants();
	});
</script>

<div class="dashboard">
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

	.page-header {
		display: flex;
		align-items: center;
		justify-content: space-between;
		margin-bottom: 24px;
	}

	.page-header h1 {
		font-size: 28px;
		font-weight: 700;
		margin: 0;
	}

	.add-btn {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		padding: 10px 18px;
		background: #6B8F71;
		color: #fff;
		border: none;
		border-radius: 8px;
		font-size: 15px;
		font-weight: 500;
		text-decoration: none;
		cursor: pointer;
		transition: background 0.15s;
	}

	.add-btn:hover {
		background: #4A6B4F;
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
		color: #8C7E6E;
		margin: 0 0 24px;
	}

	.plant-grid {
		display: grid;
		grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
		gap: 16px;
	}

	.plant-card {
		background: #FFFFFF;
		border: 1px solid #E5DDD3;
		border-radius: 12px;
		overflow: hidden;
		text-decoration: none;
		color: inherit;
		cursor: pointer;
		transition: transform 0.15s, box-shadow 0.15s;
	}

	.plant-card:hover {
		transform: translateY(-2px);
		box-shadow: 0 4px 12px rgba(0, 0, 0, 0.06);
	}

	.plant-card-photo {
		height: 120px;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.plant-icon {
		width: 56px;
		height: 56px;
	}

	.photo-img {
		width: 100%;
		height: 100%;
		object-fit: cover;
	}

	.plant-card-body {
		padding: 12px 14px;
	}

	.plant-card-name {
		font-size: 15px;
		font-weight: 600;
		margin-bottom: 2px;
		white-space: nowrap;
		overflow: hidden;
		text-overflow: ellipsis;
	}

	.plant-card-location {
		font-size: 13px;
		color: #8C7E6E;
	}

	.error {
		color: #C45B5B;
		padding: 16px;
	}

	@media (max-width: 768px) {
		.page-header h1 {
			font-size: 22px;
		}

		.add-btn {
			padding: 8px 14px;
			font-size: 14px;
		}

		.plant-grid {
			grid-template-columns: repeat(2, 1fr);
			gap: 12px;
		}

		.plant-card-photo {
			height: 72px;
		}
	}
</style>
