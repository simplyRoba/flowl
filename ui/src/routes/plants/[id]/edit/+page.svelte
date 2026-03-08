<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import type { CreatePlant } from "$lib/api";
  import {
    plantsError,
    updatePlant,
    uploadPhoto,
    deletePhoto,
  } from "$lib/stores/plants";
  import { translations } from "$lib/stores/locale";
  import { pushNotification } from "$lib/stores/notifications";
  import PlantForm from "$lib/components/PlantForm.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";

  interface Props {
    data: {
      plant: import("$lib/api").Plant | null;
      notFound: boolean;
      loadError: string | null;
    };
  }

  const props = $props();
  let data = $derived((props as Props).data);

  let plant = $state<import("$lib/api").Plant | null>(null);
  let loadError = $state<string | null>(null);
  let notFound = $state(false);
  let saving = $state(false);

  $effect(() => {
    plant = data.plant;
    loadError = data.loadError;
    notFound = data.notFound;
    saving = false;
  });

  async function handleSave(data: CreatePlant, photo?: File) {
    if (!plant) return;
    saving = true;
    const updatedPlant = await updatePlant(plant.id, data);
    if (!updatedPlant) {
      pushNotification({
        title: $translations.plant.editPlant,
        variant: "error",
        message: $translations.error.updatePlant,
      });
      plantsError.set(null);
      saving = false;
      return;
    }

    plant = updatedPlant;

    if (photo) {
      const uploaded = await uploadPhoto(updatedPlant.id, photo);
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

      plant = uploaded;
    }

    goto(resolve(`/plants/${updatedPlant.id}`));
    saving = false;
  }

  async function handleRemovePhoto() {
    if (!plant) return;
    const removed = await deletePhoto(plant.id);
    if (removed) {
      plant = { ...plant, photo_url: null };
    }
  }
</script>

<div class="page">
  <PageHeader
    backHref={plant ? `/plants/${plant.id}` : "/"}
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

  {#if notFound}
    <p class="error">{$translations.plant.notFound}</p>
  {:else if loadError}
    <p class="error">{loadError}</p>
  {:else if plant}
    {#key plant.id}
      <PlantForm
        initial={plant}
        onsave={handleSave}
        onremovephoto={handleRemovePhoto}
        {saving}
        showFooterActions={false}
      />
    {/key}
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
