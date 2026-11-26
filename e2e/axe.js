import AxeBuilder from "@axe-core/playwright";

/**
 * @param {import('@playwright/test').Page} page
 */
export async function assertNoCriticalSerious(page) {
  const results = await new AxeBuilder({ page })
    .withTags(["wcag2a", "wcag2aa", "wcag22aa"])
    .analyze();
  const bad = results.violations.filter(
    (v) => v.impact === "critical" || v.impact === "serious",
  );
  if (bad.length > 0) {
    const summary = bad.map((v) => `${v.id} (${v.impact})`).join(", ");
    throw new Error(`axe: violaciones critical/serious: ${summary}`);
  }
}