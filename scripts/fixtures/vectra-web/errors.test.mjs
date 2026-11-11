import { describe, test } from "node:test";
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = path.join(__dirname, "../../..");
const OPENAPI_PATH = path.join(REPO_ROOT, "docs/openapi.yaml");
const errorsUrl = pathToFileURL(
  path.join(REPO_ROOT, "vectra-web/js/errors.js"),
).href;

const {
  MESSAGES,
  messageFor,
  createErrorUi,
  errorCodesFromOpenApi,
  missingErrorCodes,
  showError,
} = await import(errorsUrl);

function fakeToast() {
  const state = { variant: null, text: "" };
  return {
    setAttribute(name, value) {
      if (name === "variant") state.variant = value;
    },
    get textContent() {
      return state.text;
    },
    set textContent(value) {
      state.text = String(value ?? "");
    },
    state,
  };
}

function fakeRoot(fields = {}) {
  return {
    querySelector(selector) {
      const m = String(selector).match(/^vx-field\[name="([^"]+)"\]$/);
      if (m) return fields[m[1]] ?? null;
      return null;
    },
  };
}

function fakeField() {
  const attrs = {};
  return {
    setAttribute(name, value) {
      attrs[name] = String(value);
    },
    getAttribute(name) {
      return attrs[name] ?? null;
    },
  };
}

describe("errors_map_covers_openapi_error_codes", () => {
  test("enum ErrorCode del openapi tiene mensaje en MESSAGES", () => {
    const yaml = fs.readFileSync(OPENAPI_PATH, "utf8");
    const codes = errorCodesFromOpenApi(yaml);
    assert.ok(codes.length > 0, "no se extrajeron codes de openapi");
    const missing = missingErrorCodes(codes, MESSAGES);
    assert.deepEqual(missing, [], `faltan mensajes para: ${missing.join(", ")}`);
  });
});

describe("errors_map_includes_network", () => {
  test("MESSAGES incluye network", () => {
    assert.equal(typeof MESSAGES.network, "string");
    assert.ok(MESSAGES.network.length > 0);
  });
});

describe("errors_message_for_not_found_is_spanish", () => {
  test("not_found no es el code técnico", () => {
    assert.equal(messageFor("not_found"), "Recurso no encontrado");
    assert.notEqual(messageFor("not_found"), "not_found");
  });
});

describe("errors_show_error_sets_toast_error_variant", () => {
  test("version_conflict en toast error", () => {
    const toast = fakeToast();
    const { showError: uiShow } = createErrorUi({ toast, root: fakeRoot() });
    uiShow({ code: "version_conflict" });
    assert.equal(toast.state.variant, "error");
    assert.equal(
      toast.state.text,
      "El producto fue modificado por otra terminal",
    );
  });
});

describe("errors_show_error_assigns_field_errors_to_vx_field", () => {
  test("invalid_input asigna error al vx-field por name", () => {
    const toast = fakeToast();
    const skuField = fakeField();
    const { showError: uiShow } = createErrorUi({
      toast,
      root: fakeRoot({ sku: skuField }),
    });
    uiShow({
      code: "invalid_input",
      errors: [{ field: "sku", message: "requerido" }],
    });
    assert.equal(skuField.getAttribute("error"), "requerido");
    assert.equal(toast.state.text, "La solicitud no es válida");
  });
});

describe("errors_map_missing_openapi_code_fails", () => {
  test("mapa sin timeout nombra el code faltante", () => {
    const yaml = fs.readFileSync(OPENAPI_PATH, "utf8");
    const codes = errorCodesFromOpenApi(yaml);
    const incomplete = { ...MESSAGES };
    delete incomplete.timeout;
    const missing = missingErrorCodes(codes, incomplete);
    assert.ok(missing.includes("timeout"), `missing=${missing.join(",")}`);
  });
});

describe("errors_unknown_code_uses_fallback_not_raw_code", () => {
  test("messageFor desconocido", () => {
    assert.equal(messageFor("no_such_code"), messageFor("internal"));
    assert.notEqual(messageFor("no_such_code"), "no_such_code");
  });

  test("showError con code desconocido", () => {
    const toast = fakeToast();
    const { showError: uiShow } = createErrorUi({ toast, root: fakeRoot() });
    uiShow({ code: "no_such_code" });
    assert.equal(toast.state.text, "Error interno");
    assert.notEqual(toast.state.text, "no_such_code");
  });
});

describe("errors_abort_does_not_show_toast", () => {
  test("AbortError no modifica toast", () => {
    const toast = fakeToast();
    toast.textContent = "antes";
    toast.setAttribute("variant", "info");
    const { showError: uiShow } = createErrorUi({ toast, root: fakeRoot() });
    uiShow({ name: "AbortError", code: "network" });
    assert.equal(toast.state.text, "antes");
    assert.equal(toast.state.variant, "info");
  });
});

describe("errors_unmatched_field_does_not_throw", () => {
  test("field sin vx-field no lanza", () => {
    const toast = fakeToast();
    const { showError: uiShow } = createErrorUi({ toast, root: fakeRoot() });
    assert.doesNotThrow(() =>
      uiShow({
        code: "invalid_input",
        errors: [{ field: "attributes[1].value", message: "inválido" }],
      }),
    );
    assert.equal(toast.state.text, "La solicitud no es válida");
  });
});

describe("errors_toast_text_is_not_technical_code", () => {
  test("no usa title técnico ni el code", () => {
    const toast = fakeToast();
    const { showError: uiShow } = createErrorUi({ toast, root: fakeRoot() });
    uiShow({ code: "not_found", title: "not_found" });
    assert.equal(toast.state.text, "Recurso no encontrado");
    assert.notEqual(toast.state.text, "not_found");
  });
});

test("module exports showError", () => {
  assert.equal(typeof showError, "function");
  assert.equal(typeof messageFor, "function");
  assert.equal(typeof createErrorUi, "function");
});
