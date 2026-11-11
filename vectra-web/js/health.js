export const HEALTH_POLL_MS = 10000;

const VALID_AGGREGATE = new Set(["ok", "degraded", "starting", "down"]);

/**
 * @param {unknown} body
 */
export function capabilitiesFromHealth(body) {
  const fallback = {
    status: "unknown",
    capabilities: {
      textSearch: true,
      ai: false,
      voice: false,
      vision: false,
    },
  };
  if (!body || typeof body !== "object") {
    return fallback;
  }
  const record = /** @type {{ status?: string, components?: Record<string, { status?: string }> }} */ (
    body
  );
  if (!record.components || typeof record.components !== "object") {
    return fallback;
  }
  const status = VALID_AGGREGATE.has(record.status ?? "")
    ? record.status
    : "unknown";
  const comp = record.components;
  const ai = comp.llm?.status === "up";
  const voice = comp.speech?.status === "up";
  const vision =
    comp.vision != null && comp.vision?.status === "up";
  return {
    status,
    capabilities: {
      textSearch: true,
      ai,
      voice,
      vision,
    },
  };
}

/**
 * @param {string} status
 */
export function statusLabel(status) {
  switch (status) {
    case "ok":
      return "Sistema operativo";
    case "degraded":
      return "Sistema degradado";
    case "starting":
      return "Arrancando";
    case "down":
      return "Sistema no disponible";
    default:
      return "Estado desconocido";
  }
}

/**
 * @param {Document} doc
 * @param {{ status: string, capabilities: { ai: boolean, voice: boolean, vision: boolean } }} snapshot
 */
export function applyHealthUi(doc, snapshot) {
  const label = statusLabel(snapshot.status);
  for (const el of doc.querySelectorAll("[data-health]")) {
    el.textContent = label;
  }
  const omnibox = doc.querySelector("vx-omnibox");
  if (!omnibox) {
    return;
  }
  const { capabilities } = snapshot;
  if (capabilities.ai) {
    omnibox.removeAttribute("ai-unavailable");
  } else {
    omnibox.setAttribute("ai-unavailable", "");
  }
  if (capabilities.voice) {
    omnibox.removeAttribute("voice-unavailable");
  } else {
    omnibox.setAttribute("voice-unavailable", "");
  }
  if (capabilities.vision) {
    omnibox.setAttribute("vision", "");
  } else {
    omnibox.removeAttribute("vision");
  }
}

/**
 * @param {(input: RequestInfo | URL, init?: RequestInit) => Promise<Response>} fetchFn
 */
async function fetchHealthSnapshot(fetchFn) {
  try {
    const response = await fetchFn("/health");
    const text = await response.text();
    if (text === "") {
      return capabilitiesFromHealth(null);
    }
    let json;
    try {
      json = JSON.parse(text);
    } catch {
      return capabilitiesFromHealth(null);
    }
    if (response.status !== 200 && response.status !== 503) {
      return capabilitiesFromHealth(null);
    }
    return capabilitiesFromHealth(json);
  } catch {
    return capabilitiesFromHealth(null);
  }
}

/**
 * @param {{
 *   fetch: typeof fetch,
 *   window: Window,
 *   document: Document,
 *   intervalMs?: number,
 *   setInterval?: typeof setInterval,
 *   clearInterval?: typeof clearInterval,
 * }} opts
 */
export function startHealth({
  fetch: fetchFn,
  window,
  document: doc,
  intervalMs = HEALTH_POLL_MS,
  setInterval: setIntervalFn = (fn, ms) => setInterval(fn, ms),
  clearInterval: clearIntervalFn = (id) => clearInterval(id),
}) {
  /** @type {{ status: string, capabilities: { textSearch: boolean, ai: boolean, voice: boolean, vision: boolean } }} */
  let lastSnapshot = capabilitiesFromHealth(null);

  const publish = (snapshot) => {
    lastSnapshot = snapshot;
    doc.dispatchEvent(
      new CustomEvent("vx-health", {
        bubbles: true,
        composed: true,
        detail: snapshot,
      }),
    );
    applyHealthUi(doc, snapshot);
  };

  const refresh = () => {
    void (async () => {
      const snapshot = await fetchHealthSnapshot(fetchFn);
      publish(snapshot);
    })();
  };

  refresh();

  const intervalId = setIntervalFn(refresh, intervalMs);

  const onFocus = () => {
    refresh();
  };

  const onVisibility = () => {
    if (doc.visibilityState === "visible") {
      refresh();
    }
  };

  const onHashChange = () => {
    applyHealthUi(doc, lastSnapshot);
  };

  window.addEventListener("focus", onFocus);
  doc.addEventListener("visibilitychange", onVisibility);
  window.addEventListener("hashchange", onHashChange);

  return () => {
    clearIntervalFn(intervalId);
    window.removeEventListener("focus", onFocus);
    doc.removeEventListener("visibilitychange", onVisibility);
    window.removeEventListener("hashchange", onHashChange);
  };
}
