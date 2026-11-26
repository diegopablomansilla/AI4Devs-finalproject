// @ts-check
import { test, expect } from "@playwright/test";
import { getBaseURL } from "../stack.js";
import {
  TORNILLO_SKU,
  waitForSearchIndex,
  omniboxInput,
  typeSearchQuery,
  productCards,
  searchListbox,
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

test("search_typo_tornilo_finds_tornillo_autoperforante", async ({ page }) => {
  await typeSearchQuery(page, "tornilo");
  const card = page.locator(`vx-product-card[sku="${TORNILLO_SKU}"]`);
  await expect(card).toBeVisible({ timeout: 15_000 });
  await expect(card).toContainText("Tornillo autoperforante");
  await expect(page.locator("vx-side-panel[open]")).toHaveCount(0);
});

test("search_card_shows_stock_40", async ({ page }) => {
  await typeSearchQuery(page, "tornilo");
  const card = page.locator(`vx-product-card[sku="${TORNILLO_SKU}"]`);
  await expect(card).toBeVisible({ timeout: 15_000 });
  const variant = await card.getAttribute("variant");
  expect(variant === null || variant === "full").toBe(true);
  const stockAttr = await card.getAttribute("stock");
  expect(stockNumberFromAttr(stockAttr)).toBe(40);
  await expect(
    card.locator('[data-field="stock"]'),
  ).toContainText("40");
});

test("search_arrow_down_selects_first_card", async ({ page }) => {
  await typeSearchQuery(page, "tornilo");
  const card = page.locator(
    `[data-region="results"] vx-product-card[sku="${TORNILLO_SKU}"]`,
  );
  await expect(card).toBeVisible({ timeout: 15_000 });
  const listbox = searchListbox(page);
  await expect(listbox).toBeVisible();
  const firstInList = productCards(page).first();
  await expect(firstInList).toBeVisible();
  await omniboxInput(page).press("ArrowDown");
  const firstId = await firstInList.getAttribute("id");
  expect(firstId).toBeTruthy();
  const activeId = await listbox.getAttribute("aria-activedescendant");
  expect(activeId).toBe(firstId);
  await expect(firstInList).toHaveAttribute("aria-selected", "true");
  await expect(page.locator("vx-side-panel[open]")).toHaveCount(0);
});

test("search_escape_from_listbox_restores_omnibox", async ({ page }) => {
  const card = page.locator(
    `[data-region="results"] vx-product-card[sku="${TORNILLO_SKU}"]`,
  );
  await typeSearchQuery(page, "tornilo");
  await expect(card).toBeVisible({ timeout: 20_000 });
  const listbox = searchListbox(page);
  await listbox.focus();
  await page.keyboard.press("Escape");
  await expect(omniboxInput(page)).toBeFocused();
  await expect(card).toBeVisible();
});

test("search_unknown_query_shows_no_cards", async ({ page }) => {
  await typeSearchQuery(page, "zzzzz sinproducto");
  await expect(productCards(page)).toHaveCount(0, { timeout: 15_000 });
});

test("search_sku_typo_does_not_match_tornillo", async ({ page }) => {
  const input = omniboxInput(page);
  await input.click();
  await input.fill("TOR-AUTO-10XI");
  await expect(
    page.locator(`vx-product-card[sku="${TORNILLO_SKU}"]`),
  ).toHaveCount(0, { timeout: 15_000 });
});

test("search_empty_query_clears_cards", async ({ page }) => {
  await typeSearchQuery(page, "tornilo");
  await expect(productCards(page)).not.toHaveCount(0, { timeout: 15_000 });
  await omniboxInput(page).fill("");
  await expect(productCards(page)).toHaveCount(0, { timeout: 10_000 });
});

test("search_no_facet_filter_chips", async ({ page }) => {
  await typeSearchQuery(page, "tornilo");
  await expect(productCards(page)).not.toHaveCount(0, { timeout: 15_000 });
  await expect(
    page.locator('[data-view="operator"] vx-chip'),
  ).toHaveCount(0);
});