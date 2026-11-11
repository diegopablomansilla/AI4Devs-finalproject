import { describe, test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  parseRootVars,
  validateVxContract,
  validateContrastPairs,
  parseColor,
} from "./tokens-contrast.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const TOKENS_CSS = path.join(REPO_ROOT, "vectra-web/css/tokens.css");
const BASE_CSS = path.join(REPO_ROOT, "vectra-web/css/base.css");
const LOW_CONTRAST = path.join(__dirname, "low-contrast.tokens.css");

function readOrEmpty(p) {
  try {
    return fs.readFileSync(p, "utf8");
  } catch {
    return "";
  }
}

describe("tokens_root_declares_vx_contract", () => {
  test("vectra-web/css/tokens.css cumple contrato", () => {
    const css = readOrEmpty(TOKENS_CSS);
    assert.ok(css.length > 0, "falta vectra-web/css/tokens.css");
    const vars = parseRootVars(css);
    const { ok, errors } = validateVxContract(vars);
    assert.equal(ok, true, errors.join("; "));
  });
});

describe("tokens_text_surface_pairs_meet_wcag_aa", () => {
  test("nueve pares >= 4.5:1", () => {
    const css = readOrEmpty(TOKENS_CSS);
    const vars = parseRootVars(css);
    const { ok, failures } = validateContrastPairs(vars);
    const detail = failures
      .map((f) => `${f.name}: ${f.ratio.toFixed(2)}`)
      .join("; ");
    assert.equal(ok, true, `contraste insuficiente: ${detail}`);
  });
});

describe("tokens_low_contrast_pair_fails", () => {
  test("fixture low-contrast.tokens.css", () => {
    const css = fs.readFileSync(LOW_CONTRAST, "utf8");
    const vars = parseRootVars(css);
    const { ok, failures } = validateContrastPairs(vars);
    assert.equal(ok, false);
    assert.ok(failures.length > 0);
    const f = failures[0];
    assert.ok(f.name.length > 0);
    assert.ok(f.ratio < 4.5);
    assert.ok(f.ratio > 0);
  });
});

describe("tokens_reduced_transparency_disables_blur", () => {
  test("base.css define fallback sólido", () => {
    const base = readOrEmpty(BASE_CSS);
    assert.ok(base.length > 0, "falta vectra-web/css/base.css");
    assert.match(
      base,
      /@media\s*\(\s*prefers-reduced-transparency:\s*reduce\s*\)/,
      "falta @media (prefers-reduced-transparency: reduce)",
    );
    const mediaBlocks = base.match(
      /@media\s*\(\s*prefers-reduced-transparency:\s*reduce\s*\)\s*\{[^}]*\}/gs,
    );
    assert.ok(mediaBlocks && mediaBlocks.length > 0);
    const block = mediaBlocks[0];
    assert.match(block, /\.vx-surface/);
    assert.match(block, /--vx-color-surface-solid/);
    const blurInside = /backdrop-filter\s*:\s*[^;]*blur/i.test(block);
    assert.equal(blurInside, false, "backdrop-filter con blur dentro del media");
  });
});

describe("base_css_applies_bg_text_focus_and_touch_min", () => {
  test("referencias var(--vx-*) y controles", () => {
    const base = readOrEmpty(BASE_CSS);
    assert.ok(base.length > 0);
    assert.match(base, /var\(--vx-color-bg\)/);
    assert.match(base, /var\(--vx-color-text\)/);
    assert.match(base, /\.vx-surface/);
    assert.match(base, /:focus-visible/);
    assert.match(base, /var\(--vx-color-focus-ring\)/);
    assert.match(base, /var\(--vx-touch-min\)/);
    for (const sel of ["a", "button", "input", "select", "textarea"]) {
      const re = new RegExp(`${sel}[^{]*\\{[^}]*var\\(--vx-touch-min\\)`, "s");
      assert.match(
        base,
        re,
        `falta var(--vx-touch-min) en reglas para ${sel}`,
      );
    }
  });
});

describe("tokens contract unhappy", () => {
  test("superficie opaca falla contrato", () => {
    const vars = {
      "--vx-color-bg": "#0A0A0A",
      "--vx-color-surface": "#1C1C1E",
      "--vx-color-surface-solid": "#1C1C1E",
      "--vx-color-text": "#F5F5F7",
      "--vx-color-text-muted": "#C7C7CC",
      "--vx-color-accent": "#0A84FF",
      "--vx-color-error": "#FF453A",
      "--vx-color-success": "#30D158",
      "--vx-color-focus-ring": "#64D2FF",
      "--vx-color-on-accent": "#FFFFFF",
      "--vx-color-on-error": "#FFFFFF",
      "--vx-color-on-success": "#0A0A0A",
      "--vx-space-1": "4px",
      "--vx-space-2": "8px",
      "--vx-space-3": "16px",
      "--vx-space-4": "24px",
      "--vx-radius-sm": "8px",
      "--vx-radius-md": "16px",
      "--vx-touch-min": "44px",
      "--vx-surface-blur": "20px",
    };
    const surf = parseColor(vars["--vx-color-surface"]);
    assert.ok(surf && surf.a >= 1);
    const { ok } = validateVxContract(vars);
    assert.equal(ok, false);
  });
});
