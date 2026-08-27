import { fetchJson, type Plant } from "$lib/api";
import type { PageLoad } from "./$types";

interface PlantEditPageData {
  plant: Plant | null;
  notFound: boolean;
  loadErrorCode: string | null;
}

function emptyResult(
  overrides: Partial<PlantEditPageData> = {},
): PlantEditPageData {
  return { plant: null, notFound: false, loadErrorCode: null, ...overrides };
}

export const load: PageLoad = async ({ fetch, params }) => {
  const id = Number(params.id);
  if (!Number.isInteger(id) || id <= 0) return emptyResult({ notFound: true });
  try {
    return emptyResult({
      plant: await fetchJson<Plant>(fetch, `/api/plants/${id}`),
    });
  } catch (error) {
    if (error instanceof Error && "status" in error && error.status === 404) {
      return emptyResult({ notFound: true });
    }
    return emptyResult({
      loadErrorCode:
        error instanceof Error && "code" in error
          ? String(error.code)
          : "UNKNOWN_ERROR",
    });
  }
};
