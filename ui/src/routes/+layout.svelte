<script lang="ts">
	import { Sprout, Leaf, BookOpen, Settings } from 'lucide-svelte';

	let { children } = $props();
</script>

<svelte:head>
	<title>flowl</title>
</svelte:head>

<div class="app">
	<nav class="sidebar">
		<div class="logo"><Sprout size={20} /></div>
		<a href="/" class="nav-item active"><Leaf size={20} /></a>
		<a href="/log" class="nav-item"><BookOpen size={20} /></a>
		<a href="/settings" class="nav-item bottom"><Settings size={20} /></a>
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

	.content {
		flex: 1;
		overflow-y: auto;
		padding: 24px;
	}

	@media (max-width: 768px) {
		.app {
			flex-direction: column;
		}

		.sidebar {
			width: 100%;
			height: 56px;
			flex-direction: row;
			justify-content: center;
			order: 1;
			border-right: none;
			border-top: 1px solid #E5DDD3;
			padding: 0 16px;
			gap: 16px;
		}

		.logo {
			display: none;
		}

		.nav-item {
			width: 40px;
			height: 40px;
		}

		.nav-item.bottom {
			margin-top: 0;
			margin-left: auto;
		}

		.content {
			order: 0;
			padding: 16px;
		}
	}
</style>
