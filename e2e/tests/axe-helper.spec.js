// @ts-check
import { test, expect } from "@playwright/test";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { assertNoCriticalSerious } from "../axe.js";

const fixturesDir = path.join(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
  "fixtures",
);

test("axe_helper_fails_on_serious_violation", async ({ page }) => {
  const violationPage = pathToFileURL(
    path.join(fixturesDir, "axe-violation.html"),
  ).href;
  await page.goto(violationPage);
  await expect(assertNoCriticalSerious(page)).rejects.toThrow();
});