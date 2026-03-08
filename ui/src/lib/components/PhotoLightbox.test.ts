import { cleanup, render, screen, fireEvent } from "@testing-library/svelte";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import PhotoLightbox from "./PhotoLightbox.svelte";

beforeEach(() => {
  // jsdom doesn't implement showModal/close — stub them
  HTMLDialogElement.prototype.showModal = vi.fn(function (
    this: HTMLDialogElement,
  ) {
    this.setAttribute("open", "");
  });
  HTMLDialogElement.prototype.close = vi.fn(function (this: HTMLDialogElement) {
    this.removeAttribute("open");
  });
});

afterEach(() => {
  cleanup();
});

describe("PhotoLightbox", () => {
  it("calls showModal when open is true", () => {
    render(PhotoLightbox, {
      props: { open: true, src: "/photo.jpg", alt: "A plant" },
    });
    expect(HTMLDialogElement.prototype.showModal).toHaveBeenCalled();
  });

  it("does not call showModal when open is false", () => {
    render(PhotoLightbox, {
      props: { open: false, src: "/photo.jpg", alt: "A plant" },
    });
    expect(HTMLDialogElement.prototype.showModal).not.toHaveBeenCalled();
  });

  it("renders a close button", () => {
    render(PhotoLightbox, {
      props: { open: true, src: "/photo.jpg", alt: "A plant" },
    });
    expect(screen.getByRole("button", { name: "Close" })).toBeTruthy();
  });

  it("calls onclose when the close button is clicked", async () => {
    const onclose = vi.fn();
    render(PhotoLightbox, {
      props: { open: true, src: "/photo.jpg", alt: "A plant", onclose },
    });
    await fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(onclose).toHaveBeenCalled();
  });

  it("renders the image with correct src and alt", () => {
    render(PhotoLightbox, {
      props: { open: true, src: "/photo.jpg", alt: "A plant" },
    });
    const img = screen.getByAltText("A plant");
    expect(img.getAttribute("src")).toBe("/photo.jpg");
  });
});
