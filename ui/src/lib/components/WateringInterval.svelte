<script lang="ts">
	import { Minus, Plus } from 'lucide-svelte';

	const PRESETS = [
		{ days: 3, label: '3 days', desc: 'Thirsty' },
		{ days: 7, label: '7 days', desc: 'Weekly' },
		{ days: 14, label: '14 days', desc: 'Biweekly' },
		{ days: 30, label: '30 days', desc: 'Monthly' }
	];

	let { value = 7, onchange }: { value: number; onchange: (days: number) => void } = $props();

	let isPreset = $derived(PRESETS.some((p) => p.days === value));

	function decrement() {
		if (value > 1) onchange(value - 1);
	}

	function increment() {
		onchange(value + 1);
	}
</script>

<div class="watering-interval">
	<div class="presets">
		{#each PRESETS as preset}
			<button
				type="button"
				class="preset"
				class:selected={value === preset.days}
				onclick={() => onchange(preset.days)}
			>
				<span class="preset-label">{preset.label}</span>
				<span class="preset-desc">{preset.desc}</span>
			</button>
		{/each}
	</div>

	<div class="custom">
		<span class="custom-label">Custom:</span>
		<button type="button" class="stepper-btn" onclick={decrement} disabled={value <= 1}>
			<Minus size={16} />
		</button>
		<span class="stepper-value" class:custom-active={!isPreset}>{value}</span>
		<button type="button" class="stepper-btn" onclick={increment}>
			<Plus size={16} />
		</button>
		<span class="stepper-unit">days</span>
	</div>
</div>

<style>
	.watering-interval {
		display: flex;
		flex-direction: column;
		gap: 16px;
	}

	.presets {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.preset {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 2px;
		padding: 12px 16px;
		border: 1px solid #E5DDD3;
		border-radius: 10px;
		background: #FFFFFF;
		cursor: pointer;
		transition: border-color 0.15s, background 0.15s;
		flex: 1;
		min-width: 80px;
	}

	.preset:hover {
		border-color: #8C7E6E;
	}

	.preset.selected {
		border-color: #6B8F71;
		background: #f0f7f1;
	}

	.preset-label {
		font-size: 15px;
		font-weight: 600;
	}

	.preset-desc {
		font-size: 12px;
		color: #8C7E6E;
	}

	.custom {
		display: flex;
		align-items: center;
		gap: 8px;
	}

	.custom-label {
		font-size: 14px;
		color: #8C7E6E;
	}

	.stepper-btn {
		width: 36px;
		height: 36px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: 1px solid #E5DDD3;
		border-radius: 8px;
		background: #FFFFFF;
		cursor: pointer;
		color: #2C2418;
		transition: background 0.15s;
	}

	.stepper-btn:hover:not(:disabled) {
		background: #FAF6F1;
	}

	.stepper-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}

	.stepper-value {
		font-size: 18px;
		font-weight: 600;
		min-width: 32px;
		text-align: center;
	}

	.stepper-value.custom-active {
		color: #6B8F71;
	}

	.stepper-unit {
		font-size: 14px;
		color: #8C7E6E;
	}
</style>
