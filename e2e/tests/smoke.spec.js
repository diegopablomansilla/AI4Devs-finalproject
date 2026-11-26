// @ts-check
import { test, expect } from "@playwright/test";
import { getBaseURL } from "../stack.js";
import { assertNoCriticalSerious } from "../axe.js";

test("smoke_app_loads_omnibox_focused", async ({ page }) => {
  const base = getBaseURL();
  await page.goto(`${base}/#/`);
  const omnibox = page.locator("vx-omnibox");
  await expect(omnibox).toBeVisible();
  const input = omnibox.locator("input");
  await expect(input).toBeFocused();
});

test("smoke_axe_no_critical_serious", async ({ page }) => {
  const base = getBaseURL();
  await page.goto(`${base}/#/`);
  await expect(page.locator("vx-omnibox")).toBeVisible();
  await assertNoCriticalSerious(page);
});