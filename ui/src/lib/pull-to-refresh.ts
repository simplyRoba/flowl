export const PULL_TO_REFRESH_THRESHOLD = 128;
export const MAX_PULL_TO_REFRESH_OFFSET = 140;
export const PULL_TO_REFRESH_RELOAD_DELAY_MS = 120;

const PLANT_DETAIL_ROUTE = /^\/plants\/[^/]+$/;

export type PullIndicatorState = "idle" | "pulling" | "release" | "refreshing";

export interface StandaloneDetectionInput {
  displayModeStandalone: boolean;
  navigatorStandalone: boolean;
}

export interface TouchDetectionInput {
  coarsePointer: boolean;
  maxTouchPoints: number;
  ontouchstart: boolean;
}

export interface PullToRefreshEligibility {
  pathname: string;
  scrollTop: number;
  standalone: boolean;
  touchCapable: boolean;
  overlayOpen: boolean;
}

interface NavigatorWithStandalone extends Navigator {
  standalone?: boolean;
}

export interface RefreshingPullGestureState {
  gestureActive: false;
  touchStartY: null;
  rawPullDistance: number;
  pullOffset: number;
  pullIndicatorState: "refreshing";
}

export function isStandalonePwaSession({
  displayModeStandalone,
  navigatorStandalone,
}: StandaloneDetectionInput): boolean {
  return displayModeStandalone || navigatorStandalone;
}

export function readStandalonePwaSession(win: Window): boolean {
  const standaloneMedia = win.matchMedia("(display-mode: standalone)");
  const navigatorWithStandalone = win.navigator as NavigatorWithStandalone;

  return isStandalonePwaSession({
    displayModeStandalone: standaloneMedia.matches,
    navigatorStandalone: navigatorWithStandalone.standalone === true,
  });
}

export function isTouchCapableDevice({
  coarsePointer,
  maxTouchPoints,
  ontouchstart,
}: TouchDetectionInput): boolean {
  return coarsePointer || maxTouchPoints > 0 || ontouchstart;
}

export function readTouchCapability(win: Window): boolean {
  const coarsePointer = win.matchMedia("(pointer: coarse)").matches;

  return isTouchCapableDevice({
    coarsePointer,
    maxTouchPoints: win.navigator.maxTouchPoints,
    ontouchstart: "ontouchstart" in win,
  });
}

export function isPullToRefreshRoute(pathname: string): boolean {
  if (
    pathname === "/" ||
    pathname === "/care-journal" ||
    pathname === "/settings"
  ) {
    return true;
  }

  if (pathname === "/plants/new") {
    return false;
  }

  return PLANT_DETAIL_ROUTE.test(pathname);
}

export function hasBlockingPullToRefreshOverlay(root: ParentNode): boolean {
  return Boolean(
    root.querySelector("dialog[open], .chat-drawer, .care-entry-form"),
  );
}

export function canStartPullToRefresh({
  pathname,
  scrollTop,
  standalone,
  touchCapable,
  overlayOpen,
}: PullToRefreshEligibility): boolean {
  return (
    standalone &&
    touchCapable &&
    isPullToRefreshRoute(pathname) &&
    scrollTop <= 0 &&
    !overlayOpen
  );
}

const CONTENT_ELASTIC_RANGE = 100;

export function calculateContentOffset(distance: number): number {
  if (distance <= 0) {
    return 0;
  }

  if (distance <= PULL_TO_REFRESH_THRESHOLD) {
    return distance;
  }

  const overThreshold = distance - PULL_TO_REFRESH_THRESHOLD;

  return (
    PULL_TO_REFRESH_THRESHOLD +
    CONTENT_ELASTIC_RANGE *
      (1 - Math.exp(-overThreshold / CONTENT_ELASTIC_RANGE))
  );
}

export function calculatePullOffset(distance: number): number {
  if (distance <= 0) {
    return 0;
  }

  if (distance <= PULL_TO_REFRESH_THRESHOLD) {
    return distance;
  }

  const elasticRange = MAX_PULL_TO_REFRESH_OFFSET - PULL_TO_REFRESH_THRESHOLD;
  const overThreshold = distance - PULL_TO_REFRESH_THRESHOLD;

  return (
    PULL_TO_REFRESH_THRESHOLD +
    elasticRange * (1 - Math.exp(-overThreshold / elasticRange))
  );
}

export function getPullIndicatorState(
  distance: number,
  threshold = PULL_TO_REFRESH_THRESHOLD,
): PullIndicatorState {
  if (distance <= 0) {
    return "idle";
  }

  if (distance >= threshold) {
    return "release";
  }

  return "pulling";
}

export function shouldTriggerPullToRefresh(
  distance: number,
  threshold = PULL_TO_REFRESH_THRESHOLD,
): boolean {
  return distance >= threshold;
}

export function getRefreshingPullGestureState(
  threshold = PULL_TO_REFRESH_THRESHOLD,
): RefreshingPullGestureState {
  return {
    gestureActive: false,
    touchStartY: null,
    rawPullDistance: threshold,
    pullOffset: calculatePullOffset(threshold),
    pullIndicatorState: "refreshing",
  };
}

export function schedulePullToRefreshReload(
  win: Pick<Window, "setTimeout">,
  onReload: () => void,
  delay = PULL_TO_REFRESH_RELOAD_DELAY_MS,
): ReturnType<typeof setTimeout> {
  return win.setTimeout(() => {
    onReload();
  }, delay);
}

export function reloadCurrentPage(win: {
  location: {
    reload: () => void;
  };
}): void {
  win.location.reload();
}

export function getPullIndicatorLabel(state: PullIndicatorState): string {
  if (state === "release") {
    return "Release to refresh";
  }

  if (state === "refreshing") {
    return "Refreshing";
  }

  return "Pull to refresh";
}
