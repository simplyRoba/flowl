<script lang="ts">
	import { translations } from '$lib/stores/locale';

	interface Props {
		status: string;
		nextDue: string | null;
	}

	let { status, nextDue }: Props = $props();

	function statusLabel(s: string): string {
		if (s === 'overdue') return $translations.status.overdue;
		if (s === 'due') return $translations.status.due;
		return $translations.status.ok;
	}

	function statusSuffix(nd: string | null): string | null {
		if (!nd) return null;
		const due = new Date(nd);
		if (isNaN(due.getTime())) return null;
		const now = new Date();
		const start = new Date(now.getFullYear(), now.getMonth(), now.getDate());
		const dueStart = new Date(due.getFullYear(), due.getMonth(), due.getDate());
		const diffDays = Math.round((dueStart.getTime() - start.getTime()) / 86400000);
		if (diffDays === 0) return $translations.status.today;
		if (diffDays === 1) return $translations.status.inOneDay;
		if (diffDays > 1) return $translations.status.inNDays.replace('{n}', String(diffDays));
		const overdueDays = Math.abs(diffDays);
		return overdueDays === 1 ? $translations.status.oneDay : $translations.status.nDays.replace('{n}', String(overdueDays));
	}

	let suffix = $derived(statusSuffix(nextDue));
</script>

<span class="status-badge status-{status}">
	<span class="status-dot"></span>
	{statusLabel(status)}
	{#if suffix}
		 — {suffix}
	{/if}
</span>

<style>
	.status-badge {
		display: inline-flex;
		align-items: center;
		gap: 4px;
		padding: 2px 8px;
		border-radius: var(--radius-pill);
		font-size: 11px;
		font-weight: 600;
	}

	.status-dot {
		width: 6px;
		height: 6px;
		border-radius: 50%;
	}

	.status-ok {
		background: color-mix(in srgb, var(--color-success) 15%, var(--color-surface));
		color: var(--color-success);
	}

	.status-ok .status-dot {
		background: var(--color-success);
	}

	.status-due {
		background: color-mix(in srgb, var(--color-warning) 15%, var(--color-surface));
		color: var(--color-warning);
	}

	.status-due .status-dot {
		background: var(--color-warning);
	}

	.status-overdue {
		background: color-mix(in srgb, var(--color-danger) 15%, var(--color-surface));
		color: var(--color-danger);
	}

	.status-overdue .status-dot {
		background: var(--color-danger);
	}
</style>
