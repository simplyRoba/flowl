<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import { ArrowLeft } from 'lucide-svelte';
	import type { CreatePlant } from '$lib/api';
	import { currentPlant, plantsError, loadPlant, updatePlant } from '$lib/stores/plants';
	import PlantForm from '$lib/components/PlantForm.svelte';

	let saving = $state(false);
	let loaded = $state(false);

	onMount(async () => {
		const id = Number($page.params.id);
		await loadPlant(id);
		loaded = true;
	});

	async function handleSave(data: CreatePlant) {
		if (!$currentPlant) return;
		saving = true;
		const plant = await updatePlant($currentPlant.id, data);
		if (plant) {
			goto(`/plants/${plant.id}`);
		}
		saving = false;
	}
</script>

<div class="page">
	<header class="page-header">
		<a href={$currentPlant ? `/plants/${$currentPlant.id}` : '/'} class="back-link">
			<ArrowLeft size={18} /> Cancel
		</a>
		<h1>Edit Plant</h1>
	</header>

	{#if $plantsError}
		<p class="error">{$plantsError}</p>
	{:else if loaded && $currentPlant}
		<PlantForm initial={$currentPlant} onsave={handleSave} {saving} />
	{:else}
		<p class="loading">Loading...</p>
	{/if}
</div>

<style>
	.page {
		max-width: 800px;
		margin: 0 auto;
	}

	.page-header {
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
		margin-bottom: 8px;
	}

	.back-link:hover {
		color: #4A6B4F;
	}

	h1 {
		font-size: 28px;
		font-weight: 700;
		margin: 0;
	}

	.error {
		color: #C45B5B;
		padding: 16px;
	}

	.loading {
		color: #8C7E6E;
		padding: 16px;
	}
</style>
