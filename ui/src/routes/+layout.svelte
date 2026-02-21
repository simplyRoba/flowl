<script lang="ts">
	import { onMount } from 'svelte';
	import { Leaf, BookOpen, Settings } from 'lucide-svelte';
	import Logo from '$lib/components/Logo.svelte';
	import { page } from '$app/state';
	import { initTheme } from '$lib/stores/theme';
	import '$lib/styles/buttons.css';
	import '$lib/styles/chips.css';
	import '$lib/styles/inputs.css';
	import '$lib/styles/sections.css';

	let { children } = $props();

	function isActive(href: string): boolean {
		if (href === '/') return page.url.pathname === '/' || page.url.pathname.startsWith('/plants');
		return page.url.pathname.startsWith(href);
	}

	onMount(() => {
		initTheme();
	});
</script>

<svelte:head>
	<title>flowl</title>
</svelte:head>

<div class="app">
	<nav class="sidebar">
		<div class="logo"><Logo size={28} /><span class="nav-label brand">flowl</span></div>
		<a href="/" class="nav-item" class:active={isActive('/')}><Leaf size={20} /><span class="nav-label">Plants</span></a>
		<a href="/care-journal" class="nav-item" class:active={isActive('/care-journal')}><BookOpen size={20} /><span class="nav-label">Care Journal</span></a>
		<a href="/settings" class="nav-item bottom" class:active={isActive('/settings')}><Settings size={20} /><span class="nav-label">Settings</span></a>
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
		--color-text-on-danger: #FFFFFF;
		--color-text-on-image: #FFFFFF;
		--color-success-soft: color-mix(in srgb, var(--color-success) 20%, transparent);
		--color-warning-soft: color-mix(in srgb, var(--color-warning) 18%, transparent);
		--color-danger-soft: color-mix(in srgb, var(--color-danger) 16%, transparent);

		/* Typography */
		--fs-page-title: 22px;
		--fs-section-label: 13px;
		--fs-btn: 14px;
		--fs-input: 15px;
		--fs-chip: 13px;

		/* Radii */
		--radius-card: 12px;
		--radius-btn: 8px;
		--radius-pill: 999px;

		/* Motion */
		--transition-speed: 0.15s;

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
		--color-text-on-danger: #1A1612;
		--color-text-on-image: #FFFFFF;
	}

	:global(html, body) {
		margin: 0;
		min-width: 320px;
		height: 100%;
		overflow: hidden;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
		background: var(--color-background);
		color: var(--color-text);
	}

	.app {
		display: flex;
		height: 100vh;
		height: 100dvh;
	}

	.sidebar {
		width: 64px;
		flex-shrink: 0;
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
		flex: 1;
		overflow-y: auto;
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
			padding: 32px;
		}
	}

	@media (max-width: 768px) {
		.app {
			flex-direction: column;
		}

		.sidebar {
			width: 100%;
			height: 56px;
			flex-direction: row;
			justify-content: space-around;
			order: 1;
			border-right: none;
			border-top: 1px solid var(--color-border);
			padding: 0;
			gap: 0;
		}

		.logo {
			display: none;
		}

		.nav-item {
			width: auto;
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
			order: 0;
			padding: 16px;
		}
	}
</style>
