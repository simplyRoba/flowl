<script lang="ts">
	import { emojiToSvgPath } from '$lib/emoji';

	const ICONS = ['🪴', '🌿', '🌵', '🌸', '🪻', '🌱', '🌷', '🌻', '🍀'];

	let { value = '🪴', onchange }: { value: string; onchange: (icon: string) => void } = $props();
</script>

<div class="emoji-picker">
	{#each ICONS as icon}
		<button
			type="button"
			class="emoji-option"
			class:active={value === icon}
			onclick={() => onchange(icon)}
		>
			<img class="noto" src={emojiToSvgPath(icon)} alt={icon} width="24" height="24" />
		</button>
	{/each}
</div>

<style>
	.emoji-picker {
		display: flex;
		flex-wrap: wrap;
		gap: 6px;
		justify-content: center;
	}

	.emoji-option {
		width: 42px;
		height: 42px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid #E5DDD3;
		border-radius: 10px;
		background: #FFFFFF;
		cursor: pointer;
		transition: all 0.15s;
	}

	.emoji-option:hover {
		border-color: #6B8F71;
		transform: scale(1.08);
	}

	.emoji-option.active {
		border-color: #6B8F71;
		background: color-mix(in srgb, #6B8F71 10%, transparent);
		box-shadow: 0 0 0 2px color-mix(in srgb, #6B8F71 30%, transparent);
	}
</style>
