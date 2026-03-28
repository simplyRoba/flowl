/** Test whether a pathname is a cacheable API endpoint. */
export function isCacheableApi(pathname: string): boolean {
  if (
    pathname === "/api/plants" ||
    pathname === "/api/stats" ||
    pathname === "/api/locations"
  ) {
    return true;
  }
  // /api/plants/{id} or /api/plants/{id}/care
  return /^\/api\/plants\/\d+(?:\/care)?$/.test(pathname);
}

/** Test whether a pathname is a thumbnail URL. */
export function isThumbnail(pathname: string): boolean {
  return /^\/uploads\/.+_(?:200|600|1000)\.jpg$/.test(pathname);
}
