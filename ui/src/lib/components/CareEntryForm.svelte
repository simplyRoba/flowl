<script lang="ts">
  import {
    Droplets,
    Leaf,
    Shovel,
    Scissors,
    Pencil as PencilIcon,
    Camera,
    CalendarClock,
    X as XIcon,
    Sparkles,
  } from "lucide-svelte";
  import { addCareEvent } from "$lib/stores/care";
  import {
    deleteCareEventPhoto,
    updateCareEvent,
    uploadCareEventPhoto,
    type CareEvent,
    type EventType,
  } from "$lib/api";
  import { translations } from "$lib/stores/locale";
  import { isOffline } from "$lib/stores/network";
  import { pushNotification } from "$lib/stores/notifications";

  let {
    plantId,
    existingEvent,
    onsubmit,
    oncancel,
  }: {
    plantId: number;
    existingEvent?: CareEvent;
    onsubmit: (event: CareEvent) => void | Promise<void>;
    oncancel: () => void;
  } = $props();

  let isEditing = $derived(Boolean(existingEvent));
  let eventType = $state("");
  let notes = $state("");
  let photo = $state<File | null>(null);
  let photoPreview = $state<string | null>(null);
  let photoInput = $state<HTMLInputElement | null>(null);
  let existingPhotoRemoved = $state(false);
  let occurredAt = $state("");
  let showOccurredAt = $state(false);
  let submitting = $state(false);
  let eventTypeError = $state("");
  let occurredAtError = $state("");
  let initializedEventId = $state<number | null>(null);

  function localInputValue(value: string): string {
    const hasTimezone = /(?:Z|[+-]\d{2}:?\d{2})$/i.test(value);
    const date = new Date(hasTimezone ? value : `${value}Z`);
    if (Number.isNaN(date.getTime())) return "";
    const pad = (n: number) => String(n).padStart(2, "0");
    return `${date.getFullYear()}-${pad(date.getMonth() + 1)}-${pad(date.getDate())}T${pad(date.getHours())}:${pad(date.getMinutes())}:${pad(date.getSeconds())}`;
  }

  function nowLocalInputValue(): string {
    return localInputValue(new Date().toISOString());
  }

  $effect(() => {
    const event = existingEvent;
    const eventId = event?.id ?? null;
    if (eventId === initializedEventId) return;
    clearStagedPhoto();
    eventType = event?.event_type ?? "";
    notes = event?.notes ?? "";
    existingPhotoRemoved = false;
    occurredAt = event ? localInputValue(event.occurred_at) : "";
    showOccurredAt = Boolean(event);
    eventTypeError = "";
    occurredAtError = "";
    initializedEventId = eventId;
  });

  function clearStagedPhoto() {
    if (photoPreview) URL.revokeObjectURL(photoPreview);
    photo = null;
    photoPreview = null;
  }

  function stagePhoto(file: File) {
    const valid = ["image/jpeg", "image/png", "image/webp"];
    if (!valid.includes(file.type)) return;
    clearStagedPhoto();
    photo = file;
    photoPreview = URL.createObjectURL(file);
    existingPhotoRemoved = false;
  }

  function removePhoto() {
    if (photo) {
      clearStagedPhoto();
      return;
    }
    if (existingEvent?.photo_url) existingPhotoRemoved = true;
  }

  function openPhotoPicker() {
    photoInput?.click();
  }

  function handlePhotoSelect(e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) stagePhoto(file);
    input.value = "";
  }

  function resetForm() {
    clearStagedPhoto();
    eventType = existingEvent?.event_type ?? "";
    eventTypeError = "";
    notes = existingEvent?.notes ?? "";
    occurredAt = existingEvent
      ? localInputValue(existingEvent.occurred_at)
      : "";
    occurredAtError = "";
    showOccurredAt = isEditing;
    existingPhotoRemoved = false;
  }

  function handleCancel() {
    resetForm();
    oncancel();
  }

  function validateOccurredAt(): string | undefined {
    if (!showOccurredAt) return undefined;
    const value = occurredAt.trim();
    const date = new Date(value);
    if (!value || Number.isNaN(date.getTime())) {
      occurredAtError = $translations.care.invalidOccurredAt;
      return undefined;
    }
    if (date.getTime() > Date.now()) {
      occurredAtError = $translations.care.futureOccurredAt;
      return undefined;
    }
    occurredAtError = "";
    return date.toISOString();
  }

  async function handleSubmit() {
    if (submitting || $isOffline) return;
    if (!eventType) {
      eventTypeError = $translations.care.selectTypeError;
      return;
    }

    const occurredAtIso = validateOccurredAt();
    if (showOccurredAt && !occurredAtIso) return;

    submitting = true;
    try {
      const photoFile = photo;
      let event: CareEvent;
      if (existingEvent) {
        event = await updateCareEvent(plantId, existingEvent.id, {
          event_type: eventType as EventType,
          notes: notes.trim() || null,
          occurred_at: occurredAtIso!,
        });

        try {
          if (existingPhotoRemoved) {
            await deleteCareEventPhoto(plantId, event.id);
            event = { ...event, photo_url: null };
          } else if (photoFile) {
            event = await uploadCareEventPhoto(plantId, event.id, photoFile);
          }
        } catch {
          pushNotification({
            title: $translations.plant.careJournalSection,
            variant: "error",
            message: $translations.error.updateCareEventPhoto,
          });
          return;
        }
      } else {
        const createdEvent = await addCareEvent(plantId, {
          event_type: eventType as EventType,
          notes: notes.trim() || undefined,
          occurred_at: occurredAtIso,
        });
        if (!createdEvent) {
          pushNotification({
            title: $translations.plant.careJournalSection,
            variant: "error",
            message: $translations.error.addCareEvent,
          });
          return;
        }

        event = createdEvent;
        if (photoFile) {
          event = await uploadCareEventPhoto(plantId, event.id, photoFile);
        }
      }

      resetForm();
      await onsubmit(event);
    } catch {
      pushNotification({
        title: $translations.plant.careJournalSection,
        variant: "error",
        message: isEditing
          ? $translations.error.updateCareEvent
          : $translations.error.addCareEvent,
      });
    } finally {
      submitting = false;
    }
  }
</script>

<div class="care-entry-form">
  <div
    class="type-chips"
    class:type-chips-error={Boolean(eventTypeError)}
    role="group"
    aria-label={isEditing
      ? $translations.plant.editLogEntry
      : $translations.plant.addLogEntry}
    aria-describedby={eventTypeError ? "care-entry-type-error" : undefined}
  >
    {#each [{ value: "watered", label: $translations.care.watered, icon: Droplets }, { value: "fertilized", label: $translations.care.fertilized, icon: Leaf }, { value: "repotted", label: $translations.care.repotted, icon: Shovel }, { value: "pruned", label: $translations.care.pruned, icon: Scissors }, { value: "custom", label: $translations.care.custom, icon: PencilIcon }, ...(existingEvent?.event_type === "ai-consultation" ? [{ value: "ai-consultation", label: $translations.care.aiConsultation, icon: Sparkles }] : [])] as chip (chip.value)}
      <button
        class="chip chip-solid"
        class:active={eventType === chip.value}
        class:chip-invalid={Boolean(eventTypeError)}
        aria-pressed={eventType === chip.value}
        onclick={() => {
          eventType = chip.value;
          eventTypeError = "";
        }}
      >
        <chip.icon size={14} />
        {chip.label}
      </button>
    {/each}
  </div>

  {#if eventTypeError}
    <div id="care-entry-type-error" class="field-error">{eventTypeError}</div>
  {/if}

  <textarea
    class="input log-notes"
    placeholder={$translations.plant.notesOptional}
    bind:value={notes}
    rows="2"></textarea>

  <div class="toolbar">
    <div class="toolbar-left">
      {#if photoPreview || (existingEvent?.photo_url && !existingPhotoRemoved)}
        <div class="toolbar-compound">
          <div class="toolbar-thumb">
            <img src={photoPreview ?? existingEvent?.photo_url} alt="" />
          </div>
          <button
            class="toolbar-dismiss"
            onclick={removePhoto}
            aria-label={$translations.plant.removeLogPhoto}
          >
            <XIcon size={12} />
          </button>
          <button
            class="toolbar-dismiss toolbar-replace"
            onclick={openPhotoPicker}
            aria-label={$translations.plant.replaceLogPhoto}
          >
            <Camera size={12} />
          </button>
          <input
            bind:this={photoInput}
            type="file"
            accept="image/jpeg,image/png,image/webp"
            onchange={handlePhotoSelect}
            class="file-input-hidden"
          />
        </div>
      {:else}
        <button
          class="toolbar-btn"
          onclick={openPhotoPicker}
          aria-label={$translations.plant.addLogPhoto}
        >
          <Camera size={16} />
        </button>
        <input
          bind:this={photoInput}
          type="file"
          accept="image/jpeg,image/png,image/webp"
          onchange={handlePhotoSelect}
          class="file-input-hidden"
        />
      {/if}

      {#if showOccurredAt}
        <div class="toolbar-compound">
          <input
            class="toolbar-date-input"
            type="datetime-local"
            max={nowLocalInputValue()}
            step="1"
            bind:value={occurredAt}
            aria-label={$translations.plant.when}
            aria-invalid={Boolean(occurredAtError)}
            aria-describedby={occurredAtError
              ? "care-entry-occurred-at-error"
              : undefined}
          />
          {#if !isEditing}
            <button
              class="toolbar-dismiss"
              onclick={() => {
                showOccurredAt = false;
                occurredAt = "";
                occurredAtError = "";
              }}
              aria-label={$translations.common.cancel}
            >
              <XIcon size={12} />
            </button>
          {/if}
        </div>
      {:else}
        <button
          class="toolbar-btn"
          onclick={() => {
            showOccurredAt = true;
            if (!occurredAt) occurredAt = nowLocalInputValue();
          }}
          aria-label={$translations.plant.when}
        >
          <CalendarClock size={16} />
        </button>
      {/if}
    </div>

    <div class="toolbar-right">
      <button class="btn btn-outline" onclick={handleCancel}
        >{$translations.common.cancel}</button
      >
      <button
        class="btn btn-primary"
        onclick={handleSubmit}
        disabled={submitting || $isOffline || !eventType}
      >
        {submitting ? $translations.common.saving : $translations.common.save}
      </button>
    </div>
  </div>

  {#if occurredAtError}
    <div id="care-entry-occurred-at-error" class="field-error">
      {occurredAtError}
    </div>
  {/if}
</div>

<style>
  .care-entry-form {
    margin-top: 12px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border-subtle);
  }

  .type-chips {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
    margin-bottom: 10px;
  }

  .type-chips-error {
    margin-bottom: 6px;
  }

  .chip-invalid {
    border-color: var(--color-danger);
  }

  .field-error {
    color: var(--color-danger);
    font-size: 13px;
    margin: 8px 0 0;
  }

  .log-notes {
    width: 100%;
    resize: vertical;
    margin-bottom: 10px;
  }

  .toolbar {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: center;
  }

  .toolbar-left {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-shrink: 0;
  }

  .toolbar-right {
    display: flex;
    gap: 6px;
    align-items: center;
    margin-left: auto;
  }

  .toolbar-btn {
    box-sizing: border-box;
    width: 36px;
    height: 36px;
    padding: 0;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-btn);
    background: var(--color-surface);
    color: var(--color-text-muted);
    transition:
      background var(--transition-speed),
      border-color var(--transition-speed),
      color var(--transition-speed);
  }

  .toolbar-btn:hover {
    background: var(--color-primary-tint);
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .toolbar-compound {
    box-sizing: border-box;
    display: inline-flex;
    align-items: center;
    border: 1px solid var(--color-primary);
    border-radius: var(--radius-btn);
    overflow: hidden;
    height: 36px;
  }

  .toolbar-thumb {
    width: 36px;
    height: 34px;
    overflow: hidden;
  }

  .toolbar-thumb img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .toolbar-date-input {
    border: none;
    background: none;
    padding: 2px 8px;
    font-size: 13px;
    height: 34px;
    width: 165px;
    color: var(--color-text);
    font-family: inherit;
  }

  .toolbar-dismiss {
    width: 34px;
    height: 34px;
    border: none;
    border-left: 1px solid var(--color-border-subtle);
    background: none;
    color: var(--color-text-muted);
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 0;
    transition:
      color var(--transition-speed),
      background var(--transition-speed);
  }

  .toolbar-dismiss:hover {
    color: var(--color-danger);
    background: color-mix(in srgb, var(--color-danger) 8%, transparent);
  }

  .toolbar-replace:hover {
    color: var(--color-primary);
    background: var(--color-primary-tint);
  }

  .file-input-hidden {
    display: none;
  }

  @media (max-width: 768px) {
    .toolbar-btn {
      width: 44px;
      height: 44px;
    }

    .toolbar-compound {
      height: 44px;
    }

    .toolbar-thumb {
      width: 44px;
      height: 42px;
    }

    .toolbar-date-input {
      height: 42px;
    }

    .toolbar-dismiss {
      width: 42px;
      height: 42px;
    }
  }
</style>
