import { describe, test, before } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";
import { installDomGlobals } from "./dom-stub.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const highlightUrl = pathToFileURL(
  path.join(REPO_ROOT, "vectra-web/js/highlight.js"),
).href;

installDomGlobals();

/** @type {typeof import("../../../vectra-web/js/highlight.js")} */
let highlightMod;

describe("highlight", () => {
  before(async () => {
    highlightMod = await import(highlightUrl);
  });

  test("highlight_restores_mark_around_match", () => {
    const out = highlightMod.sanitizeHighlight("<mark>Perno</mark> hexagonal");
    assert.match(out, /<mark>Perno<\/mark>/);
    const host = document.createElement("div");
    highlightMod.appendSanitizedHtml(host, "<mark>Perno</mark> hexagonal");
    const mark = host.querySelector("mark");
    assert.ok(mark);
    assert.equal(mark.textContent, "Perno");
    assert.match(host.textContent ?? "", /hexagonal/);
  });

  test("highlight_img_onerror_is_plain_text", () => {
    const host = document.createElement("div");
    highlightMod.appendSanitizedHtml(host, '<img onerror="alert(1)">');
    assert.equal(host.querySelector("img"), null);
    assert.match(host.textContent ?? "", /&lt;img/);
  });

  test("highlight_script_tag_is_plain_text", () => {
    const host = document.createElement("div");
    highlightMod.appendSanitizedHtml(host, '<script>alert(1)</script>');
    assert.equal(host.querySelector("script"), null);
    assert.match(host.textContent ?? "", /script/);
  });

  test("highlight_em_not_restored", () => {
    const host = document.createElement("div");
    highlightMod.appendSanitizedHtml(host, "<em>x</em>");
    assert.equal(host.querySelector("em"), null);
    assert.match(host.textContent ?? "", /x/);
  });

  test("highlight_mark_with_attributes_not_restored", () => {
    const host = document.createElement("div");
    highlightMod.appendSanitizedHtml(host, '<mark onclick="x">y</mark>');
    assert.equal(host.querySelector("mark"), null);
    assert.match(host.textContent ?? "", /y/);
  });
});