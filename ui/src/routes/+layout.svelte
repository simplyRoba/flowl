<script lang="ts">
	import { onMount } from 'svelte';
	import { Leaf, BookOpen, Settings } from 'lucide-svelte';
	import Logo from '$lib/components/Logo.svelte';
	import { page } from '$app/state';
	import { initTheme, isThemePreference } from '$lib/stores/theme';
	import { initLocale, isLocale, translations } from '$lib/stores/locale';
	import { fetchSettings } from '$lib/api';
	import '$lib/styles/buttons.css';
	import '$lib/styles/chips.css';
	import '$lib/styles/inputs.css';
	import '$lib/styles/sections.css';

	let { children } = $props();

	function isActive(href: string): boolean {
		if (href === '/') return page.url.pathname === '/' || page.url.pathname.startsWith('/plants');
		return page.url.pathname.startsWith(href);
	}

	onMount(async () => {
		try {
			const settings = await fetchSettings();
			const theme = isThemePreference(settings.theme) ? settings.theme : undefined;
			const locale = isLocale(settings.locale) ? settings.locale : undefined;
			initTheme(theme);
			initLocale(locale);
		} catch {
			initTheme();
			initLocale();
		}
	});
</script>

<svelte:head>
	<title>flowl</title>
</svelte:head>

<div class="app">
	<nav class="sidebar">
		<div class="logo"><Logo size={28} /><span class="nav-label brand">flowl</span></div>
		<a href="/" class="nav-item" class:active={isActive('/')}><Leaf size={20} /><span class="nav-label">{$translations.nav.plants}</span></a>
		<a href="/care-journal" class="nav-item" class:active={isActive('/care-journal')}><BookOpen size={20} /><span class="nav-label">{$translations.nav.careJournal}</span></a>
		<a href="/settings" class="nav-item bottom" class:active={isActive('/settings')}><Settings size={20} /><span class="nav-label">{$translations.nav.settings}</span></a>
	</nav>
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	:global(:root) {
		color-scheme: light;
		--color-background: #FAF6F1;
		--color-surface: #FFFFFF;
		--color-surface-muted: color-mix(in srgb, var(--color-surface) 86%, var(--color-background));
		--color-border: #E5DDD3;
		--color-border-subtle: color-mix(in srgb, var(--color-border) 70%, var(--color-background));
		--color-text: #2C2418;
		--color-text-muted: #8C7E6E;
		--color-primary: #6B8F71;
		--color-primary-tint: color-mix(in srgb, var(--color-primary) 10%, transparent);
		--color-primary-dark: #4A6B4F;
		--color-secondary: #C4775B;
		--color-water: #5B9BC4;
		--color-water-strong: #4C89B1;
		--color-success: #7AB87A;
		--color-warning: #D4A843;
		--color-danger: #C45B5B;
		--color-text-on-primary: #FFFFFF;
		--color-text-on-water: #FFFFFF;
		--color-text-on-ai: #FFFFFF;
		--color-text-on-danger: #FFFFFF;
		--color-text-on-image: #FFFFFF;
		--color-success-soft: color-mix(in srgb, var(--color-success) 20%, transparent);
		--color-warning-soft: color-mix(in srgb, var(--color-warning) 18%, transparent);
		--color-danger-soft: color-mix(in srgb, var(--color-danger) 16%, transparent);
		--color-ai: #9B7ED8;
		--color-ai-tint: color-mix(in srgb, var(--color-ai) 10%, transparent);
		--color-ai-soft: color-mix(in srgb, var(--color-ai) 15%, transparent);

		/* Typography */
		--fs-page-title: 22px;
		--fs-section-label: 13px;
		--fs-btn: 14px;
		--fs-input: 16px;
		--fs-chip: 13px;

		/* Radii */
		--radius-card: 12px;
		--radius-btn: 8px;
		--radius-pill: 999px;

		/* Motion */
		--transition-speed: 0.15s;

		/* Layout */
		--nav-bottom-height: 56px;
		--safe-area-bottom: env(safe-area-inset-bottom, 0px);
		--nav-bottom-total: calc(var(--nav-bottom-height) + var(--safe-area-bottom));

		/* Content widths */
		--content-width-narrow: 640px;
		--content-width-default: 800px;
		--content-width-wide: 1200px;
	}

	:global([data-theme='dark']) {
		color-scheme: dark;
		--color-background: #1A1612;
		--color-surface: #252019;
		--color-surface-muted: color-mix(in srgb, var(--color-surface) 90%, var(--color-background));
		--color-border: #3A3228;
		--color-border-subtle: color-mix(in srgb, var(--color-border) 70%, var(--color-background));
		--color-text: #EDE6DB;
		--color-text-muted: #9C8E7E;
		--color-primary: #8BB592;
		--color-primary-tint: color-mix(in srgb, var(--color-primary) 10%, transparent);
		--color-primary-dark: #A3CDA9;
		--color-secondary: #D49478;
		--color-water: #78B4D8;
		--color-water-strong: color-mix(in srgb, var(--color-water) 85%, #000);
		--color-success: #8BC48B;
		--color-warning: #D4B054;
		--color-danger: #D47878;
		--color-text-on-primary: #1A1612;
		--color-text-on-water: #1A1612;
		--color-text-on-ai: #1A1612;
		--color-text-on-danger: #1A1612;
		--color-text-on-image: #FFFFFF;
		--color-ai: #B89EE8;
		--color-ai-tint: color-mix(in srgb, var(--color-ai) 12%, transparent);
		--color-ai-soft: color-mix(in srgb, var(--color-ai) 18%, transparent);
	}

	:global(html, body) {
		margin: 0;
		min-width: 320px;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
		background: var(--color-background);
		color: var(--color-text);
	}

	.app {
		display: block;
	}

	.sidebar {
		position: fixed;
		top: 0;
		left: 0;
		bottom: 0;
		z-index: 100;
		width: 64px;
		background: var(--color-surface);
		border-right: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 16px 0;
		gap: 8px;
		color: var(--color-text);
	}

	.logo {
		margin-bottom: 16px;
		color: var(--color-primary);
	}

	.nav-item {
		width: 44px;
		height: 44px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 10px;
		text-decoration: none;
		color: var(--color-text-muted);
		transition: background 0.15s, color 0.15s;
	}

	.nav-item:hover {
		background: var(--color-surface-muted);
		color: var(--color-text);
	}

	.nav-item.active {
		background: var(--color-primary);
		color: var(--color-text-on-primary);
	}

	.nav-item.bottom {
		margin-top: auto;
	}

	.nav-label {
		display: none;
	}

	.content {
		margin-left: 64px;
		padding: 24px;
	}

	@media (min-width: 1280px) {
		:global(:root) {
			--content-width-narrow: 720px;
			--content-width-default: 960px;
			--content-width-wide: 1400px;
		}

		.sidebar {
			width: 200px;
			align-items: stretch;
			padding: 16px 12px;
		}

		.logo {
			display: flex;
			align-items: center;
			gap: 10px;
			padding: 0 8px;
		}

		.nav-label {
			display: inline;
			font-size: 14px;
			font-weight: 500;
		}

		.nav-label.brand {
			font-size: 18px;
			font-weight: 700;
			color: var(--color-primary);
		}

		.nav-item {
			width: auto;
			justify-content: flex-start;
			gap: 10px;
			padding: 0 12px;
		}

		.content {
			margin-left: 200px;
			padding: 32px;
		}
	}

	@media (max-width: 768px) {
		.sidebar {
			top: auto;
			left: 0;
			right: 0;
			bottom: 0;
			width: 100%;
			height: var(--nav-bottom-height);
			flex-direction: row;
			justify-content: space-around;
			border-right: none;
			border-top: 1px solid var(--color-border);
			padding: 0;
			padding-bottom: var(--safe-area-bottom);
			gap: 0;
		}

		.logo {
			display: none;
		}

		.nav-item {
			flex: 1;
			height: auto;
			flex-direction: column;
			gap: 2px;
			border-radius: 0;
			background: none;
			color: var(--color-text-muted);
		}

		.nav-item:hover {
			background: none;
		}

		.nav-item.active {
			background: none;
			color: var(--color-primary);
		}

		.nav-label {
			display: inline;
			font-size: 11px;
			font-weight: 400;
		}

		.nav-item.bottom {
			margin-top: 0;
			margin-left: 0;
		}

		.content {
			margin-left: 0;
			padding: 16px;
			padding-bottom: calc(var(--nav-bottom-total) + 16px);
		}
	}
</style>
