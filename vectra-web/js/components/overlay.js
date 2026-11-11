/**
 * Trampa de foco, Escape y restore para vx-modal / vx-side-panel (T-01-04).
 */

/**
 * @param {ShadowRoot | null} root
 * @returns {HTMLElement[]}
 */
function getFocusableElements(root) {
  if (!root) {
    return [];
  }
  const nodes = root.querySelectorAll(
    'button:not([disabled]), [href], input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
  );
  return [...nodes];
}

/**
 * @param {HTMLElement} host
 * @param {{ getDialogRoot: () => ShadowRoot | null }} options
 */
export function createOverlayController(host, options) {
  /** @type {HTMLElement | null} */
  let savedOpener = null;
  /** @type {((e: KeyboardEvent) => void) | null} */
  let onKeyDown = null;

  function focusFirst() {
    const list = getFocusableElements(options.getDialogRoot());
    list[0]?.focus();
  }

  /**
   * @param {KeyboardEvent} e
   */
  function handleKeyDown(e) {
    if (e.key === "Escape") {
      hide();
      return;
    }
    if (e.key !== "Tab") {
      return;
    }
    const list = getFocusableElements(options.getDialogRoot());
    if (list.length === 0) {
      return;
    }
    const current = document.activeElement;
    const idx = list.indexOf(current);
    if (idx === -1) {
      return;
    }
    if (!e.shiftKey && idx === list.length - 1) {
      e.preventDefault();
      list[0].focus();
    } else if (e.shiftKey && idx === 0) {
      e.preventDefault();
      list[list.length - 1].focus();
    }
  }

  function hide() {
    if (!host.hasAttribute("open")) {
      return;
    }
    host.removeAttribute("open");
    if (onKeyDown) {
      host.removeEventListener("keydown", onKeyDown);
      onKeyDown = null;
    }
    host.dispatchEvent(
      new CustomEvent("vx-closed", { bubbles: true, composed: true }),
    );
    if (savedOpener && savedOpener.isConnected !== false) {
      savedOpener.focus();
    }
    savedOpener = null;
  }

  /**
   * @param {HTMLElement | undefined} opener
   */
  function show(opener) {
    savedOpener = opener ?? document.activeElement;
    host.setAttribute("open", "");
    focusFirst();
    if (onKeyDown) {
      host.removeEventListener("keydown", onKeyDown);
    }
    onKeyDown = handleKeyDown;
    host.addEventListener("keydown", onKeyDown);
  }

  return { show, hide };
}
