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