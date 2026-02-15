<script lang="ts">
	import { goto } from '$app/navigation';
	import { ArrowLeft } from 'lucide-svelte';
	import type { CreatePlant } from '$lib/api';
	import { createPlant } from '$lib/stores/plants';
	import PlantForm from '$lib/components/PlantForm.svelte';

	let saving = $state(false);

	async function handleSave(data: CreatePlant) {
		saving = true;
		const plant = await createPlant(data);
		if (plant) {
			goto(`/plants/${plant.id}`);
		}
		saving = false;
	}
</script>

<div class="page">
	<header class="page-header">
		<a href="/" class="back-link"><ArrowLeft size={18} /> Back</a>
		<h1>Add Plant</h1>
	</header>

	<PlantForm onsave={handleSave} {saving} />
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

	@media (max-width: 768px) {
		h1 {
			font-size: 22px;
		}

		.page-header {
			margin-bottom: 16px;
		}
	}
</style>
