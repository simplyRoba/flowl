<script lang="ts">
	import { X } from 'lucide-svelte';

	let { open = false, src = '', alt = '', onclose }: { open?: boolean; src?: string; alt?: string; onclose?: () => void } = $props();

	let zoom = $state(1);
	let translateX = $state(0);
	let translateY = $state(0);
	let isPanning = $state(false);
	let panStartX = $state(0);
	let panStartY = $state(0);
	let panOriginX = $state(0);
	let panOriginY = $state(0);
	let pinchStartDistance = $state<number | null>(null);
	let pinchStartZoom = $state(1);
	let imageEl: HTMLImageElement | null = null;
	let bodyOverflow = $state('');

	const MIN_ZOOM = 1;
	const MAX_ZOOM = 3;

	function requestClose() {
		onclose?.();
	}

	function clamp(value: number, min: number, max: number): number {
		return Math.min(max, Math.max(min, value));
	}

	function clampTranslate(nextX: number, nextY: number) {
		if (!imageEl) {
			return { x: nextX, y: nextY };
		}
		const baseWidth = imageEl.clientWidth;
		const baseHeight = imageEl.clientHeight;
		const maxX = Math.max(0, (baseWidth * zoom - baseWidth) / 2);
		const maxY = Math.max(0, (baseHeight * zoom - baseHeight) / 2);
		return {
			x: clamp(nextX, -maxX, maxX),
			y: clamp(nextY, -maxY, maxY)
		};
	}

	function handleWheel(event: WheelEvent) {
		if (!open) return;
		event.preventDefault();
		zoom = clamp(zoom + event.deltaY * -0.002, MIN_ZOOM, MAX_ZOOM);
		const clamped = clampTranslate(translateX, translateY);
		translateX = clamped.x;
		translateY = clamped.y;
	}

	function handlePointerDown(event: PointerEvent) {
		if (!open || zoom <= 1) return;
		isPanning = true;
		panStartX = event.clientX;
		panStartY = event.clientY;
		panOriginX = translateX;
		panOriginY = translateY;
	}

	function handleWindowPointerMove(event: PointerEvent) {
		if (!isPanning) return;
		const nextX = panOriginX + (event.clientX - panStartX);
		const nextY = panOriginY + (event.clientY - panStartY);
		const clamped = clampTranslate(nextX, nextY);
		translateX = clamped.x;
		translateY = clamped.y;
	}

	function handleWindowPointerUp() {
		isPanning = false;
	}

	function touchDistance(touches: TouchList): number {
		const [a, b] = [touches[0], touches[1]];
		const dx = a.clientX - b.clientX;
		const dy = a.clientY - b.clientY;
		return Math.hypot(dx, dy);
	}

	function handleWindowTouchStart(event: TouchEvent) {
		if (!open) return;
		if (event.touches.length === 2) {
			pinchStartDistance = touchDistance(event.touches);
			pinchStartZoom = zoom;
		}
	}

	function handleWindowTouchMove(event: TouchEvent) {
		if (!open || event.touches.length !== 2 || !pinchStartDistance) return;
		event.preventDefault();
		zoom = clamp(
			pinchStartZoom * (touchDistance(event.touches) / pinchStartDistance),
			MIN_ZOOM,
			MAX_ZOOM
		);
		const clamped = clampTranslate(translateX, translateY);
		translateX = clamped.x;
		translateY = clamped.y;
	}

	function handleWindowTouchEnd() {
		pinchStartDistance = null;
	}

	function handleWindowKeydown(event: KeyboardEvent) {
		if (event.key === 'Escape') {
			requestClose();
		}
	}

	$effect(() => {
		if (!open) return;
		zoom = 1;
		translateX = 0;
		translateY = 0;
	});

	$effect(() => {
		if (typeof document === 'undefined') return;
		if (open) {
			bodyOverflow = document.body.style.overflow;
			document.body.style.overflow = 'hidden';
			return;
		}
		document.body.style.overflow = bodyOverflow;
	});

	$effect(() => {
		if (!open || typeof window === 'undefined') return;
		window.addEventListener('keydown', handleWindowKeydown);
		window.addEventListener('pointermove', handleWindowPointerMove);
		window.addEventListener('pointerup', handleWindowPointerUp);
		window.addEventListener('touchstart', handleWindowTouchStart, { passive: true });
		window.addEventListener('touchmove', handleWindowTouchMove, { passive: false });
		window.addEventListener('touchend', handleWindowTouchEnd);
		return () => {
			window.removeEventListener('keydown', handleWindowKeydown);
			window.removeEventListener('pointermove', handleWindowPointerMove);
			window.removeEventListener('pointerup', handleWindowPointerUp);
			window.removeEventListener('touchstart', handleWindowTouchStart);
			window.removeEventListener('touchmove', handleWindowTouchMove);
			window.removeEventListener('touchend', handleWindowTouchEnd);
		};
	});
</script>

{#if open && src}
	<div class="lightbox" onclick={requestClose} role="dialog" aria-modal="true" aria-label="Plant photo">
		<button type="button" class="lightbox-close" aria-label="Close" onclick={requestClose}>
			<X size={24} />
		</button>
		<div class="lightbox-content" onclick={(event) => event.stopPropagation()}>
			<img
				src={src}
				alt={alt}
				class="lightbox-image"
				bind:this={imageEl}
				onwheel={handleWheel}
				onpointerdown={handlePointerDown}
				style={`transform: translate(${translateX}px, ${translateY}px) scale(${zoom});`}
			/>
		</div>
	</div>
{/if}

<style>
	.lightbox {
		position: fixed;
		inset: 0;
		background: color-mix(in srgb, var(--color-background) 82%, transparent);
		display: flex;
		align-items: center;
		justify-content: center;
		z-index: 1000;
	}

	.lightbox-close {
		position: absolute;
		top: 16px;
		right: 16px;
		width: 40px;
		height: 40px;
		display: flex;
		align-items: center;
		justify-content: center;
		border: none;
		border-radius: var(--radius-btn);
		background: var(--color-surface);
		color: var(--color-text-muted);
		cursor: pointer;
		z-index: 1;
		transition: color var(--transition-speed);
	}

	.lightbox-close:hover {
		color: var(--color-text);
	}

	.lightbox-content {
		max-width: 90vw;
		max-height: 90vh;
		display: flex;
		align-items: center;
		justify-content: center;
		overflow: hidden;
		border-radius: var(--radius-card);
	}

	.lightbox-image {
		max-width: 90vw;
		max-height: 90vh;
		object-fit: contain;
		cursor: grab;
		user-select: none;
		touch-action: none;
		will-change: transform;
	}
</style>
