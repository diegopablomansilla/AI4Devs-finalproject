/**
 * Escapa HTML y restaura solo <mark> / </mark> (T-03-16).
 * @param {unknown} html
 * @returns {string}
 */
export function sanitizeHighlight(html) {
  const s = String(html ?? "");
  const escaped = s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");
  return escaped
    .replace(/&lt;mark&gt;/gi, "<mark>")
    .replace(/&lt;\/mark&gt;/gi, "</mark>");
}

/**
 * @param {Element} parent
 * @param {unknown} html
 */
export function appendSanitizedHtml(parent, html) {
  const safe = sanitizeHighlight(html);
  const parts = safe.split(/(<mark>|<\/mark>)/gi);
  let inMark = false;
  for (const part of parts) {
    if (!part) {
      continue;
    }
    const lower = part.toLowerCase();
    if (lower === "<mark>") {
      inMark = true;
      continue;
    }
    if (lower === "</mark>") {
      inMark = false;
      continue;
    }
    if (inMark) {
      const mark = document.createElement("mark");
      mark.textContent = part;
      parent.appendChild(mark);
    } else {
      parent.appendChild(document.createTextNode(part));
    }
  }
}