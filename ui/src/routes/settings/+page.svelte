<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { Check, Pencil, Trash2, Palette, MapPin, Database, Info, Radio, Download, Upload, Wrench } from 'lucide-svelte';
	import { locations, locationsError, loadLocations, deleteLocation, updateLocation } from '$lib/stores/locations';
	import {
		themePreference,
		setThemePreference,
		type ThemePreference
	} from '$lib/stores/theme';
	import { fetchAppInfo, fetchStats, fetchMqttStatus, repairMqtt, importData, type AppInfo, type Stats, type MqttStatus } from '$lib/api';
	import ModalDialog from '$lib/components/ModalDialog.svelte';

	const themeOptions: { value: ThemePreference; label: string }[] = [
		{ value: 'light', label: 'Light' },
		{ value: 'dark', label: 'Dark' },
		{ value: 'system', label: 'System' }
	];

	let editingId: number | null = $state(null);
	let editValue = $state('');
	let editError = $state('');
	let appInfo: AppInfo | null = $state(null);
	let stats: Stats | null = $state(null);
	let mqttStatus: MqttStatus | null = $state(null);
	let repairLoading = $state(false);
	let repairMessage = $state('');
	let repairError = $state('');
	let importLoading = $state(false);
	let importMessage = $state('');
	let importError = $state('');
	let fileInput: HTMLInputElement = $state() as HTMLInputElement;

	// Dialog state
	let deleteDialogOpen = $state(false);
	let deleteTarget: { id: number; name: string; plantCount: number } | null = $state(null);
	let importDialogOpen = $state(false);
	let importFile: File | null = $state(null);
	let repairDialogOpen = $state(false);

	onMount(() => {
		loadLocations();
		fetchAppInfo()
			.then((info) => { appInfo = info; })
			.catch(() => { /* hide About section on failure */ });
		fetchStats()
			.then((s) => { stats = s; })
			.catch(() => { /* hide Data section on failure */ });
		fetchMqttStatus()
			.then((m) => { mqttStatus = m; })
			.catch(() => { /* hide MQTT section on failure */ });
	});

	async function startEditing(id: number, name: string) {
		editingId = id;
		editValue = name;
		editError = '';
		await tick();
		const input = document.querySelector<HTMLInputElement>('.edit-input');
		input?.select();
	}

	let cancelled = false;

	async function commitEdit(id: number, originalName: string) {
		if (cancelled) {
			cancelled = false;
			return;
		}
		const trimmed = editValue.trim();
		if (!trimmed || trimmed === originalName) {
			editingId = null;
			editError = '';
			return;
		}
		const result = await updateLocation(id, trimmed);
		if (result) {
			editingId = null;
			editError = '';
		} else {
			editError = get(locationsError) || 'Failed to rename location';
			locationsError.set(null);
		}
	}

	function cancelEdit(target: HTMLInputElement) {
		cancelled = true;
		editingId = null;
		editError = '';
		target.blur();
	}

	function handleEditKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter') {
			e.preventDefault();
			(e.target as HTMLInputElement).blur();
		} else if (e.key === 'Escape') {
			cancelEdit(e.target as HTMLInputElement);
		}
	}

	function handleRepairClick() {
		repairDialogOpen = true;
	}

	async function handleRepairConfirm() {
		repairDialogOpen = false;
		repairLoading = true;
		repairMessage = '';
		repairError = '';
		try {
			const result = await repairMqtt();
			repairMessage = `Cleared ${result.cleared}, published ${result.published}`;
		} catch (e: unknown) {
			repairError = e instanceof Error ? e.message : 'Repair failed';
		} finally {
			repairLoading = false;
		}
	}

	function handleExport() {
		window.location.href = '/api/data/export';
	}

	function handleImportClick() {
		fileInput.click();
	}

	function handleFileSelected(e: Event) {
		const input = e.target as HTMLInputElement;
		const file = input.files?.[0];
		if (!file) return;

		// Reset for next use
		input.value = '';

		importFile = file;
		importDialogOpen = true;
	}

	async function handleImportConfirm() {
		importDialogOpen = false;
		const file = importFile;
		importFile = null;
		if (!file) return;

		importLoading = true;
		importMessage = '';
		importError = '';
		try {
			const result = await importData(file);
			importMessage = `Imported ${result.plants} plants, ${result.photos} photos, ${result.care_events} log entries, ${result.locations} locations`;
			fetchStats()
				.then((s) => { stats = s; })
				.catch(() => {});
			loadLocations();
		} catch (e: unknown) {
			importError = e instanceof Error ? e.message : 'Import failed';
		} finally {
			importLoading = false;
		}
	}

	async function handleDelete(id: number, name: string, plantCount: number) {
		if (plantCount === 0) {
			await deleteLocation(id);
			return;
		}
		deleteTarget = { id, name, plantCount };
		deleteDialogOpen = true;
	}

	async function handleDeleteConfirm() {
		deleteDialogOpen = false;
		if (!deleteTarget) return;
		await deleteLocation(deleteTarget.id);
		deleteTarget = null;
	}
</script>

<div class="page">
	<header class="page-header">
		<h1>Settings</h1>
	</header>

	<section class="section settings-section">
		<h2 class="section-title"><Palette size={14} /> Appearance</h2>
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

	<section class="section settings-section">
		<h2 class="section-title"><MapPin size={14} /> Locations</h2>

		{#if $locationsError}
			<p class="error">{$locationsError}</p>
		{:else if $locations.length === 0}
			<p class="empty">No locations yet. Create locations when adding plants.</p>
		{:else}
			<ul class="location-list">
				{#each $locations as location (location.id)}
					<li class="location-item">
						{#if editingId === location.id}
							<div class="edit-group">
								<div class="edit-row">
									<input
										class="input edit-input"
										class:input-error={editError}
										type="text"
										bind:value={editValue}
										onblur={() => commitEdit(location.id, location.name)}
										onkeydown={handleEditKeydown}
										oninput={() => { editError = ''; }}
									/>
									<button
										class="btn btn-icon"
										onmousedown={(e) => { e.preventDefault(); commitEdit(location.id, location.name); }}
									>
										<Check size={16} />
									</button>
								</div>
								{#if editError}
									<span class="field-error">{editError}</span>
								{/if}
							</div>
						{:else}
							<div class="location-info">
								<span class="location-name">{location.name}</span>
								{#if location.plant_count > 0}
									<span class="plant-count">{location.plant_count} plant{location.plant_count === 1 ? '' : 's'}</span>
								{/if}
							</div>
							<div class="location-actions">
								<button
									class="btn btn-icon"
									onclick={() => startEditing(location.id, location.name)}
								>
									<Pencil size={16} />
								</button>
								<button
									class="btn btn-icon btn-danger"
									onclick={() => handleDelete(location.id, location.name, location.plant_count)}
								>
									<Trash2 size={16} />
								</button>
							</div>
						{/if}
					</li>
				{/each}
			</ul>
		{/if}
	</section>

	{#if mqttStatus}
		<section class="section settings-section">
			<h2 class="section-title"><Radio size={14} /> MQTT</h2>
			<div class="about-row">
				<span class="setting-label">Status</span>
				<span class="mqtt-status">
					{#if mqttStatus.status === 'connected'}
						<span class="status-dot status-connected"></span> Connected
					{:else if mqttStatus.status === 'disconnected'}
						<span class="status-dot status-disconnected"></span> Disconnected
					{:else}
						Disabled
					{/if}
				</span>
			</div>
			{#if mqttStatus.status !== 'disabled'}
				<div class="about-row">
					<span class="setting-label">Broker</span>
					<span>{mqttStatus.broker}</span>
				</div>
				<div class="about-row">
					<span class="setting-label">Topic prefix</span>
					<span>{mqttStatus.topic_prefix}</span>
				</div>
				<div class="about-row">
					<span class="setting-label">Repair</span>
					<span class="repair-actions">
						{#if repairMessage}
							<span class="repair-success">{repairMessage}</span>
						{/if}
						{#if repairError}
							<span class="repair-error">{repairError}</span>
						{/if}
						<button
							class="btn btn-outline btn-sm"
							disabled={mqttStatus.status !== 'connected' || repairLoading}
							title={mqttStatus.status !== 'connected' ? 'MQTT must be connected' : undefined}
							onclick={handleRepairClick}
						>
							{#if repairLoading}
								Repairing...
							{:else}
								<Wrench size={14} /> Repair
							{/if}
						</button>
					</span>
				</div>
			{/if}
		</section>
	{/if}

	{#if stats}
		<section class="section settings-section">
			<h2 class="section-title"><Database size={14} /> Data</h2>
			<div class="about-row">
				<span class="setting-label">Backup</span>
				<span class="backup-actions">
					{#if importMessage}
					<span class="backup-success">{importMessage}</span>
					{/if}
					{#if importError}
						<span class="backup-error">{importError}</span>
					{/if}
					<button
						class="btn btn-outline btn-sm"
						disabled={importLoading}
						onclick={handleImportClick}
					>
						{#if importLoading}
							Importing...
						{:else}
							<Upload size={14} /> Import
						{/if}
					</button>
					<input
						type="file"
						accept=".zip"
						class="hidden"
						bind:this={fileInput}
						onchange={handleFileSelected}
					/>
					<button class="btn btn-outline btn-sm" onclick={handleExport}>
						<Download size={14} /> Export
					</button>
				</span>
			</div>
			<div class="about-row">
				<span class="setting-label">Plants</span>
				<span>{stats.plant_count} {stats.plant_count === 1 ? 'plant' : 'plants'}, {stats.care_event_count} {stats.care_event_count === 1 ? 'care journal entry' : 'care journal entries'}, {stats.location_count} {stats.location_count === 1 ? 'location' : 'locations'}</span>
			</div>
		</section>
	{/if}

	{#if appInfo}
		<section class="section settings-section">
			<h2 class="section-title"><Info size={14} /> About</h2>
			<div class="about-row">
				<span class="setting-label">Version</span>
				<span>{appInfo.version}</span>
			</div>
			<div class="about-row">
				<span class="setting-label">Source</span>
				<a href={appInfo.repository} target="_blank" rel="noopener noreferrer">
					{appInfo.repository.replace('https://', '')}
				</a>
			</div>
			<div class="about-row">
				<span class="setting-label">License</span>
				<span>{appInfo.license}</span>
			</div>
		</section>
	{/if}
</div>

<ModalDialog
	open={deleteDialogOpen}
	title="Delete location"
	message={deleteTarget
		? deleteTarget.plantCount > 0
			? `Delete "${deleteTarget.name}"? ${deleteTarget.plantCount} plant${deleteTarget.plantCount === 1 ? '' : 's'} will lose ${deleteTarget.plantCount === 1 ? 'its' : 'their'} location.`
			: `Delete "${deleteTarget.name}"?`
		: ''}
	mode="confirm"
	variant="danger"
	confirmLabel="Delete"
	onconfirm={handleDeleteConfirm}
	oncancel={() => { deleteDialogOpen = false; deleteTarget = null; }}
/>

<ModalDialog
	open={importDialogOpen}
	title="Import data"
	message={importFile ? `Import "${importFile.name}"? All existing data and photos will be replaced.` : ''}
	mode="confirm"
	variant="danger"
	confirmLabel="Import"
	onconfirm={handleImportConfirm}
	oncancel={() => { importDialogOpen = false; importFile = null; }}
/>

<ModalDialog
	open={repairDialogOpen}
	title="Repair MQTT"
	message="Clear all retained MQTT topics and republish fresh state for all plants?"
	mode="confirm"
	variant="warning"
	confirmLabel="Repair"
	onconfirm={handleRepairConfirm}
	oncancel={() => { repairDialogOpen = false; }}
/>

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

	.settings-section {
		margin-bottom: 20px;
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
		font-weight: 500;
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
		background: var(--color-primary-tint);
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
		gap: 6px;
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

	.location-actions {
		display: flex;
		gap: 6px;
	}

	.edit-group {
		flex: 1;
	}

	.edit-row {
		display: flex;
		align-items: center;
		gap: 6px;
	}


	.edit-input {
		flex: 1;
	}

	.field-error {
		font-size: var(--fs-chip);
		color: var(--color-danger);
	}

	.about-row {
		display: flex;
		align-items: center;
		justify-content: space-between;
		padding: 10px 0;
		border-bottom: 1px solid var(--color-border);
		font-size: 15px;
	}

	.about-row:last-child {
		border-bottom: none;
		padding-bottom: 0;
	}

	.about-row:first-of-type {
		padding-top: 0;
	}

	.about-row > span:not(.setting-label) {
		color: var(--color-text-muted);
	}

	.mqtt-status {
		display: flex;
		align-items: center;
		gap: 6px;
	}

	.status-dot {
		display: inline-block;
		width: 8px;
		height: 8px;
		border-radius: 50%;
	}

	.status-connected {
		background-color: var(--color-success);
	}

	.status-disconnected {
		background-color: var(--color-text-muted);
	}

	.repair-actions {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.repair-success {
		font-size: var(--fs-chip);
		color: var(--color-success);
	}

	.repair-error {
		font-size: var(--fs-chip);
		color: var(--color-danger);
	}

	.backup-actions {
		display: flex;
		align-items: center;
		gap: 8px;
		flex-wrap: wrap;
	}

	.backup-success {
		font-size: var(--fs-chip);
		color: var(--color-success);
	}

	.backup-error {
		font-size: var(--fs-chip);
		color: var(--color-danger);
	}

	.hidden {
		display: none;
	}

	.about-row a {
		color: var(--color-primary);
		text-decoration: none;
	}

	.about-row a:hover {
		text-decoration: underline;
	}

	@media (max-width: 768px) {
		.settings-section {
			padding: 16px;
		}
	}
</style>
