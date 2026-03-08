<script lang="ts">
  import { resolve } from "$app/paths";
  import { onMount } from "svelte";
  import { Leaf, BookOpen, Settings } from "lucide-svelte";
  import Logo from "$lib/components/Logo.svelte";
  import ToastHost from "$lib/components/ToastHost.svelte";
  import { page } from "$app/state";
  import { initTheme, isThemePreference } from "$lib/stores/theme";
  import { initLocale, isLocale, translations } from "$lib/stores/locale";
  import { fetchSettings } from "$lib/api";
  import {
    calculatePullOffset,
    canStartPullToRefresh,
    getPullIndicatorLabel,
    getPullIndicatorState,
    getRefreshingPullGestureState,
    hasBlockingPullToRefreshOverlay,
    isPullToRefreshRoute,
    PULL_TO_REFRESH_RELOAD_DELAY_MS,
    PULL_TO_REFRESH_THRESHOLD,
    reloadCurrentPage,
    readStandalonePwaSession,
    schedulePullToRefreshReload,
    readTouchCapability,
    shouldTriggerPullToRefresh,
    type PullIndicatorState,
  } from "$lib/pull-to-refresh";
  import "$lib/styles/buttons.css";
  import "$lib/styles/chips.css";
  import "$lib/styles/inputs.css";
  import "$lib/styles/sections.css";

  let { children } = $props();
  let isStandalonePwa = $state(false);
  let isTouchCapable = $state(false);
  let pullOffset = $state(0);
  let rawPullDistance = $state(0);
  let pullIndicatorState = $state<PullIndicatorState>("idle");

  let touchStartY: number | null = null;
  let gestureActive = $state(false);
  let refreshTimeout: ReturnType<typeof setTimeout> | null = null;

  const canUsePullToRefresh = $derived(
    isStandalonePwa &&
      isTouchCapable &&
      isPullToRefreshRoute(page.url.pathname),
  );
  const pullIndicatorVisible = $derived(pullIndicatorState !== "idle");
  const pullIndicatorLabel = $derived(
    getPullIndicatorLabel(pullIndicatorState),
  );
  const spinnerRotation = $derived(
    pullIndicatorState === "pulling"
      ? Math.min(rawPullDistance / PULL_TO_REFRESH_THRESHOLD, 1) * 360
      : 0,
  );

  function isActive(href: string): boolean {
    if (href === "/")
      return (
        page.url.pathname === "/" || page.url.pathname.startsWith("/plants")
      );
    return page.url.pathname.startsWith(href);
  }

  onMount(async () => {
    try {
      const settings = await fetchSettings();
      const theme = isThemePreference(settings.theme)
        ? settings.theme
        : undefined;
      const locale = isLocale(settings.locale) ? settings.locale : undefined;
      initTheme(theme);
      initLocale(locale);
    } catch {
      initTheme();
      initLocale();
    }
  });

  function addMediaListener(
    query: MediaQueryList,
    listener: (event: MediaQueryListEvent) => void,
  ) {
    if (typeof query.addEventListener === "function") {
      query.addEventListener("change", listener);
      return;
    }

    const legacyQuery = query as MediaQueryList & {
      addListener?: (event: (event: MediaQueryListEvent) => void) => void;
    };

    legacyQuery.addListener?.(listener);
  }

  function removeMediaListener(
    query: MediaQueryList,
    listener: (event: MediaQueryListEvent) => void,
  ) {
    if (typeof query.removeEventListener === "function") {
      query.removeEventListener("change", listener);
      return;
    }

    const legacyQuery = query as MediaQueryList & {
      removeListener?: (event: (event: MediaQueryListEvent) => void) => void;
    };

    legacyQuery.removeListener?.(listener);
  }

  function clearRefreshTimeout() {
    if (refreshTimeout === null) {
      return;
    }

    clearTimeout(refreshTimeout);
    refreshTimeout = null;
  }

  function resetPullGesture() {
    touchStartY = null;
    gestureActive = false;
    rawPullDistance = 0;
    pullOffset = 0;
    pullIndicatorState = "idle";
  }

  function getScrollTop(): number {
    return Math.max(
      window.scrollY,
      document.documentElement.scrollTop,
      document.body.scrollTop,
    );
  }

  function hasBlockingOverlay(): boolean {
    return hasBlockingPullToRefreshOverlay(document);
  }

  function syncPullToRefreshCapabilities() {
    isStandalonePwa = readStandalonePwaSession(window);
    isTouchCapable = readTouchCapability(window);
  }

  function getEligibility() {
    return canStartPullToRefresh({
      pathname: page.url.pathname,
      scrollTop: getScrollTop(),
      standalone: isStandalonePwa,
      touchCapable: isTouchCapable,
      overlayOpen: hasBlockingOverlay(),
    });
  }

  function handleTouchStart(event: TouchEvent) {
    if (event.touches.length !== 1 || !getEligibility()) {
      resetPullGesture();
      return;
    }

    clearRefreshTimeout();
    touchStartY = event.touches[0].clientY;
    gestureActive = true;
    rawPullDistance = 0;
    pullOffset = 0;
    pullIndicatorState = "idle";
  }

  function handleTouchMove(event: TouchEvent) {
    if (!gestureActive || touchStartY === null) {
      return;
    }

    if (event.touches.length !== 1 || hasBlockingOverlay()) {
      resetPullGesture();
      return;
    }

    const distance = event.touches[0].clientY - touchStartY;

    if (distance <= 0) {
      rawPullDistance = 0;
      pullOffset = 0;
      pullIndicatorState = "idle";
      return;
    }

    rawPullDistance = distance;
    pullOffset = calculatePullOffset(distance);
    pullIndicatorState = getPullIndicatorState(distance);
    event.preventDefault();
  }

  function handleTouchEnd() {
    if (!gestureActive) {
      return;
    }

    if (shouldTriggerPullToRefresh(rawPullDistance)) {
      const refreshingGesture = getRefreshingPullGestureState();

      gestureActive = refreshingGesture.gestureActive;
      touchStartY = refreshingGesture.touchStartY;
      rawPullDistance = refreshingGesture.rawPullDistance;
      pullOffset = refreshingGesture.pullOffset;
      pullIndicatorState = refreshingGesture.pullIndicatorState;
      refreshTimeout = schedulePullToRefreshReload(
        window,
        () => {
          reloadCurrentPage(window);
        },
        PULL_TO_REFRESH_RELOAD_DELAY_MS,
      );
      return;
    }

    resetPullGesture();
  }

  function handleTouchCancel() {
    resetPullGesture();
  }

  onMount(() => {
    syncPullToRefreshCapabilities();

    const standaloneQuery = window.matchMedia("(display-mode: standalone)");
    const coarsePointerQuery = window.matchMedia("(pointer: coarse)");
    const handleCapabilityChange = () => {
      syncPullToRefreshCapabilities();
    };

    addMediaListener(standaloneQuery, handleCapabilityChange);
    addMediaListener(coarsePointerQuery, handleCapabilityChange);

    window.addEventListener("touchstart", handleTouchStart, { passive: true });
    window.addEventListener("touchmove", handleTouchMove, { passive: false });
    window.addEventListener("touchend", handleTouchEnd);
    window.addEventListener("touchcancel", handleTouchCancel);

    return () => {
      clearRefreshTimeout();
      removeMediaListener(standaloneQuery, handleCapabilityChange);
      removeMediaListener(coarsePointerQuery, handleCapabilityChange);
      window.removeEventListener("touchstart", handleTouchStart);
      window.removeEventListener("touchmove", handleTouchMove);
      window.removeEventListener("touchend", handleTouchEnd);
      window.removeEventListener("touchcancel", handleTouchCancel);
    };
  });

  $effect(() => {
    const _pathname = page.url.pathname;

    if (pullIndicatorState !== "refreshing") {
      resetPullGesture();
    }
  });

  $effect(() => {
    if (!canUsePullToRefresh && pullIndicatorState !== "refreshing") {
      resetPullGesture();
    }
  });
</script>

<svelte:head>
  <title>flowl</title>
</svelte:head>

<div class="app-shell">
  <div
    class="pull-indicator"
    class:visible={pullIndicatorVisible}
    class:armed={pullIndicatorState === "release"}
    class:refreshing={pullIndicatorState === "refreshing"}
    aria-live="polite"
    aria-hidden={!pullIndicatorVisible}
    data-testid="pull-to-refresh-indicator"
    style:transform={pullIndicatorVisible
      ? `translate(-50%, ${Math.max(12, pullOffset)}px)`
      : undefined}
  >
    <span
      class="pull-indicator-spinner"
      aria-hidden="true"
      style:transform={pullIndicatorState === "pulling"
        ? `rotate(${spinnerRotation}deg)`
        : undefined}
    ></span>
    <span>{pullIndicatorLabel}</span>
  </div>

  <div class="app">
    <nav class="sidebar">
      <div class="logo">
        <Logo size={28} /><span class="nav-label brand">flowl</span>
      </div>
      <a href={resolve("/")} class="nav-item" class:active={isActive("/")}
        ><Leaf size={20} /><span class="nav-label"
          >{$translations.nav.plants}</span
        ></a
      >
      <a
        href={resolve("/care-journal")}
        class="nav-item"
        class:active={isActive("/care-journal")}
        ><BookOpen size={20} /><span class="nav-label"
          >{$translations.nav.careJournal}</span
        ></a
      >
      <a
        href={resolve("/settings")}
        class="nav-item bottom"
        class:active={isActive("/settings")}
        ><Settings size={20} /><span class="nav-label"
          >{$translations.nav.settings}</span
        ></a
      >
    </nav>
    <main
      class="content"
      class:settling={!gestureActive}
      style:transform={pullOffset > 0
        ? `translateY(${pullOffset}px)`
        : undefined}
    >
      {@render children()}
    </main>
    <ToastHost />
  </div>
</div>

<style>
  :global(:root) {
    color-scheme: light;
    --color-background: #faf6f1;
    --color-surface: #ffffff;
    --color-surface-muted: color-mix(
      in srgb,
      var(--color-surface) 86%,
      var(--color-background)
    );
    --color-border: #e5ddd3;
    --color-border-subtle: color-mix(
      in srgb,
      var(--color-border) 70%,
      var(--color-background)
    );
    --color-text: #2c2418;
    --color-text-muted: #8c7e6e;
    --color-primary: #6b8f71;
    --color-primary-tint: color-mix(
      in srgb,
      var(--color-primary) 10%,
      transparent
    );
    --color-primary-dark: #4a6b4f;
    --color-secondary: #c4775b;
    --color-water: #5b9bc4;
    --color-water-strong: #4c89b1;
    --color-success: #7ab87a;
    --color-warning: #d4a843;
    --color-danger: #c45b5b;
    --color-text-on-primary: #ffffff;
    --color-text-on-water: #ffffff;
    --color-text-on-ai: #ffffff;
    --color-text-on-danger: #ffffff;
    --color-text-on-image: #ffffff;
    --color-success-soft: color-mix(
      in srgb,
      var(--color-success) 20%,
      transparent
    );
    --color-warning-soft: color-mix(
      in srgb,
      var(--color-warning) 18%,
      transparent
    );
    --color-danger-soft: color-mix(
      in srgb,
      var(--color-danger) 16%,
      transparent
    );
    --color-ai: #9b7ed8;
    --color-ai-tint: color-mix(in srgb, var(--color-ai) 10%, transparent);
    --color-ai-soft: color-mix(in srgb, var(--color-ai) 15%, transparent);

    /* Typography */
    --fs-page-title: 22px;
    --fs-section-label: 13px;
    --fs-btn: 14px;
    --fs-input: 16px;
    --fs-chip: 13px;

    /* Radii */
    --radius-card: 12px;
    --radius-btn: 8px;
    --radius-pill: 999px;

    /* Motion */
    --transition-speed: 0.15s;

    /* Layout */
    --nav-bottom-height: 56px;
    --safe-area-bottom: env(safe-area-inset-bottom, 0px);
    --nav-bottom-total: calc(
      var(--nav-bottom-height) + var(--safe-area-bottom)
    );

    /* Content widths */
    --content-width-narrow: 640px;
    --content-width-default: 800px;
    --content-width-wide: 1200px;
  }

  :global([data-theme="dark"]) {
    color-scheme: dark;
    --color-background: #1a1612;
    --color-surface: #252019;
    --color-surface-muted: color-mix(
      in srgb,
      var(--color-surface) 90%,
      var(--color-background)
    );
    --color-border: #3a3228;
    --color-border-subtle: color-mix(
      in srgb,
      var(--color-border) 70%,
      var(--color-background)
    );
    --color-text: #ede6db;
    --color-text-muted: #9c8e7e;
    --color-primary: #8bb592;
    --color-primary-tint: color-mix(
      in srgb,
      var(--color-primary) 10%,
      transparent
    );
    --color-primary-dark: #a3cda9;
    --color-secondary: #d49478;
    --color-water: #78b4d8;
    --color-water-strong: color-mix(in srgb, var(--color-water) 85%, #000);
    --color-success: #8bc48b;
    --color-warning: #d4b054;
    --color-danger: #d47878;
    --color-text-on-primary: #1a1612;
    --color-text-on-water: #1a1612;
    --color-text-on-ai: #1a1612;
    --color-text-on-danger: #1a1612;
    --color-text-on-image: #ffffff;
    --color-ai: #b89ee8;
    --color-ai-tint: color-mix(in srgb, var(--color-ai) 12%, transparent);
    --color-ai-soft: color-mix(in srgb, var(--color-ai) 18%, transparent);
  }

  :global(html, body) {
    margin: 0;
    min-width: 320px;
    font-family:
      -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu,
      Cantarell, sans-serif;
    background: var(--color-background);
    color: var(--color-text);
  }

  .app-shell {
    position: relative;
  }

  .pull-indicator {
    position: fixed;
    top: 0;
    left: 50%;
    z-index: 160;
    display: flex;
    align-items: center;
    gap: 10px;
    min-width: 148px;
    padding: 10px 14px;
    border: 1px solid var(--color-border-subtle);
    border-radius: 999px;
    background: color-mix(in srgb, var(--color-surface) 94%, transparent);
    box-shadow: 0 12px 24px rgba(0, 0, 0, 0.08);
    color: var(--color-text);
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.01em;
    opacity: 0;
    pointer-events: none;
    backdrop-filter: blur(10px);
    transition:
      transform 0.18s ease,
      opacity 0.18s ease;
  }

  .pull-indicator.visible {
    opacity: 1;
  }

  .pull-indicator-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-primary);
    border-radius: 999px;
    transition: transform 0.18s ease;
  }

  .pull-indicator.armed .pull-indicator-spinner {
    border-top-color: var(--color-secondary);
    border-right-color: var(--color-secondary);
    transform: scale(1.3);
  }

  .pull-indicator.refreshing .pull-indicator-spinner {
    animation: pull-refresh-spin 0.8s linear infinite;
  }

  .app {
    display: block;
  }

  .sidebar {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    z-index: 100;
    width: 64px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 16px 0;
    gap: 8px;
    color: var(--color-text);
  }

  .logo {
    margin-bottom: 16px;
    color: var(--color-primary);
  }

  .nav-item {
    width: 44px;
    height: 44px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 10px;
    text-decoration: none;
    color: var(--color-text-muted);
    transition:
      background 0.15s,
      color 0.15s;
  }

  .nav-item:hover {
    background: var(--color-surface-muted);
    color: var(--color-text);
  }

  .nav-item.active {
    background: var(--color-primary);
    color: var(--color-text-on-primary);
  }

  .nav-item.bottom {
    margin-top: auto;
  }

  .nav-label {
    display: none;
  }

  .content {
    margin-left: 64px;
    padding: 24px;
  }

  .content.settling {
    transition: transform 0.18s ease;
  }

  @keyframes pull-refresh-spin {
    from {
      transform: rotate(0deg);
    }

    to {
      transform: rotate(360deg);
    }
  }

  @media (min-width: 1280px) {
    :global(:root) {
      --content-width-narrow: 720px;
      --content-width-default: 960px;
      --content-width-wide: 1400px;
    }

    .sidebar {
      width: 200px;
      align-items: stretch;
      padding: 16px 12px;
    }

    .logo {
      display: flex;
      align-items: center;
      gap: 10px;
      padding: 0 8px;
    }

    .nav-label {
      display: inline;
      font-size: 14px;
      font-weight: 500;
    }

    .nav-label.brand {
      font-size: 18px;
      font-weight: 700;
      color: var(--color-primary);
    }

    .nav-item {
      width: auto;
      justify-content: flex-start;
      gap: 10px;
      padding: 0 12px;
    }

    .content {
      margin-left: 200px;
      padding: 32px;
    }
  }

  @media (max-width: 768px) {
    .sidebar {
      top: auto;
      left: 0;
      right: 0;
      bottom: 0;
      width: 100%;
      height: var(--nav-bottom-height);
      flex-direction: row;
      justify-content: space-around;
      border-right: none;
      border-top: 1px solid var(--color-border);
      padding: 0;
      padding-bottom: var(--safe-area-bottom);
      gap: 0;
    }

    .logo {
      display: none;
    }

    .nav-item {
      flex: 1;
      height: auto;
      flex-direction: column;
      gap: 2px;
      border-radius: 0;
      background: none;
      color: var(--color-text-muted);
    }

    .nav-item:hover {
      background: none;
    }

    .nav-item.active {
      background: none;
      color: var(--color-primary);
    }

    .nav-label {
      display: inline;
      font-size: 11px;
      font-weight: 400;
    }

    .nav-item.bottom {
      margin-top: 0;
      margin-left: 0;
    }

    .content {
      margin-left: 0;
      padding: 16px;
      padding-bottom: calc(var(--nav-bottom-total) + 16px);
    }
  }
</style>
