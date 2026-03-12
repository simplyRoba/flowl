import { test, expect } from "./fixtures";

test.describe("Plant lifecycle", () => {
  test("create plant, verify on dashboard, water it, status changes", async ({
    page,
    flowl,
  }) => {
    // Navigate to create plant form
    await page.goto(flowl.baseURL);
    await page.getByRole("link", { name: /add plant/i }).click();
    await page.waitForURL("**/plants/new");

    // Fill in the plant name and save
    await page.getByLabel("Name *").fill("Test Fern");
    await page.getByRole("button", { name: "Save" }).click();

    // Should redirect to plant detail page
    await page.waitForURL("**/plants/*");
    await expect(page.locator("h2").filter({ hasText: "Test Fern" })).toBeVisible();

    // New plant should show "Due" status (never watered)
    await expect(page.locator(".status-badge").filter({ hasText: "Due" })).toBeVisible();

    // Go back to dashboard and verify plant appears
    await page.goto(flowl.baseURL);
    await expect(page.locator(".plant-card-name", { hasText: "Test Fern" })).toBeVisible();

    // Navigate to plant detail via card
    await page.locator(".plant-card", { hasText: "Test Fern" }).click();
    await expect(page.locator("h2").filter({ hasText: "Test Fern" })).toBeVisible();

    // Water the plant
    await page.locator(".btn-water").click();

    // Status should change to "Ok"
    await expect(page.locator(".status-badge").filter({ hasText: "Ok" })).toBeVisible();

    // Care journal should show the watering event
    await expect(page.locator(".timeline-label").first()).toHaveText("Watered");
  });

  test("delete plant removes it from dashboard", async ({ page, flowl }) => {
    // Create a plant first
    await page.goto(`${flowl.baseURL}/plants/new`);
    await page.getByLabel("Name *").fill("Doomed Plant");
    await page.getByRole("button", { name: "Save" }).click();
    await page.waitForURL("**/plants/*");

    // Click delete button (trash icon in header)
    await page.locator(".btn-danger").first().click();

    // Confirm deletion in dialog
    await page.getByRole("button", { name: "Delete" }).click();

    // Should redirect to dashboard
    await page.waitForURL("**/");

    // Plant should no longer appear
    await expect(page.locator(".plant-card-name", { hasText: "Doomed Plant" })).not.toBeVisible();

    // Dashboard should show empty state again
    await expect(page.getByText("No plants yet")).toBeVisible();
  });
});
