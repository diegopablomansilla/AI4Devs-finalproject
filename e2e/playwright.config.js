// @ts-check
import { defineConfig, devices } from "@playwright/test";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.dirname(fileURLToPath(import.meta.url));

/** @type {import('@playwright/test').PlaywrightTestConfig} */
export default defineConfig({
  testDir: "tests",
  fullyParallel: false,
  workers: 1,
  globalSetup: path.join(rootDir, "harness.mjs"),
  use: {
    ...devices["Desktop Chrome"],
  },
  projects: [{ name: "chromium", use: { browserName: "chromium" } }],
});