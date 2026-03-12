export type WateringStatus = "ok" | "due" | "overdue";
export type LightNeeds = "direct" | "indirect" | "low";
export type Difficulty = "easy" | "moderate" | "demanding";
export type PetSafety = "safe" | "caution" | "toxic";
export type GrowthSpeed = "slow" | "moderate" | "fast";
export type SoilType = "standard" | "cactus-mix" | "orchid-bark" | "peat-moss";
export type SoilMoisture = "dry" | "moderate" | "moist";
export type EventType =
  | "watered"
  | "fertilized"
  | "repotted"
  | "pruned"
  | "custom"
  | "ai-consultation";

export interface Location {
  id: number;
  name: string;
  plant_count: number;
}

export interface Plant {
  id: number;
  name: string;
  species: string | null;
  icon: string;
  photo_url: string | null;
  location_id: number | null;
  location_name: string | null;
  watering_interval_days: number;
  watering_status: WateringStatus;
  last_watered: string | null;
  next_due: string | null;
  light_needs: LightNeeds;
  difficulty: Difficulty | null;
  pet_safety: PetSafety | null;
  growth_speed: GrowthSpeed | null;
  soil_type: SoilType | null;
  soil_moisture: SoilMoisture | null;
  notes: string | null;
  created_at: string;
  updated_at: string;
}

export interface CreatePlant {
  name: string;
  species?: string;
  icon?: string;
  location_id?: number | null;
  watering_interval_days?: number;
  light_needs?: LightNeeds;
  difficulty?: Difficulty | null;
  pet_safety?: PetSafety | null;
  growth_speed?: GrowthSpeed | null;
  soil_type?: SoilType | null;
  soil_moisture?: SoilMoisture | null;
  notes?: string;
}

export interface UpdatePlant {
  name?: string;
  species?: string;
  icon?: string;
  location_id?: number | null;
  watering_interval_days?: number;
  light_needs?: LightNeeds;
  difficulty?: Difficulty | null;
  pet_safety?: PetSafety | null;
  growth_speed?: GrowthSpeed | null;
  soil_type?: SoilType | null;
  soil_moisture?: SoilMoisture | null;
  notes?: string;
}

export interface AppInfo {
  version: string;
  repository: string;
  license: string;
}

export interface Stats {
  plant_count: number;
  care_event_count: number;
  location_count: number;
  photo_count: number;
}

export interface MqttStatus {
  status: "connected" | "disconnected" | "disabled";
  broker: string | null;
  topic_prefix: string | null;
}

export interface AiStatus {
  enabled: boolean;
  base_url: string | null;
  model: string | null;
}

export interface CareProfile {
  watering_interval_days: number | null;
  light_needs: LightNeeds | null;
  difficulty: Difficulty | null;
  pet_safety: PetSafety | null;
  growth_speed: GrowthSpeed | null;
  soil_type: SoilType | null;
  soil_moisture: SoilMoisture | null;
}

export interface IdentifyResult {
  common_name: string;
  scientific_name: string;
  confidence: number | null;
  summary: string | null;
  care_profile: CareProfile | null;
}

export interface IdentifyResponse {
  suggestions: IdentifyResult[];
}

export interface ChatMessage {
  role: string;
  content: string;
  image?: string;
}

export class ApiError extends Error {
  status: number;
  code: string;

  constructor(status: number, code: string, message: string) {
    super(message);
    this.status = status;
    this.code = code;
  }
}

async function request<T>(
  method: string,
  url: string,
  body?: unknown,
): Promise<T> {
  const init: RequestInit = { method };
  if (body instanceof FormData) {
    init.body = body;
  } else if (body !== undefined) {
    init.headers = { "Content-Type": "application/json" };
    init.body = JSON.stringify(body);
  }

  const resp = await fetch(url, init);

  if (!resp.ok) {
    const data = await resp.json().catch(() => ({ message: resp.statusText }));
    throw new ApiError(
      resp.status,
      data.code || "UNKNOWN_ERROR",
      data.message || resp.statusText,
    );
  }

  if (resp.status === 204) {
    return undefined as T;
  }

  return resp.json();
}

function parseFilename(contentDisposition: string | null): string {
  if (!contentDisposition) return "flowl-export.zip";

  const utf8Match = /filename\*=UTF-8''([^;]+)/i.exec(contentDisposition);
  if (utf8Match?.[1]) {
    return decodeURIComponent(utf8Match[1]);
  }

  const filenameMatch = /filename="?([^";]+)"?/i.exec(contentDisposition);
  return filenameMatch?.[1] ?? "flowl-export.zip";
}

export function fetchAppInfo(): Promise<AppInfo> {
  return request("GET", "/api/info");
}

export function fetchStats(): Promise<Stats> {
  return request("GET", "/api/stats");
}

export function fetchMqttStatus(): Promise<MqttStatus> {
  return request("GET", "/api/mqtt/status");
}

export function fetchAiStatus(): Promise<AiStatus> {
  return request("GET", "/api/ai/status");
}

export async function identifyPlant(photos: File[]): Promise<IdentifyResponse> {
  const formData = new FormData();
  for (const photo of photos) {
    formData.append("photos", photo);
  }

  const resp = await fetch("/api/ai/identify", {
    method: "POST",
    body: formData,
  });

  if (!resp.ok) {
    const data = await resp.json().catch(() => ({ message: resp.statusText }));
    throw new ApiError(
      resp.status,
      data.code || "UNKNOWN_ERROR",
      data.message || resp.statusText,
    );
  }

  return resp.json();
}

export interface MqttRepairResult {
  cleared: number;
  published: number;
}

export function repairMqtt(): Promise<MqttRepairResult> {
  return request("POST", "/api/mqtt/repair");
}

export function fetchPlants(): Promise<Plant[]> {
  return request("GET", "/api/plants");
}

export function fetchPlant(id: number): Promise<Plant> {
  return request("GET", `/api/plants/${id}`);
}

export function createPlant(data: CreatePlant): Promise<Plant> {
  return request("POST", "/api/plants", data);
}

export function updatePlant(id: number, data: UpdatePlant): Promise<Plant> {
  return request("PUT", `/api/plants/${id}`, data);
}

export function deletePlant(id: number): Promise<void> {
  return request("DELETE", `/api/plants/${id}`);
}

export function waterPlant(id: number): Promise<Plant> {
  return request("POST", `/api/plants/${id}/water`);
}

export function uploadPlantPhoto(plantId: number, file: File): Promise<Plant> {
  const body = new FormData();
  body.append("file", file);
  return request("POST", `/api/plants/${plantId}/photo`, body);
}

export function deletePlantPhoto(plantId: number): Promise<void> {
  return request("DELETE", `/api/plants/${plantId}/photo`);
}

// --- Import/Export ---

export interface ImportResult {
  locations: number;
  plants: number;
  care_events: number;
  photos: number;
}

export function importData(file: File): Promise<ImportResult> {
  const body = new FormData();
  body.append("file", file);
  return request("POST", "/api/data/import", body);
}

export async function exportData(): Promise<void> {
  const resp = await fetch("/api/data/export", { method: "GET" });

  if (!resp.ok) {
    const data = await resp.json().catch(() => ({ message: resp.statusText }));
    throw new ApiError(
      resp.status,
      data.code || "UNKNOWN_ERROR",
      data.message || resp.statusText,
    );
  }

  const blob = await resp.blob();
  const filename = parseFilename(resp.headers.get("Content-Disposition"));
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");

  link.href = url;
  link.download = filename;
  link.style.display = "none";

  document.body.append(link);
  link.click();
  link.remove();
  URL.revokeObjectURL(url);
}

// --- Settings ---

export interface UserSettings {
  theme: string;
  locale: string;
}

export function fetchSettings(): Promise<UserSettings> {
  return request("GET", "/api/settings");
}

export function updateSettings(
  data: Partial<UserSettings>,
): Promise<UserSettings> {
  return request("PUT", "/api/settings", data);
}

// --- Care Events ---

export interface CareEvent {
  id: number;
  plant_id: number;
  plant_name: string;
  event_type: EventType;
  notes: string | null;
  photo_url: string | null;
  occurred_at: string;
  created_at: string;
}

export interface CreateCareEvent {
  event_type: EventType;
  notes?: string;
  occurred_at?: string;
}

export interface CareEventsPage {
  events: CareEvent[];
  has_more: boolean;
}

export function fetchCareEvents(plantId: number): Promise<CareEvent[]> {
  return request("GET", `/api/plants/${plantId}/care`);
}

export function fetchAllCareEvents(
  limit?: number,
  before?: number,
  types?: EventType[],
): Promise<CareEventsPage> {
  const params = new URLSearchParams();
  if (limit !== undefined) params.set("limit", String(limit));
  if (before !== undefined) params.set("before", String(before));
  if (types) {
    for (const t of types) params.append("type", t);
  }
  const qs = params.toString();
  return request("GET", `/api/care${qs ? `?${qs}` : ""}`);
}

export function createCareEvent(
  plantId: number,
  data: CreateCareEvent,
): Promise<CareEvent> {
  return request("POST", `/api/plants/${plantId}/care`, data);
}

export function deleteCareEvent(
  plantId: number,
  eventId: number,
): Promise<void> {
  return request("DELETE", `/api/plants/${plantId}/care/${eventId}`);
}

export function uploadCareEventPhoto(
  plantId: number,
  eventId: number,
  file: File,
): Promise<CareEvent> {
  const body = new FormData();
  body.append("file", file);
  return request("POST", `/api/plants/${plantId}/care/${eventId}/photo`, body);
}

export function deleteCareEventPhoto(
  plantId: number,
  eventId: number,
): Promise<void> {
  return request("DELETE", `/api/plants/${plantId}/care/${eventId}/photo`);
}

// --- Locations ---

export function fetchLocations(): Promise<Location[]> {
  return request("GET", "/api/locations");
}

export function createLocation(name: string): Promise<Location> {
  return request("POST", "/api/locations", { name });
}

export function updateLocation(id: number, name: string): Promise<Location> {
  return request("PUT", `/api/locations/${id}`, { name });
}

export function deleteLocation(id: number): Promise<void> {
  return request("DELETE", `/api/locations/${id}`);
}

// --- AI Chat ---

export async function* chatPlant(
  plantId: number,
  message: string,
  history: ChatMessage[],
  signal?: AbortSignal,
  image?: string,
): AsyncGenerator<string> {
  const historyClean = history.map(({ role, content }) => ({ role, content }));
  const body: Record<string, unknown> = {
    plant_id: plantId,
    message,
    history: historyClean,
  };
  if (image) body.image = image;
  const resp = await fetch("/api/ai/chat", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
    signal,
  });

  if (!resp.ok) {
    const data = await resp.json().catch(() => ({ message: resp.statusText }));
    throw new ApiError(
      resp.status,
      data.code || "UNKNOWN_ERROR",
      data.message || resp.statusText,
    );
  }

  const reader = resp.body!.getReader();
  const decoder = new TextDecoder();
  let buf = "";

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    buf += decoder.decode(value, { stream: true });
    const lines = buf.split("\n");
    buf = lines.pop()!;
    for (const line of lines) {
      if (!line.startsWith("data: ")) continue;
      let data: Record<string, unknown>;
      try {
        data = JSON.parse(line.slice(6));
      } catch {
        continue;
      }
      if (data.done) return;
      if (data.error) {
        const err = data.error as { code?: string; message?: string };
        throw new ApiError(
          500,
          err.code || "AI_STREAM_ERROR",
          err.message || "AI response interrupted",
        );
      }
      if (data.delta) yield data.delta as string;
    }
  }
}

export async function summarizeChat(
  plantId: number,
  history: ChatMessage[],
): Promise<string> {
  const data: { summary: string } = await request("POST", "/api/ai/summarize", {
    plant_id: plantId,
    history,
  });
  return data.summary;
}
