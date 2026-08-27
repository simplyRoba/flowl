import { beforeEach, describe, expect, it, vi } from "vitest";

const { goto } = vi.hoisted(() => ({ goto: vi.fn() }));
vi.mock("$app/navigation", () => ({ goto }));
vi.mock("./stores/network", () => ({ recheckHealth: vi.fn() }));

import {
  ApiError,
  chatPlant,
  classifyResponse,
  exportData,
  fetchBlob,
  fetchJson,
  identifyPlant,
  importData,
  summarizeChat,
  uploadPlantPhoto,
} from "./api";
import { resetAuthNavigationForTests } from "./auth";
import { recheckHealth } from "./stores/network";

function response(
  status: number,
  body: unknown,
  statusText = "Error",
): Response {
  return new Response(JSON.stringify(body), {
    status,
    statusText,
    headers: { "Content-Type": "application/json" },
  });
}

describe("authentication response classification", () => {
  beforeEach(() => {
    goto.mockReset();
    vi.mocked(recheckHealth).mockReset();
    resetAuthNavigationForTests();
    window.history.replaceState({}, "", "/plants/7?tab=care#log");
  });

  it("redirects only for the exact 401 authentication contract", async () => {
    await expect(
      classifyResponse(
        response(401, {
          code: "AUTHENTICATION_REQUIRED",
          message: "Authentication is required",
        }),
      ),
    ).rejects.toBeInstanceOf(ApiError);
    expect(goto).toHaveBeenCalledWith(
      "/login?return_to=%2Fplants%2F7%3Ftab%3Dcare%23log",
      { replaceState: true },
    );
  });

  it.each([
    response(401, { code: "OTHER", message: "Nope" }),
    response(500, { code: "AUTHENTICATION_REQUIRED", message: "Nope" }),
    new Response("not json", { status: 401, statusText: "Unauthorized" }),
  ])(
    "does not redirect for other received failures",
    async (failedResponse) => {
      await expect(classifyResponse(failedResponse)).rejects.toBeInstanceOf(
        ApiError,
      );
      expect(goto).not.toHaveBeenCalled();
    },
  );

  it("classifies direct export, chat, identify, upload, and photo requests before bodies", async () => {
    const errorResponse = () => ({
      ok: false,
      status: 401,
      statusText: "Unauthorized",
      json: vi.fn().mockResolvedValue({
        code: "AUTHENTICATION_REQUIRED",
        message: "Authentication is required",
      }),
      blob: vi.fn(),
      body: { getReader: vi.fn() },
      headers: new Headers(),
    });
    const fetchMock = vi.fn();
    globalThis.fetch = fetchMock;

    fetchMock.mockResolvedValueOnce(errorResponse());
    await expect(exportData()).rejects.toBeInstanceOf(ApiError);
    expect(fetchMock.mock.results[0]?.value).toBeDefined();

    const exportResponse = await fetchMock.mock.results[0]?.value;
    expect(exportResponse.blob).not.toHaveBeenCalled();

    fetchMock.mockResolvedValueOnce(errorResponse());
    await expect(chatPlant(1, "help", []).next()).rejects.toBeInstanceOf(
      ApiError,
    );
    const chatResponse = await fetchMock.mock.results[1]?.value;
    expect(chatResponse.body.getReader).not.toHaveBeenCalled();

    fetchMock.mockResolvedValueOnce(errorResponse());
    await expect(
      identifyPlant([new File(["x"], "plant.jpg")]),
    ).rejects.toBeInstanceOf(ApiError);

    fetchMock.mockResolvedValueOnce(errorResponse());
    await expect(
      uploadPlantPhoto(1, new File(["x"], "plant.jpg")),
    ).rejects.toBeInstanceOf(ApiError);

    const photoResponse = errorResponse();
    await expect(
      fetchBlob("/uploads/1.jpg", vi.fn().mockResolvedValue(photoResponse)),
    ).rejects.toBeInstanceOf(ApiError);
    expect(photoResponse.blob).not.toHaveBeenCalled();
  });

  it("rechecks health after protected photo transport rejection", async () => {
    await expect(
      fetchBlob(
        "/uploads/1.jpg",
        vi.fn().mockRejectedValue(new TypeError("offline")),
      ),
    ).rejects.toThrow("offline");
    expect(recheckHealth).toHaveBeenCalledTimes(1);
  });

  it("classifies summarize and import responses before consuming their payloads", async () => {
    const exactExpiry = () => ({
      ok: false,
      status: 401,
      statusText: "Unauthorized",
      json: vi.fn().mockResolvedValue({
        code: "AUTHENTICATION_REQUIRED",
        message: "Authentication is required",
      }),
    });
    const fetchMock = vi.fn();
    globalThis.fetch = fetchMock;

    fetchMock.mockResolvedValueOnce(exactExpiry());
    await expect(summarizeChat(1, [])).rejects.toBeInstanceOf(ApiError);
    expect(goto).toHaveBeenCalledTimes(1);

    resetAuthNavigationForTests();
    goto.mockReset();
    fetchMock.mockResolvedValueOnce(exactExpiry());
    await expect(
      importData(new File(["zip"], "flowl.zip", { type: "application/zip" })),
    ).rejects.toBeInstanceOf(ApiError);
    expect(goto).toHaveBeenCalledTimes(1);
  });

  it.each([
    ["export", () => exportData()],
    ["identify", () => identifyPlant([new File(["x"], "plant.jpg")])],
    ["chat", () => chatPlant(1, "help", []).next()],
    ["summarize", () => summarizeChat(1, [])],
    [
      "import",
      () =>
        importData(new File(["zip"], "flowl.zip", { type: "application/zip" })),
    ],
    ["upload", () => uploadPlantPhoto(1, new File(["x"], "plant.jpg"))],
    [
      "photo",
      () =>
        fetchBlob(
          "/uploads/1.jpg",
          vi.fn().mockRejectedValue(new TypeError("offline")),
        ),
    ],
  ])(
    "rechecks health without auth navigation after %s transport rejection",
    async (_path, request) => {
      globalThis.fetch = vi.fn().mockRejectedValue(new TypeError("offline"));

      await expect(request()).rejects.toThrow("offline");

      expect(recheckHealth).toHaveBeenCalledTimes(1);
      expect(goto).not.toHaveBeenCalled();
    },
  );

  it("only rechecks health after transport rejection from injected fetch", async () => {
    await expect(
      fetchJson(
        vi.fn().mockRejectedValue(new TypeError("offline")),
        "/api/plants/7",
      ),
    ).rejects.toThrow("offline");
    expect(recheckHealth).toHaveBeenCalledTimes(1);
    expect(goto).not.toHaveBeenCalled();
  });
});
