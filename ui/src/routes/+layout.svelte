<script lang="ts">
	let { children } = $props();
</script>

<svelte:head>
	<title>flowl</title>
</svelte:head>

<div class="app">
	<nav class="sidebar">
		<div class="logo">🌱</div>
		<a href="/" class="nav-item active">🪴</a>
		<a href="/log" class="nav-item">📓</a>
		<a href="/settings" class="nav-item bottom">⚙️</a>
	</nav>
	<main class="content">
		{@render children()}
	</main>
</div>

<style>
	:global(body) {
		margin: 0;
		font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
		background: #FAF6F1;
		color: #2C2418;
	}

	.app {
		display: flex;
		min-height: 100vh;
	}

	.sidebar {
		width: 64px;
		background: #FFFFFF;
		border-right: 1px solid #E5DDD3;
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 16px 0;
		gap: 8px;
	}

	.logo {
		font-size: 20px;
		margin-bottom: 16px;
	}

	.nav-item {
		width: 44px;
		height: 44px;
		display: flex;
		align-items: center;
		justify-content: center;
		border-radius: 10px;
		font-size: 18px;
		text-decoration: none;
		transition: background 0.15s;
	}

	.nav-item:hover {
		background: #E5DDD3;
	}

	.nav-item.active {
		background: #6B8F71;
	}

	.nav-item.bottom {
		margin-top: auto;
	}

	.content {
		flex: 1;
		padding: 24px;
	}
</style>
