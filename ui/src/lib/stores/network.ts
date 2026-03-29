import { writable } from "svelte/store";

export const isOffline = writable(false);

const HEALTH_URL = "/health";
const POLL_INTERVAL = 60_000;
const FETCH_TIMEOUT = 5_000;

let polling = false;
let timer: ReturnType<typeof setInterval> | null = null;

async function checkHealth(): Promise<boolean> {
  try {
    const controller = new AbortController();
    const id = setTimeout(() => controller.abort(), FETCH_TIMEOUT);
    const res = await fetch(HEALTH_URL, { signal: controller.signal });
    clearTimeout(id);
    return res.ok;
  } catch {
    return false;
  }
}

async function update(): Promise<void> {
  const healthy = await checkHealth();
  isOffline.set(!healthy);
}

/** Trigger an immediate health check (e.g. after an API call fails). */
export function recheckHealth(): void {
  update();
}

export function startHealthPolling(): () => void {
  if (polling) return () => {};
  polling = true;

  // Initial check
  update();

  // React immediately to browser online/offline events
  const handleOnline = () => {
    update();
  };
  const handleOffline = () => {
    isOffline.set(true);
    // Still verify via health once the browser thinks we're back
  };

  window.addEventListener("online", handleOnline);
  window.addEventListener("offline", handleOffline);

  timer = setInterval(update, POLL_INTERVAL);

  return () => {
    polling = false;
    if (timer) {
      clearInterval(timer);
      timer = null;
    }
    window.removeEventListener("online", handleOnline);
    window.removeEventListener("offline", handleOffline);
  };
}
