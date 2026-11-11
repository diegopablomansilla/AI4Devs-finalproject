const TERMINAL_KEY = "vectra.terminal";

export class ApiError extends Error {
  /**
   * @param {{ status: number, code: string, title?: string, errors?: Array<{ field: string, message: string }> }} fields
   */
  constructor({ status, code, title, errors }) {
    const resolvedTitle = title ?? "";
    super(resolvedTitle || code);
    this.name = "ApiError";
    this.status = status;
    this.code = code;
    this.title = resolvedTitle;
    this.errors = errors ?? [];
  }
}

/**
 * @param {{ fetch: typeof fetch, storage?: { getItem: (key: string) => string | null } | null }} deps
 */
export function createApi({ fetch: fetchFn, storage }) {
  function terminalHeader() {
    try {
      const raw = storage?.getItem?.(TERMINAL_KEY);
      if (raw == null) return null;
      const trimmed = String(raw).trim();
      return trimmed === "" ? null : trimmed;
    } catch {
      return null;
    }
  }

  /**
   * @param {string} method
   * @param {string} path
   * @param {{ body?: unknown, signal?: AbortSignal }} [options]
   */
  async function request(method, path, options = {}) {
    const { body, signal } = options;
    const headers = new Headers();
    if (body !== undefined) {
      headers.set("Content-Type", "application/json");
    }
    const terminal = terminalHeader();
    if (terminal) {
      headers.set("X-Vectra-Terminal", terminal);
    }

    /** @type {RequestInit} */
    const init = { method, headers, signal };
    if (body !== undefined) {
      init.body = JSON.stringify(body);
    }

    let response;
    try {
      response = await fetchFn(path, init);
    } catch (err) {
      if (err?.name === "AbortError") {
        throw err;
      }
      throw new ApiError({
        status: 0,
        code: "network",
        title: "network",
        errors: [],
      });
    }

    const text = await response.text();

    if (response.ok) {
      if (text === "") {
        return null;
      }
      try {
        return JSON.parse(text);
      } catch {
        throw new ApiError({
          status: response.status,
          code: "internal",
          title: "",
          errors: [],
        });
      }
    }

    try {
      const json = JSON.parse(text);
      if (json && typeof json.code === "string") {
        throw new ApiError({
          status: json.status ?? response.status,
          code: json.code,
          title: json.title ?? "",
          errors: Array.isArray(json.errors) ? json.errors : [],
        });
      }
    } catch (err) {
      if (err instanceof ApiError) {
        throw err;
      }
    }

    throw new ApiError({
      status: response.status,
      code: "internal",
      title: "",
      errors: [],
    });
  }

  return { request, ApiError };
}

const defaultApi = createApi({
  fetch: (...args) => globalThis.fetch(...args),
  storage: globalThis.localStorage,
});

export const request = defaultApi.request;
