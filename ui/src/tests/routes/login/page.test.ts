import { cleanup, render, screen, waitFor } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import Page from "../../../routes/login/+page.svelte";
import loginSource from "../../../routes/login/+page.svelte?raw";
import { setLocale } from "$lib/stores/locale";

const { goto, fetchAuthConfig } = vi.hoisted(() => ({
  goto: vi.fn(),
  fetchAuthConfig: vi.fn(),
}));

let pageUrl = new URL("http://localhost/login");

vi.mock("$app/navigation", () => ({ goto }));
vi.mock("$app/paths", () => ({ resolve: (path: string) => path }));
vi.mock("$app/state", () => ({
  page: {
    get url() {
      return pageUrl;
    },
  },
}));
vi.mock("$lib/auth", () => ({
  fetchAuthConfig,
  safeLocalTarget: (target: string | null) =>
    target?.startsWith("/") && !target.startsWith("//") ? target : "/",
}));

beforeEach(() => {
  vi.clearAllMocks();
  pageUrl = new URL("http://localhost/login?return_to=%2Fplants%2F7");
  setLocale("en");
  fetchAuthConfig.mockResolvedValue({
    enabled: true,
    provider_name: "Garden SSO",
  });
});

afterEach(() => {
  cleanup();
  setLocale("en");
});

describe("login page", () => {
  it("renders the configured provider in its single full-width action", async () => {
    render(Page);

    await waitFor(() => {
      expect(
        screen.getByRole("link", { name: "Continue with Garden SSO" }),
      ).toBeTruthy();
    });

    const action = screen.getByRole("link", {
      name: "Continue with Garden SSO",
    });
    expect(action.getAttribute("href")).toBe(
      "/auth/login?return_to=%2Fplants%2F7",
    );
    expect(document.querySelectorAll(".provider-action")).toHaveLength(1);
    expect(document.querySelector('input[type="password"]')).toBeNull();
    expect(document.querySelector(".login-page")).toBeTruthy();
    expect(document.querySelector(".login-card")).toBeTruthy();
  });

  it.each([
    [
      "?error=authentication_failed",
      "Authentication failed. Please try again.",
    ],
    [
      "?error=provider_unavailable",
      "The sign-in provider is temporarily unavailable.",
    ],
    ["?logged_out=1", "You have been signed out."],
  ])("renders the translated generic state %s", async (query, message) => {
    pageUrl = new URL(`http://localhost/login${query}`);
    render(Page);

    expect(await screen.findByText(message)).toBeTruthy();
  });

  it("keeps a single-column mobile card and promotes it to a two-column card at 48rem", () => {
    const mobileCard = loginSource.match(
      /\.login-card \{([\s\S]*?)\n {2}\}/,
    )?.[1];
    const desktopRules = loginSource.match(
      /@media \(min-width: 48rem\) \{([\s\S]*?)\n {2}\}\n<\/style>/,
    )?.[1];

    expect(mobileCard).toContain("width: min(100%, 400px)");
    expect(mobileCard).not.toContain("display: grid");
    expect(desktopRules).toMatch(
      /\.login-card \{[\s\S]*?width: min\(100%, 880px\);[\s\S]*?min-height: 380px;[\s\S]*?display: grid;[\s\S]*?grid-template-columns: minmax\(0, 1\.15fr\) minmax\(300px, 0\.85fr\);/,
    );
    expect(desktopRules).toMatch(
      /\.brand-area \{[\s\S]*?justify-content: center;[\s\S]*?text-align: left;[\s\S]*?border-right: 1px solid var\(--color-border\);/,
    );
    expect(desktopRules).toMatch(
      /\.action-area \{[\s\S]*?display: flex;[\s\S]*?align-items: center;/,
    );
  });

  it.each([
    ["de", "Weiter mit Garden SSO"],
    ["es", "Continuar con Garden SSO"],
  ] as const)(
    "interpolates the provider in the active %s locale",
    async (locale, label) => {
      setLocale(locale);
      render(Page);

      expect(await screen.findByRole("link", { name: label })).toBeTruthy();
    },
  );

  it("returns to the application root when auth is disabled", async () => {
    fetchAuthConfig.mockResolvedValue({ enabled: false, provider_name: null });
    render(Page);

    await waitFor(() => {
      expect(goto).toHaveBeenCalledWith("/", { replaceState: true });
    });
  });

  it("uses a generic provider-unavailable state if config cannot be read", async () => {
    fetchAuthConfig.mockRejectedValue(new Error("offline"));
    render(Page);

    expect(
      await screen.findByText(
        "The sign-in provider is temporarily unavailable.",
      ),
    ).toBeTruthy();
  });
});
