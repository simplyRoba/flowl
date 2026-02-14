<script lang="ts">
	import { Plus } from 'lucide-svelte';
	import type { Location } from '$lib/api';

	let {
		locations,
		value = null,
		onchange,
		oncreate
	}: {
		locations: Location[];
		value: number | null;
		onchange: (id: number | null) => void;
		oncreate?: (name: string) => Promise<Location | null>;
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
	<button
		type="button"
		class="chip"
		class:selected={value === null}
		onclick={() => onchange(null)}
	>
		None
	</button>
	{#each locations as loc (loc.id)}
		<button
			type="button"
			class="chip"
			class:selected={value === loc.id}
			onclick={() => onchange(loc.id)}
		>
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
			<button type="submit" class="chip new-chip">Add</button>
			<button type="button" class="chip" onclick={() => { adding = false; newName = ''; }}>
				Cancel
			</button>
		</form>
	{:else}
		<button type="button" class="chip new-chip" onclick={() => { adding = true; }}>
			<Plus size={14} /> New
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
		border: 1px solid #E5DDD3;
		border-radius: 999px;
		background: #FFFFFF;
		font-size: 14px;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
		white-space: nowrap;
	}

	.chip:hover {
		border-color: #8C7E6E;
	}

	.chip.selected {
		border-color: #6B8F71;
		background: #f0f7f1;
		color: #4A6B4F;
		font-weight: 500;
	}

	.new-chip {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		color: #6B8F71;
		border-style: dashed;
	}

	.new-location {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	.new-input {
		padding: 8px 12px;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		font-size: 14px;
		outline: none;
		width: 140px;
	}

	.new-input:focus {
		border-color: #6B8F71;
	}
</style>
