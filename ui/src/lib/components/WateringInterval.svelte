<script lang="ts">
	import { Minus, Plus, Droplet } from 'lucide-svelte';

	const PRESETS = [
		{ days: 3, label: '3 days', shortLabel: '3d', desc: 'Thirsty' },
		{ days: 7, label: '7 days', shortLabel: '7d', desc: 'Weekly' },
		{ days: 14, label: '14 days', shortLabel: '14d', desc: 'Biweekly' },
		{ days: 30, label: '30 days', shortLabel: '30d', desc: 'Monthly' }
	];

	let { value = 7, onchange }: { value: number; onchange: (days: number) => void } = $props();

	function decrement() {
		if (value > 1) onchange(value - 1);
	}

	function increment() {
		onchange(value + 1);
	}
</script>

<div class="watering-interval">
	<div class="interval-presets">
		{#each PRESETS as preset}
			<button
				type="button"
				class="interval-preset"
				class:active={value === preset.days}
				onclick={() => onchange(preset.days)}
			>
				<span class="preset-icon"><Droplet size={16} /></span>
				<span class="preset-value">
					<span class="preset-long">{preset.label}</span>
					<span class="preset-short">{preset.shortLabel}</span>
				</span>
				<span class="preset-label">{preset.desc}</span>
			</button>
		{/each}
	</div>

	<div class="interval-custom">
		<span class="stepper-label">
			<span class="stepper-long">Or set custom:</span>
			<span class="stepper-short">Custom:</span>
		</span>
		<div class="stepper">
			<button type="button" class="btn btn-icon stepper-btn" onclick={decrement} disabled={value <= 1}>
				<Minus size={16} />
			</button>
			<input
				class="stepper-value"
				type="number"
				min="1"
				value={value}
				oninput={(e) => {
					const next = Number((e.currentTarget as HTMLInputElement).value);
					if (!Number.isNaN(next) && next > 0) onchange(next);
				}}
			/>
			<button type="button" class="btn btn-icon stepper-btn" onclick={increment}>
				<Plus size={16} />
			</button>
		</div>
		<span class="stepper-label">days</span>
	</div>
</div>

<style>
	.watering-interval {
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.interval-presets {
		display: flex;
		gap: 8px;
		flex-wrap: wrap;
	}

	.interval-preset {
		display: flex;
		flex-direction: column;
		align-items: center;
		gap: 4px;
		padding: 10px 8px;
		border: 1px solid var(--color-border);
		border-radius: 10px;
		background: var(--color-surface);
		cursor: pointer;
		transition: all var(--transition-speed);
		flex: 1;
		min-width: 0;
	}

	.interval-preset:hover {
		border-color: var(--color-primary);
	}

	.interval-preset.active {
		border-color: var(--color-primary);
		background: var(--color-primary-tint);
		color: var(--color-primary);
	}

	.interval-preset .preset-icon {
		font-size: 18px;
	}

	.preset-value {
		font-size: 13px;
		font-weight: 600;
	}

	.preset-short {
		display: none;
	}

	.preset-label {
		font-size: 12px;
		color: var(--color-text-muted);
	}

	.interval-preset.active .preset-label {
		color: var(--color-primary);
	}

	.interval-custom {
		display: flex;
		align-items: center;
		gap: 12px;
	}

	.stepper-label {
		font-size: 14px;
		color: var(--color-text-muted);
	}

	.stepper-short {
		display: none;
	}

	.stepper {
		display: flex;
		align-items: center;
		gap: 0;
		border-radius: var(--radius-btn);
		overflow: hidden;
	}

	.stepper-btn {
		border-radius: 0;
		background: var(--color-surface-muted);
	}

	.stepper-btn:first-child {
		border-right: none;
		border-radius: var(--radius-btn) 0 0 var(--radius-btn);
	}

	.stepper-btn:last-child {
		border-left: none;
		border-radius: 0 var(--radius-btn) var(--radius-btn) 0;
	}

	.stepper-btn:hover:not(:disabled) {
		background: var(--color-primary-tint);
		color: var(--color-primary);
		border-color: var(--color-primary);
	}

	.stepper-btn:first-child:hover:not(:disabled) + .stepper-value {
		border-left-color: var(--color-primary);
	}

	.stepper:has(.stepper-btn:last-child:hover:not(:disabled)) .stepper-value {
		border-right-color: var(--color-primary);
	}

	.stepper-value {
		font-size: 18px;
		font-weight: 600;
		width: 52px;
		text-align: center;
		background: var(--color-surface);
		border: 1px solid var(--color-border);
		height: 40px;
		font-family: inherit;
		color: var(--color-text);
		outline: none;
		box-sizing: border-box;
	}

	.stepper-value:focus {
		border-color: var(--color-primary);
	}

	.stepper-value::-webkit-outer-spin-button,
	.stepper-value::-webkit-inner-spin-button {
		-webkit-appearance: none;
		margin: 0;
	}

	.stepper-value[type='number'] {
		-moz-appearance: textfield;
		appearance: textfield;
	}

	@media (max-width: 768px) {
		.preset-long {
			display: none;
		}

		.preset-short {
			display: inline;
		}

		.stepper-long {
			display: none;
		}

		.stepper-short {
			display: inline;
		}
	}
</style>
