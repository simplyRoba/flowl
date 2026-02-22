<script lang="ts">
	import { onMount, tick } from 'svelte';
	import { get } from 'svelte/store';
	import { Globe } from 'lucide-svelte';
	import { Check, Pencil, Trash2, Palette, MapPin, Database, Info, Radio, Download, Upload, Wrench } from 'lucide-svelte';
	import { locations, locationsError, loadLocations, deleteLocation, updateLocation } from '$lib/stores/locations';
	import {
		themePreference,
		setThemePreference,
		type ThemePreference
	} from '$lib/stores/theme';
	import { translations, locale, setLocale, type Locale } from '$lib/stores/locale';
	import { plural } from '$lib/i18n';
	import { fetchAppInfo, fetchStats, fetchMqttStatus, repairMqtt, importData, type AppInfo, type Stats, type MqttStatus } from '$lib/api';
	import ModalDialog from '$lib/components/ModalDialog.svelte';

	const themeOptions: { value: ThemePreference; labelKey: 'themeLight' | 'themeDark' | 'themeSystem' }[] = [
		{ value: 'light', labelKey: 'themeLight' },
		{ value: 'dark', labelKey: 'themeDark' },
		{ value: 'system', labelKey: 'themeSystem' }
	];

	const localeOptions: { value: Locale; label: string }[] = [
		{ value: 'en', label: 'English' },
		{ value: 'de', label: 'Deutsch' },
		{ value: 'es', label: 'Español' }
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
			editError = get(locationsError) || get(translations).settings.failedToRename;
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
			const t = get(translations);
			repairMessage = t.settings.repairResult.replace('{cleared}', String(result.cleared)).replace('{published}', String(result.published));
		} catch (e: unknown) {
			repairError = e instanceof Error ? e.message : get(translations).settings.repairFailed;
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
			const t = get(translations);
			importMessage = t.settings.importResult
				.replace('{plants}', String(result.plants))
				.replace('{photos}', String(result.photos))
				.replace('{care_events}', String(result.care_events))
				.replace('{locations}', String(result.locations));
			fetchStats()
				.then((s) => { stats = s; })
				.catch(() => {});
			loadLocations();
		} catch (e: unknown) {
			importError = e instanceof Error ? e.message : get(translations).settings.importFailed;
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
		<h1>{$translations.settings.title}</h1>
	</header>

	<section class="section settings-section">
		<h2 class="section-title"><Palette size={14} /> {$translations.settings.appearance}</h2>
		<div class="setting-row">
			<div>
				<div class="setting-label">{$translations.settings.theme}</div>
			</div>
			<div class="theme-selector" role="radiogroup" aria-label={$translations.settings.theme}>
				{#each themeOptions as option}
					<button
						type="button"
						class="theme-option"
						class:active={$themePreference === option.value}
						aria-pressed={$themePreference === option.value}
						onclick={() => setThemePreference(option.value)}
					>
						{$translations.settings[option.labelKey]}
					</button>
				{/each}
			</div>
		</div>
		<div class="setting-divider"></div>
		<div class="setting-row">
			<div>
				<div class="setting-label">{$translations.settings.language}</div>
			</div>
			<div class="theme-selector" role="radiogroup" aria-label={$translations.settings.language}>
				{#each localeOptions as option}
					<button
						type="button"
						class="theme-option"
						class:active={$locale === option.value}
						aria-pressed={$locale === option.value}
						onclick={() => setLocale(option.value)}
					>
						{option.label}
					</button>
				{/each}
			</div>
		</div>
	</section>

	<section class="section settings-section">
		<h2 class="section-title"><MapPin size={14} /> {$translations.settings.locations}</h2>

		{#if $locationsError}
			<p class="error">{$locationsError}</p>
		{:else if $locations.length === 0}
			<p class="empty">{$translations.settings.noLocations}</p>
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
									<span class="plant-count">{plural($translations.settings.plantCount, location.plant_count)}</span>
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
			<h2 class="section-title"><Radio size={14} /> {$translations.settings.mqtt}</h2>
			<div class="about-row">
				<span class="setting-label">{$translations.settings.mqttStatus}</span>
				<span class="mqtt-status">
					{#if mqttStatus.status === 'connected'}
						<span class="status-dot status-connected"></span> {$translations.settings.connected}
					{:else if mqttStatus.status === 'disconnected'}
						<span class="status-dot status-disconnected"></span> {$translations.settings.disconnected}
					{:else}
						{$translations.settings.disabled}
					{/if}
				</span>
			</div>
			{#if mqttStatus.status !== 'disabled'}
				<div class="about-row">
					<span class="setting-label">{$translations.settings.broker}</span>
					<span>{mqttStatus.broker}</span>
				</div>
				<div class="about-row">
					<span class="setting-label">{$translations.settings.topicPrefix}</span>
					<span>{mqttStatus.topic_prefix}</span>
				</div>
				<div class="about-row">
					<div class="setting-info">
						<div class="setting-label">{$translations.settings.repair}</div>
						<div class="setting-description">{$translations.settings.repairDesc}</div>
					</div>
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
							title={mqttStatus.status !== 'connected' ? $translations.settings.mqttMustBeConnected : undefined}
							onclick={handleRepairClick}
						>
							{#if repairLoading}
								{$translations.settings.repairing}
							{:else}
								<Wrench size={14} /> {$translations.settings.repair}
							{/if}
						</button>
					</span>
				</div>
			{/if}
		</section>
	{/if}

	{#if stats}
		<section class="section settings-section">
			<h2 class="section-title"><Database size={14} /> {$translations.settings.data}</h2>
			<div class="about-row">
				<div class="setting-info">
					<div class="setting-label">{$translations.settings.backup}</div>
					<div class="setting-description">{$translations.settings.backupDesc}</div>
				</div>
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
							{$translations.settings.importing}
						{:else}
							<Upload size={14} /> {$translations.settings.importBtn}
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
						<Download size={14} /> {$translations.settings.exportBtn}
					</button>
				</span>
			</div>
			<div class="about-row">
				<span class="setting-label">{$translations.settings.statsLabel}</span>
				<span>{plural($translations.settings.statsPlants, stats.plant_count)}, {plural($translations.settings.statsCareEvents, stats.care_event_count)}, {plural($translations.settings.statsLocations, stats.location_count)}</span>
			</div>
		</section>
	{/if}

	{#if appInfo}
		<section class="section settings-section">
			<h2 class="section-title"><Info size={14} /> {$translations.settings.about}</h2>
			<div class="about-row">
				<span class="setting-label">{$translations.settings.version}</span>
				<span>{appInfo.version}</span>
			</div>
			<div class="about-row">
				<span class="setting-label">{$translations.settings.source}</span>
				<a href={appInfo.repository} target="_blank" rel="noopener noreferrer">
					{appInfo.repository.replace('https://', '')}
				</a>
			</div>
			<div class="about-row">
				<span class="setting-label">{$translations.settings.license}</span>
				<span>{appInfo.license}</span>
			</div>
		</section>
	{/if}
</div>

<ModalDialog
	open={deleteDialogOpen}
	title={$translations.settings.deleteLocation}
	message={deleteTarget
		? deleteTarget.plantCount > 0
			? $translations.settings.deleteLocationConfirmPlants
				.replace('{name}', deleteTarget.name)
				.replace('{count}', plural($translations.settings.plantCount, deleteTarget.plantCount))
				.replace('{pronoun}', deleteTarget.plantCount === 1 ? $translations.settings.deleteLocationPronoun.one : $translations.settings.deleteLocationPronoun.other)
			: $translations.settings.deleteLocationConfirm.replace('{name}', deleteTarget.name)
		: ''}
	mode="confirm"
	variant="danger"
	confirmLabel={$translations.common.delete}
	onconfirm={handleDeleteConfirm}
	oncancel={() => { deleteDialogOpen = false; deleteTarget = null; }}
/>

<ModalDialog
	open={importDialogOpen}
	title={$translations.settings.importData}
	message={importFile ? $translations.settings.importConfirm.replace('{name}', importFile.name) : ''}
	mode="confirm"
	variant="danger"
	confirmLabel={$translations.settings.importBtn}
	onconfirm={handleImportConfirm}
	oncancel={() => { importDialogOpen = false; importFile = null; }}
/>

<ModalDialog
	open={repairDialogOpen}
	title={$translations.settings.repairTitle}
	message={$translations.settings.repairConfirm}
	mode="confirm"
	variant="warning"
	confirmLabel={$translations.settings.repair}
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

	.setting-info {
		display: flex;
		flex-direction: column;
		gap: 2px;
	}

	.setting-description {
		font-size: var(--fs-chip);
		color: var(--color-text-muted);
	}

	.theme-selector {
		display: inline-flex;
		padding: 4px;
		border-radius: var(--radius-pill);
		border: 1px solid var(--color-border);
		background: var(--color-surface-muted);
		gap: 4px;
	}

	.setting-divider {
		height: 1px;
		background: var(--color-border-subtle);
		margin: 14px 0;
		width: 100%;
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
		align-items: flex-start;
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
