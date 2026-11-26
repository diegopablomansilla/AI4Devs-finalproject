import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const rootDir = path.dirname(fileURLToPath(import.meta.url));
const stackPath = path.join(rootDir, ".stack.json");

/**
 * @returns {{ baseURL: string, vortexPid?: number, meiliPid?: number, tempDir?: string }}
 */
export function readStack() {
  const raw = fs.readFileSync(stackPath, "utf8");
  return JSON.parse(raw);
}

export function getBaseURL() {
  const stack = readStack();
  if (!stack.baseURL) {
    throw new Error("e2e: falta baseURL en .stack.json");
  }
  return stack.baseURL;
}