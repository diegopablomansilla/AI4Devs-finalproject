/** Mensajes en español por `code` (OpenAPI ErrorCode + `network` de cliente). */

export const MESSAGES = {
  invalid_input: "La solicitud no es válida",
  not_found: "Recurso no encontrado",
  conflict: "Conflicto con el estado actual",
  version_conflict: "El producto fue modificado por otra terminal",
  duplicate_sku: "El SKU ya existe",
  duplicate_slug: "El slug ya existe",
  duplicate_relation: "La relación ya existe",
  self_relation: "Un producto no puede relacionarse consigo mismo",
  category_cycle: "La categoría formaría un ciclo",
  attribute_in_use: "El atributo está en uso",
  negative_stock: "El stock no puede ser negativo",
  payload_too_large: "El cuerpo es demasiado grande",
  unsupported_media_type: "El tipo de contenido no está soportado",
  unavailable: "El servicio no está disponible",
  ai_unavailable: "El modo IA no está disponible",
  speech_unavailable: "El reconocimiento de voz no está disponible",
  timeout: "La operación superó el tiempo de espera",
  internal: "Error interno",
  network: "No hay conexión con el servidor",
};

/**
 * @param {string} yamlText
 * @returns {string[]}
 */
export function errorCodesFromOpenApi(yamlText) {
  const anchor = yamlText.indexOf("ErrorCode:");
  if (anchor === -1) {
    return [];
  }
  const block = yamlText.slice(anchor);
  const enumPos = block.indexOf("enum:");
  if (enumPos === -1) {
    return [];
  }
  const lines = block.slice(enumPos + "enum:".length).split("\n");
  /** @type {string[]} */
  const codes = [];
  for (const line of lines) {
    const match = line.match(/^\s+-\s+(\w+)\s*$/);
    if (match) {
      codes.push(match[1]);
      continue;
    }
    if (codes.length > 0 && !/^\s+-\s/.test(line) && line.trim() !== "") {
      break;
    }
  }
  return codes;
}

/**
 * @param {string[]} codes
 * @param {Record<string, string>} messages
 * @returns {string[]}
 */
export function missingErrorCodes(codes, messages) {
  /** @type {string[]} */
  const missing = [];
  for (const code of codes) {
    const msg = messages[code];
    if (typeof msg !== "string" || msg.trim() === "") {
      missing.push(code);
    }
  }
  return missing;
}

/**
 * @param {string | undefined | null} code
 * @returns {string}
 */
export function messageFor(code) {
  if (code != null && typeof MESSAGES[code] === "string") {
    return MESSAGES[code];
  }
  return MESSAGES.internal;
}

/**
 * @param {string} field
 */
function fieldSelector(field) {
  const escaped =
    typeof CSS !== "undefined" && typeof CSS.escape === "function"
      ? CSS.escape(field)
      : field.replace(/\\/g, "\\\\").replace(/"/g, '\\"');
  return `vx-field[name="${escaped}"]`;
}

/**
 * @param {{ toast?: HTMLElement | null, root?: ParentNode & { querySelector?: typeof Document.prototype.querySelector, body?: HTMLElement, createElement?: typeof Document.prototype.createElement } | null }} deps
 */
export function createErrorUi({ toast: fixedToast, root: fixedRoot } = {}) {
  /**
   * @param {{ code?: string, errors?: Array<{ field?: string, message?: string }>, name?: string, title?: string }} err
   */
  function showError(err) {
    if (err?.name === "AbortError") {
      return;
    }

    const root = fixedRoot ?? globalThis.document;
    let toast = fixedToast ?? null;
    if (!toast && root?.querySelector) {
      toast = root.querySelector("vx-toast");
    }
    if (!toast && root?.body && root?.createElement) {
      toast = root.createElement("vx-toast");
      root.body.appendChild(toast);
    }

    const message = messageFor(err?.code);
    if (toast) {
      toast.setAttribute("variant", "error");
      toast.textContent = message;
    }

    const fieldErrors = Array.isArray(err?.errors) ? err.errors : [];
    for (const item of fieldErrors) {
      const field = item?.field;
      if (!field || !root?.querySelector) {
        continue;
      }
      const fieldEl = root.querySelector(fieldSelector(field));
      if (fieldEl?.setAttribute) {
        fieldEl.setAttribute("error", String(item.message ?? ""));
      }
    }
  }

  return { showError };
}

export const { showError } = createErrorUi();
