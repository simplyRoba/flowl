<script lang="ts">
	import { Leaf, BookOpen, Settings } from 'lucide-svelte';
	import Logo from '$lib/components/Logo.svelte';
	import { page } from '$app/state';

	let { children } = $props();

	function isActive(href: string): boolean {
		if (href === '/') return page.url.pathname === '/' || page.url.pathname.startsWith('/plants');
		return page.url.pathname.startsWith(href);
	}
</script>

<svelte:head>
	<title>flowl</title>
</svelte:head>

<div class="app">
	<nav class="sidebar">
		<div class="logo"><Logo size={28} /><span class="nav-label brand">flowl</span></div>
		<a href="/" class="nav-item" class:active={isActive('/')}><Leaf size={20} /><span class="nav-label">Plants</span></a>
		<a href="/log" class="nav-item" class:active={isActive('/log')}><BookOpen size={20} /><span class="nav-label">Log</span></a>
		<a href="/settings" class="nav-item bottom" class:active={isActive('/settings')}><Settings size={20} /><span class="nav-label">Settings</span></a>
	</nav>
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	:global(html, body) {
		margin: 0;
		height: 100%;
		overflow: hidden;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
		background: #FAF6F1;
		color: #2C2418;
	}

	.app {
		display: flex;
		height: 100vh;
		height: 100dvh;
	}

	.sidebar {
		width: 64px;
		flex-shrink: 0;
		background: #FFFFFF;
		border-right: 1px solid #E5DDD3;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 16px 0;
		gap: 8px;
		color: #2C2418;
	}

	.logo {
		margin-bottom: 16px;
		color: #6B8F71;
	}

	.nav-item {
		width: 44px;
		height: 44px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 10px;
		text-decoration: none;
		color: #8C7E6E;
		transition: background 0.15s, color 0.15s;
	}

	.nav-item:hover {
		background: #E5DDD3;
		color: #2C2418;
	}

	.nav-item.active {
		background: #6B8F71;
		color: #FFFFFF;
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
			color: #6B8F71;
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
			border-top: 1px solid #E5DDD3;
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
			color: #8C7E6E;
		}

		.nav-item:hover {
			background: none;
		}

		.nav-item.active {
			background: none;
			color: #6B8F71;
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
