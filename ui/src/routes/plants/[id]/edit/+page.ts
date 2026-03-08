import type { Plant } from "$lib/api";
import type { PageLoad } from "./$types";

interface RouteLoadError {
  status: number;
  message: string;
}

interface PlantEditPageData {
  plant: Plant | null;
  notFound: boolean;
  loadError: string | null;
}

async function fetchJson<T>(fetchFn: typeof fetch, url: string): Promise<T> {
  const response = await fetchFn(url);

  if (!response.ok) {
    const data = await response
      .json()
      .catch(() => ({ message: response.statusText }));
    throw {
      status: response.status,
      message: data.message || response.statusText,
    } satisfies RouteLoadError;
  }

  return response.json() as Promise<T>;
}

function emptyResult(
  overrides: Partial<PlantEditPageData> = {},
): PlantEditPageData {
  return {
    plant: null,
    notFound: false,
    loadError: null,
    ...overrides,
  };
}

export const load: PageLoad = async ({ fetch, params }) => {
  const id = Number(params.id);
  if (!Number.isInteger(id) || id <= 0) {
    return emptyResult({ notFound: true });
  }

  try {
    const plant = await fetchJson<Plant>(fetch, `/api/plants/${id}`);
    return emptyResult({ plant });
  } catch (error) {
    if (
      typeof error === "object" &&
      error !== null &&
      "status" in error &&
      error.status === 404
    ) {
      return emptyResult({ notFound: true });
    }

    return emptyResult({
      loadError:
        error instanceof Error
          ? error.message
          : typeof error === "object" && error !== null && "message" in error
            ? String(error.message)
            : null,
    });
  }
};
