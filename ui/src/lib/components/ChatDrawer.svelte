<script lang="ts">
  import { onMount } from "svelte";
  import { Sparkles, X, Send, BookOpen, Camera } from "lucide-svelte";
  import { translations } from "$lib/stores/locale";
  import {
    chatPlant,
    summarizeChat,
    createCareEvent,
    uploadCareEventPhoto,
    type ChatMessage,
    type Plant,
  } from "$lib/api";
  import { pushNotification } from "$lib/stores/notifications";

  let {
    plant,
    open = false,
    onclose,
    onsave,
  }: {
    plant: Plant;
    open: boolean;
    onclose: () => void;
    onsave?: () => void;
  } = $props();

  let messages: { role: string; content: string; image?: string }[] = $state(
    [],
  );
  let inputText = $state("");
  let streaming = $state(false);
  let abortController: AbortController | null = $state(null);
  let messagesEl: HTMLDivElement | undefined = $state();
  let inputEl: HTMLInputElement | undefined = $state();
  let dialogEl: HTMLDialogElement | undefined = $state();

  // Save note state
  let summarizing = $state(false);
  let summaryText = $state("");
  let editingSummary = $state(false);
  let savingNote = $state(false);
  let noteSavedMessage = $state("");
  let noteError = $state(false);

  // Photo attachment state
  let attachedPhoto: File | null = $state(null);
  let attachedPreview: string | null = $state(null);
  let isDraggingFile = $state(false);

  // Last user-sent photo (for save-note attachment)
  let lastUserPhoto: File | null = $state(null);
  let lastUserPhotoPreview: string | null = $state(null);
  let saveNotePhoto = $state(true); // whether to attach photo on save

  let hasAssistantMessage = $derived(
    messages.some((m) => m.role === "assistant" && m.content),
  );
  let showSaveNote = $derived(
    hasAssistantMessage && !streaming && !editingSummary,
  );

  // Drag-to-dismiss state
  let dragStartY = $state(0);
  let dragOffset = $state(0);
  let dragging = $state(false);

  // Mobile detection
  let isMobile = $state(false);

  const MAX_HISTORY = 20;

  let chips = $derived.by(() => {
    const c: { text: string; danger?: boolean }[] = [];
    if (plant.watering_status === "overdue") {
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
    const el = messagesEl;
    if (!el) return;

    requestAnimationFrame(() => {
      if (!el.isConnected) return;
      el.scrollTop = el.scrollHeight;
    });
  }

  function getHistory(): ChatMessage[] {
    const hist = messages.map((m) => ({ role: m.role, content: m.content }));
    if (hist.length > MAX_HISTORY) {
      return hist.slice(hist.length - MAX_HISTORY);
    }
    return hist;
  }

  const VALID_IMAGE_TYPES = ["image/jpeg", "image/png", "image/webp"];

  function stagePhoto(file: File) {
    if (!VALID_IMAGE_TYPES.includes(file.type)) return;
    if (attachedPreview) URL.revokeObjectURL(attachedPreview);
    attachedPhoto = file;
    attachedPreview = URL.createObjectURL(file);
  }

  function clearPhoto() {
    if (attachedPreview) URL.revokeObjectURL(attachedPreview);
    attachedPhoto = null;
    attachedPreview = null;
  }

  function fileToBase64(file: File): Promise<string> {
    return new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const dataUrl = reader.result as string;
        resolve(dataUrl.split(",")[1]);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });
  }

  function handlePhotoSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) stagePhoto(file);
    input.value = "";
  }

  function handleFileDragEnter(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = true;
  }

  function handleFileDragLeave(e: DragEvent) {
    if (e.currentTarget === e.target) {
      isDraggingFile = false;
    }
  }

  function handleFileDrop(e: DragEvent) {
    e.preventDefault();
    isDraggingFile = false;
    const file = e.dataTransfer?.files?.[0];
    if (file && VALID_IMAGE_TYPES.includes(file.type)) {
      stagePhoto(file);
    }
  }

  async function sendMessage(text: string) {
    if (!text.trim() || streaming) return;

    const userMsg = text.trim();
    const photo = attachedPhoto;
    if (photo) {
      lastUserPhoto = photo;
      if (lastUserPhotoPreview) URL.revokeObjectURL(lastUserPhotoPreview);
      lastUserPhotoPreview = URL.createObjectURL(photo);
      saveNotePhoto = true;
    }
    inputText = "";
    clearPhoto();
    noteSavedMessage = "";
    noteError = false;

    let imageBase64: string | undefined;
    let imageDataUrl: string | undefined;
    if (photo) {
      imageBase64 = await fileToBase64(photo);
      imageDataUrl = `data:${photo.type};base64,${imageBase64}`;
    }

    messages.push({ role: "user", content: userMsg, image: imageDataUrl });
    scrollToBottom();

    streaming = true;
    const controller = new AbortController();
    abortController = controller;

    // Add placeholder for AI response
    messages.push({ role: "assistant", content: "" });
    scrollToBottom();

    try {
      const history = getHistory().slice(0, -1); // exclude the empty assistant placeholder
      // Also exclude the user message we just added — it goes as the `message` param
      const historyWithoutCurrent = history.slice(0, -1);

      for await (const delta of chatPlant(
        plant.id,
        userMsg,
        historyWithoutCurrent,
        controller.signal,
        imageBase64,
      )) {
        messages[messages.length - 1].content += delta;
        scrollToBottom();
      }
    } catch (err: unknown) {
      if (err instanceof DOMException && err.name === "AbortError") return;
      const errorContent = messages[messages.length - 1].content;
      if (!errorContent) {
        // Replace empty assistant message with error
        messages[messages.length - 1] = {
          role: "assistant",
          content: $translations.chat.errorMessage,
        };
      } else {
        messages.push({
          role: "assistant",
          content: $translations.chat.errorMessage,
        });
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
    if (e.key === "Enter" && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  }

  function handleChipClick(text: string) {
    sendMessage(text);
  }

  async function handleSaveNote() {
    if (summarizing) return;
    summarizing = true;
    noteSavedMessage = "";
    noteError = false;
    try {
      const history = getHistory();
      const summary = await summarizeChat(plant.id, history);
      summaryText = summary;
      editingSummary = true;
    } catch {
      noteSavedMessage = $translations.chat.noteSaveFailed;
      noteError = true;
      scrollToBottom();
    } finally {
      summarizing = false;
    }
  }

  async function handleConfirmSave() {
    if (savingNote || !summaryText.trim()) return;
    savingNote = true;
    try {
      const event = await createCareEvent(plant.id, {
        event_type: "ai-consultation",
        notes: summaryText.trim(),
      });
      if (lastUserPhoto && saveNotePhoto && event) {
        await uploadCareEventPhoto(plant.id, event.id, lastUserPhoto);
      }
      editingSummary = false;
      summaryText = "";
      lastUserPhoto = null;
      if (lastUserPhotoPreview) URL.revokeObjectURL(lastUserPhotoPreview);
      lastUserPhotoPreview = null;
      onsave?.();
      pushNotification({
        variant: "success",
        message: $translations.chat.noteSaved,
      });
      handleClose();
    } catch {
      noteSavedMessage = $translations.chat.noteSaveFailed;
      noteError = true;
      scrollToBottom();
    } finally {
      savingNote = false;
    }
  }

  function handleCancelSave() {
    editingSummary = false;
    summaryText = "";
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

  // Lock body scroll when mobile dialog is open
  let savedOverflow = "";
  $effect(() => {
    if (typeof document === "undefined") return;
    if (open && isMobile) {
      savedOverflow = document.body.style.overflow;
      document.body.style.overflow = "hidden";
    } else {
      document.body.style.overflow = savedOverflow;
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
    const mq = window.matchMedia("(max-width: 768px)");
    isMobile = mq.matches;
    const handler = (e: MediaQueryListEvent) => {
      isMobile = e.matches;
    };
    mq.addEventListener("change", handler);
    return () => {
      mq.removeEventListener("change", handler);
      if (abortController) {
        abortController.abort();
      }
      if (attachedPreview) {
        URL.revokeObjectURL(attachedPreview);
      }
      if (lastUserPhotoPreview) {
        URL.revokeObjectURL(lastUserPhotoPreview);
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
    <div class="chat-header-right">
      {#if showSaveNote}
        <button
          class="btn btn-sm chat-save-note-btn"
          onclick={handleSaveNote}
          disabled={summarizing}
        >
          <BookOpen size={14} />
          {summarizing
            ? $translations.chat.savingNote
            : $translations.chat.saveNote}
        </button>
      {/if}
      <button
        class="chat-close"
        onclick={handleClose}
        aria-label={$translations.chat.close}
      >
        <X size={18} />
      </button>
    </div>
  </div>

  {#if showChips}
    <div class="quick-chips">
      <span class="quick-chips-label">{$translations.chat.quickQuestions}</span>
      {#each chips as chip (chip.text)}
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

  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div
    class="chat-messages"
    class:dragging-file={isDraggingFile}
    bind:this={messagesEl}
    ondragenter={handleFileDragEnter}
    ondragover={handleFileDragEnter}
    ondragleave={handleFileDragLeave}
    ondrop={handleFileDrop}
  >
    {#if messages.length === 0}
      <div class="empty-state">
        <Sparkles size={32} />
        <p>{$translations.chat.emptyState.replace("{name}", plant.name)}</p>
      </div>
    {:else}
      {#each messages as msg, index (index)}
        {#if msg.content}
          <div
            class="message"
            class:user={msg.role === "user"}
            class:assistant={msg.role === "assistant"}
          >
            {#if msg.image && msg.role === "user"}
              <img class="message-photo" src={msg.image} alt="" />
            {/if}
            {msg.content}
          </div>
        {/if}
      {/each}
      {#if streaming && messages[messages.length - 1]?.content === ""}
        <div class="typing-indicator">
          <span class="typing-dot"></span>
          <span class="typing-dot"></span>
          <span class="typing-dot"></span>
        </div>
      {/if}
      {#if noteSavedMessage}
        <div class="note-status" class:note-error={noteError}>
          {noteSavedMessage}
        </div>
      {/if}
    {/if}
  </div>

  {#if editingSummary}
    <div class="summary-editor">
      <textarea
        class="summary-textarea"
        bind:value={summaryText}
        placeholder={$translations.chat.summaryPlaceholder}
        disabled={savingNote}
      ></textarea>
      {#if lastUserPhotoPreview && saveNotePhoto}
        <div class="summary-photo-preview">
          <div class="photo-preview-thumb">
            <img src={lastUserPhotoPreview} alt="" />
            <button
              class="photo-preview-remove"
              onclick={() => {
                saveNotePhoto = false;
              }}
              aria-label={$translations.chat.removePhoto}
            >
              <X size={12} />
            </button>
          </div>
          <span class="summary-photo-label"
            >{$translations.chat.attachChatPhoto}</span
          >
        </div>
      {/if}
      <div class="summary-actions">
        <button
          class="btn btn-outline"
          onclick={handleCancelSave}
          disabled={savingNote}
        >
          {$translations.chat.cancelSummary}
        </button>
        <button
          class="btn btn-ai"
          onclick={handleConfirmSave}
          disabled={savingNote || !summaryText.trim()}
        >
          {savingNote
            ? $translations.common.saving
            : $translations.chat.saveSummary}
        </button>
      </div>
    </div>
  {:else}
    {#if attachedPreview}
      <div class="photo-preview-strip">
        <div class="photo-preview-thumb">
          <img src={attachedPreview} alt="" />
          <button
            class="photo-preview-remove"
            onclick={clearPhoto}
            aria-label={$translations.chat.removePhoto}
          >
            <X size={12} />
          </button>
        </div>
      </div>
    {/if}
    <div class="chat-input-area">
      <label
        class="attach-btn"
        class:disabled={streaming}
        title={$translations.chat.attachPhoto}
      >
        <Camera size={16} />
        <input
          type="file"
          accept="image/jpeg,image/png,image/webp"
          onchange={handlePhotoSelect}
          disabled={streaming}
          class="file-input"
        />
      </label>
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
  {/if}
{/snippet}

{#if open && !isMobile}
  <div class="chat-drawer">
    {@render chatContent()}
  </div>
{/if}

<dialog
  bind:this={dialogEl}
  class="chat-dialog-mobile"
  oncancel={(e) => {
    e.preventDefault();
    handleClose();
  }}
>
  {#if open && isMobile}
    <div
      class="dialog-sheet"
      style:transform={dragOffset > 0
        ? `translateY(${dragOffset}px)`
        : undefined}
      style:transition={dragging ? "none" : "transform 0.15s ease-out"}
    >
      {@render chatContent()}
    </div>
  {/if}
</dialog>

<style>
  /* ── Desktop drawer ── */
  .chat-drawer {
    position: fixed;
    top: 0;
    right: 0;
    bottom: 0;
    width: clamp(400px, 30vw, 560px);
    z-index: 90;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    animation: slideInRight 0.2s ease-out;
  }

  @keyframes slideInRight {
    from {
      transform: translateX(100%);
    }
    to {
      transform: translateX(0);
    }
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
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }

  .dialog-sheet {
    display: flex;
    flex-direction: column;
    height: 100%;
    box-sizing: border-box;
    background: var(--color-surface);
    border-radius: 16px 16px 0 0;
    box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.15);
    overflow: hidden;
    padding-bottom: var(--safe-area-bottom);
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

  .chat-header-right {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .chat-header-title {
    font-size: 16px;
    font-weight: 600;
    color: var(--color-text);
  }

  .chat-save-note-btn {
    background: none;
    color: var(--color-ai);
  }

  .chat-save-note-btn:hover:not(:disabled) {
    opacity: 0.8;
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
    opacity: 0.6;
  }

  @media (max-width: 768px) {
    .chat-close {
      display: none;
    }
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
    overscroll-behavior: contain;
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
    0%,
    60%,
    100% {
      opacity: 0.4;
      transform: translateY(0);
    }
    30% {
      opacity: 1;
      transform: translateY(-4px);
    }
  }

  /* ── Drag-and-drop overlay ── */
  .chat-messages.dragging-file {
    outline: 2px dashed var(--color-ai);
    outline-offset: -4px;
    background: color-mix(in srgb, var(--color-ai) 5%, transparent);
  }

  /* ── Message photo ── */
  .message-photo {
    display: block;
    max-width: 200px;
    border-radius: 8px;
    margin-bottom: 6px;
  }

  /* ── Photo preview strip ── */
  .photo-preview-strip {
    padding: 8px 16px 0;
    flex-shrink: 0;
  }

  .photo-preview-thumb {
    position: relative;
    display: inline-block;
  }

  .photo-preview-thumb img {
    width: 48px;
    height: 48px;
    object-fit: cover;
    border-radius: 8px;
    border: 1px solid var(--color-border);
  }

  .photo-preview-remove {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-danger);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    transition: all var(--transition-speed);
  }

  .photo-preview-remove:hover {
    background: color-mix(
      in srgb,
      var(--color-danger) 10%,
      var(--color-surface)
    );
    border-color: var(--color-danger);
    transform: scale(1.15);
  }

  /* ── Attach button ── */
  .attach-btn {
    width: 38px;
    height: 38px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    transition: all 0.15s;
  }

  .attach-btn:hover:not(.disabled) {
    border-color: var(--color-ai);
    color: var(--color-ai);
    background: var(--color-ai-tint);
  }

  .attach-btn.disabled {
    opacity: 0.4;
    cursor: default;
    pointer-events: none;
  }

  .file-input {
    display: none;
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
    color: var(--color-text-on-ai);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    padding: 1px 2px 0 0;
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

  .note-status {
    padding: 0 16px 8px;
    font-size: 13px;
    color: var(--color-success);
    text-align: center;
  }

  .note-status.note-error {
    color: var(--color-danger);
  }

  /* ── Summary editor ── */
  .summary-editor {
    padding: 12px 16px;
    border-top: 1px solid var(--color-border);
    flex-shrink: 0;
  }

  .summary-textarea {
    width: 100%;
    box-sizing: border-box;
    min-height: 160px;
    padding: 10px 12px;
    border-radius: 8px;
    border: 1px solid var(--color-border);
    background: var(--color-background);
    font-size: 14px;
    font-family: inherit;
    color: var(--color-text);
    resize: vertical;
    outline: none;
  }

  .summary-textarea:focus {
    border-color: var(--color-ai);
  }

  .summary-textarea:disabled {
    opacity: 0.5;
  }

  .summary-photo-preview {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
  }

  .summary-photo-label {
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .summary-actions {
    display: flex;
    gap: 8px;
    margin-top: 12px;
  }

  .summary-actions .btn {
    flex: 1;
  }
</style>
