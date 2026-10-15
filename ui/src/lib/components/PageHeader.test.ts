import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

const paths = vi.hoisted(() => ({ base: "" }));

vi.mock("$app/paths", () => ({
  // Mirrors Kit 3: route IDs start with "/", plain paths get a "/" prefix.
  resolve: (value: string) =>
    `${paths.base}${value.startsWith("/") ? value : `/${value}`}`,
}));

import PageHeader from "./PageHeader.svelte";

afterEach(() => {
  cleanup();
  paths.base = "";
});

describe("PageHeader", () => {
  it.each(["/", "/settings", "/plants/42"] as const)(
    "resolves both back links to %s",
    (backHref) => {
      render(PageHeader, { props: { backHref } });
      const links = screen.getAllByRole("link");
      expect(links).toHaveLength(2);
      for (const link of links) {
        expect(link.getAttribute("href")).toBe(backHref);
      }
    },
  );

  it("resolves both back links relative to the app base", () => {
    paths.base = "/flowl";
    render(PageHeader, { props: { backHref: "/settings" } });
    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(2);
    for (const link of links) {
      expect(link.getAttribute("href")).toBe("/flowl/settings");
    }
  });

  it("updates both resolved back links when the destination changes", async () => {
    const { rerender } = render(PageHeader, {
      props: { backHref: "/settings" },
    });

    await rerender({ backHref: "/plants/42" });

    const links = screen.getAllByRole("link");
    expect(links).toHaveLength(2);
    for (const link of links) {
      expect(link.getAttribute("href")).toBe("/plants/42");
    }
  });

  it("uses default 'Back' label when backLabel is not provided", () => {
    render(PageHeader, { props: { backHref: "/" } });
    expect(screen.getAllByText("Back").length).toBeGreaterThan(0);
  });

  it("uses custom backLabel when provided", () => {
    render(PageHeader, {
      props: { backHref: "/", backLabel: "Dashboard" },
    });
    expect(screen.getAllByText("Dashboard").length).toBeGreaterThan(0);
  });
});
