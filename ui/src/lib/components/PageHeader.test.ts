import { cleanup, render, screen } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";

vi.mock("$app/paths", () => ({
  resolve: (value: string) => value,
}));

import PageHeader from "./PageHeader.svelte";

afterEach(() => {
  cleanup();
});

describe("PageHeader", () => {
  it("renders a back link with the given href", () => {
    render(PageHeader, { props: { backHref: "/settings" } });
    const links = screen.getAllByRole("link");
    expect(links.some((l) => l.getAttribute("href") === "/settings")).toBe(
      true,
    );
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
