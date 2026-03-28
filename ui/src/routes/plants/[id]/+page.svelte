<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/stores";
  import {
    ArrowLeft,
    Pencil,
    Trash2,
    Droplet,
    Droplets,
    MapPin,
    Sun,
    CloudSun,
    Cloud,
    Leaf,
    Shovel,
    Scissors,
    BookOpen,
    Pencil as PencilIcon,
    Info,
    Gauge,
    PawPrint,
    TrendingUp,
    Layers,
    Repeat,
    CalendarCheck,
    CalendarClock,
    Sparkles,
    ChevronRight,
  } from "lucide-svelte";
  import { plantsError, deletePlant, waterPlant } from "$lib/stores/plants";
  import { careError } from "$lib/stores/care";
  import { resolveError } from "$lib/stores/errors";
  import { translations } from "$lib/stores/locale";
  import { isOffline } from "$lib/stores/network";
  import { pushNotification } from "$lib/stores/notifications";
  import {
    deleteCareEvent,
    fetchCareEvents,
    fetchPlant,
    type CareEvent,
    type Plant,
  } from "$lib/api";
  import { emojiToSvgPath } from "$lib/emoji";
  import { thumbUrl, thumbSrcset } from "$lib/thumbUrl";
  import StatusBadge from "$lib/components/StatusBadge.svelte";
  import PageHeader from "$lib/components/PageHeader.svelte";
  import PhotoLightbox from "$lib/components/PhotoLightbox.svelte";
  import ModalDialog from "$lib/components/ModalDialog.svelte";
  import ChatDrawer from "$lib/components/ChatDrawer.svelte";
  import CareEntryForm from "$lib/components/CareEntryForm.svelte";
  import { aiStatus, loadAiStatus } from "$lib/stores/ai";
  import {
    groupCareEvents,
    isGroup,
    type WateringGroup,
  } from "$lib/careGrouping";
  import { SvelteSet } from "svelte/reactivity";

  interface Props {
    data: {
      plant: Plant | null;
      notFound: boolean;
      loadErrorCode: string | null;
    };
  }

  const props = $props();
  let data = $derived((props as Props).data);

  type BackPath = "/" | "/care-journal" | "/plants" | "/settings";

  let plant = $state<Plant | null>(null);
  let plantLoadErrorCode = $state<string | null>(null);
  let careEvents = $state<CareEvent[]>([]);
  let careLoading = $state(false);
  let notFound = $state(false);
  let deleting = $state(false);
  let watering = $state(false);
  let showLogForm = $state(false);
  let showAllEvents = $state(false);
  let deletingEventId = $state<number | null>(null);
  let deleteEventDialogTarget = $state<CareEvent | null>(null);
  let backHref = $state<BackPath>("/");
  let lightboxOpen = $state(false);
  let deleteDialogOpen = $state(false);
  let aiEnabled = $derived($aiStatus?.enabled ?? false);
  let chatOpen = $state(false);
  let lightboxSrc = $state("");
  const BACK_PATHS = new Set<BackPath>([
    "/",
    "/care-journal",
    "/plants",
    "/settings",
  ]);

  const EVENT_LIMIT = 20;

  onMount(() => {
    loadAiStatus();
  });

  async function loadCareEvents(plantId: number) {
    careLoading = true;
    careEvents = [];
    try {
      careEvents = await fetchCareEvents(plantId);
    } catch {
      careEvents = [];
    }
    careLoading = false;
  }

  $effect(() => {
    plant = data.plant;
    plantLoadErrorCode = data.loadErrorCode;
    notFound = data.notFound;
    showLogForm = false;
    showAllEvents = false;
    deletingEventId = null;
    deleteEventDialogTarget = null;
    deleteDialogOpen = false;
    lightboxOpen = false;
    lightboxSrc = "";
    chatOpen = false;
    expandedGroups.clear();
    if (data.plant) {
      loadCareEvents(data.plant.id);
    } else {
      careEvents = [];
      careLoading = false;
    }
  });

  $effect(() => {
    const from = $page.url.searchParams.get("from") as BackPath | null;
    backHref = from && BACK_PATHS.has(from) ? from : "/";
  });

  async function refreshPlantDetails(plantId: number) {
    const [nextPlant] = await Promise.all([
      fetchPlant(plantId),
      loadCareEvents(plantId),
    ]);

    plant = nextPlant;
    plantLoadErrorCode = null;
  }

  function handleDelete() {
    deleteDialogOpen = true;
  }

  async function handleDeleteConfirm() {
    deleteDialogOpen = false;
    if (!plant) return;
    deleting = true;
    try {
      const plantName = plant.name;
      const success = await deletePlant(plant.id);
      if (success) {
        pushNotification({
          title: $translations.plant.deletePlant,
          variant: "success",
          message: $translations.notifications.plantDeleted.replace(
            "{name}",
            plantName,
          ),
        });
        goto(resolve("/"));
      }
    } finally {
      deleting = false;
    }
  }

  function lightLabel(needs: string) {
    if (needs === "direct") return $translations.plant.lightDirect;
    if (needs === "low") return $translations.plant.lightLow;
    return $translations.plant.lightIndirect;
  }

  function lightIcon(needs: string) {
    if (needs === "direct") return Sun;
    if (needs === "low") return Cloud;
    return CloudSun;
  }

  async function handleWater() {
    if (!plant || watering) return;
    watering = true;
    try {
      const wateredPlant = await waterPlant(plant.id);
      if (!wateredPlant) {
        pushNotification({
          title: $translations.plant.wateringSection,
          variant: "error",
          message: $plantsError || $translations.error.waterPlant,
        });
        plantsError.set(null);
        return;
      }
      plant = wateredPlant;
      loadCareEvents(wateredPlant.id);
    } finally {
      watering = false;
    }
  }

  function formatDate(dateStr: string | null): string {
    if (!dateStr) return $translations.plant.never;
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    return date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  }

  function eventTypeLabel(type: string): string {
    if (type === "watered") return $translations.care.watered;
    if (type === "fertilized") return $translations.care.fertilized;
    if (type === "repotted") return $translations.care.repotted;
    if (type === "pruned") return $translations.care.pruned;
    if (type === "ai-consultation") return $translations.care.aiConsultation;
    return $translations.care.custom;
  }

  function formatShortDate(dateStr: string, includeYear = true): string {
    const date = new Date(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    const opts: Intl.DateTimeFormatOptions = {
      month: "short",
      day: "numeric",
    };
    if (includeYear) opts.year = "2-digit";
    return date.toLocaleDateString(undefined, opts);
  }

  function handleEventDelete(event: CareEvent) {
    deleteEventDialogTarget = event;
  }

  async function handleEventDeleteConfirm() {
    const event = deleteEventDialogTarget;
    deleteEventDialogTarget = null;
    if (!plant || !event || deletingEventId === event.id) return;
    deletingEventId = event.id;
    try {
      await deleteCareEvent(plant.id, event.id);
      await refreshPlantDetails(plant.id);
    } catch (error) {
      careError.set(resolveError(error, "deleteCareEvent"));
      pushNotification({
        title: $translations.plant.careJournalSection,
        variant: "error",
        message: $careError || $translations.error.deleteCareEvent,
      });
      careError.set(null);
    }
    deletingEventId = null;
  }

  let expandedGroups: Set<string> = new SvelteSet();

  let displayEvents = $derived(
    showAllEvents ? careEvents : careEvents.slice(0, EVENT_LIMIT),
  );

  let groupedTimeline = $derived(groupCareEvents(displayEvents));

  let hasMoreEvents = $derived(careEvents.length > EVENT_LIMIT);

  function careGroupKey(group: WateringGroup): string {
    return `${group.plantId}-${group.firstAt}`;
  }

  function careGroupLabel(group: WateringGroup): string {
    return $translations.care.wateredCount.replace(
      "{count}",
      String(group.count),
    );
  }

  function toggleCareGroup(key: string) {
    if (expandedGroups.has(key)) {
      expandedGroups.delete(key);
    } else {
      expandedGroups.add(key);
    }
  }

  let LightNeedsIcon = $derived(plant ? lightIcon(plant.light_needs) : Sun);

  function difficultyLabel(val: string): string {
    if (val === "easy") return $translations.plant.difficultyEasy;
    if (val === "moderate") return $translations.plant.difficultyModerate;
    return $translations.plant.difficultyDemanding;
  }

  function petSafetyLabel(val: string): string {
    if (val === "safe") return $translations.plant.petSafe;
    if (val === "caution") return $translations.plant.petCaution;
    return $translations.plant.petToxic;
  }

  function growthSpeedLabel(val: string): string {
    if (val === "slow") return $translations.plant.growthSlow;
    if (val === "moderate") return $translations.plant.growthModerate;
    return $translations.plant.growthFast;
  }

  function soilMoistureLabel(val: string): string {
    if (val === "dry") return $translations.plant.moistureDry;
    if (val === "moderate") return $translations.plant.moistureModerate;
    return $translations.plant.moistureMoist;
  }

  function soilTypeLabel(val: string): string {
    if (val === "standard") return $translations.plant.soilStandard;
    if (val === "cactus-mix") return $translations.plant.soilCactus;
    if (val === "orchid-bark") return $translations.plant.soilOrchid;
    return $translations.plant.soilPeat;
  }

  function openLightbox(src?: string) {
    const url = src || plant?.photo_url;
    if (!url) return;
    lightboxSrc = url;
    lightboxOpen = true;
  }

  function closeLightbox() {
    lightboxOpen = false;
  }
</script>

{#if notFound}
  <div class="not-found">
    <h2>{$translations.plant.notFound}</h2>
    <p>{$translations.plant.notFoundHint}</p>
    <a href={resolve("/")} class="back-link"
      ><ArrowLeft size={16} /> {$translations.plant.backToPlants}</a
    >
  </div>
{:else if plant}
  <div class="detail">
    <div class="detail-content">
      <PageHeader
        backHref={backHref as BackPath}
        backLabel={$translations.common.back}
      >
        <a
          href={resolve(`/plants/${plant.id}/edit`)}
          class="btn btn-icon"
          class:disabled={$isOffline}
          aria-label={$translations.plant.editPlant}
          aria-disabled={$isOffline}
          onclick={(e) => {
            if ($isOffline) e.preventDefault();
          }}
        >
          <Pencil size={16} />
        </a>
        <button
          class="btn btn-icon btn-danger"
          aria-label={$translations.plant.deletePlant}
          onclick={handleDelete}
          disabled={$isOffline || deleting}
        >
          <Trash2 size={16} />
        </button>
      </PageHeader>

      <div class="detail-hero">
        <div class="detail-photo">
          {#if plant.photo_url}
            <button
              type="button"
              class="detail-photo-button"
              aria-label={$translations.plant.openPhoto}
              onclick={() => openLightbox()}
            >
              <img
                src={thumbUrl(plant.photo_url, 200)}
                srcset={thumbSrcset(plant.photo_url)}
                sizes="(max-width: 768px) 100vw, (min-width: 1280px) 300px, 260px"
                alt={plant.name}
                class="detail-photo-img"
                onerror={(e) => {
                  const img = e.currentTarget as HTMLImageElement;
                  img.onerror = null;
                  img.src = plant!.photo_url!;
                }}
              />
            </button>
          {:else}
            <img
              src={emojiToSvgPath(plant.icon)}
              alt={plant.name}
              class="detail-photo-icon"
            />
          {/if}
        </div>
        <div class="detail-info">
          <div class="detail-name">
            <h2>{plant.name}</h2>
            {#if plant.species}
              <span class="detail-species">{plant.species}</span>
            {/if}
          </div>
          {#if plant.location_name}
            <p class="detail-location">
              <MapPin size={14} />
              {plant.location_name}
            </p>
          {/if}
          <div class="detail-status">
            <StatusBadge
              status={plant.watering_status}
              nextDue={plant.next_due ?? null}
            />
          </div>
          <div class="hero-actions">
            <button
              class="btn btn-water btn-lg"
              onclick={handleWater}
              disabled={$isOffline || watering}
            >
              <Droplet size={16} />
              {watering
                ? $translations.dashboard.watering
                : $translations.plant.waterNow}
            </button>
            {#if aiEnabled}
              <button
                class="btn btn-ai btn-lg"
                onclick={() => (chatOpen = true)}
                disabled={$isOffline}
              >
                <Sparkles size={16} />
                {$translations.chat.askAi}
              </button>
            {/if}
          </div>
        </div>
      </div>

      <div class="detail-sections">
        <div class="detail-grid">
          <div class="section">
            <div class="section-title">
              <Droplet size={14} />
              {$translations.plant.wateringSection}
            </div>
            <div class="detail-row">
              <span class="detail-row-label"
                >{$translations.plant.interval}</span
              ><span class="detail-row-value"
                >{$translations.plant.everyNDays.replace(
                  "{n}",
                  String(plant.watering_interval_days),
                )}
                <Repeat size={14} /></span
              >
            </div>
            <div class="detail-row">
              <span class="detail-row-label"
                >{$translations.plant.lastWatered}</span
              ><span class="detail-row-value"
                >{formatDate(plant.last_watered)}
                <CalendarCheck size={14} /></span
              >
            </div>
            <div class="detail-row">
              <span class="detail-row-label">{$translations.plant.nextDue}</span
              ><span class="detail-row-value"
                >{plant.next_due
                  ? formatDate(plant.next_due)
                  : $translations.plant.na}
                <CalendarClock size={14} /></span
              >
            </div>
            {#if plant.soil_moisture}
              <div class="detail-row">
                <span class="detail-row-label"
                  >{$translations.plant.soilMoisture}</span
                >
                <span class="detail-row-value"
                  >{soilMoistureLabel(plant.soil_moisture)}
                  <Droplets size={14} /></span
                >
              </div>
            {/if}
          </div>
          <div class="section">
            <div class="section-title">
              <Info size={14} />
              {$translations.plant.careInfoSection}
            </div>
            <div class="detail-row">
              <span class="detail-row-label">{$translations.plant.light}</span>
              <span class="detail-row-value">
                {lightLabel(plant.light_needs)}
                <LightNeedsIcon size={14} />
              </span>
            </div>
            {#if plant.difficulty}
              <div class="detail-row">
                <span class="detail-row-label"
                  >{$translations.plant.difficulty}</span
                >
                <span class="detail-row-value"
                  >{difficultyLabel(plant.difficulty)}
                  <Gauge size={14} /></span
                >
              </div>
            {/if}
            {#if plant.pet_safety}
              <div class="detail-row">
                <span class="detail-row-label"
                  >{$translations.plant.petSafety}</span
                >
                <span class="detail-row-value"
                  >{petSafetyLabel(plant.pet_safety)}
                  <PawPrint size={14} /></span
                >
              </div>
            {/if}
            {#if plant.growth_speed}
              <div class="detail-row">
                <span class="detail-row-label"
                  >{$translations.plant.growth}</span
                >
                <span class="detail-row-value"
                  >{growthSpeedLabel(plant.growth_speed)}
                  <TrendingUp size={14} /></span
                >
              </div>
            {/if}
            {#if plant.soil_type}
              <div class="detail-row">
                <span class="detail-row-label">{$translations.plant.soil}</span>
                <span class="detail-row-value"
                  >{soilTypeLabel(plant.soil_type)}
                  <Layers size={14} /></span
                >
              </div>
            {/if}
          </div>
        </div>

        {#if plant.notes}
          <div class="section">
            <div class="section-title">
              <PencilIcon size={14} />
              {$translations.plant.notesSection}
            </div>
            <div class="detail-notes">{plant.notes}</div>
          </div>
        {/if}

        <div class="section care-journal">
          <div class="section-title">
            <BookOpen size={14} />
            {$translations.plant.careJournalSection}
          </div>

          {#if careLoading}
            <div class="skeleton-list">
              {#each { length: 4 } as _, i (i)}
                <div class="skeleton-entry">
                  <div
                    class="shimmer skeleton-icon"
                    style="width: 24px; height: 24px; border-radius: 6px"
                  ></div>
                  <div class="shimmer-lines">
                    <div class="shimmer" style="width: 50%"></div>
                    <div class="shimmer" style="width: 30%"></div>
                  </div>
                </div>
              {/each}
            </div>
          {:else if careEvents.length === 0}
            <p class="journal-empty">{$translations.plant.noCareEvents}</p>
          {:else}
            <ul class="timeline">
              {#each groupedTimeline as item (isGroup(item) ? careGroupKey(item) : item.id)}
                {#if isGroup(item)}
                  {@const key = careGroupKey(item)}
                  {@const expanded = expandedGroups.has(key)}
                  <li class="timeline-item timeline-group-summary">
                    <button
                      class="timeline-group-btn"
                      onclick={() => toggleCareGroup(key)}
                    >
                      <span class="timeline-icon">
                        <Droplet size={12} />
                      </span>
                      <span class="timeline-text">
                        <span class="timeline-top">
                          <span class="timeline-label"
                            >{careGroupLabel(item)}</span
                          >
                          <span class="timeline-date"
                            >{formatShortDate(
                              item.firstAt,
                              new Date(item.firstAt).getFullYear() !==
                                new Date(item.lastAt).getFullYear(),
                            )} – {formatShortDate(item.lastAt)}</span
                          >
                        </span>
                      </span>
                      <span class="timeline-group-chevron" class:expanded>
                        <ChevronRight size={14} />
                      </span>
                    </button>
                  </li>
                  {#if expanded}
                    <li class="timeline-nested-group">
                      {#each item.events as event (event.id)}
                        <div class="timeline-nested">
                          <span class="timeline-icon timeline-icon-sm">
                            <Droplet size={10} />
                          </span>
                          <span class="timeline-text">
                            <span class="timeline-top">
                              <span class="timeline-label"
                                >{$translations.care.watered}</span
                              >
                              <span class="timeline-date"
                                >{formatShortDate(event.occurred_at)}</span
                              >
                            </span>
                          </span>
                          <span class="timeline-actions">
                            <button
                              class="btn btn-ghost event-delete"
                              onclick={() => handleEventDelete(event)}
                              disabled={$isOffline ||
                                deletingEventId === event.id}
                              aria-label={$translations.plant.deleteLogEntry}
                            >
                              <Trash2 size={16} />
                            </button>
                          </span>
                        </div>
                      {/each}
                    </li>
                  {/if}
                {:else}
                  <li class="timeline-item">
                    <span class="timeline-icon">
                      {#if item.event_type === "watered"}
                        <Droplet size={12} />
                      {:else if item.event_type === "fertilized"}
                        <Leaf size={12} />
                      {:else if item.event_type === "repotted"}
                        <Shovel size={12} />
                      {:else if item.event_type === "pruned"}
                        <Scissors size={12} />
                      {:else if item.event_type === "ai-consultation"}
                        <Sparkles size={12} />
                      {:else}
                        <PencilIcon size={12} />
                      {/if}
                    </span>
                    <span class="timeline-text">
                      <span class="timeline-top">
                        <span class="timeline-label"
                          >{eventTypeLabel(item.event_type)}</span
                        >
                        <span class="timeline-date"
                          >{formatShortDate(item.occurred_at)}</span
                        >
                      </span>
                      {#if item.photo_url}
                        <button
                          class="timeline-photo"
                          onclick={() => openLightbox(item.photo_url!)}
                        >
                          <img
                            src={thumbUrl(item.photo_url, 200)}
                            srcset={thumbSrcset(item.photo_url)}
                            sizes="72px"
                            alt=""
                            onerror={(e) => {
                              const img = e.currentTarget as HTMLImageElement;
                              img.onerror = null;
                              img.src = item.photo_url!;
                            }}
                          />
                        </button>
                      {/if}
                      {#if item.notes}
                        <span class="timeline-sub">{item.notes}</span>
                      {/if}
                    </span>
                    <span class="timeline-actions">
                      <button
                        class="btn btn-ghost event-delete"
                        onclick={() => handleEventDelete(item)}
                        disabled={$isOffline || deletingEventId === item.id}
                        aria-label={$translations.plant.deleteLogEntry}
                      >
                        <Trash2 size={16} />
                      </button>
                    </span>
                  </li>
                {/if}
              {/each}
            </ul>
            {#if hasMoreEvents && !showAllEvents}
              <button
                class="btn btn-ghost"
                onclick={() => (showAllEvents = true)}
                >{$translations.plant.showMore}</button
              >
            {/if}
          {/if}

          {#if showLogForm}
            <CareEntryForm
              plantId={plant.id}
              onsubmit={async () => {
                const plantId = plant!.id;
                await refreshPlantDetails(plantId);
                showLogForm = false;
              }}
              oncancel={() => {
                showLogForm = false;
              }}
            />
          {:else}
            <button
              class="btn btn-ghost"
              onclick={() => (showLogForm = true)}
              disabled={$isOffline}
            >
              {$translations.plant.addLogEntry}
            </button>
          {/if}
        </div>
      </div>

      <PhotoLightbox
        open={lightboxOpen}
        src={lightboxSrc}
        alt={plant.name}
        onclose={closeLightbox}
      />
    </div>

    <ChatDrawer
      {plant}
      open={chatOpen}
      onclose={() => {
        chatOpen = false;
      }}
      onsave={() => refreshPlantDetails(plant!.id)}
    />
  </div>
{:else if plantLoadErrorCode || $plantsError}
  <p class="error">
    {plantLoadErrorCode
      ? ($translations.errorCode[
          plantLoadErrorCode as keyof typeof $translations.errorCode
        ] ?? $translations.error.loadPlant)
      : $plantsError}
  </p>
{:else}
  <p class="loading">{$translations.common.loading}</p>
{/if}

<ModalDialog
  open={deleteDialogOpen}
  title={$translations.plant.deletePlant}
  message={plant
    ? $translations.plant.deleteConfirm.replace("{name}", plant.name)
    : ""}
  mode="confirm"
  variant="danger"
  confirmLabel={$translations.common.delete}
  onconfirm={handleDeleteConfirm}
  oncancel={() => {
    deleteDialogOpen = false;
  }}
/>

<ModalDialog
  open={deleteEventDialogTarget !== null}
  title={$translations.plant.deleteLogEntry}
  message={$translations.plant.deleteLogEntryConfirm}
  mode="confirm"
  variant="danger"
  confirmLabel={$translations.common.delete}
  onconfirm={handleEventDeleteConfirm}
  oncancel={() => {
    deleteEventDialogTarget = null;
  }}
/>

<style>
  .detail {
    max-width: var(--content-width-default);
    margin: 0 auto;
  }

  .detail-content {
    min-width: 0;
  }

  .hero-actions {
    display: flex;
    gap: 8px;
    margin-top: 4px;
  }

  .detail-hero {
    display: flex;
    align-items: flex-start;
    gap: 20px;
    margin-bottom: 24px;
  }

  .detail-photo {
    width: 260px;
    height: 260px;
    flex-shrink: 0;
    border-radius: var(--radius-card);
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .detail-photo-button {
    width: 100%;
    height: 100%;
    border: none;
    background: transparent;
    padding: 0;
    cursor: zoom-in;
  }

  .detail-photo-img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    border-radius: var(--radius-card);
  }

  .detail-photo-icon {
    width: 110px;
    height: 110px;
  }

  .detail-name {
    display: flex;
    align-items: baseline;
    flex-wrap: wrap;
    gap: 6px;
    margin-bottom: 6px;
  }

  .detail-info h2 {
    font-size: var(--fs-page-title);
    font-weight: 700;
    margin: 0;
  }

  .detail-species {
    color: var(--color-text-muted);
    font-size: 14px;
    font-style: italic;
  }

  .detail-info {
    flex: 1;
  }

  .detail-location {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    color: var(--color-text-muted);
    font-size: 14px;
    margin: 0 0 10px;
  }

  .detail-status {
    margin-bottom: 14px;
  }

  .detail-sections {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .detail-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 16px;
  }

  .detail-row {
    display: flex;
    justify-content: space-between;
    padding: 6px 0;
    font-size: 14px;
  }

  .detail-row-label {
    color: var(--color-text-muted);
  }

  .detail-row-value {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }

  .detail-notes {
    font-size: 14px;
    line-height: 1.5;
    color: var(--color-text);
    white-space: pre-wrap;
  }

  .not-found {
    text-align: center;
    padding: 64px 24px;
  }

  .not-found h2 {
    font-size: var(--fs-page-title);
    font-weight: 600;
    margin: 0 0 8px;
  }

  .not-found p {
    color: var(--color-text-muted);
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

  .care-journal {
    margin-bottom: 16px;
  }

  .journal-empty {
    color: var(--color-text-muted);
    font-size: 14px;
    margin: 8px 0 0;
  }

  .timeline {
    list-style: none;
    margin: 0;
    padding: 0;
  }

  .timeline-item {
    display: flex;
    gap: 10px;
    padding: 8px 0;
    font-size: 14px;
    border-bottom: 1px solid var(--color-border);
    align-items: flex-start;
  }

  .timeline-item:last-child {
    border-bottom: none;
  }

  .timeline-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 12px;
    flex-shrink: 0;
    background: var(--color-surface-muted);
    color: var(--color-text-muted);
  }

  .timeline-text {
    flex: 1;
    min-width: 0;
  }

  .timeline-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
    min-height: 24px;
    margin-bottom: 2px;
  }

  .timeline-label {
    font-weight: 500;
  }

  .timeline-date {
    font-size: 12px;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }

  .timeline-photo {
    float: right;
    width: 72px;
    height: 72px;
    border-radius: 8px;
    overflow: hidden;
    border: none;
    padding: 0;
    background: none;
    cursor: zoom-in;
    margin-left: 12px;
    margin-bottom: 4px;
  }

  .timeline-photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .timeline-actions {
    display: flex;
    align-items: center;
    min-height: 24px;
  }

  .event-delete {
    color: var(--color-text-muted);
  }

  .event-delete:hover:not(:disabled) {
    color: var(--color-danger);
    opacity: 1;
  }

  .timeline-sub {
    display: block;
    color: var(--color-text-muted);
    font-size: 13px;
    margin-top: 2px;
  }

  /* ---- Group summary ---- */
  .timeline-group-summary {
    padding: 0;
  }

  .timeline-group-summary:has(+ .timeline-nested-group) {
    border-bottom: none;
  }

  .timeline-nested-group {
    padding-left: 34px;
    border-bottom: 1px solid var(--color-border);
  }

  .timeline-nested-group:last-child {
    border-bottom: none;
  }

  .timeline-group-btn {
    display: flex;
    gap: 10px;
    align-items: center;
    width: 100%;
    padding: 8px 0;
    background: none;
    border: none;
    font-family: inherit;
    font-size: 14px;
    cursor: pointer;
    text-align: left;
    color: inherit;
  }

  .timeline-group-btn:hover {
    background: var(--color-surface-muted);
  }

  .timeline-group-chevron {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    color: var(--color-text-muted);
    transition: transform var(--transition-speed);
  }

  .timeline-group-chevron.expanded {
    transform: rotate(90deg);
  }

  .timeline-nested {
    display: flex;
    gap: 10px;
    padding: 6px 0;
    font-size: 14px;
    align-items: center;
    border-bottom: 1px solid
      color-mix(in srgb, var(--color-border) 50%, transparent);
  }

  .timeline-nested:last-child {
    border-bottom: none;
  }

  .timeline-icon-sm {
    width: 18px;
    height: 18px;
    border-radius: 4px;
  }

  .btn-ghost:not(.event-delete) {
    margin-top: 8px;
  }

  @media (min-width: 1280px) {
    .detail-photo {
      width: 300px;
      height: 300px;
    }
  }

  @media (max-width: 768px) {
    .detail {
      padding-bottom: 64px;
    }

    .detail-hero {
      flex-direction: column;
      gap: 16px;
    }

    .detail-photo {
      width: 100%;
      height: 220px;
    }

    .detail-info h2 {
      font-size: var(--fs-page-title);
    }

    .detail-info {
      width: 100%;
    }

    .hero-actions :global(.btn) {
      flex: 1;
      justify-content: center;
    }

    .detail-grid {
      grid-template-columns: 1fr;
    }
  }
</style>
