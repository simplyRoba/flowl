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
			class="location-chip"
			class:active={value === null}
			onclick={() => onchange(null)}
		>
			None
		</button>
	{/if}
	{#each locations as loc (loc.id)}
		<button
			type="button"
			class="location-chip"
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
			<button type="submit" class="location-chip add-location">Add</button>
			<button type="button" class="location-chip" onclick={() => { adding = false; newName = ''; }}>
				Cancel
			</button>
		</form>
	{:else}
		<button type="button" class="location-chip add-location" onclick={() => { adding = true; }}>
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

	.location-chip {
		padding: 8px 14px;
		border: 1px solid #E5DDD3;
		border-radius: 999px;
		background: #FFFFFF;
		font-size: 13px;
		font-weight: 500;
		cursor: pointer;
		transition: all 0.15s;
		white-space: nowrap;
		display: inline-flex;
		align-items: center;
		gap: 5px;
		color: #2C2418;
	}

	.location-chip:hover {
		border-color: #6B8F71;
	}

	.location-chip.active {
		border-color: #6B8F71;
		background: color-mix(in srgb, #6B8F71 10%, transparent);
		color: #6B8F71;
	}

	.location-chip.add-location {
		border-style: dashed;
		color: #8C7E6E;
	}

	.location-chip.add-location:hover {
		border-color: #6B8F71;
		color: #6B8F71;
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
		font-size: 13px;
		outline: none;
		width: 140px;
	}

	.new-input:focus {
		border-color: #6B8F71;
	}

	.chip-icon {
		width: 14px;
		height: 14px;
	}
</style>
