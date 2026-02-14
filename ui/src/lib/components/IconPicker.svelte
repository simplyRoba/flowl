<script lang="ts">
	import { emojiToSvgPath } from '$lib/emoji';

	const ICONS = ['🪴', '🌿', '🌵', '🌸', '🪻', '🌱', '🌷', '🌻', '🍀'];

	let { value = '🪴', onchange }: { value: string; onchange: (icon: string) => void } = $props();
</script>

<div class="icon-picker">
	{#each ICONS as icon}
		<button
			type="button"
			class="icon-option"
			class:selected={value === icon}
			onclick={() => onchange(icon)}
		>
			<img src={emojiToSvgPath(icon)} alt={icon} width="28" height="28" />
		</button>
	{/each}
</div>

<style>
	.icon-picker {
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
	}

	.icon-option {
		width: 48px;
		height: 48px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 2px solid #E5DDD3;
		border-radius: 10px;
		background: #FFFFFF;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
	}

	.icon-option:hover {
		border-color: #8C7E6E;
	}

	.icon-option.selected {
		border-color: #6B8F71;
		background: #f0f7f1;
	}
</style>
