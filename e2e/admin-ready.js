// @ts-check
import { getBaseURL } from "./stack.js";

/**
 * @param {import("@playwright/test").Page} page
 * @param {string} name
 * @param {string} value
 */
export async function fillVxField(page, name, value) {
  const field = page.locator(`vx-field[name="${name}"]`);
  const input = field.locator("input");
  await input.click();
  await input.fill(value);
  await input.dispatchEvent("change");
  await field.evaluate((el, v) => el.setAttribute("value", v), value);
}

/**
 * @param {import("@playwright/test").Page} page
 */
export async function openCreateProductSheet(page) {
  await page.goto(`${getBaseURL()}/#/admin/productos`);
  await page.locator('[data-view="admin"]').waitFor({ state: "visible" });
  await page.locator('button[data-action="create"]').click();
  await page.locator('[data-form="create-product"]').waitFor({
    state: "visible",
  });
  await page
    .locator('select[name="categoryId"] option')
    .nth(1)
    .waitFor({ state: "attached" });
}

/**
 * @param {import("@playwright/test").Page} page
 */
export async function fillMinimalCreateProduct(page, sku, name) {
  await fillVxField(page, "sku", sku);
  await fillVxField(page, "name", name);
  await fillVxField(page, "productType", "Prueba");
  await fillVxField(page, "price", "99.00");
  const category = page.locator('select[name="categoryId"]');
  await category.selectOption({ index: 1 });
}

export const E2E_CREATE_SKU = "E2E-ALTA-001";
export const E2E_CREATE_NAME = "Producto e2e alta visible";

/**
 * @param {import("@playwright/test").Page} page
 * @param {string} sku
 */
/**
 * @param {string} baseURL
 * @param {string} sku
 */
export async function expectSkuInAdminApi(baseURL, sku) {
  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    let cursor = null;
    for (let page = 0; page < 50; page++) {
      const qs = new URLSearchParams({ limit: "100", active: "true" });
      if (cursor) {
        qs.set("cursor", cursor);
      }
      const res = await fetch(`${baseURL}/api/v1/products?${qs}`, {
        signal: AbortSignal.timeout(10_000),
      });
      if (!res.ok) {
        break;
      }
      const data = await res.json();
      const items = Array.isArray(data?.items) ? data.items : [];
      if (items.some((item) => item?.sku === sku)) {
        return;
      }
      cursor = data?.nextCursor ?? null;
      if (cursor == null) {
        break;
      }
    }
    await new Promise((r) => setTimeout(r, 400));
  }
  throw new Error(`e2e: SKU ${sku} no aparece en GET /api/v1/products`);
}

/**
 * @param {import("@playwright/test").Page} page
 * @param {string} sku
 */
export async function expectSkuInAdminTable(page, sku) {
  const table = page.locator(".vx-product-table");
  await table.waitFor({ state: "visible", timeout: 10_000 });
  for (let i = 0; i < 30; i++) {
    const text = await table.innerText();
    if (text.includes(sku)) {
      return;
    }
    const more = page.locator('[data-action="load-more"]');
    if ((await more.count()) === 0) {
      break;
    }
    await more.click();
    await page.waitForTimeout(400);
  }
  const finalText = await table.innerText();
  if (!finalText.includes(sku)) {
    throw new Error(`e2e: SKU ${sku} no visible en tabla admin`);
  }
}