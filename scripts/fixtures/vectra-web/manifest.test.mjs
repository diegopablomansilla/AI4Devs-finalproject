import { describe, test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import {
  validateManifestObject,
  validateManifestIconsOnDisk,
  pngIhdrDimensions,
} from "./manifest.mjs";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const WEB = path.join(REPO_ROOT, "vectra-web");
const MANIFEST_PATH = path.join(WEB, "manifest.webmanifest");
const INDEX_PATH = path.join(WEB, "index.html");
const APP_JS = path.join(WEB, "js/app.js");

function readManifestJson() {
  const raw = fs.readFileSync(MANIFEST_PATH, "utf8");
  return JSON.parse(raw);
}

describe("manifest_json_is_valid", () => {
  test("vectra-web/manifest.webmanifest parsea", () => {
    assert.ok(fs.existsSync(MANIFEST_PATH), "falta vectra-web/manifest.webmanifest");
    let parsed;
    assert.doesNotThrow(() => {
      parsed = readManifestJson();
    });
    assert.equal(typeof parsed, "object");
  });
});

describe("manifest_declares_vectra_standalone_and_colors", () => {
  test("Listo cuando T-01-13", () => {
    const manifest = readManifestJson();
    const { ok, errors } = validateManifestObject(manifest);
    assert.equal(ok, true, errors.join("; "));
  });
});

describe("manifest_declares_192_512_and_maskable_icons", () => {
  test("Listo cuando T-01-13", () => {
    const manifest = readManifestJson();
    const { ok, errors } = validateManifestObject(manifest);
    assert.equal(ok, true, errors.join("; "));
    const maskable = manifest.icons.filter((i) =>
      String(i.purpose).includes("maskable"),
    );
    assert.ok(maskable.length >= 2);
  });
});

describe("manifest_icon_files_match_declared_sizes", () => {
  test("IHDR coincide con sizes", () => {
    const manifest = readManifestJson();
    const { ok, errors } = validateManifestIconsOnDisk(manifest, WEB);
    assert.equal(ok, true, errors.join("; "));
  });
});

describe("manifest_invalid_json_fails", () => {
  test("fixture", () => {
    assert.throws(() => JSON.parse("{ not json"), SyntaxError);
  });
});

describe("manifest_without_standalone_fails", () => {
  test("oráculo rechaza display browser", () => {
    const manifest = {
      name: "Vectra",
      short_name: "Vectra",
      display: "browser",
      background_color: "#0A0A0A",
      theme_color: "#0A0A0A",
      start_url: "/",
      lang: "es",
      icons: [
        {
          src: "/static/{{VERSION}}/icons/icon-192.png",
          sizes: "192x192",
          type: "image/png",
          purpose: "any",
        },
        {
          src: "/static/{{VERSION}}/icons/icon-512.png",
          sizes: "512x512",
          type: "image/png",
          purpose: "any",
        },
        {
          src: "/static/{{VERSION}}/icons/icon-192-maskable.png",
          sizes: "192x192",
          type: "image/png",
          purpose: "maskable",
        },
        {
          src: "/static/{{VERSION}}/icons/icon-512-maskable.png",
          sizes: "512x512",
          type: "image/png",
          purpose: "maskable",
        },
      ],
    };
    const { ok } = validateManifestObject(manifest);
    assert.equal(ok, false);
  });
});

describe("manifest_without_maskable_icon_fails", () => {
  test("oráculo rechaza sin maskable", () => {
    const manifest = {
      name: "Vectra",
      short_name: "Vectra",
      display: "standalone",
      background_color: "#0A0A0A",
      theme_color: "#0A0A0A",
      start_url: "/",
      lang: "es",
      icons: [
        {
          src: "/static/{{VERSION}}/icons/icon-192.png",
          sizes: "192x192",
          type: "image/png",
          purpose: "any",
        },
        {
          src: "/static/{{VERSION}}/icons/icon-512.png",
          sizes: "512x512",
          type: "image/png",
          purpose: "any",
        },
      ],
    };
    const { ok } = validateManifestObject(manifest);
    assert.equal(ok, false);
  });
});

describe("manifest_icon_src_file_missing_fails", () => {
  test("oráculo detecta src huérfano", () => {
    const manifest = {
      name: "Vectra",
      short_name: "Vectra",
      display: "standalone",
      background_color: "#0A0A0A",
      theme_color: "#0A0A0A",
      start_url: "/",
      lang: "es",
      icons: [
        {
          src: "/static/{{VERSION}}/icons/no-such.png",
          sizes: "192x192",
          type: "image/png",
          purpose: "any",
        },
      ],
    };
    const { ok, errors } = validateManifestIconsOnDisk(manifest, WEB);
    assert.equal(ok, false);
    assert.ok(errors.some((e) => e.includes("no-such.png")));
  });
});

describe("manifest_wrong_icon_dimensions_fail", () => {
  test("IHDR 1x1 no es 192x192", () => {
    const onePixel = Buffer.from(
      "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mP8z8BQDwAEhQGAhKmMIQAAAABJRU5ErkJggg==",
      "base64",
    );
    const dims = pngIhdrDimensions(onePixel);
    assert.deepEqual(dims, { width: 1, height: 1 });
    assert.notEqual(dims.width, 192);
  });
});

describe("manifest_does_not_register_service_worker", () => {
  test("No hace T-01-13", () => {
    assert.ok(fs.existsSync(INDEX_PATH));
    const index = fs.readFileSync(INDEX_PATH, "utf8");
    const app = fs.readFileSync(APP_JS, "utf8");
    assert.ok(!index.includes("serviceWorker.register"));
    assert.ok(!app.includes("serviceWorker.register"));
    assert.equal(fs.existsSync(path.join(WEB, "sw.js")), false);
  });
});
