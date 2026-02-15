<script lang="ts">
	import { onMount } from 'svelte';
	import { Trash2 } from 'lucide-svelte';
	import { locations, locationsError, loadLocations, deleteLocation } from '$lib/stores/locations';

	onMount(() => {
		loadLocations();
	});

	async function handleDelete(id: number, name: string, plantCount: number) {
		const message = plantCount > 0
			? `Delete "${name}"? ${plantCount} plant${plantCount === 1 ? '' : 's'} will lose ${plantCount === 1 ? 'its' : 'their'} location.`
			: `Delete "${name}"?`;

		if (!confirm(message)) return;
		await deleteLocation(id);
	}
</script>

<div class="page">
	<header class="page-header">
		<h1>Settings</h1>
	</header>

	<section class="settings-card">
		<h2>Locations</h2>

		{#if $locationsError}
			<p class="error">{$locationsError}</p>
		{:else if $locations.length === 0}
			<p class="empty">No locations yet. Create locations when adding plants.</p>
		{:else}
			<ul class="location-list">
				{#each $locations as location (location.id)}
					<li class="location-item">
						<div class="location-info">
							<span class="location-name">{location.name}</span>
							{#if location.plant_count > 0}
								<span class="plant-count">{location.plant_count} plant{location.plant_count === 1 ? '' : 's'}</span>
							{/if}
						</div>
						<button
							class="delete-btn"
							onclick={() => handleDelete(location.id, location.name, location.plant_count)}
						>
							<Trash2 size={16} />
						</button>
					</li>
				{/each}
			</ul>
		{/if}
	</section>
</div>

<style>
	.page {
		max-width: 800px;
		margin: 0 auto;
	}

	.page-header {
		margin-bottom: 24px;
	}

	.page-header h1 {
		font-size: 28px;
		font-weight: 700;
		margin: 0;
	}

	.settings-card {
		background: #FFFFFF;
		border: 1px solid #E5DDD3;
		border-radius: 12px;
		padding: 20px;
	}

	.settings-card h2 {
		font-size: 13px;
		font-weight: 600;
		color: #8C7E6E;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin: 0 0 16px;
	}

	.empty {
		color: #8C7E6E;
		font-size: 14px;
		margin: 0;
	}

	.error {
		color: #C45B5B;
		font-size: 14px;
		margin: 0;
	}

	.location-list {
		list-style: none;
		padding: 0;
		margin: 0;
	}

	.location-item {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 12px 0;
		border-bottom: 1px solid #E5DDD3;
	}

	.location-item:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.location-item:first-child {
		padding-top: 0;
	}

	.location-info {
		display: flex;
		align-items: center;
		gap: 10px;
	}

	.location-name {
		font-size: 15px;
		font-weight: 500;
	}

	.plant-count {
		font-size: 12px;
		color: #8C7E6E;
		background: #FAF6F1;
		padding: 2px 8px;
		border-radius: 10px;
		border: 1px solid #E5DDD3;
	}

	.delete-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 8px;
		border: 1px solid #E5DDD3;
		background: #FFFFFF;
		color: #8C7E6E;
		cursor: pointer;
		transition: background 0.15s, color 0.15s, border-color 0.15s;
	}

	.delete-btn:hover {
		color: #C45B5B;
		border-color: #C45B5B;
		background: #fef2f2;
	}

	@media (max-width: 768px) {
		.page-header h1 {
			font-size: 22px;
		}

		.settings-card {
			padding: 16px;
		}
	}
</style>
