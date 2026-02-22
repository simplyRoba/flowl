<script lang="ts">
	import { MapPin } from 'lucide-svelte';
	import type { Location } from '$lib/api';
	import { translations } from '$lib/stores/locale';

	let {
		locations,
		value = null,
		onchange,
		oncreate,
		showNone = true
	}: {
		locations: Location[];
		value: number | null;
		onchange: (id: number | null) => void;
		oncreate?: (name: string) => Promise<Location | null>;
		showNone?: boolean;
	} = $props();

	let adding = $state(false);
	let newName = $state('');

	async function handleCreate() {
		if (!newName.trim() || !oncreate) return;
		const loc = await oncreate(newName.trim());
		if (loc) {
			onchange(loc.id);
			newName = '';
			adding = false;
		}
	}
</script>

<div class="location-chips">
	{#if showNone}
		<button
			type="button"
			class="chip"
			class:active={value === null}
			onclick={() => onchange(null)}
		>
			{$translations.form.none}
		</button>
	{/if}
	{#each locations as loc (loc.id)}
		<button
			type="button"
			class="chip"
			class:active={value === loc.id}
			onclick={() => onchange(loc.id)}
		>
			<MapPin size={14} class="chip-icon" />
			{loc.name}
		</button>
	{/each}
	{#if adding}
		<form class="new-location" onsubmit={(e) => { e.preventDefault(); handleCreate(); }}>
			<input
				type="text"
				bind:value={newName}
				placeholder={$translations.form.locationName}
				class="input new-input"
			/>
			<button type="submit" class="chip add-btn">{$translations.form.add}</button>
			<button type="button" class="chip" onclick={() => { adding = false; newName = ''; }}>
				{$translations.common.cancel}
			</button>
		</form>
	{:else}
		<button type="button" class="chip chip-dashed" onclick={() => { adding = true; }}>
			{$translations.form.newLocation}
		</button>
	{/if}
</div>

<style>
	.location-chips {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		align-items: center;
	}

	.chip {
		padding: 8px 14px;
	}

	.new-location {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.add-btn {
		background: var(--color-primary);
		color: var(--color-text-on-primary);
		border-color: var(--color-primary);
	}

	.add-btn:hover {
		background: var(--color-primary-dark);
		border-color: var(--color-primary-dark);
	}

	.new-input {
		width: 140px;
		padding: 8px 14px;
		font-size: var(--fs-chip);
		border-radius: var(--radius-pill);
	}

</style>
