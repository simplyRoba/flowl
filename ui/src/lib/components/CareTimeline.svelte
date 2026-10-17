<script lang="ts">
  import { onMount, untrack } from "svelte";
  import { SvelteSet } from "svelte/reactivity";
  import {
    Droplet,
    Leaf,
    Shovel,
    Scissors,
    Pencil,
    Sparkles,
  } from "lucide-svelte";
  import type { CareEvent, EventType } from "#lib/api.js";
  import {
    CARE_TIMELINE_WINDOW_DAYS,
    createCareTimelineGround,
    layoutCareTimelinePlan,
    planCareTimelineEvents,
    normalizeCareTimelineEvents,
    selectCareTimelineEvents,
  } from "#lib/careTimeline.js";
  import { locale, translations } from "#lib/stores/locale.js";

  export interface WateringFeedback {
    plantId: number;
    eventId: number;
    request: number;
  }

  export interface Props {
    plantId: number;
    events: readonly CareEvent[];
    loading?: boolean;
    historyError?: boolean;
    wateringFeedback?: WateringFeedback | null;
    foregroundElements?: readonly HTMLElement[];
    onMinimumHeight?: (height: number) => void;
  }
  let {
    plantId,
    events,
    loading = false,
    historyError = false,
    wateringFeedback = null,
    foregroundElements = [],
    onMinimumHeight,
  }: Props = $props();
  const id = $props.id();
  let host: HTMLDivElement;
  let size = $state({ width: 320, height: 140 });
  let shields = $state<
    { x: number; y: number; width: number; height: number }[]
  >([]);
  let now = $state(Date.now());
  let motionReady = $state(false);
  let revealing = $state(false);
  let reducedMotion = false;
  let visitPlantId: number | undefined;
  let visitEntered = false;
  let entranceStartedAt = 0;
  let entranceTimer: ReturnType<typeof setTimeout> | undefined;

  const feedbackAnimations = new SvelteSet<Animation>();
  let playedWateringRequest: string | undefined;

  function finishFeedback() {
    for (const animation of feedbackAnimations) animation.cancel();
    feedbackAnimations.clear();
  }

  function showWateringFeedback(
    node: SVGElement,
    feedback: WateringFeedback | null,
  ) {
    let running: Animation[] = [];
    const cancel = () => {
      for (const animation of running) {
        if (feedbackAnimations.delete(animation)) animation.cancel();
      }
      running = [];
    };
    const update = (next: WateringFeedback | null) => {
      if (!next || next.plantId !== plantId) return;
      // Confirmed watering must not wait for the visit entrance's time stagger.
      const icon = node.querySelector<SVGElement>(".event-icon");
      const stem = node.querySelector<SVGElement>(".stem");
      node.style.animation = "none";
      if (icon) icon.style.animation = "none";
      if (stem) stem.style.animation = "none";
      node.dataset.waterFeedback = "true";
      const request = `${next.plantId}:${next.request}`;
      if (playedWateringRequest === request) return;
      playedWateringRequest = request;
      cancel();
      if (!motionReady || reducedMotion || !icon || !stem) return;
      running = [
        node.animate([{ opacity: 0.4 }, { opacity: 1 }], {
          duration: 180,
          easing: "ease-out",
        }),
        stem.animate([{ transform: "scaleY(0)" }, { transform: "scaleY(1)" }], {
          duration: 360,
          easing: "ease-out",
        }),
        icon.animate(
          [
            { opacity: 1, transform: "scale(0.55)" },
            { opacity: 1, transform: "scale(1)", offset: 0.45 },
            { opacity: 0.46, transform: "scale(1)" },
          ],
          { duration: 900, easing: "cubic-bezier(0.16, 1, 0.3, 1)" },
        ),
      ];
      for (const animation of running) {
        feedbackAnimations.add(animation);
        animation.onfinish = () => feedbackAnimations.delete(animation);
      }
    };
    untrack(() => update(feedback));
    return {
      update: (next: WateringFeedback | null) => untrack(() => update(next)),
      destroy: cancel,
    };
  }

  function finishEntrance() {
    clearTimeout(entranceTimer);
    entranceTimer = undefined;
    revealing = false;
  }

  // Recreated SVG nodes (including history returning after an empty refresh)
  // join the current entrance instead of starting their own animation clock.
  function joinEntrance(node: SVGElement) {
    untrack(() => {
      node.style.setProperty(
        "--entrance-elapsed",
        `${revealing && visitPlantId === plantId ? Math.max(0, performance.now() - entranceStartedAt) : 0}ms`,
      );
    });
  }

  $effect(() => {
    const currentPlantId = plantId;
    const hasHistory = eligible.length > 0;
    const ready = motionReady;
    untrack(() => {
      if (visitPlantId !== currentPlantId) {
        finishEntrance();
        visitPlantId = currentPlantId;
        visitEntered = false;
      }
      if (!ready || !hasHistory || visitEntered) return;
      visitEntered = true;
      if (reducedMotion) return;
      entranceStartedAt = performance.now();
      revealing = true;
      entranceTimer = setTimeout(finishEntrance, 2600);
    });
  });
  const styles = {
    watered: { icon: Droplet, color: "var(--color-water)" },
    fertilized: { icon: Leaf, color: "var(--color-primary)" },
    repotted: { icon: Shovel, color: "var(--color-secondary)" },
    pruned: { icon: Scissors, color: "var(--color-text-muted)" },
    custom: { icon: Pencil, color: "var(--color-text-muted)" },
    "ai-consultation": { icon: Sparkles, color: "var(--color-ai)" },
  } satisfies Record<EventType, { icon: typeof Droplet; color: string }>;

  // Neither the minute clock nor projection/locale changes reparse raw history.
  const sourceEvents = $derived(events);
  const sourcePlantId = $derived(plantId);
  const history = $derived(
    normalizeCareTimelineEvents(sourceEvents, sourcePlantId),
  );
  const eligible = $derived(selectCareTimelineEvents(history, now));
  // Keep complete markers inside the opaque area of the 6% edge fades.
  // Plan collisions on the actual, inset time span before sampling the ground.
  const timeInset = $derived(size.width * 0.1);
  const plan = $derived.by(() => {
    const projected = planCareTimelineEvents(
      eligible,
      now,
      size.width - timeInset * 2,
    );
    return {
      ...projected,
      placements: projected.placements.map((point) => ({
        ...point,
        x: point.x + timeInset,
      })),
    };
  });
  const minimumHeight = $derived(plan.minimumHeight);
  const layoutHeight = $derived(Math.max(size.height, minimumHeight));
  const ground = $derived(
    createCareTimelineGround(plantId, size.width, layoutHeight),
  );
  const points = $derived(layoutCareTimelinePlan(plan, ground));
  const periodFormat = $derived(
    new Intl.DateTimeFormat($locale, {
      year: "numeric",
      month: "short",
      day: "numeric",
    }),
  );
  const description = $derived(
    `${periodFormat.format(now - CARE_TIMELINE_WINDOW_DAYS * 86_400_000)} – ${periodFormat.format(now)}` +
      (historyError
        ? ` · ${$translations.error.loadCareEvents}`
        : loading
          ? ` · ${$translations.common.loading}`
          : ""),
  );

  $effect(() => {
    // Fresh care can arrive between minute ticks. Imported future records must
    // never pull the display clock forward to their occurrence timestamp.
    void history;
    untrack(() => {
      now = Date.now();
    });
  });

  let reportedHeight: number | undefined;
  let reportedPlantId: number | undefined;
  let reportedCallback: Props["onMinimumHeight"];
  $effect(() => {
    const height = minimumHeight;
    const report = onMinimumHeight;
    if (
      height !== reportedHeight ||
      plantId !== reportedPlantId ||
      report !== reportedCallback
    ) {
      reportedHeight = height;
      reportedPlantId = plantId;
      reportedCallback = report;
      untrack(() => report?.(height));
    }
  });

  $effect(() => {
    const elements = foregroundElements;
    void $locale;
    const targets = [
      ...new Set([
        host,
        ...elements.filter((element) => element.isConnected),
        ...elements.flatMap((element) =>
          element.isConnected && element.parentElement
            ? [element.parentElement]
            : [],
        ),
      ]),
    ];
    const measure = (width?: number, height?: number) =>
      untrack(() => {
        const origin = host.getBoundingClientRect();
        const measuredWidth = width ?? origin.width;
        const measuredHeight = height ?? origin.height;
        const nextSize = {
          width:
            measuredWidth > 0 && Number.isFinite(measuredWidth)
              ? measuredWidth
              : 320,
          height:
            measuredHeight > 0 && Number.isFinite(measuredHeight)
              ? measuredHeight
              : 140,
        };
        if (nextSize.width !== size.width || nextSize.height !== size.height)
          size = nextSize;
        const nextShields = elements.flatMap((element) => {
          if (!element.isConnected) return [];
          const bounds = element.getBoundingClientRect();
          return bounds.width > 0 && bounds.height > 0
            ? [
                {
                  x: bounds.left - origin.left - 12,
                  y: bounds.top - origin.top - 12,
                  width: bounds.width + 24,
                  height: bounds.height + 24,
                },
              ]
            : [];
        });
        if (
          nextShields.length !== shields.length ||
          nextShields.some((bounds, index) => {
            const previous = shields[index];
            return (
              bounds.x !== previous.x ||
              bounds.y !== previous.y ||
              bounds.width !== previous.width ||
              bounds.height !== previous.height
            );
          })
        )
          shields = nextShields;
      });
    const resize = () => measure();
    measure();
    const observer =
      typeof ResizeObserver === "undefined"
        ? undefined
        : new ResizeObserver((entries) => {
            const entry = entries.find((entry) => entry.target === host);
            // Parent/foreground notifications can signal movement, not host resizing.
            if (entry)
              measure(entry.contentRect.width, entry.contentRect.height);
            else
              untrack(() => {
                // Preserve the measured SVG dimensions while refreshing content bounds.
                measure(size.width, size.height);
              });
          });
    for (const target of targets) observer?.observe(target);
    window.addEventListener("resize", resize);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", resize);
    };
  });

  onMount(() => {
    const media = window.matchMedia?.("(prefers-reduced-motion: reduce)");
    reducedMotion = media?.matches ?? false;
    const motionChanged = (event: MediaQueryListEvent) => {
      reducedMotion = event.matches;
      if (reducedMotion) {
        finishEntrance();
        finishFeedback();
      }
    };
    media?.addEventListener("change", motionChanged);
    motionReady = true;
    let timer: ReturnType<typeof setInterval> | undefined;
    const visibilityChanged = () => {
      clearInterval(timer);
      timer = undefined;
      if (!document.hidden) {
        now = Date.now();
        timer = setInterval(() => {
          now = Date.now();
        }, 60_000);
      }
    };
    document.addEventListener("visibilitychange", visibilityChanged);
    visibilityChanged();
    return () => {
      finishEntrance();
      finishFeedback();
      media?.removeEventListener("change", motionChanged);
      clearInterval(timer);
      document.removeEventListener("visibilitychange", visibilityChanged);
    };
  });
</script>

<div
  class="care-timeline"
  class:entrance-pending={!motionReady}
  class:revealing
  style:min-height={`${minimumHeight}px`}
  bind:this={host}
>
  {#if eligible.length}
    {#key plantId}
      <svg
        width="100%"
        height="100%"
        viewBox={`0 0 ${size.width} ${layoutHeight}`}
        role="img"
        aria-labelledby={`${id}-title`}
        aria-describedby={`${id}-description`}
        focusable="false"
      >
        <title id={`${id}-title`}
          >{$translations.plant.careJournalSection}</title
        >
        <desc id={`${id}-description`}>{description}</desc>
        <defs>
          <clipPath id={`${id}-ground-reveal`} clipPathUnits="userSpaceOnUse">
            <rect
              class="ground-reveal-clip"
              width={size.width}
              height={layoutHeight}
              use:joinEntrance
            />
          </clipPath>
          <linearGradient id={`${id}-edges`} x1="0" y1="0" x2="1" y2="0">
            <stop offset="0" stop-color="white" stop-opacity="0" />
            <stop offset="0.06" stop-color="white" stop-opacity="1" />
            <stop offset="0.94" stop-color="white" stop-opacity="1" />
            <stop offset="1" stop-color="white" stop-opacity="0" />
          </linearGradient>
          <filter
            id={`${id}-blur`}
            filterUnits="userSpaceOnUse"
            x="-36"
            y="-36"
            width={size.width + 72}
            height={layoutHeight + 72}
          >
            <feGaussianBlur stdDeviation="12" />
          </filter>
          <mask
            id={`${id}-mask`}
            style="mask-type: luminance"
            maskContentUnits="userSpaceOnUse"
            maskUnits="userSpaceOnUse"
            x="0"
            y="0"
            width={size.width}
            height={layoutHeight}
          >
            <rect
              width={size.width}
              height={layoutHeight}
              fill={`url(#${id}-edges)`}
            />
            <g fill="black" filter={`url(#${id}-blur)`}>
              {#each shields as bounds, index (index)}
                <rect {...bounds} />
              {/each}
            </g>
          </mask>
        </defs>
        <g mask={`url(#${id}-mask)`} aria-hidden="true">
          <g class="ground-reveal" clip-path={`url(#${id}-ground-reveal)`}>
            <path
              class="ground-shadow"
              d={ground.path}
              transform="translate(0 3)"
            />
            <path class="baseline ground dotted" d={ground.path} />
          </g>
          {#each points as point (point.event.id)}
            {@const style = styles[point.event.eventType]}
            {@const Icon = style.icon}
            <g
              class="event"
              use:joinEntrance
              use:showWateringFeedback={wateringFeedback?.eventId ===
              point.event.id
                ? wateringFeedback
                : null}
              style:--event-delay={`${Math.max(0, Math.min(1, point.x / size.width)) * 2200}ms`}
              data-event-id={point.event.id}
              data-event-type={point.event.eventType}
              transform={`translate(${point.x} ${point.y})`}
              color={style.color}
            >
              <line
                class="stem dotted"
                x1="0"
                x2="0"
                y1="9"
                y2={point.groundY - point.y}
              />
              <ellipse
                class="contact-shadow"
                cx="0"
                cy={point.groundY - point.y + 2}
                rx="6"
                ry="1.6"
                fill="currentColor"
                opacity="0.1"
              />
              <circle
                class="contact"
                cx="0"
                cy={point.groundY - point.y}
                r="1.3"
                fill="currentColor"
                opacity="0.3"
              />
              <Icon
                class="event-icon"
                {@attach joinEntrance}
                x={-7}
                y={-7}
                size={14}
                strokeWidth={1.6}
                opacity={0.46}
              />
            </g>
          {/each}
        </g>
      </svg>
    {/key}
  {/if}
</div>

<style>
  .care-timeline {
    position: absolute;
    inset: 0;
    pointer-events: none;
    overflow: hidden;
  }
  svg {
    display: block;
  }
  .entrance-pending {
    visibility: hidden;
  }
  .revealing .ground-reveal-clip {
    transform-origin: 0 0;
    animation: ground-entrance 2200ms linear both;
    animation-delay: calc(0ms - var(--entrance-elapsed, 0ms));
  }
  .revealing .event {
    animation: event-entrance 220ms ease-out both;
    animation-delay: calc(var(--event-delay) - var(--entrance-elapsed, 0ms));
  }
  .revealing .event .stem {
    animation: stem-entrance 360ms ease-out both;
    animation-delay: calc(var(--event-delay) - var(--entrance-elapsed, 0ms));
  }
  .event :global(.event-icon) {
    transform-box: fill-box;
    transform-origin: center;
  }
  .revealing .event :global(.event-icon) {
    animation: icon-entrance 300ms ease-out both;
    animation-delay: calc(
      var(--event-delay) + 80ms - var(--entrance-elapsed, 0ms)
    );
  }
  @keyframes ground-entrance {
    from {
      transform: scaleX(0);
    }
    to {
      transform: scaleX(1);
    }
  }
  @keyframes event-entrance {
    from {
      opacity: 0;
    }
    to {
      opacity: 1;
    }
  }
  @keyframes stem-entrance {
    from {
      transform: scaleY(0);
    }
    to {
      transform: scaleY(1);
    }
  }
  @keyframes icon-entrance {
    from {
      transform: scale(0.65);
    }
    to {
      transform: scale(1);
    }
  }
  .dotted {
    fill: none;
    stroke-width: 1;
    stroke-dasharray: 1 5;
    stroke-linecap: round;
  }
  .baseline {
    stroke: var(--color-text-muted);
    opacity: 0.2;
  }
  .ground-shadow {
    fill: none;
    stroke: var(--color-text-muted);
    stroke-width: 4;
    opacity: 0.045;
    filter: blur(2px);
  }
  .stem {
    stroke: currentColor;
    opacity: 0.28;
    transform-box: fill-box;
    transform-origin: center bottom;
  }
</style>
