import { cleanup, render, screen, fireEvent } from "@testing-library/svelte";
import { afterEach, describe, expect, it, vi } from "vitest";
import IconPicker from "./IconPicker.svelte";

vi.mock("$lib/emoji", () => ({
  emojiToSvgPath: (emoji: string) => `/noto/${emoji}.svg`,
}));

afterEach(() => {
  cleanup();
});

describe("IconPicker", () => {
  it("renders all 9 emoji options", () => {
    const onchange = vi.fn();
    render(IconPicker, { props: { value: "🪴", onchange } });
    const buttons = screen.getAllByRole("button");
    expect(buttons).toHaveLength(9);
  });

  it("marks the matching icon as active", () => {
    const onchange = vi.fn();
    const { container } = render(IconPicker, {
      props: { value: "🌵", onchange },
    });
    const active = container.querySelector(".emoji-option.active");
    expect(active).toBeTruthy();
    expect(active?.querySelector("img")?.alt).toBe("🌵");
  });

  it("calls onchange with the clicked icon", async () => {
    const onchange = vi.fn();
    render(IconPicker, { props: { value: "🪴", onchange } });
    const cactusBtn = screen.getByRole("button", { name: "🌵" });
    await fireEvent.click(cactusBtn);
    expect(onchange).toHaveBeenCalledWith("🌵");
  });
});
