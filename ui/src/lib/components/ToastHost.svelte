<script lang="ts">
  import { X } from "lucide-svelte";
  import { onDestroy } from "svelte";
  import { SvelteMap, SvelteSet } from "svelte/reactivity";
  import { translations } from "$lib/stores/locale";
  import {
    dismissNotification,
    visibleNotifications,
    type Notification,
  } from "$lib/stores/notifications";

  const AUTO_DISMISS_MS = 3500;
  const timers = new SvelteMap<string, ReturnType<typeof setTimeout>>();
  const timerStartedAt = new SvelteMap<string, number>();
  const remainingTime = new SvelteMap<string, number>();
  const pausedNotifications = new SvelteSet<string>();

  function liveRole(notification: Notification): "status" | "alert" {
    return notification.variant === "error" ? "alert" : "status";
  }

  function livePoliteness(notification: Notification): "polite" | "assertive" {
    return notification.variant === "error" ? "assertive" : "polite";
  }

  function handleAction(notification: Notification) {
    notification.action?.onClick();
    dismissNotification(notification.id);
  }

  function clearTimer(id: string) {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
  }

  function scheduleDismiss(id: string, delay = AUTO_DISMISS_MS) {
    if (typeof window === "undefined") return;

    clearTimer(id);
    timerStartedAt.set(id, Date.now());
    remainingTime.set(id, delay);

    const timer = window.setTimeout(() => {
      dismissNotification(id);
      timers.delete(id);
      timerStartedAt.delete(id);
      remainingTime.delete(id);
    }, delay);

    timers.set(id, timer);
  }

  function pauseDismiss(id: string) {
    pausedNotifications.add(id);

    const startedAt = timerStartedAt.get(id);
    const remaining = remainingTime.get(id);
    if (startedAt === undefined || remaining === undefined) return;

    const elapsed = Date.now() - startedAt;
    remainingTime.set(id, Math.max(0, remaining - elapsed));
    clearTimer(id);
  }

  function resumeDismiss(notification: Notification) {
    if (notification.variant === "error") return;
    pausedNotifications.delete(notification.id);
    scheduleDismiss(notification.id, remainingTime.get(notification.id));
  }

  $effect(() => {
    if (typeof window === "undefined") return;

    const activeIds = new Set($visibleNotifications.map((item) => item.id));

    for (const notification of $visibleNotifications) {
      if (
        notification.variant === "error" ||
        pausedNotifications.has(notification.id) ||
        timers.has(notification.id)
      ) {
        continue;
      }

      scheduleDismiss(notification.id, remainingTime.get(notification.id));
    }

    for (const [id, timer] of timers) {
      if (!activeIds.has(id)) {
        clearTimeout(timer);
        timers.delete(id);
        timerStartedAt.delete(id);
        remainingTime.delete(id);
        pausedNotifications.delete(id);
      }
    }
  });

  onDestroy(() => {
    for (const timer of timers.values()) {
      clearTimeout(timer);
    }
    timers.clear();
    timerStartedAt.clear();
    remainingTime.clear();
    pausedNotifications.clear();
  });
</script>

{#if $visibleNotifications.length > 0}
  <div class="toast-host" aria-label="Notifications">
    {#each $visibleNotifications as notification (notification.id)}
      <div
        class="toast"
        class:toast-success={notification.variant === "success"}
        class:toast-error={notification.variant === "error"}
        class:toast-info={notification.variant === "info"}
        role={liveRole(notification)}
        aria-live={livePoliteness(notification)}
        onmouseenter={() => pauseDismiss(notification.id)}
        onmouseleave={() => resumeDismiss(notification)}
      >
        <div class="toast-message">{notification.message}</div>

        <div class="toast-actions">
          {#if notification.action}
            <button
              type="button"
              class="toast-action"
              onclick={() => handleAction(notification)}
            >
              {notification.action.label}
            </button>
          {/if}

          <button
            type="button"
            class="toast-close"
            aria-label={$translations.common.close}
            onclick={() => dismissNotification(notification.id)}
          >
            <X size={14} />
          </button>
        </div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .toast-host {
    position: fixed;
    right: 16px;
    bottom: 16px;
    z-index: 160;
    display: flex;
    flex-direction: column;
    gap: 10px;
    width: min(360px, calc(100vw - 32px));
    pointer-events: none;
  }

  .toast {
    pointer-events: auto;
    display: flex;
    align-items: flex-start;
    gap: 12px;
    justify-content: space-between;
    padding: 12px 14px;
    border-radius: 14px;
    border: 1px solid var(--color-border);
    background: color-mix(in srgb, var(--color-surface) 96%, transparent);
    box-shadow: 0 18px 40px rgba(44, 36, 24, 0.14);
    backdrop-filter: blur(10px);
  }

  .toast-success {
    border-left: 4px solid var(--color-success);
  }

  .toast-info {
    border-left: 4px solid var(--color-primary);
  }

  .toast-error {
    border-left: 4px solid var(--color-danger);
  }

  .toast-message {
    flex: 1;
    font-size: 14px;
    line-height: 1.4;
    color: var(--color-text);
  }

  .toast-actions {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-shrink: 0;
  }

  .toast-action,
  .toast-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    border: none;
    background: none;
    color: var(--color-text-muted);
    cursor: pointer;
    font: inherit;
    font-size: 13px;
    padding: 2px 4px;
  }

  .toast-action {
    color: var(--color-primary);
    font-weight: 600;
  }

  .toast-close:hover,
  .toast-action:hover {
    color: var(--color-text);
  }

  @media (max-width: 768px) {
    .toast-host {
      top: max(12px, env(safe-area-inset-top, 0px));
      right: 16px;
      bottom: auto;
      left: 16px;
      width: auto;
    }
  }
</style>
