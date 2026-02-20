<script lang="ts">
	import { MapPin } from 'lucide-svelte';
	import type { Location } from '$lib/api';

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
			None
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
				placeholder="Location name"
				class="new-input"
			/>
			<button type="submit" class="chip chip-dashed">Add</button>
			<button type="button" class="chip" onclick={() => { adding = false; newName = ''; }}>
				Cancel
			</button>
		</form>
	{:else}
		<button type="button" class="chip chip-dashed" onclick={() => { adding = true; }}>
			+ New
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

	.new-input {
		padding: 8px 12px;
		border: 1px solid var(--color-border);
		border-radius: var(--radius-btn);
		font-size: var(--fs-chip);
		outline: none;
		width: 140px;
	}

	.new-input:focus {
		border-color: var(--color-primary);
	}

</style>
