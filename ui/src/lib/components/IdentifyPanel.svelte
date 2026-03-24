<script lang="ts">
  import {
    Camera,
    X,
    Sparkles,
    Check,
    TriangleAlert,
    ChevronLeft,
    ChevronRight,
  } from "lucide-svelte";
  import type { IdentifyResult } from "$lib/api";
  import { ApiError, identifyPlant } from "$lib/api";
  import { resolveError } from "$lib/stores/errors";
  import { translations } from "$lib/stores/locale";

  let {
    photoFile = null,
    photoPreview = null,
    existingPhotoUrl = null,
    onapply,
    onundo,
  }: {
    photoFile: File | null;
    photoPreview: string | null;
    existingPhotoUrl: string | null;
    onapply: (suggestion: IdentifyResult) => number;
    onundo: () => void;
  } = $props();

  let identifyState = $state<
    "idle" | "loading" | "result" | "applied" | "error"
  >("idle");
  let identifyResults = $state<IdentifyResult[]>([]);
  let currentSuggestion = $state(0);
  let identifyError = $state("");
  let identifyErrorRetryable = $state(true);
  let appliedCount = $state(0);

  let activeSuggestion = $derived(identifyResults[currentSuggestion] ?? null);
  let suggestionCount = $derived(identifyResults.length);

  // Extra photo slots
  let extraPhoto1 = $state<File | null>(null);
  let extraPhoto2 = $state<File | null>(null);
  let extraPreview1 = $state<string | null>(null);
  let extraPreview2 = $state<string | null>(null);

  const VALID_LIGHT = ["direct", "indirect", "low"];
  const VALID_DIFFICULTY = ["easy", "moderate", "demanding"];
  const VALID_PET_SAFETY = ["safe", "caution", "toxic"];
  const VALID_GROWTH = ["slow", "moderate", "fast"];
  const VALID_SOIL_TYPE = [
    "standard",
    "cactus-mix",
    "orchid-bark",
    "peat-moss",
  ];
  const VALID_SOIL_MOISTURE = ["dry", "moderate", "moist"];

  type FillChip = { label: string; value: string };

  let willFillChips = $derived.by((): FillChip[] => {
    if (!activeSuggestion) return [];
    const t = $translations;
    const chips: FillChip[] = [];
    chips.push({
      label: t.form.speciesShort,
      value: activeSuggestion.scientific_name,
    });
    const cp = activeSuggestion.care_profile;
    if (cp) {
      if (cp.watering_interval_days != null)
        chips.push({
          label: t.form.watering,
          value: `${cp.watering_interval_days}d`,
        });
      if (cp.light_needs && VALID_LIGHT.includes(cp.light_needs))
        chips.push({ label: t.form.lightNeeds, value: cp.light_needs });
      if (cp.difficulty && VALID_DIFFICULTY.includes(cp.difficulty))
        chips.push({ label: t.form.difficulty, value: cp.difficulty });
      if (cp.pet_safety && VALID_PET_SAFETY.includes(cp.pet_safety))
        chips.push({ label: t.form.petSafety, value: cp.pet_safety });
      if (cp.growth_speed && VALID_GROWTH.includes(cp.growth_speed))
        chips.push({ label: t.form.growthSpeed, value: cp.growth_speed });
      if (cp.soil_type && VALID_SOIL_TYPE.includes(cp.soil_type))
        chips.push({ label: t.form.soilType, value: cp.soil_type });
      if (cp.soil_moisture && VALID_SOIL_MOISTURE.includes(cp.soil_moisture))
        chips.push({ label: t.form.soilMoisture, value: cp.soil_moisture });
    }
    return chips;
  });

  function setExtraPhoto(slot: 1 | 2, file: File) {
    const url = URL.createObjectURL(file);
    if (slot === 1) {
      if (extraPreview1) URL.revokeObjectURL(extraPreview1);
      extraPhoto1 = file;
      extraPreview1 = url;
    } else {
      if (extraPreview2) URL.revokeObjectURL(extraPreview2);
      extraPhoto2 = file;
      extraPreview2 = url;
    }
  }

  function removeExtraPhoto(slot: 1 | 2) {
    if (slot === 1) {
      if (extraPreview1) URL.revokeObjectURL(extraPreview1);
      extraPhoto1 = null;
      extraPreview1 = null;
    } else {
      if (extraPreview2) URL.revokeObjectURL(extraPreview2);
      extraPhoto2 = null;
      extraPreview2 = null;
    }
  }

  function handleExtraSelect(slot: 1 | 2, e: Event) {
    const input = e.target as HTMLInputElement;
    const file = input.files?.[0];
    if (file) setExtraPhoto(slot, file);
    input.value = "";
  }

  async function handleIdentify() {
    identifyState = "loading";
    identifyError = "";

    try {
      const photos: File[] = [];

      if (photoFile) {
        photos.push(photoFile);
      } else if (existingPhotoUrl) {
        const resp = await fetch(existingPhotoUrl);
        const blob = await resp.blob();
        photos.push(new File([blob], "photo.jpg", { type: blob.type }));
      }

      if (extraPhoto1) photos.push(extraPhoto1);
      if (extraPhoto2) photos.push(extraPhoto2);

      if (photos.length === 0) {
        identifyState = "idle";
        return;
      }

      const response = await identifyPlant(photos);
      identifyResults = response.suggestions;
      currentSuggestion = 0;
      identifyState = "result";
    } catch (e: unknown) {
      identifyError = resolveError(e, "identifyPlant");
      identifyErrorRetryable =
        !(e instanceof ApiError) ||
        e.status === 500 ||
        e.status === 503 ||
        e.status === 429;
      identifyState = "error";
    }
  }

  function handleApply() {
    if (!activeSuggestion) return;
    appliedCount = onapply(activeSuggestion);
    identifyState = "applied";
  }

  function handleUndoClick() {
    onundo();
    identifyState = "idle";
    identifyResults = [];
    currentSuggestion = 0;
  }

  function handleDismiss() {
    identifyState = "idle";
    identifyResults = [];
    currentSuggestion = 0;
  }

  function prevSuggestion() {
    if (suggestionCount <= 1) return;
    currentSuggestion =
      (currentSuggestion - 1 + suggestionCount) % suggestionCount;
  }

  function nextSuggestion() {
    if (suggestionCount <= 1) return;
    currentSuggestion = (currentSuggestion + 1) % suggestionCount;
  }

  let swipeStartX = 0;
  let swiping = false;

  function handleSwipeStart(e: PointerEvent) {
    swipeStartX = e.clientX;
    swiping = true;
  }

  function handleSwipeEnd(e: PointerEvent) {
    if (!swiping) return;
    swiping = false;
    const dx = e.clientX - swipeStartX;
    if (Math.abs(dx) > 50) {
      if (dx < 0) nextSuggestion();
      else prevSuggestion();
    }
  }
</script>

<div class="identify-section">
  {#if identifyState === "idle"}
    <button type="button" class="identify-btn" onclick={handleIdentify}>
      <Sparkles size={18} />
      {$translations.identify.identifyPlant}
    </button>
    <div class="extra-photos-label">
      {$translations.identify.extraPhotosHint}
    </div>
    <div class="extra-photos">
      <div class="extra-photo-slot extra-photo-filled extra-photo-main">
        {#if photoPreview}
          <img src={photoPreview} alt={$translations.form.photoPreview} />
        {:else if existingPhotoUrl}
          <img src={existingPhotoUrl} alt="" />
        {/if}
      </div>
      {#if extraPreview1}
        <div class="extra-photo-slot extra-photo-filled">
          <img src={extraPreview1} alt={$translations.identify.closeUp} />
          <button
            type="button"
            class="extra-photo-remove"
            onclick={() => removeExtraPhoto(1)}
          >
            <X size={12} />
          </button>
        </div>
      {:else}
        <label class="extra-photo-slot">
          <Camera size={18} />
          <span>{$translations.identify.closeUp}</span>
          <input
            type="file"
            accept="image/jpeg,image/png,image/webp"
            class="file-input"
            onchange={(e) => handleExtraSelect(1, e)}
          />
        </label>
      {/if}
      {#if extraPreview2}
        <div class="extra-photo-slot extra-photo-filled">
          <img src={extraPreview2} alt={$translations.identify.stemPot} />
          <button
            type="button"
            class="extra-photo-remove"
            onclick={() => removeExtraPhoto(2)}
          >
            <X size={12} />
          </button>
        </div>
      {:else}
        <label class="extra-photo-slot">
          <Camera size={18} />
          <span>{$translations.identify.stemPot}</span>
          <input
            type="file"
            accept="image/jpeg,image/png,image/webp"
            class="file-input"
            onchange={(e) => handleExtraSelect(2, e)}
          />
        </label>
      {/if}
    </div>
  {:else if identifyState === "loading"}
    <div class="identify-loading-header">
      <span class="spinner"></span>
      <Sparkles size={16} />
      {$translations.identify.identifying}
    </div>
    <div class="loading-photos">
      {#if photoPreview}
        <img src={photoPreview} alt="" class="loading-thumb" />
      {:else if existingPhotoUrl}
        <img src={existingPhotoUrl} alt="" class="loading-thumb" />
      {/if}
      {#if extraPreview1}
        <img src={extraPreview1} alt="" class="loading-thumb" />
      {/if}
      {#if extraPreview2}
        <img src={extraPreview2} alt="" class="loading-thumb" />
      {/if}
    </div>
    <div class="shimmer-lines">
      <div class="shimmer"></div>
      <div class="shimmer"></div>
      <div class="shimmer"></div>
    </div>
  {:else if identifyState === "result" && activeSuggestion}
    <div class="suggestion-header">
      <Sparkles size={14} />
      {$translations.identify.aiSuggestion}
      {#if suggestionCount > 1}
        <span class="suggestion-counter"
          >{$translations.identify.suggestionCount
            .replace("{current}", String(currentSuggestion + 1))
            .replace("{total}", String(suggestionCount))}</span
        >
      {/if}
    </div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      class="suggestion-body"
      onpointerdown={suggestionCount > 1 ? handleSwipeStart : undefined}
      onpointerup={suggestionCount > 1 ? handleSwipeEnd : undefined}
    >
      <div class="suggestion-name">
        <span class="suggestion-scientific"
          >{activeSuggestion.scientific_name}</span
        >
        {#if activeSuggestion.confidence != null}
          <span class="suggestion-confidence"
            >{$translations.identify.confidence.replace(
              "{n}",
              String(Math.round(activeSuggestion.confidence * 100)),
            )}</span
          >
        {/if}
      </div>
      {#if activeSuggestion.common_name}
        <div class="suggestion-common">
          "{activeSuggestion.common_name}"
        </div>
      {/if}
      {#if activeSuggestion.summary}
        <div class="suggestion-summary">{activeSuggestion.summary}</div>
      {/if}
      {#if willFillChips.length > 0}
        <div class="will-fill">
          <div class="will-fill-label">
            {$translations.identify.willFill}
          </div>
          <div class="fill-chips">
            {#each willFillChips as chip (`${chip.label}-${chip.value}`)}
              <span class="fill-chip"
                ><Check size={11} /> {chip.label} ({chip.value})</span
              >
            {/each}
          </div>
        </div>
      {/if}
    </div>
    {#if suggestionCount > 1}
      <div class="suggestion-nav">
        <button
          type="button"
          class="nav-btn"
          onclick={prevSuggestion}
          aria-label={$translations.identify.prevSuggestion}
        >
          <ChevronLeft size={18} />
        </button>
        <div class="nav-dots">
          {#each identifyResults as _, i (i)}
            <button
              type="button"
              class="nav-dot"
              class:active={i === currentSuggestion}
              onclick={() => {
                currentSuggestion = i;
              }}
              aria-label={$translations.identify.suggestionCount
                .replace("{current}", String(i + 1))
                .replace("{total}", String(suggestionCount))}
            ></button>
          {/each}
        </div>
        <button
          type="button"
          class="nav-btn"
          onclick={nextSuggestion}
          aria-label={$translations.identify.nextSuggestion}
        >
          <ChevronRight size={18} />
        </button>
      </div>
    {/if}
    <div class="suggestion-actions">
      <button type="button" class="btn btn-ai" onclick={handleApply}
        >{$translations.identify.applyToForm}</button
      >
      <button type="button" class="btn btn-outline" onclick={handleDismiss}
        >{$translations.identify.dismiss}</button
      >
    </div>
  {:else if identifyState === "applied"}
    <div class="applied-banner">
      <Check size={18} />
      <span
        >{$translations.identify.applied.replace(
          "{n}",
          String(appliedCount),
        )}</span
      >
      <button type="button" class="applied-undo" onclick={handleUndoClick}
        >{$translations.identify.undo}</button
      >
    </div>
  {:else if identifyState === "error"}
    <div class="identify-error">
      <TriangleAlert size={18} />
      <span>{identifyError || $translations.identify.errorMessage}</span>
      {#if identifyErrorRetryable}
        <button
          type="button"
          class="btn btn-outline btn-ai btn-sm"
          onclick={handleIdentify}>{$translations.identify.retry}</button
        >
      {:else}
        <button
          type="button"
          class="btn btn-outline btn-sm"
          onclick={() => (identifyState = "idle")}
          >{$translations.identify.dismiss}</button
        >
      {/if}
    </div>
  {/if}
</div>

<style>
  .identify-section {
    border: 1px dashed var(--color-border);
    border-radius: var(--radius-card);
    padding: 16px;
    background: var(--color-ai-tint);
    margin-top: 4px;
    margin-bottom: 12px;
    width: 100%;
    box-sizing: border-box;
  }

  .identify-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    padding: 12px 16px;
    border: 1px solid
      color-mix(in srgb, var(--color-ai) 40%, var(--color-border));
    border-radius: var(--radius-btn);
    background: var(--color-ai-soft);
    color: var(--color-text);
    font-size: 15px;
    font-weight: 600;
    font-family: inherit;
    cursor: pointer;
    transition: all var(--transition-speed);
  }

  .identify-btn:hover {
    border-color: var(--color-ai);
    background: color-mix(in srgb, var(--color-ai) 20%, transparent);
  }

  .identify-btn :global(svg) {
    color: var(--color-ai);
  }

  .extra-photos-label {
    font-size: var(--fs-chip);
    color: var(--color-text-muted);
    margin-top: 12px;
    margin-bottom: 8px;
  }

  .extra-photos {
    display: flex;
    gap: 10px;
    flex-wrap: wrap;
  }

  .extra-photo-slot {
    width: 88px;
    height: 88px;
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-btn);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    color: var(--color-text-muted);
    font-size: 11px;
    cursor: pointer;
    transition: all var(--transition-speed);
    background: color-mix(in srgb, var(--color-surface) 60%, transparent);
    text-align: center;
    line-height: 1.2;
  }

  .extra-photo-slot:hover {
    border-color: var(--color-primary);
    color: var(--color-primary);
  }

  .extra-photo-filled {
    border-style: solid;
    position: relative;
    overflow: visible;
    padding: 0;
    cursor: default;
  }

  .extra-photo-filled:hover {
    border-color: var(--color-border);
    color: var(--color-text-muted);
  }

  .extra-photo-filled img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: 6px;
  }

  .extra-photo-remove {
    position: absolute;
    top: -6px;
    right: -6px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-danger);
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    padding: 0;
    transition: all var(--transition-speed);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
  }

  .extra-photo-remove:hover {
    background: color-mix(
      in srgb,
      var(--color-danger) 10%,
      var(--color-surface)
    );
    border-color: var(--color-danger);
    transform: scale(1.15);
  }

  .file-input {
    display: none;
  }

  /* ---- Loading state ---- */
  .identify-loading-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 16px;
    color: var(--color-text);
  }

  .identify-loading-header :global(svg) {
    color: var(--color-ai);
  }

  .spinner {
    display: inline-block;
    width: 16px;
    height: 16px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-ai);
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .loading-photos {
    display: flex;
    gap: 8px;
    margin-bottom: 16px;
  }

  .loading-thumb {
    width: 56px;
    height: 56px;
    border-radius: 6px;
    border: 1px solid var(--color-border);
    object-fit: cover;
  }

  /* ---- Suggestion card ---- */
  .suggestion-header {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: var(--fs-section-label);
    font-weight: 600;
    color: var(--color-ai);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 14px;
  }

  .suggestion-name {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    margin-bottom: 2px;
  }

  .suggestion-scientific {
    font-size: 18px;
    font-weight: 700;
    color: var(--color-text);
  }

  .suggestion-confidence {
    font-size: var(--fs-chip);
    font-weight: 600;
    color: var(--color-success);
    background: var(--color-success-soft);
    padding: 2px 10px;
    border-radius: var(--radius-pill);
    white-space: nowrap;
  }

  .suggestion-common {
    font-size: 14px;
    color: var(--color-text-muted);
    margin-bottom: 10px;
    font-style: italic;
  }

  .suggestion-summary {
    font-size: 14px;
    color: var(--color-text-muted);
    line-height: 1.5;
    margin-bottom: 16px;
  }

  .will-fill {
    margin-bottom: 16px;
  }

  .will-fill-label {
    font-size: var(--fs-chip);
    font-weight: 600;
    color: var(--color-text-muted);
    margin-bottom: 8px;
  }

  .fill-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .fill-chip {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 4px 10px;
    border-radius: var(--radius-pill);
    font-size: 12px;
    font-weight: 500;
    background: var(--color-primary-tint);
    color: var(--color-primary);
    border: 1px solid color-mix(in srgb, var(--color-primary) 25%, transparent);
  }

  .suggestion-body {
    touch-action: pan-y;
    user-select: none;
  }

  .suggestion-counter {
    margin-left: auto;
    font-size: var(--fs-chip);
    font-weight: 500;
    color: var(--color-text-muted);
    text-transform: none;
    letter-spacing: 0;
  }

  .suggestion-nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin: 14px 0 16px;
  }

  .nav-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    border: 1px solid var(--color-border);
    background: var(--color-surface);
    color: var(--color-text-muted);
    cursor: pointer;
    padding: 0;
    transition: all var(--transition-speed);
  }

  .nav-btn:hover {
    border-color: var(--color-ai);
    color: var(--color-ai);
    background: var(--color-ai-tint);
  }

  .nav-dots {
    display: flex;
    align-items: center;
    gap: 8px;
  }

  .nav-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    border: 1.5px solid var(--color-ai);
    background: transparent;
    padding: 0;
    cursor: pointer;
    transition: all var(--transition-speed);
  }

  .nav-dot.active {
    background: var(--color-ai);
  }

  .nav-dot:hover:not(.active) {
    background: color-mix(in srgb, var(--color-ai) 40%, transparent);
  }

  .suggestion-actions {
    display: flex;
    gap: 10px;
  }

  .suggestion-actions :global(.btn) {
    flex: 1;
  }

  /* ---- Applied state ---- */
  .applied-banner {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 10px 16px;
    background: color-mix(in srgb, var(--color-success) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-success) 30%, transparent);
    border-radius: var(--radius-btn);
    font-size: 14px;
    font-weight: 500;
    color: var(--color-text);
  }

  .applied-banner :global(svg) {
    color: var(--color-success);
    flex-shrink: 0;
  }

  .applied-undo {
    margin-left: auto;
    font-size: var(--fs-chip);
    color: var(--color-text-muted);
    cursor: pointer;
    text-decoration: underline;
    background: none;
    border: none;
    font-family: inherit;
    white-space: nowrap;
  }

  .applied-undo:hover {
    color: var(--color-text);
  }

  /* ---- Error state ---- */
  .identify-error {
    padding: 12px 16px;
    background: color-mix(in srgb, var(--color-danger) 8%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-danger) 30%, transparent);
    border-radius: var(--radius-btn);
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 14px;
    color: var(--color-text);
  }

  .identify-error :global(svg) {
    color: var(--color-danger);
    flex-shrink: 0;
  }

  .identify-error :global(.btn) {
    margin-left: auto;
    flex-shrink: 0;
  }

  @media (max-width: 768px) {
    .extra-photo-slot {
      width: 80px;
      height: 80px;
    }

    .suggestion-actions {
      flex-direction: column;
    }

    .suggestion-actions :global(.btn) {
      min-height: 44px;
    }

    .identify-error {
      flex-wrap: wrap;
    }

    .identify-error :global(.btn) {
      margin-left: 0;
      width: 100%;
    }

    .applied-banner {
      flex-wrap: wrap;
    }
  }
</style>
