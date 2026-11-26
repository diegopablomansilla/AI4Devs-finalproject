import { expect } from "@playwright/test";
import { getBaseURL } from "./stack.js";

export const TORNILLO_SKU = "TOR-AUTO-10X1";
export const TYPO_QUERY = "tornilo";

/**
 * Poll until Meilisearch returns the seed tornillo for the typo query.
 *
 * @param {string} [baseURL]
 * @param {{ timeoutMs?: number, intervalMs?: number }} [options]
 */
export async function waitForSearchIndex(baseURL, options = {}) {
  const url = baseURL ?? getBaseURL();
  const timeoutMs = options.timeoutMs ?? 90_000;
  const intervalMs = options.intervalMs ?? 500;

  if (timeoutMs <= 0) {
    throw new Error("e2e: search index wait disabled (timeoutMs <= 0)");
  }

  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    try {
      const res = await fetch(
        `${url}/api/v1/search?q=${encodeURIComponent(TYPO_QUERY)}&limit=50`,
        { signal: AbortSignal.timeout(5000) },
      );
      if (res.ok) {
        const data = await res.json();
        const hits = Array.isArray(data?.hits) ? data.hits : [];
        if (hits.some((h) => h?.sku === TORNILLO_SKU)) {
          return;
        }
      }
    } catch {
      /* retry */
    }
    await new Promise((r) => setTimeout(r, intervalMs));
  }

  throw new Error(
    `e2e: índice sin ${TORNILLO_SKU} para q=${TYPO_QUERY} tras ${timeoutMs}ms`,
  );
}

/**
 * Poll until Meilisearch returns a hit for the given SKU.
 *
 * @param {string} [baseURL]
 * @param {string} sku
 * @param {{ timeoutMs?: number, intervalMs?: number, query?: string }} [options]
 */
export async function waitForSkuInIndex(baseURL, sku, options = {}) {
  const url = baseURL ?? getBaseURL();
  const timeoutMs = options.timeoutMs ?? 90_000;
  const intervalMs = options.intervalMs ?? 500;
  const query = options.query ?? sku;

  if (timeoutMs <= 0) {
    throw new Error("e2e: sku index wait disabled (timeoutMs <= 0)");
  }

  const deadline = Date.now() + timeoutMs;

  while (Date.now() < deadline) {
    try {
      const res = await fetch(
        `${url}/api/v1/search?q=${encodeURIComponent(query)}&limit=50`,
        { signal: AbortSignal.timeout(5000) },
      );
      if (res.ok) {
        const data = await res.json();
        const hits = Array.isArray(data?.hits) ? data.hits : [];
        if (hits.some((h) => h?.sku === sku)) {
          return;
        }
      }
    } catch {
      /* retry */
    }
    await new Promise((r) => setTimeout(r, intervalMs));
  }

  throw new Error(
    `e2e: índice sin SKU ${sku} para q=${query} tras ${timeoutMs}ms`,
  );
}

/**
 * @param {string} baseURL
 * @param {string} sku
 */
export async function findProductIdBySku(baseURL, sku) {
  const res = await fetch(
    `${baseURL}/api/v1/search?q=${encodeURIComponent(sku)}&limit=20`,
    { signal: AbortSignal.timeout(10_000) },
  );
  if (!res.ok) {
    throw new Error(`e2e: search for ${sku} returned ${res.status}`);
  }
  const data = await res.json();
  const hits = Array.isArray(data?.hits) ? data.hits : [];
  const hit = hits.find((h) => h?.sku === sku);
  if (!hit?.id) {
    throw new Error(`e2e: no product id for sku ${sku}`);
  }
  return String(hit.id);
}

/**
 * @param {string} baseURL
 * @param {string} sku
 */
export async function zeroProductStock(baseURL, sku) {
  const productId = await findProductIdBySku(baseURL, sku);
  const getRes = await fetch(`${baseURL}/api/v1/products/${productId}`, {
    signal: AbortSignal.timeout(10_000),
  });
  if (!getRes.ok) {
    throw new Error(`e2e: GET product ${productId} returned ${getRes.status}`);
  }
  const product = await getRes.json();
  const stockNum = stockNumberFromAttr(String(product?.stock ?? ""));
  if (!Number.isFinite(stockNum) || stockNum <= 0) {
    return;
  }
  const quantity = `-${stockNum}`;
  const postRes = await fetch(
    `${baseURL}/api/v1/products/${productId}/stock-movements`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        "X-Vectra-Terminal": "e2e",
      },
      body: JSON.stringify({
        type: "adjustment",
        quantity,
        reason: "e2e T-09-11",
      }),
      signal: AbortSignal.timeout(15_000),
    },
  );
  if (!postRes.ok) {
    const body = await postRes.text();
    throw new Error(
      `e2e: stock adjustment for ${sku} failed ${postRes.status}: ${body}`,
    );
  }

  const deadline = Date.now() + 30_000;
  while (Date.now() < deadline) {
    const check = await fetch(`${baseURL}/api/v1/products/${productId}`, {
      signal: AbortSignal.timeout(5000),
    });
    if (check.ok) {
      const refreshed = await check.json();
      const after = stockNumberFromAttr(String(refreshed?.stock ?? ""));
      if (after === 0) {
        return;
      }
    }
    await new Promise((r) => setTimeout(r, 400));
  }
  throw new Error(`e2e: stock of ${sku} did not reach 0 in time`);
}

/**
 * @param {import("@playwright/test").Page} page
 * @param {string} query
 * @param {string} sku
 */
export async function openProductFromSearch(page, query, sku) {
  await typeSearchQuery(page, query);
  const card = page.locator(`[data-region="results"] vx-product-card[sku="${sku}"]`);
  await expectCardVisible(card);
  await card.click();
}

/**
 * @param {import("@playwright/test").Locator} card
 */
async function expectCardVisible(card) {
  await card.waitFor({ state: "visible", timeout: 15_000 });
}

/**
 * @param {import("@playwright/test").Page} page
 */
export function openSidePanel(page) {
  return page.locator('vx-side-panel[open]');
}

/**
 * @param {import("@playwright/test").Page} page
 */
export function panelCompactCards(page) {
  return page.locator('vx-side-panel[open] vx-product-card[variant="compact"]');
}

/**
 * @param {import("@playwright/test").Page} page
 */
export function substitutesHeading(page) {
  return page.locator('vx-side-panel[open] .vx-recs-heading', {
    hasText: "Sustitutos",
  });
}

/**
 * @param {import("@playwright/test").Page} page
 */
export function omniboxInput(page) {
  return page.locator("vx-omnibox").locator("input");
}

/**
 * @param {import("@playwright/test").Page} page
 * @param {string} query
 */
export async function typeSearchQuery(page, query) {
  const input = omniboxInput(page);
  await input.click();
  await input.fill("");
  await input.pressSequentially(query, { delay: 15 });
}

/**
 * Operator UI: typo query must surface the seed autoperforante card.
 *
 * @param {import("@playwright/test").Page} page
 * @param {{ timeout?: number }} [options]
 */
export async function expectTypoTornilloCard(page, options = {}) {
  const timeout = options.timeout ?? 20_000;
  await waitForSearchIndex(getBaseURL(), { timeoutMs: 90_000 });
  await page.locator("vx-omnibox").waitFor({ state: "visible" });
  await typeSearchQuery(page, TYPO_QUERY);
  const card = page.locator(
    `[data-region="results"] vx-product-card[sku="${TORNILLO_SKU}"]`,
  );
  await expect(card).toBeVisible({ timeout });
  return card;
}

/**
 * @param {import("@playwright/test").Page} page
 * @param {import("@playwright/test").Locator} skuCard
 */
export async function waitForExactlyOneSkuCard(page, skuCard) {
  await expect
    .poll(async () => skuCard.count(), { timeout: 30_000 })
    .toBe(1);
  await expect(skuCard).toBeVisible();
}

/**
 * @param {import("@playwright/test").Page} page
 */
export function productCards(page) {
  return page.locator('[data-region="results"] vx-product-card');
}

/**
 * @param {import("@playwright/test").Page} page
 */
export function searchListbox(page) {
  return page.locator('[data-region="results"] [role="listbox"]');
}

/**
 * @param {string | null} stockAttr
 */
export function stockNumberFromAttr(stockAttr) {
  if (stockAttr == null || stockAttr === "") {
    return Number.NaN;
  }
  const normalized = String(stockAttr).trim().replace(",", ".");
  return Number.parseFloat(normalized);
}