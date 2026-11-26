// @ts-check
import { test, expect } from "@playwright/test";
import { getBaseURL } from "../stack.js";
import {
  waitForSearchIndex,
  waitForSkuInIndex,
  typeSearchQuery,
  TORNILLO_SKU,
  TYPO_QUERY,
  expectTypoTornilloCard,
  waitForExactlyOneSkuCard,
} from "../search-ready.js";
import {
  openCreateProductSheet,
  fillMinimalCreateProduct,
  expectSkuInAdminApi,
  expectSkuInAdminTable,
  E2E_CREATE_SKU,
  E2E_CREATE_NAME,
} from "../admin-ready.js";

test.describe.configure({ timeout: 120_000 });

test.beforeAll(async () => {
  test.setTimeout(120_000);
  await waitForSearchIndex();
});

test("wait_for_sku_index_rejects_zero_timeout", async () => {
  await expect(
    waitForSkuInIndex(getBaseURL(), "UNUSED", { timeoutMs: 0 }),
  ).rejects.toThrow(/timeoutMs <= 0/);
});

test("product_create_incomplete_stays_on_sheet", async ({ page }) => {
  await openCreateProductSheet(page);
  await page.locator('[data-form="create-product"] button[type="submit"]').click();
  await expect(page.locator('[data-form="create-product"]')).toBeVisible();
  await expect(page.locator(".vx-product-sheet-title")).toContainText(
    "Nuevo producto",
  );
  await expect(page.locator(`td[data-sku="${E2E_CREATE_SKU}"]`)).toHaveCount(0);
});

test("product_create_duplicate_sku_does_not_create", async ({ page }) => {
  await openCreateProductSheet(page);
  await fillMinimalCreateProduct(page, TORNILLO_SKU, "Duplicado e2e");
  await page.locator('[data-form="create-product"] button[type="submit"]').click();
  await expect(page.locator('[data-form="create-product"]')).toBeVisible({
    timeout: 10_000,
  });
  await expect(page.getByText("El SKU ya existe")).toBeVisible({
    timeout: 10_000,
  });
  await page.goto(`${getBaseURL()}/#/`);
  await expectTypoTornilloCard(page, { timeout: 30_000 });
  const tornilloCard = page.locator(
    `[data-region="results"] vx-product-card[sku="${TORNILLO_SKU}"]`,
  );
  await waitForExactlyOneSkuCard(page, tornilloCard);
});

test("product_create_appears_in_search", async ({ page }) => {
  const base = getBaseURL();
  await openCreateProductSheet(page);
  await fillMinimalCreateProduct(page, E2E_CREATE_SKU, E2E_CREATE_NAME);
  const createResponse = page.waitForResponse(
    (res) =>
      res.url().includes("/api/v1/products") &&
      res.request().method() === "POST",
  );
  const listResponse = page.waitForResponse(
    (res) =>
      res.url().includes("/api/v1/products?") &&
      res.request().method() === "GET",
  );
  await page.locator('[data-form="create-product"] button[type="submit"]').click();
  const response = await createResponse;
  expect(response.status()).toBe(201);
  await listResponse;
  await expectSkuInAdminApi(base, E2E_CREATE_SKU);
  await page.goto(`${base}/#/admin/productos`);
  await expect(page.locator(".vx-product-title")).toContainText("Productos");
  await expectSkuInAdminTable(page, E2E_CREATE_SKU);
  await waitForSkuInIndex(base, E2E_CREATE_SKU);
  await page.goto(`${base}/#/`);
  await waitForSearchIndex();
  await typeSearchQuery(page, E2E_CREATE_SKU);
  await expect(
    page.locator(`[data-region="results"] vx-product-card[sku="${E2E_CREATE_SKU}"]`),
  ).toBeVisible({ timeout: 15_000 });
});