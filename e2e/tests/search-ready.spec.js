// @ts-check
import { test, expect } from "@playwright/test";
import { waitForSearchIndex } from "../search-ready.js";

test("search_index_wait_disabled_fails_immediately", async () => {
  await expect(
    waitForSearchIndex(undefined, { timeoutMs: 0 }),
  ).rejects.toThrow(/timeoutMs <= 0|disabled/i);
});