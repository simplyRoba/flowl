<script lang="ts">
  import { onMount } from "svelte";
  import { get } from "svelte/store";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import type { CreatePlant } from "$lib/api";
  import {
    currentPlant,
    plantsError,
    loadPlant,
    updatePlant,
    uploadPhoto,
    deletePhoto,
  } from "$lib/stores/plants";
  import { translations } from "$lib/stores/locale";
  import { pushNotification } from "$lib/stores/notifications";
  import PlantForm from "$lib/components/PlantForm.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  let saving = $state(false);
  let loaded = $state(false);
  let loadError = $state<string | null>(null);

  onMount(async () => {
    const id = Number($page.params.id);
    const plant = await loadPlant(id);
    if (!plant) {
      loadError = get(plantsError) ?? get(translations).error.loadPlant;
    }
    loaded = true;
  });

  async function handleSave(data: CreatePlant, photo?: File) {
    if (!$currentPlant) return;
    saving = true;
    const plant = await updatePlant($currentPlant.id, data);
    if (!plant) {
      pushNotification({
        title: $translations.plant.editPlant,
        variant: "error",
        message: $translations.error.updatePlant,
      });
      plantsError.set(null);
      saving = false;
      return;
    }

    if (photo) {
      const uploaded = await uploadPhoto(plant.id, photo);
      if (!uploaded) {
        pushNotification({
          title: $translations.form.media,
          variant: "error",
          message: $translations.error.uploadPhoto,
        });
        plantsError.set(null);
        saving = false;
        return;
      }
    }

    goto(resolve(`/plants/${plant.id}`));
    saving = false;
  }

  async function handleRemovePhoto() {
    if (!$currentPlant) return;
    await deletePhoto($currentPlant.id);
  }
</script>

<div class="page">
  <PageHeader
    backHref={$currentPlant ? `/plants/${$currentPlant.id}` : "/"}
    backLabel={$translations.common.cancel}
  >
    <button
      type="submit"
      form="plant-form"
      class="btn btn-primary"
      disabled={saving}
    >
      {saving ? $translations.common.saving : $translations.common.save}
    </button>
  </PageHeader>

  <h1>{$translations.plant.editPlant}</h1>

  {#if loadError}
    <p class="error">{loadError}</p>
  {:else if loaded && $currentPlant}
    <PlantForm
      initial={$currentPlant}
      onsave={handleSave}
      onremovephoto={handleRemovePhoto}
      {saving}
      showFooterActions={false}
    />
  {:else}
    <p class="loading">{$translations.common.loading}</p>
  {/if}
</div>

<style>
  .page {
    max-width: var(--content-width-narrow);
    margin: 0 auto;
  }

  h1 {
    font-size: var(--fs-page-title);
    font-weight: 700;
    margin: 0 0 24px;
  }

  .error {
    color: var(--color-danger);
    padding: 16px;
  }

  .loading {
    color: var(--color-text-muted);
    padding: 16px;
  }

  @media (max-width: 768px) {
    .page {
      padding-bottom: 64px;
    }

    h1 {
      margin-bottom: 16px;
    }
  }
</style>
