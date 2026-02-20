<script lang="ts">
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { page } from '$app/stores';
	import type { CreatePlant } from '$lib/api';
	import { currentPlant, plantsError, loadPlant, updatePlant, uploadPhoto, deletePhoto } from '$lib/stores/plants';
	import PlantForm from '$lib/components/PlantForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';

	let saving = $state(false);
	let loaded = $state(false);

	onMount(async () => {
		const id = Number($page.params.id);
		await loadPlant(id);
		loaded = true;
	});

	async function handleSave(data: CreatePlant, photo?: File) {
		if (!$currentPlant) return;
		saving = true;
		const plant = await updatePlant($currentPlant.id, data);
		if (plant) {
			if (photo) {
				await uploadPhoto(plant.id, photo);
			}
			goto(`/plants/${plant.id}`);
		}
		saving = false;
	}

	async function handleRemovePhoto() {
		if (!$currentPlant) return;
		await deletePhoto($currentPlant.id);
	}
</script>

<div class="page">
	<PageHeader backHref={$currentPlant ? `/plants/${$currentPlant.id}` : '/'} backLabel="Cancel">
		<button type="submit" form="plant-form" class="btn btn-primary" disabled={saving}>
			{saving ? 'Saving...' : 'Save'}
		</button>
	</PageHeader>

	<h1>Edit Plant</h1>

	{#if $plantsError}
		<p class="error">{$plantsError}</p>
	{:else if loaded && $currentPlant}
		<PlantForm initial={$currentPlant} onsave={handleSave} onremovephoto={handleRemovePhoto} {saving} showFooterActions={false} />
	{:else}
		<p class="loading">Loading...</p>
	{/if}
</div>

<style>
	.page {
		max-width: var(--content-width-narrow);
		margin: 0 auto;
	}

	h1 {
		font-size: var(--fs-page-title);
		font-weight: 700;
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

	@media (max-width: 768px) {
		.page {
			padding-bottom: 64px;
		}

		h1 {
			margin-bottom: 16px;
		}
	}
</style>
