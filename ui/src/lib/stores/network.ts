import { writable, get } from "svelte/store";

const initialOffline =
  typeof navigator !== "undefined" ? !navigator.onLine : false;
export const isOffline = writable(initialOffline);

const HEALTH_URL = "/health";
const RECOVERY_INTERVAL = 10_000;
const FETCH_TIMEOUT = 5_000;

let recoveryTimer: ReturnType<typeof setInterval> | null = null;

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

function stopRecoveryPoll(): void {
  if (recoveryTimer) {
    clearInterval(recoveryTimer);
    recoveryTimer = null;
  }
}

function startRecoveryPoll(): void {
  if (recoveryTimer) return;
  recoveryTimer = setInterval(async () => {
    const healthy = await checkHealth();
    if (healthy) {
      isOffline.set(false);
      stopRecoveryPoll();
    }
  }, RECOVERY_INTERVAL);
}

/** Trigger an immediate health check (e.g. after an API call fails). */
export function recheckHealth(): void {
  checkHealth().then((healthy) => {
    isOffline.set(!healthy);
    if (!healthy) {
      startRecoveryPoll();
    } else {
      stopRecoveryPoll();
    }
  });
}

export function startNetworkMonitor(): () => void {
  // Initial check
  if (get(isOffline)) {
    startRecoveryPoll();
  }
  checkHealth().then((healthy) => {
    isOffline.set(!healthy);
    if (!healthy) startRecoveryPoll();
  });

  // React immediately to browser online/offline events
  const handleOnline = () => {
    recheckHealth();
  };
  const handleOffline = () => {
    isOffline.set(true);
    startRecoveryPoll();
  };

  window.addEventListener("online", handleOnline);
  window.addEventListener("offline", handleOffline);

  return () => {
    stopRecoveryPoll();
    window.removeEventListener("online", handleOnline);
    window.removeEventListener("offline", handleOffline);
  };
}
