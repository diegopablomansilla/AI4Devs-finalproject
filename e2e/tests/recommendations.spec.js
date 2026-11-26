// @ts-check
import { test, expect } from "@playwright/test";
import { getBaseURL } from "../stack.js";
import {
  waitForSearchIndex,
  typeSearchQuery,
  openProductFromSearch,
  openSidePanel,
  panelCompactCards,
  substitutesHeading,
  omniboxInput,
  zeroProductStock,
  stockNumberFromAttr,
} from "../search-ready.js";

test.describe.configure({ timeout: 120_000 });

test.beforeAll(async () => {
  test.setTimeout(120_000);
  await waitForSearchIndex();
});

test.beforeEach(async ({ page }) => {
  await page.goto(`${getBaseURL()}/#/`);
  await expect(page.locator("vx-omnibox")).toBeVisible();
});

test("recs_open_panel_shows_three_blocks", async ({ page }) => {
  await openProductFromSearch(page, "tornilo", "TOR-AUTO-10X1");
  const panel = openSidePanel(page);
  await expect(panel).toBeVisible({ timeout: 10_000 });
  await expect(panel).toContainText("Similares");
  await expect(panel).toContainText("Sustitutos");
  await expect(panel).toContainText("Se usa junto con");
  await expect(
    panel.locator('vx-product-card[sku="TOR-4-16-AC-001"][variant="compact"]'),
  ).toBeVisible();
  await expect(page.getByRole("button", { name: "Agregar" })).toHaveCount(0);
});

test("recs_perno_m8_shows_complement", async ({ page }) => {
  await openProductFromSearch(page, "PER-M8X40-ZN", "PER-M8X40-ZN");
  const panel = openSidePanel(page);
  await expect(panel).toBeVisible({ timeout: 10_000 });
  await expect(
    panel.locator('vx-product-card[sku="ARA-PERNO-01"][variant="compact"]'),
  ).toBeVisible();
});

test("recs_perno_acero_shows_substitute", async ({ page }) => {
  await openProductFromSearch(page, "PER-ACERO-01", "PER-ACERO-01");
  const panel = openSidePanel(page);
  await expect(panel).toBeVisible({ timeout: 10_000 });
  const sub = panel.locator(
    'vx-product-card[sku="PER-ACERO-02"][variant="compact"]',
  );
  await expect(sub).toBeVisible();
  const stockAttr = await sub.getAttribute("stock");
  expect(stockNumberFromAttr(stockAttr)).toBe(12);
});

test("recs_opening_recommended_product_keeps_panel", async ({ page }) => {
  await openProductFromSearch(page, "tornilo", "TOR-AUTO-10X1");
  const panel = openSidePanel(page);
  await expect(panel).toBeVisible({ timeout: 10_000 });
  await panel
    .locator('vx-product-card[sku="TOR-4-16-AC-001"][variant="compact"]')
    .click();
  await expect(panel).toBeVisible();
  await expect(panel).toContainText("Similares");
  await expect(panel).toContainText("Sustitutos");
  await expect(panel).toContainText("Se usa junto con");
});

test("recs_close_returns_focus_to_omnibox", async ({ page }) => {
  await openProductFromSearch(page, "tornilo", "TOR-AUTO-10X1");
  await expect(openSidePanel(page)).toBeVisible({ timeout: 10_000 });
  await page.keyboard.press("Escape");
  await expect(openSidePanel(page)).toHaveCount(0, { timeout: 5000 });
  await expect(omniboxInput(page)).toBeFocused();
});

test("recs_substitutes_not_highlighted_when_in_stock", async ({ page }) => {
  await openProductFromSearch(page, "PER-ACERO-01", "PER-ACERO-01");
  await expect(openSidePanel(page)).toBeVisible({ timeout: 10_000 });
  const heading = substitutesHeading(page);
  await expect(heading).toBeVisible();
  await expect(heading).not.toHaveAttribute("data-highlight-substitutes", "");
  await expect(heading).not.toContainText("Destacados");
});

test("recs_highlights_substitutes_when_stock_zero", async ({ page }) => {
  await zeroProductStock(getBaseURL(), "PER-ACERO-01");
  await page.goto(`${getBaseURL()}/#/`);
  await openProductFromSearch(page, "PER-ACERO-01", "PER-ACERO-01");
  await expect(openSidePanel(page)).toBeVisible({ timeout: 10_000 });
  const heading = substitutesHeading(page);
  await expect(heading).toHaveAttribute("data-highlight-substitutes", "");
  await expect(heading).toContainText("Destacados");
  const sub = openSidePanel(page).locator(
    'vx-product-card[sku="PER-ACERO-02"][variant="compact"]',
  );
  await expect(sub).toBeVisible();
  const cards = panelCompactCards(page);
  const count = await cards.count();
  for (let i = 0; i < count; i++) {
    const stockAttr = await cards.nth(i).getAttribute("stock");
    const stock = stockNumberFromAttr(stockAttr);
    expect(stock).not.toBe(0);
  }
});

test("recs_product_without_relations_shows_empty_blocks", async ({ page }) => {
  await openProductFromSearch(page, "TOR-4-16-ZI-002", "TOR-4-16-ZI-002");
  const panel = openSidePanel(page);
  await expect(panel).toBeVisible({ timeout: 10_000 });
  await expect(panel).toContainText("Similares");
  await expect(panel).toContainText("Sustitutos");
  await expect(panel).toContainText("Se usa junto con");
  await expect(panelCompactCards(page)).toHaveCount(0);
});