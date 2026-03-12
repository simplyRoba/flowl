import { test, expect } from "./fixtures";

test.describe("Dashboard", () => {
  test("loads and shows empty state", async ({ page, flowl }) => {
    await page.goto(flowl.baseURL);

    // The dashboard should render with the "no plants" empty state
    await expect(page.getByText("No plants yet")).toBeVisible();
    await expect(page.getByText("Add your first plant")).toBeVisible();
  });

  test("add plant button navigates to form", async ({ page, flowl }) => {
    await page.goto(flowl.baseURL);

    await page.getByRole("link", { name: /add plant/i }).click();
    await page.waitForURL("**/plants/new");
  });

  test("health endpoint responds", async ({ flowl }) => {
    const res = await fetch(`${flowl.baseURL}/health`);
    expect(res.status).toBe(200);
    const body = await res.json();
    expect(body.status).toBe("ok");
  });
});
