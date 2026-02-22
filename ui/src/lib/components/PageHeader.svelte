<script lang="ts">
	import { ArrowLeft } from 'lucide-svelte';
	import type { Snippet } from 'svelte';
	import { translations } from '$lib/stores/locale';

	let {
		backHref,
		backLabel = '',
		children
	}: {
		backHref: string;
		backLabel?: string;
		children?: Snippet;
	} = $props();

	let resolvedLabel = $derived(backLabel || $translations.common.back);
</script>

<!-- Desktop: inline header row -->
<div class="page-header-inline">
	<a href={backHref} class="back-link">
		<ArrowLeft size={16} /> {resolvedLabel}
	</a>
	{#if children}
		<div class="header-actions">
			{@render children()}
		</div>
	{/if}
</div>

<!-- Mobile: fixed bottom action bar -->
<div class="action-bar">
	<a href={backHref} class="action-bar-back">
		<ArrowLeft size={16} /> {resolvedLabel}
	</a>
	{#if children}
		<div class="action-bar-actions">
			{@render children()}
		</div>
	{/if}
</div>

<style>
	/* Desktop inline header */
	.page-header-inline {
		display: flex;
		align-items: center;
		justify-content: space-between;
		min-height: 52px;
		margin: -24px -24px 16px;
		padding: 12px 24px;
		position: sticky;
		top: -24px;
		z-index: 10;
		background: var(--color-background);
	}

	@media (min-width: 1280px) {
		.page-header-inline {
			margin: -32px -32px 16px;
			padding: 12px 32px;
			top: -32px;
		}
	}

	.back-link {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--color-primary);
		text-decoration: none;
		font-size: 15px;
		font-weight: 500;
	}

	.back-link:hover {
		color: var(--color-primary-dark);
	}

	.header-actions {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	/* Mobile bottom action bar */
	.action-bar {
		display: none;
	}

	.action-bar-back {
		display: inline-flex;
		align-items: center;
		gap: 6px;
		color: var(--color-primary);
		text-decoration: none;
		font-size: 15px;
		font-weight: 500;
	}

	.action-bar-back:hover {
		color: var(--color-primary-dark);
	}

	.action-bar-actions {
		display: flex;
		gap: 8px;
		align-items: center;
	}

	@media (max-width: 768px) {
		.page-header-inline {
			display: none;
		}

		.action-bar {
			display: flex;
			align-items: center;
			justify-content: space-between;
			position: fixed;
			bottom: 56px;
			left: 0;
			right: 0;
			z-index: 10;
			min-height: 52px;
			padding: 8px 16px;
			background: var(--color-surface);
			border-top: 1px solid var(--color-border);
			box-sizing: border-box;
		}
	}
</style>
