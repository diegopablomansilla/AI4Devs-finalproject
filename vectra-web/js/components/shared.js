/** Shared design-token stylesheets for Shadow DOM (T-01-03). */

let styleLoader = null;
/** @type {CSSStyleSheet[] | null} */
let cache = null;

/**
 * @param {(url: URL) => Promise<string>} fn
 */
export function setSharedStyleLoader(fn) {
  styleLoader = fn;
  cache = null;
}

async function defaultLoad(url) {
  const res = await fetch(url);
  if (!res.ok) {
    throw new Error(`no se pudo cargar ${url.pathname}`);
  }
  return res.text();
}

/**
 * @returns {Promise<CSSStyleSheet[]>}
 */
export async function loadSharedStyleSheets() {
  if (cache) {
    return cache;
  }
  const load = styleLoader ?? defaultLoad;
  const tokensUrl = new URL("../../css/tokens.css", import.meta.url);
  const baseUrl = new URL("../../css/base.css", import.meta.url);
  const [tokensText, baseText] = await Promise.all([
    load(tokensUrl),
    load(baseUrl),
  ]);
  const tokensSheet = new CSSStyleSheet();
  const baseSheet = new CSSStyleSheet();
  if (typeof tokensSheet.replace === "function") {
    await tokensSheet.replace(tokensText);
    await baseSheet.replace(baseText);
  } else {
    tokensSheet.replaceSync(tokensText);
    baseSheet.replaceSync(baseText);
  }
  cache = [tokensSheet, baseSheet];
  return cache;
}

/**
 * @param {string} cssText
 * @returns {CSSStyleSheet}
 */
export function createLocalSheet(cssText) {
  const sheet = new CSSStyleSheet();
  if (typeof sheet.replaceSync === "function") {
    sheet.replaceSync(cssText);
  } else {
    sheet.replace(cssText);
  }
  return sheet;
}

/**
 * @param {ShadowRoot} shadowRoot
 * @param {string} localCss
 */
export async function applyComponentStyles(shadowRoot, localCss) {
  const shared = await loadSharedStyleSheets();
  const local = createLocalSheet(localCss);
  shadowRoot.adoptedStyleSheets = [...shared, local];
}
