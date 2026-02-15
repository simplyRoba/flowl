<script lang="ts">
	import { onMount } from 'svelte';
	import { Sun, CloudSun, Cloud } from 'lucide-svelte';
	import type { Plant, CreatePlant, Location } from '$lib/api';
	import { locations, loadLocations, createLocation } from '$lib/stores/locations';
	import IconPicker from './IconPicker.svelte';
	import LocationChips from './LocationChips.svelte';
	import WateringInterval from './WateringInterval.svelte';

	let {
		initial = null,
		onsave,
		saving = false
	}: {
		initial?: Plant | null;
		onsave: (data: CreatePlant) => void;
		saving?: boolean;
	} = $props();

	let name = $state('');
	let species = $state('');
	let icon = $state('🪴');
	let locationId = $state<number | null>(null);
	let wateringDays = $state(7);
	let lightNeeds = $state('indirect');
	let notes = $state('');
	let nameError = $state('');

	// Initialize/re-initialize from initial prop (for edit form)
	$effect(() => {
		if (initial) {
			name = initial.name;
			species = initial.species ?? '';
			icon = initial.icon;
			locationId = initial.location_id;
			wateringDays = initial.watering_interval_days;
			lightNeeds = initial.light_needs;
			notes = initial.notes ?? '';
		}
	});

	onMount(() => {
		loadLocations();
	});

	function handleSubmit(e: Event) {
		e.preventDefault();
		if (!name.trim()) {
			nameError = 'Name is required';
			return;
		}
		nameError = '';

		const data: CreatePlant = {
			name: name.trim(),
			icon
		};
		if (species.trim()) data.species = species.trim();
		data.location_id = locationId;
		data.watering_interval_days = wateringDays;
		data.light_needs = lightNeeds;
		if (notes.trim()) data.notes = notes.trim();

		onsave(data);
	}

	async function handleCreateLocation(locationName: string): Promise<Location | null> {
		return await createLocation(locationName);
	}
</script>

<form class="plant-form" onsubmit={handleSubmit}>
	<section class="form-section">
		<h3>Identity</h3>
		<label class="field">
			<span class="label">Name <span class="required">*</span></span>
			<input
				type="text"
				bind:value={name}
				placeholder="e.g. Monstera Deliciosa"
				class="input"
				class:error={nameError}
				oninput={() => { nameError = ''; }}
			/>
			{#if nameError}
				<span class="field-error">{nameError}</span>
			{/if}
		</label>

		<label class="field">
			<span class="label">Species</span>
			<input
				type="text"
				bind:value={species}
				placeholder="e.g. Monstera"
				class="input"
			/>
		</label>

		<div class="field">
			<span class="label">Icon</span>
			<IconPicker value={icon} onchange={(v) => { icon = v; }} />
		</div>
	</section>

	<section class="form-section">
		<h3>Location</h3>
		<LocationChips
			locations={$locations}
			value={locationId}
			onchange={(v) => { locationId = v; }}
			oncreate={handleCreateLocation}
		/>
	</section>

	<section class="form-section">
		<h3>Watering</h3>
		<WateringInterval value={wateringDays} onchange={(v) => { wateringDays = v; }} />
	</section>

	<section class="form-section">
		<h3>Light Needs</h3>
		<div class="light-options">
			<button
				type="button"
				class="light-option"
				class:selected={lightNeeds === 'direct'}
				onclick={() => { lightNeeds = 'direct'; }}
			>
				<Sun size={20} />
				<span class="light-label">Direct</span>
				<span class="light-desc">Full sun</span>
			</button>
			<button
				type="button"
				class="light-option"
				class:selected={lightNeeds === 'indirect'}
				onclick={() => { lightNeeds = 'indirect'; }}
			>
				<CloudSun size={20} />
				<span class="light-label">Indirect</span>
				<span class="light-desc">Filtered</span>
			</button>
			<button
				type="button"
				class="light-option"
				class:selected={lightNeeds === 'low'}
				onclick={() => { lightNeeds = 'low'; }}
			>
				<Cloud size={20} />
				<span class="light-label">Low</span>
				<span class="light-desc">Shade</span>
			</button>
		</div>
	</section>

	<section class="form-section">
		<h3>Notes</h3>
		<textarea
			bind:value={notes}
			placeholder="Care tips, observations..."
			class="input textarea"
			rows="4"
		></textarea>
	</section>

	<button type="submit" class="save-btn" disabled={saving}>
		{saving ? 'Saving...' : 'Save'}
	</button>
</form>

<style>
	.plant-form {
		display: flex;
		flex-direction: column;
		gap: 24px;
		max-width: 640px;
	}

	.form-section {
		background: #FFFFFF;
		border: 1px solid #E5DDD3;
		border-radius: 12px;
		padding: 20px;
	}

	.form-section h3 {
		font-size: 13px;
		font-weight: 600;
		color: #8C7E6E;
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin: 0 0 16px;
	}

	.field {
		display: flex;
		flex-direction: column;
		gap: 6px;
		margin-bottom: 16px;
	}

	.field:last-child {
		margin-bottom: 0;
	}

	.label {
		font-size: 14px;
		font-weight: 500;
	}

	.required {
		color: #C45B5B;
	}

	.input {
		padding: 10px 12px;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		font-size: 15px;
		font-family: inherit;
		outline: none;
		transition: border-color 0.15s;
	}

	.input:focus {
		border-color: #6B8F71;
	}

	.input.error {
		border-color: #C45B5B;
	}

	.textarea {
		width: 100%;
		resize: vertical;
		min-height: 80px;
	}

	.field-error {
		font-size: 13px;
		color: #C45B5B;
	}

	.light-options {
		display: flex;
		gap: 8px;
	}

	.light-option {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 16px 12px;
		border: 1px solid #E5DDD3;
		border-radius: 10px;
		background: #FFFFFF;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
		color: #2C2418;
	}

	.light-option:hover {
		border-color: #8C7E6E;
	}

	.light-option.selected {
		border-color: #6B8F71;
		background: #f0f7f1;
	}

	.light-label {
		font-size: 14px;
		font-weight: 600;
	}

	.light-desc {
		font-size: 12px;
		color: #8C7E6E;
	}

	.save-btn {
		padding: 12px 24px;
		background: #6B8F71;
		color: #fff;
		border: none;
		border-radius: 8px;
		font-size: 15px;
		font-weight: 500;
		cursor: pointer;
		transition: background 0.15s;
		align-self: flex-start;
	}

	.save-btn:hover:not(:disabled) {
		background: #4A6B4F;
	}

	.save-btn:disabled {
		opacity: 0.6;
		cursor: default;
	}

	@media (max-width: 768px) {
		.plant-form {
			gap: 16px;
		}

		.form-section {
			padding: 16px;
		}

		.light-options {
			gap: 6px;
		}

		.light-option {
			padding: 12px 8px;
		}

		.save-btn {
			align-self: stretch;
			text-align: center;
		}
	}
</style>
