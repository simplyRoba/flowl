<script lang="ts">
	import { onMount } from 'svelte';
	import { Trash2 } from 'lucide-svelte';
	import { locations, locationsError, loadLocations, deleteLocation } from '$lib/stores/locations';
	import {
		themePreference,
		setThemePreference,
		type ThemePreference
	} from '$lib/stores/theme';

	const themeOptions: { value: ThemePreference; label: string }[] = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' }
	];

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
		<h2>Appearance</h2>
		<div class="setting-row">
			<div>
				<div class="setting-label">Theme</div>
			</div>
			<div class="theme-selector" role="radiogroup" aria-label="Theme">
				{#each themeOptions as option}
					<button
						type="button"
						class="theme-option"
						class:active={$themePreference === option.value}
						aria-pressed={$themePreference === option.value}
						onclick={() => setThemePreference(option.value)}
					>
						{option.label}
					</button>
				{/each}
			</div>
		</div>
	</section>

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
		max-width: var(--content-width-default);
		margin: 0 auto;
	}

	.page-header {
		margin-bottom: 24px;
	}

	.page-header h1 {
		font-size: var(--fs-page-title);
		font-weight: 700;
		margin: 0;
	}

	.settings-card {
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		border-radius: var(--radius-card);
		padding: 20px;
		margin-bottom: 20px;
	}

	.settings-card h2 {
		font-size: var(--fs-chip);
		font-weight: 600;
		color: var(--color-text-muted);
		text-transform: uppercase;
		letter-spacing: 0.5px;
		margin: 0 0 16px;
	}

	.setting-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		gap: 16px;
		flex-wrap: wrap;
	}

	.setting-label {
		font-size: 15px;
		font-weight: 600;
	}

	.setting-hint {
		font-size: 13px;
		color: var(--color-text-muted);
		margin-top: 4px;
	}

	.theme-selector {
		display: inline-flex;
		padding: 4px;
		border-radius: var(--radius-pill);
		border: 1px solid var(--color-border);
		background: var(--color-surface-muted);
		gap: 4px;
	}

	.theme-option {
		border: none;
		background: transparent;
		color: var(--color-text-muted);
		padding: 6px 14px;
		border-radius: var(--radius-pill);
		font-size: var(--fs-chip);
		font-weight: 600;
		cursor: pointer;
		transition: background var(--transition-speed), color var(--transition-speed);
	}

	.theme-option:hover {
		color: var(--color-text);
		background: color-mix(in srgb, var(--color-primary) 10%, transparent);
	}

	.theme-option.active {
		background: var(--color-primary);
		color: var(--color-text-on-primary);
	}

	.empty {
		color: var(--color-text-muted);
		font-size: 14px;
		margin: 0;
	}

	.error {
		color: var(--color-danger);
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
		border-bottom: 1px solid var(--color-border);
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
		color: var(--color-text-muted);
		background: var(--color-surface-muted);
		padding: 2px 8px;
		border-radius: 10px;
		border: 1px solid var(--color-border);
	}

	.delete-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: var(--radius-btn);
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		transition: background var(--transition-speed), color var(--transition-speed), border-color var(--transition-speed);
	}

	.delete-btn:hover {
		color: var(--color-danger);
		border-color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 10%, transparent);
	}

	@media (max-width: 768px) {
		.settings-card {
			padding: 16px;
		}
	}
</style>
