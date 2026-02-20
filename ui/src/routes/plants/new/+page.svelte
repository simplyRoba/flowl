<script lang="ts">
	import { goto } from '$app/navigation';
	import type { CreatePlant } from '$lib/api';
	import { createPlant, uploadPhoto } from '$lib/stores/plants';
	import PlantForm from '$lib/components/PlantForm.svelte';
	import PageHeader from '$lib/components/PageHeader.svelte';

	let saving = $state(false);

	async function handleSave(data: CreatePlant, photo?: File) {
		saving = true;
		const plant = await createPlant(data);
		if (plant) {
			if (photo) {
				await uploadPhoto(plant.id, photo);
			}
			goto(`/plants/${plant.id}`);
		}
		saving = false;
	}
</script>

<div class="page">
	<PageHeader backHref="/" backLabel="Back">
		<button type="submit" form="plant-form" class="btn btn-primary" disabled={saving}>
			{saving ? 'Saving...' : 'Save'}
		</button>
	</PageHeader>

	<h1>Add Plant</h1>

	<PlantForm onsave={handleSave} {saving} showLocationNone={false} showFooterActions={false} />
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

	@media (max-width: 768px) {
		.page {
			padding-bottom: 64px;
		}

		h1 {
			margin-bottom: 16px;
		}
	}
</style>
