<script lang="ts">
  import { MapPin } from "lucide-svelte";
  import type { Location } from "$lib/api";
  import { translations } from "$lib/stores/locale";

  let {
      locations,
      value = null,
      onchange,
      oncreate,
      showNone = true,
  }: {
    locations: Location[];
    value: number | null;
    onchange: (id: number | null) => void;
    oncreate?: (name: string) => Promise<{ location: Location } | { error: string }>;
    showNone?: boolean;
  } = $props();

  let adding = $state(false);
  let newName = $state("");
  let createError = $state("");

  function normalizeLocationName(name: string): string {
    return name.trim().toLocaleLowerCase();
  }

  function locationExistsMessage(name: string): string {
    const form = $translations.form as typeof $translations.form & {
      locationExists: string;
    };
    return form.locationExists.replace("{name}", name);
  }

  async function handleCreate() {
    const trimmed = newName.trim();
    if (!trimmed || !oncreate) return;

    const existing = locations.find(
      (loc) => normalizeLocationName(loc.name) === normalizeLocationName(trimmed),
    );
    if (existing) {
      createError = locationExistsMessage(existing.name);
      return;
    }

    const result = await oncreate(trimmed);
    if ("location" in result) {
      onchange(result.location.id);
      newName = "";
      createError = "";
      adding = false;
    } else {
      createError = result.error;
    }
  }
</script>

<div class="location-chips">
  {#if showNone}
    <button
      type="button"
      class="chip"
      class:active={value === null}
      onclick={() => onchange(null)}
    >
      {$translations.form.none}
    </button>
  {/if}
  {#each locations as loc (loc.id)}
    <button
      type="button"
      class="chip"
      class:active={value === loc.id}
      onclick={() => onchange(loc.id)}
    >
      <MapPin size={14} class="chip-icon" />
      {loc.name}
    </button>
  {/each}
  {#if adding}
    <form
      class="new-location"
      onsubmit={(e) => {
        e.preventDefault();
        handleCreate();
      }}
    >
      <div class="new-location-row">
        <input
          type="text"
          bind:value={newName}
          placeholder={$translations.form.locationName}
          class="input new-input"
          class:input-error={createError}
          oninput={() => {
            createError = "";
          }}
        />
        <button type="submit" class="chip add-btn"
          >{$translations.form.add}</button
        >
        <button
          type="button"
          class="chip"
          onclick={() => {
            adding = false;
            newName = "";
            createError = "";
          }}
        >
          {$translations.common.cancel}
        </button>
      </div>
      {#if createError}
        <span class="field-error" aria-live="polite">{createError}</span>
      {/if}
    </form>
  {:else}
    <button
      type="button"
      class="chip chip-dashed"
      onclick={() => {
        adding = true;
        createError = "";
      }}
    >
      {$translations.form.newLocation}
    </button>
  {/if}
</div>

<style>
  .location-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: flex-start;
  }

  .chip {
    padding: 8px 14px;
  }

  .new-location {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .new-location-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .add-btn {
    background: var(--color-primary);
    color: var(--color-text-on-primary);
    border-color: var(--color-primary);
  }

  .add-btn:hover {
    background: var(--color-primary-dark);
    border-color: var(--color-primary-dark);
  }

  .new-input {
    width: 140px;
    padding: 8px 14px;
    font-size: var(--fs-chip);
    border-radius: var(--radius-pill);
  }

  .field-error {
    font-size: var(--fs-chip);
    color: var(--color-danger);
  }
</style>
