<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import type { CreatePlant } from "$lib/api";
  import { createPlant, updatePlant, uploadPhoto } from "$lib/stores/plants";
  import { translations } from "$lib/stores/locale";
  import { pushNotification } from "$lib/stores/notifications";
  import { isOffline } from "$lib/stores/network";
  import PlantForm from "$lib/components/PlantForm.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import OfflineMessage from "$lib/components/OfflineMessage.svelte";

  let saving = $state(false);
  let draftPlantId: number | null = $state(null);

  async function handleSave(data: CreatePlant, photo?: File) {
    saving = true;
    const plant = draftPlantId
      ? await updatePlant(draftPlantId, data)
      : await createPlant(data);

    if (!plant) {
      pushNotification({
        title: draftPlantId
          ? $translations.plant.editPlant
          : $translations.plant.addPlant,
        variant: "error",
        message: draftPlantId
          ? $translations.error.updatePlant
          : $translations.error.createPlant,
      });
      saving = false;
      return;
    }

    if (photo) {
      const uploaded = await uploadPhoto(plant.id, photo);
      if (!uploaded) {
        draftPlantId = plant.id;
        pushNotification({
          title: $translations.form.media,
          variant: "error",
          message: $translations.error.uploadPhoto,
        });
        saving = false;
        return;
      }
    }

    draftPlantId = null;
    goto(resolve(`/plants/${plant.id}`));
    saving = false;
  }
</script>

<div class="page">
  <PageHeader backHref="/" backLabel={$translations.common.cancel}>
    <button
      type="submit"
      form="plant-form"
      class="btn btn-primary"
      disabled={saving || $isOffline}
    >
      {saving ? $translations.common.saving : $translations.common.save}
    </button>
  </PageHeader>

  <h1>{$translations.plant.addPlant}</h1>

  {#if $isOffline}
    <OfflineMessage />
  {:else}
    <PlantForm
      onsave={handleSave}
      {saving}
      showLocationNone={false}
      showFooterActions={false}
    />
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

  @media (max-width: 768px) {
    .page {
      padding-bottom: 64px;
    }

    h1 {
      margin-bottom: 16px;
    }
  }
</style>
