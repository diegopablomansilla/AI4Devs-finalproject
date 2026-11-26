import { describe, test } from "node:test";
import assert from "node:assert/strict";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const apiUrl = pathToFileURL(
  path.join(__dirname, "../../../vectra-web/js/api.js"),
).href;

const { ApiError, createApi, request } = await import(apiUrl);

const TERMINAL_KEY = "vectra.terminal";

function problemJson(status, code, extra = {}) {
  const body = {
    type: "https://vectra.local/problems/not-found",
    title: "Recurso no encontrado",
    status,
    code,
    ...extra,
  };
  return new Response(JSON.stringify(body), {
    status,
    headers: { "Content-Type": "application/problem+json" },
  });
}

describe("api client", () => {
  test("api_request_parses_json_ok", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async () =>
        new Response(JSON.stringify({ hits: [] }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
    });
    const data = await req("GET", "/api/v1/search");
    assert.deepEqual(data, { hits: [] });
  });

  test("api_request_posts_json_body", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async (_path, init) => {
        assert.equal(init.method, "POST");
        assert.equal(_path, "/api/v1/products");
        assert.equal(init.headers.get("Content-Type"), "application/json");
        assert.equal(init.body, JSON.stringify({ sku: "P-1" }));
        return new Response(JSON.stringify({ id: "01" }), { status: 201 });
      },
    });
    await req("POST", "/api/v1/products", { body: { sku: "P-1" } });
  });

  test("api_request_sends_terminal_header", async () => {
    const { request: req } = createApi({
      storage: { getItem: (key) => (key === TERMINAL_KEY ? "Caja-1" : null) },
      fetch: async (_path, init) => {
        assert.equal(init.headers.get("X-Vectra-Terminal"), "Caja-1");
        return new Response("{}", { status: 200 });
      },
    });
    await req("GET", "/health");
  });

  test("api_request_forwards_abort_signal", async () => {
    const ac = new AbortController();
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async (_path, init) => {
        assert.equal(init.signal, ac.signal);
        return new Response("{}", { status: 200 });
      },
    });
    await req("GET", "/api/v1/search", { signal: ac.signal });
  });

  test("api_request_problem_json_not_found", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async () =>
        problemJson(404, "not_found", { title: "Recurso no encontrado" }),
    });
    await assert.rejects(
      () => req("GET", "/no-such-route"),
      (err) => {
        assert.ok(err instanceof ApiError);
        assert.equal(err.code, "not_found");
        assert.equal(err.status, 404);
        assert.equal(err.title, "Recurso no encontrado");
        return true;
      },
    );
  });

  test("api_request_invalid_input_exposes_errors", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async () =>
        problemJson(400, "invalid_input", {
          title: "La solicitud no es válida",
          errors: [{ field: "sku", message: "requerido" }],
        }),
    });
    await assert.rejects(
      () => req("POST", "/api/v1/products", { body: {} }),
      (err) => {
        assert.ok(err instanceof ApiError);
        assert.equal(err.code, "invalid_input");
        assert.equal(err.errors.length, 1);
        assert.equal(err.errors[0].field, "sku");
        return true;
      },
    );
  });

  test("api_request_network_failure_is_network_code", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async () => {
        throw new TypeError("Failed to fetch");
      },
    });
    await assert.rejects(
      () => req("GET", "/health"),
      (err) => {
        assert.ok(err instanceof ApiError);
        assert.equal(err.code, "network");
        return true;
      },
    );
  });

  test("api_request_abort_is_not_network", async () => {
    const ac = new AbortController();
    ac.abort();
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async () => {
        const err = new DOMException("Aborted", "AbortError");
        throw err;
      },
    });
    await assert.rejects(
      () => req("GET", "/api/v1/search", { signal: ac.signal }),
      (err) => {
        assert.equal(err.name, "AbortError");
        assert.notEqual(err.code, "network");
        return true;
      },
    );
  });

  test("api_request_non_json_error_is_internal", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => null },
      fetch: async () =>
        new Response("<html>oops</html>", {
          status: 500,
          headers: { "Content-Type": "text/html" },
        }),
    });
    await assert.rejects(
      () => req("GET", "/boom"),
      (err) => {
        assert.ok(err instanceof ApiError);
        assert.equal(err.code, "internal");
        assert.equal(err.status, 500);
        return true;
      },
    );
  });

  test("api_request_omits_terminal_header_when_empty", async () => {
    const { request: req } = createApi({
      storage: { getItem: () => "" },
      fetch: async (_path, init) => {
        assert.equal(init.headers.get("X-Vectra-Terminal"), null);
        return new Response("{}", { status: 200 });
      },
    });
    await req("GET", "/health");
  });
});

test("module exports request and ApiError", () => {
  assert.equal(typeof request, "function");
  assert.equal(typeof createApi, "function");
  assert.equal(ApiError.name, "ApiError");
});
