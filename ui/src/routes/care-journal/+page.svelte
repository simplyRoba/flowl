<script lang="ts">
  import { goto } from "$app/navigation";
  import { resolve } from "$app/paths";
  import { page } from "$app/state";
  import { untrack } from "svelte";
  import {
    SvelteDate,
    SvelteSet,
    SvelteURLSearchParams,
  } from "svelte/reactivity";
  import {
    Droplet,
    Leaf,
    Shovel,
    Scissors,
    Pencil,
    Sparkles,
    ChevronRight,
  } from "lucide-svelte";
  import type { CareEvent, EventType } from "$lib/api";
  import { fetchAllCareEvents } from "$lib/api";
  import { resolveError } from "$lib/stores/errors";
  import { translations } from "$lib/stores/locale";
  import { thumbUrl, thumbSrcset } from "$lib/thumbUrl";
  import {
    groupCareEvents,
    isGroup,
    type TimelineItem,
    type WateringGroup,
  } from "$lib/careGrouping";
  import PhotoLightbox from "$lib/components/PhotoLightbox.svelte";

  let lightboxOpen = $state(false);
  let lightboxSrc = $state("");

  let events = $state<CareEvent[]>([]);
  let loading = $state(false);
  let error = $state<string | null>(null);
  let expandedGroups: Set<string> = new SvelteSet();

  const TYPE_VALUES = [
    "watered",
    "fertilized",
    "repotted",
    "pruned",
    "custom",
    "ai-consultation",
  ] as const;

  let activeTypes: Set<string> = $derived(
    new SvelteSet(page.url.searchParams.getAll("type")),
  );
  let allActive: boolean = $derived(
    activeTypes.size === 0 || activeTypes.size === TYPE_VALUES.length,
  );

  function updateUrl(types: Set<string>) {
    const search = new SvelteURLSearchParams();
    for (const t of types) search.append("type", t);
    const href =
      search.size > 0 ? `/care-journal?${search.toString()}` : "/care-journal";
    goto(resolve(href as "/care-journal"), {
      replaceState: true,
      noScroll: true,
    });
  }

  function toggleFilter(value: string) {
    const next = new SvelteSet(activeTypes);
    if (next.has(value)) {
      next.delete(value);
    } else {
      next.add(value);
    }
    if (next.size === TYPE_VALUES.length) {
      updateUrl(new SvelteSet());
    } else {
      updateUrl(next);
    }
  }

  function toggleAll() {
    if (activeTypes.size === 0) {
      updateUrl(new SvelteSet(TYPE_VALUES));
    } else {
      updateUrl(new SvelteSet());
    }
  }

  async function loadAllEvents() {
    if (loading) return;
    loading = true;
    error = null;
    expandedGroups.clear();
    const types =
      activeTypes.size > 0 ? ([...activeTypes] as EventType[]) : undefined;
    try {
      const result = await fetchAllCareEvents(10000, undefined, types);
      events = result.events;
    } catch (e) {
      error = resolveError(e, "loadCareEvents");
    }
    loading = false;
  }

  function dayLabel(dateStr: string): string {
    const date = new SvelteDate(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    const fullDate = date.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
    const today = new SvelteDate();
    today.setHours(0, 0, 0, 0);
    const yesterday = new SvelteDate(today);
    yesterday.setDate(yesterday.getDate() - 1);
    const eventDate = new SvelteDate(date);
    eventDate.setHours(0, 0, 0, 0);
    if (eventDate.getTime() === today.getTime())
      return `${$translations.care.today} — ${fullDate}`;
    if (eventDate.getTime() === yesterday.getTime())
      return `${$translations.care.yesterday} — ${fullDate}`;
    return fullDate;
  }

  function eventTypeLabel(type: string): string {
    if (type === "watered") return $translations.care.watered;
    if (type === "fertilized") return $translations.care.fertilized;
    if (type === "repotted") return $translations.care.repotted;
    if (type === "pruned") return $translations.care.pruned;
    if (type === "ai-consultation") return $translations.care.aiConsultation;
    return $translations.care.custom;
  }

  function formatShortDate(dateStr: string): string {
    const date = new SvelteDate(dateStr);
    if (isNaN(date.getTime())) return dateStr;
    return date.toLocaleDateString(undefined, {
      month: "short",
      day: "numeric",
      year: "2-digit",
    });
  }

  function groupKey(group: WateringGroup): string {
    return `${group.plantId}-${group.firstAt}`;
  }

  function groupSummaryText(group: WateringGroup): string {
    return $translations.care.wateredNTimes
      .replace("{count}", String(group.count))
      .replace("{from}", formatShortDate(group.firstAt))
      .replace("{to}", formatShortDate(group.lastAt));
  }

  function toggleGroup(key: string) {
    if (expandedGroups.has(key)) {
      expandedGroups.delete(key);
    } else {
      expandedGroups.add(key);
    }
  }

  function itemDayLabel(item: TimelineItem): string {
    if (isGroup(item)) {
      return dayLabel(item.lastAt);
    }
    return dayLabel(item.occurred_at);
  }

  interface DayGroup {
    label: string;
    items: TimelineItem[];
  }

  let timelineItems: TimelineItem[] = $derived(groupCareEvents(events));

  let groupedByDay: DayGroup[] = $derived.by(() => {
    const groups: DayGroup[] = [];
    let currentLabel = "";
    for (const item of timelineItems) {
      const label = itemDayLabel(item);
      if (label !== currentLabel) {
        groups.push({ label, items: [item] });
        currentLabel = label;
      } else {
        groups[groups.length - 1].items.push(item);
      }
    }
    return groups;
  });

  $effect(() => {
    void activeTypes.size;
    untrack(() => loadAllEvents());
  });
</script>

<div class="log-page">
  <header class="page-header">
    <h1>{$translations.care.title}</h1>
  </header>

  <div class="log-filters">
    <button
      class="chip chip-solid"
      class:active={allActive}
      onclick={toggleAll}
      aria-label={$translations.care.filterAll}
    >
      <span class="filter-label">{$translations.care.filterAll}</span>
    </button>
    {#each TYPE_VALUES as value (value)}
      <button
        class="chip chip-solid"
        class:active={activeTypes.has(value)}
        onclick={() => toggleFilter(value)}
        aria-label={eventTypeLabel(value)}
      >
        {#if value === "watered"}
          <Droplet size={14} />
        {:else if value === "fertilized"}
          <Leaf size={14} />
        {:else if value === "repotted"}
          <Shovel size={14} />
        {:else if value === "pruned"}
          <Scissors size={14} />
        {:else if value === "custom"}
          <Pencil size={14} />
        {:else if value === "ai-consultation"}
          <Sparkles size={14} />
        {/if}
        <span class="filter-label icon-has-label">{eventTypeLabel(value)}</span>
      </button>
    {/each}
  </div>

  {#if error}
    <p class="error">{error}</p>
  {:else if loading}
    <div class="skeleton-list">
      {#each { length: 6 } as _, i (i)}
        <div class="skeleton-entry">
          <div class="shimmer skeleton-icon"></div>
          <div class="skeleton-lines">
            <div class="shimmer" style="width: 40%"></div>
            <div class="shimmer" style="width: 65%"></div>
          </div>
        </div>
      {/each}
    </div>
  {:else if events.length === 0}
    <div class="empty-state">
      <p>{$translations.care.noCareEvents}</p>
    </div>
  {:else}
    <div class="log-timeline">
      {#each groupedByDay as dayGroup (dayGroup.label)}
        <div class="log-day-group">
          <div class="log-day-header">{dayGroup.label}</div>
          {#each dayGroup.items as item (isGroup(item) ? groupKey(item) : item.id)}
            {#if isGroup(item)}
              {@const key = groupKey(item)}
              {@const expanded = expandedGroups.has(key)}
              <button
                class="log-entry log-group-summary"
                onclick={() => toggleGroup(key)}
              >
                <div class="log-entry-left">
                  <div class="log-entry-icon water-icon">
                    <Droplet size={14} />
                  </div>
                </div>
                <div class="log-entry-content">
                  <a
                    href={resolve(`/plants/${item.plantId}?from=/care-journal`)}
                    class="log-entry-plant"
                    onclick={(e) => e.stopPropagation()}>{item.plantName}</a
                  >
                  <div class="log-entry-action">
                    {groupSummaryText(item)}
                  </div>
                </div>
                <div class="log-group-chevron" class:expanded>
                  <ChevronRight size={16} />
                </div>
              </button>
              {#if expanded}
                <div class="log-group-expanded">
                  {#each item.events as event (event.id)}
                    <div class="log-entry log-entry-nested">
                      <div class="log-entry-left">
                        <div class="log-entry-icon water-icon nested-icon">
                          <Droplet size={12} />
                        </div>
                      </div>
                      <div class="log-entry-content">
                        <div class="log-entry-action">
                          {$translations.care.watered}
                        </div>
                        <div class="log-entry-date">
                          {formatShortDate(event.occurred_at)}
                        </div>
                      </div>
                    </div>
                  {/each}
                </div>
              {/if}
            {:else}
              <div class="log-entry">
                <div class="log-entry-left">
                  <div
                    class="log-entry-icon
										{item.event_type === 'watered' ? 'water-icon' : ''}
										{item.event_type === 'fertilized' ? 'fertilize-icon' : ''}
										{item.event_type === 'repotted' ? 'repot-icon' : ''}
										{item.event_type === 'pruned' ? 'prune-icon' : ''}
										{item.event_type === 'custom' ? 'custom-icon' : ''}
									{item.event_type === 'ai-consultation' ? 'ai-icon' : ''}"
                  >
                    {#if item.event_type === "watered"}
                      <Droplet size={14} />
                    {:else if item.event_type === "fertilized"}
                      <Leaf size={14} />
                    {:else if item.event_type === "repotted"}
                      <Shovel size={14} />
                    {:else if item.event_type === "pruned"}
                      <Scissors size={14} />
                    {:else if item.event_type === "ai-consultation"}
                      <Sparkles size={14} />
                    {:else}
                      <Pencil size={14} />
                    {/if}
                  </div>
                </div>
                <div class="log-entry-content">
                  <a
                    href={resolve(
                      `/plants/${item.plant_id}?from=/care-journal`,
                    )}
                    class="log-entry-plant">{item.plant_name}</a
                  >
                  {#if item.photo_url}
                    <button
                      class="log-entry-photo"
                      onclick={(e) => {
                        e.stopPropagation();
                        lightboxSrc = item.photo_url!;
                        lightboxOpen = true;
                      }}
                    >
                      <img
                        src={thumbUrl(item.photo_url, 200)}
                        srcset={thumbSrcset(item.photo_url)}
                        sizes="80px"
                        alt=""
                        onerror={(e) => {
                          const img = e.currentTarget as HTMLImageElement;
                          img.onerror = null;
                          img.src = item.photo_url!;
                        }}
                      />
                    </button>
                  {/if}
                  <div class="log-entry-action">
                    {eventTypeLabel(item.event_type)}
                  </div>
                  {#if item.notes}
                    <div class="log-entry-note">{item.notes}</div>
                  {/if}
                </div>
              </div>
            {/if}
          {/each}
        </div>
      {/each}
    </div>
  {/if}
</div>

<PhotoLightbox
  open={lightboxOpen}
  src={lightboxSrc}
  alt=""
  onclose={() => {
    lightboxOpen = false;
  }}
/>

<style>
  .log-page {
    max-width: var(--content-width-default);
    margin: 0 auto;
  }

  .page-header {
    margin-bottom: 16px;
  }

  .page-header h1 {
    font-size: var(--fs-page-title);
    font-weight: 700;
    margin: 0;
  }

  .log-filters {
    display: flex;
    gap: 6px;
    margin-bottom: 16px;
    flex-wrap: wrap;
  }

  .log-timeline {
    background: transparent;
    border: none;
    border-radius: 0;
    padding: 0;
  }

  .log-day-group {
    margin-bottom: 20px;
  }

  .log-day-group:last-child {
    margin-bottom: 0;
  }

  .log-day-header {
    font-size: 13px;
    font-weight: 600;
    color: var(--color-text-muted);
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 8px;
    padding-bottom: 6px;
    border-bottom: 1px solid var(--color-border);
  }

  .log-entry {
    display: flex;
    gap: 12px;
    padding: 10px 0;
    border-bottom: 1px solid var(--color-border);
    align-items: flex-start;
  }

  .log-entry:last-child {
    border-bottom: none;
  }

  .log-entry-left {
    flex-shrink: 0;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 4px;
  }

  .log-entry-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 16px;
    flex-shrink: 0;
    background: var(--color-surface-muted);
    color: var(--color-text-muted);
  }

  .log-entry-icon.water-icon {
    background: color-mix(in srgb, var(--color-water) 15%, transparent);
  }
  .log-entry-icon.fertilize-icon {
    background: color-mix(in srgb, var(--color-secondary) 15%, transparent);
  }
  .log-entry-icon.repot-icon {
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
  }
  .log-entry-icon.prune-icon {
    background: color-mix(in srgb, var(--color-text-muted) 15%, transparent);
  }
  .log-entry-icon.custom-icon {
    background: color-mix(in srgb, var(--color-warning) 15%, transparent);
  }
  .log-entry-icon.ai-icon {
    background: color-mix(in srgb, var(--color-ai) 15%, transparent);
  }

  .log-entry-content {
    flex: 1;
    min-width: 0;
  }

  .log-entry-photo {
    float: right;
    width: 80px;
    height: 80px;
    border-radius: 8px;
    overflow: hidden;
    border: none;
    padding: 0;
    background: none;
    cursor: zoom-in;
    margin-left: 10px;
    margin-bottom: 4px;
  }

  .log-entry-photo img {
    width: 100%;
    height: 100%;
    object-fit: cover;
    display: block;
  }

  .log-entry-plant {
    font-size: 14px;
    font-weight: 600;
    color: var(--color-text);
    text-decoration: none;
  }

  .log-entry-plant:hover {
    text-decoration: underline;
  }

  .log-entry-action {
    font-size: 13px;
    color: var(--color-text-muted);
  }

  .log-entry-note {
    font-size: 13px;
    color: var(--color-text);
    margin-top: 4px;
    line-height: 1.4;
  }

  .log-entry-date {
    font-size: 12px;
    color: var(--color-text-muted);
  }

  /* ---- Group summary ---- */
  .log-group-summary {
    width: 100%;
    background: none;
    border: none;
    border-bottom: 1px solid var(--color-border);
    font-family: inherit;
    cursor: pointer;
    text-align: left;
  }

  .log-group-summary:hover {
    background: var(--color-surface-muted);
  }

  .log-group-chevron {
    flex-shrink: 0;
    display: flex;
    align-items: center;
    color: var(--color-text-muted);
    transition: transform var(--transition-speed);
  }

  .log-group-chevron.expanded {
    transform: rotate(90deg);
  }

  .log-group-expanded {
    padding-left: 48px;
    border-bottom: 1px solid var(--color-border);
  }

  .log-entry-nested {
    padding: 6px 0;
    border-bottom: 1px solid
      color-mix(in srgb, var(--color-border) 50%, transparent);
  }

  .log-entry-nested:last-child {
    border-bottom: none;
  }

  .nested-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
  }

  /* ---- Skeleton loading ---- */
  .skeleton-list {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .skeleton-entry {
    display: flex;
    gap: 12px;
    padding: 10px 0;
    align-items: center;
  }

  .skeleton-icon {
    width: 36px;
    height: 36px;
    border-radius: 10px;
    flex-shrink: 0;
  }

  .skeleton-lines {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .empty-state {
    text-align: center;
    padding: 64px 24px;
    color: var(--color-text-muted);
  }

  .error {
    color: var(--color-danger);
    padding: 16px;
  }

  @media (max-width: 768px) {
    .page-header h1 {
      font-size: 18px;
    }

    .filter-label.icon-has-label {
      display: none;
    }
  }
</style>
