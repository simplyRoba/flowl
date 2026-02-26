<script lang="ts">
	import { onMount } from 'svelte';
	import { Sparkles, X, Send } from 'lucide-svelte';
	import { translations } from '$lib/stores/locale';
	import { chatPlant, type ChatMessage, type Plant } from '$lib/api';

	let {
		plant,
		open = false,
		onclose
	}: {
		plant: Plant;
		open: boolean;
		onclose: () => void;
	} = $props();

	let messages: { role: string; content: string }[] = $state([]);
	let inputText = $state('');
	let streaming = $state(false);
	let abortController: AbortController | null = $state(null);
	let messagesEl: HTMLDivElement | undefined = $state();
	let inputEl: HTMLInputElement | undefined = $state();
	let dialogEl: HTMLDialogElement | undefined = $state();

	// Drag-to-dismiss state
	let dragStartY = $state(0);
	let dragOffset = $state(0);
	let dragging = $state(false);

	// Mobile detection
	let isMobile = $state(false);

	const MAX_HISTORY = 20;

	let chips = $derived.by(() => {
		const c: { text: string; danger?: boolean }[] = [];
		if (plant.watering_status === 'overdue') {
			c.push({ text: $translations.chat.whyOverdue, danger: true });
		}
		c.push({ text: $translations.chat.healthCheck });
		c.push({ text: $translations.chat.wateringAdvice });
		if (plant.species === null) {
			c.push({ text: $translations.chat.helpIdentify });
		} else {
			c.push({ text: $translations.chat.whenToRepot });
		}
		c.push({ text: $translations.chat.lightRequirements });
		return c;
	});

	let showChips = $derived(messages.length === 0);

	function scrollToBottom() {
		if (messagesEl) {
			requestAnimationFrame(() => {
				messagesEl!.scrollTop = messagesEl!.scrollHeight;
			});
		}
	}

	function getHistory(): ChatMessage[] {
		const hist = messages.map((m) => ({ role: m.role, content: m.content }));
		if (hist.length > MAX_HISTORY) {
			return hist.slice(hist.length - MAX_HISTORY);
		}
		return hist;
	}

	async function sendMessage(text: string) {
		if (!text.trim() || streaming) return;

		const userMsg = text.trim();
		inputText = '';
		messages.push({ role: 'user', content: userMsg });
		scrollToBottom();

		streaming = true;
		const controller = new AbortController();
		abortController = controller;

		// Add placeholder for AI response
		messages.push({ role: 'assistant', content: '' });
		scrollToBottom();

		try {
			const history = getHistory().slice(0, -1); // exclude the empty assistant placeholder
			// Also exclude the user message we just added — it goes as the `message` param
			const historyWithoutCurrent = history.slice(0, -1);

			for await (const delta of chatPlant(plant.id, userMsg, historyWithoutCurrent, controller.signal)) {
				messages[messages.length - 1].content += delta;
				scrollToBottom();
			}
		} catch (err: unknown) {
			if (err instanceof DOMException && err.name === 'AbortError') return;
			const errorContent = messages[messages.length - 1].content;
			if (!errorContent) {
				// Replace empty assistant message with error
				messages[messages.length - 1] = {
					role: 'assistant',
					content: $translations.chat.errorMessage
				};
			} else {
				messages.push({ role: 'assistant', content: $translations.chat.errorMessage });
			}
			scrollToBottom();
		} finally {
			streaming = false;
			abortController = null;
		}
	}

	function handleSubmit() {
		sendMessage(inputText);
	}

	function handleKeydown(e: KeyboardEvent) {
		if (e.key === 'Enter' && !e.shiftKey) {
			e.preventDefault();
			handleSubmit();
		}
	}

	function handleChipClick(text: string) {
		sendMessage(text);
	}

	function handleClose() {
		if (abortController) {
			abortController.abort();
		}
		onclose();
	}

	// Mobile drag-to-dismiss
	function handleDragStart(e: TouchEvent) {
		dragStartY = e.touches[0].clientY;
		dragging = true;
		dragOffset = 0;
	}

	function handleDragMove(e: TouchEvent) {
		if (!dragging) return;
		const diff = e.touches[0].clientY - dragStartY;
		dragOffset = Math.max(0, diff);
	}

	function handleDragEnd() {
		if (!dragging) return;
		dragging = false;
		if (dragOffset > 120) {
			handleClose();
		}
		dragOffset = 0;
	}

	// Manage mobile dialog
	$effect(() => {
		if (!dialogEl) return;
		if (open && isMobile) {
			if (!dialogEl.open) dialogEl.showModal();
		} else if (dialogEl.open) {
			dialogEl.close();
		}
	});

	// Focus input when opened
	$effect(() => {
		if (open && inputEl) {
			setTimeout(() => inputEl?.focus(), 300);
		}
	});

	// Mobile detection + cleanup
	onMount(() => {
		const mq = window.matchMedia('(max-width: 768px)');
		isMobile = mq.matches;
		const handler = (e: MediaQueryListEvent) => { isMobile = e.matches; };
		mq.addEventListener('change', handler);
		return () => {
			mq.removeEventListener('change', handler);
			if (abortController) {
				abortController.abort();
			}
		};
	});
</script>

{#snippet chatContent()}
	<!-- svelte-ignore a11y_no_static_element_interactions -->
	<div
		class="drag-handle"
		ontouchstart={handleDragStart}
		ontouchmove={handleDragMove}
		ontouchend={handleDragEnd}
	>
		<div class="drag-handle-bar"></div>
	</div>

	<div class="chat-header">
		<div class="chat-header-left">
			<Sparkles size={18} />
			<span class="chat-header-title">{plant.name}</span>
		</div>
		<button class="chat-close" onclick={handleClose} aria-label={$translations.chat.close}>
			<X size={18} />
		</button>
	</div>

	{#if showChips}
		<div class="quick-chips">
			<span class="quick-chips-label">{$translations.chat.quickQuestions}</span>
			{#each chips as chip}
				<button
					class="chip"
					class:chip-danger={chip.danger}
					onclick={() => handleChipClick(chip.text)}
				>
					{chip.text}
				</button>
			{/each}
		</div>
	{/if}

	<div class="chat-messages" bind:this={messagesEl}>
		{#if messages.length === 0}
			<div class="empty-state">
				<Sparkles size={32} />
				<p>{$translations.chat.emptyState.replace('{name}', plant.name)}</p>
			</div>
		{:else}
			{#each messages as msg}
				{#if msg.content}
				<div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
					{msg.content}
				</div>
				{/if}
			{/each}
			{#if streaming && messages[messages.length - 1]?.content === ''}
				<div class="typing-indicator">
					<span class="typing-dot"></span>
					<span class="typing-dot"></span>
					<span class="typing-dot"></span>
				</div>
			{/if}
		{/if}
	</div>

	<div class="chat-input-area">
		<input
			bind:this={inputEl}
			class="chat-input"
			placeholder={$translations.chat.placeholder}
			bind:value={inputText}
			onkeydown={handleKeydown}
			disabled={streaming}
		/>
		<button
			class="send-btn"
			class:disabled={!inputText.trim() || streaming}
			onclick={handleSubmit}
			disabled={!inputText.trim() || streaming}
			aria-label={$translations.chat.send}
		>
			<Send size={16} />
		</button>
	</div>
{/snippet}

{#if open && !isMobile}
	<div class="chat-drawer">
		{@render chatContent()}
	</div>
{/if}

<dialog
	bind:this={dialogEl}
	class="chat-dialog-mobile"
	oncancel={(e) => { e.preventDefault(); handleClose(); }}
>
	{#if open && isMobile}
		<div
			class="dialog-sheet"
			style:transform={dragOffset > 0 ? `translateY(${dragOffset}px)` : undefined}
			style:transition={dragging ? 'none' : 'transform 0.15s ease-out'}
		>
			{@render chatContent()}
		</div>
	{/if}
</dialog>

<style>
	/* ── Desktop drawer ── */
	.chat-drawer {
		width: 400px;
		flex-shrink: 0;
		background: var(--color-surface);
		border-left: 1px solid var(--color-border);
		display: flex;
		flex-direction: column;
		height: 100%;
		animation: slideInRight 0.2s ease-out;
	}

	@keyframes slideInRight {
		from { transform: translateX(100%); }
		to { transform: translateX(0); }
	}

	@media (max-width: 768px) {
		.chat-drawer {
			display: none;
		}
	}

	/* ── Mobile dialog ── */
	.chat-dialog-mobile {
		position: fixed;
		top: 60px;
		left: 0;
		right: 0;
		bottom: 0;
		width: 100%;
		max-width: 100%;
		max-height: none;
		height: auto;
		margin: 0;
		padding: 0;
		border: none;
		background: transparent;
		overflow: visible;
	}

	.chat-dialog-mobile[open] {
		animation: slideUp 0.25s ease-out;
	}

	.chat-dialog-mobile::backdrop {
		background: rgba(0, 0, 0, 0.3);
	}

	@keyframes slideUp {
		from { transform: translateY(100%); }
		to { transform: translateY(0); }
	}

	.dialog-sheet {
		display: flex;
		flex-direction: column;
		height: 100%;
		background: var(--color-surface);
		border-radius: 16px 16px 0 0;
		box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.15);
		overflow: hidden;
	}

	/* ── Drag handle (mobile only) ── */
	.drag-handle {
		display: none;
	}

	@media (max-width: 768px) {
		.drag-handle {
			height: 24px;
			display: flex;
			align-items: center;
			justify-content: center;
			flex-shrink: 0;
			cursor: grab;
		}
	}

	.drag-handle-bar {
		width: 36px;
		height: 4px;
		border-radius: 2px;
		background: var(--color-border);
	}

	/* ── Shared elements ── */
	.chat-header {
		padding: 16px 20px;
		border-bottom: 1px solid var(--color-border);
		display: flex;
		align-items: center;
		justify-content: space-between;
		flex-shrink: 0;
	}

	.chat-header-left {
		display: flex;
		align-items: center;
		gap: 8px;
		color: var(--color-ai);
	}

	.chat-header-title {
		font-size: 16px;
		font-weight: 600;
		color: var(--color-text);
	}

	.chat-close {
		width: 32px;
		height: 32px;
		border-radius: 8px;
		border: none;
		background: transparent;
		color: var(--color-text-muted);
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
	}

	.chat-close:hover {
		background: var(--color-ai-tint);
	}

	/* ── Quick chips ── */
	.quick-chips {
		padding: 12px 20px;
		display: flex;
		flex-wrap: wrap;
		gap: 8px;
		flex-shrink: 0;
	}

	.quick-chips-label {
		width: 100%;
		font-size: 13px;
		color: var(--color-text-muted);
		margin-bottom: 2px;
	}

	.chip {
		padding: 6px 14px;
		border-radius: 999px;
		border: 1px solid var(--color-border);
		background: var(--color-surface);
		font-size: 13px;
		color: var(--color-text);
		cursor: pointer;
		transition: all 0.15s;
	}

	.chip:hover {
		border-color: var(--color-ai);
		background: var(--color-ai-tint);
		color: var(--color-ai);
	}

	.chip-danger {
		border-color: var(--color-danger);
		color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 8%, transparent);
	}

	.chip-danger:hover {
		border-color: var(--color-danger);
		color: var(--color-danger);
		background: color-mix(in srgb, var(--color-danger) 16%, transparent);
	}

	/* ── Messages ── */
	.chat-messages {
		flex: 1;
		overflow-y: auto;
		padding: 16px 20px;
		display: flex;
		flex-direction: column;
		gap: 12px;
	}

	.empty-state {
		flex: 1;
		display: flex;
		flex-direction: column;
		align-items: center;
		justify-content: center;
		gap: 12px;
		color: var(--color-text-muted);
		text-align: center;
		font-size: 14px;
		opacity: 0.6;
	}

	.message {
		max-width: 85%;
		padding: 10px 14px;
		border-radius: 14px;
		font-size: 14px;
		line-height: 1.5;
		white-space: pre-wrap;
		word-break: break-word;
	}

	.message.user {
		align-self: flex-end;
		background: var(--color-ai);
		color: var(--color-text-on-primary);
		border-bottom-right-radius: 4px;
	}

	.message.assistant {
		align-self: flex-start;
		background: var(--color-ai-tint);
		color: var(--color-text);
		border-bottom-left-radius: 4px;
	}

	/* ── Typing indicator ── */
	.typing-indicator {
		align-self: flex-start;
		display: flex;
		gap: 4px;
		padding: 12px 16px;
		background: var(--color-ai-tint);
		border-radius: 14px;
		border-bottom-left-radius: 4px;
	}

	.typing-dot {
		width: 8px;
		height: 8px;
		border-radius: 50%;
		background: var(--color-ai);
		opacity: 0.4;
		animation: typing 1.4s ease-in-out infinite;
	}

	.typing-dot:nth-child(2) {
		animation-delay: 0.2s;
	}

	.typing-dot:nth-child(3) {
		animation-delay: 0.4s;
	}

	@keyframes typing {
		0%, 60%, 100% { opacity: 0.4; transform: translateY(0); }
		30% { opacity: 1; transform: translateY(-4px); }
	}

	/* ── Input area ── */
	.chat-input-area {
		padding: 12px 16px;
		border-top: 1px solid var(--color-border);
		display: flex;
		gap: 8px;
		align-items: center;
		flex-shrink: 0;
	}

	.chat-input {
		flex: 1;
		padding: 10px 14px;
		border-radius: 999px;
		border: 1px solid var(--color-border);
		background: var(--color-background);
		font-size: var(--fs-input);
		color: var(--color-text);
		outline: none;
		font-family: inherit;
	}

	.chat-input::placeholder {
		color: var(--color-text-muted);
	}

	.chat-input:focus {
		border-color: var(--color-ai);
	}

	.chat-input:disabled {
		opacity: 0.5;
	}

	.send-btn {
		width: 38px;
		height: 38px;
		border-radius: 50%;
		border: none;
		background: var(--color-ai);
		color: white;
		cursor: pointer;
		display: flex;
		align-items: center;
		justify-content: center;
		flex-shrink: 0;
		transition: filter 0.15s;
	}

	.send-btn:hover:not(:disabled) {
		filter: brightness(0.9);
	}

	.send-btn.disabled,
	.send-btn:disabled {
		opacity: 0.4;
		cursor: default;
	}
</style>
