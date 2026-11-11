/**
 * Navegación por teclado reutilizable (listbox + aria-activedescendant).
 * T-01-10 · HU-01-03
 */

/**
 * @param {HTMLElement | null | undefined} el
 * @returns {HTMLElement | null}
 */
function focusTarget(el) {
  if (!el) {
    return null;
  }
  if (el.shadowRoot) {
    const input = el.shadowRoot.querySelector("input");
    if (input) {
      return input;
    }
  }
  return el;
}

/**
 * @param {HTMLElement | (() => HTMLElement | null) | null | undefined} restoreFocus
 * @returns {HTMLElement | null}
 */
function resolveRestoreFocus(restoreFocus) {
  if (!restoreFocus) {
    return null;
  }
  if (typeof restoreFocus === "function") {
    return focusTarget(restoreFocus());
  }
  return focusTarget(restoreFocus);
}

/**
 * @param {HTMLElement[]} items
 */
function ensureOptions(items) {
  for (let i = 0; i < items.length; i += 1) {
    const item = items[i];
    if (!item.getAttribute("id")) {
      item.setAttribute("id", `vx-opt-${i}`);
    }
    item.setAttribute("role", "option");
  }
}

/**
 * @param {HTMLElement} list
 * @param {HTMLElement[]} items
 * @param {number} index
 */
function paintSelection(list, items, index) {
  for (let i = 0; i < items.length; i += 1) {
    items[i].setAttribute("aria-selected", i === index ? "true" : "false");
  }
  if (index >= 0 && index < items.length) {
    const id = items[index].getAttribute("id");
    if (id) {
      list.setAttribute("aria-activedescendant", id);
    }
  } else {
    list.removeAttribute("aria-activedescendant");
  }
}

/**
 * @param {{
 *   list: HTMLElement,
 *   getItems: () => HTMLElement[],
 *   controllers?: HTMLElement[],
 *   restoreFocus?: HTMLElement | (() => HTMLElement | null) | null,
 *   onOpen?: (item: HTMLElement, index: number) => void,
 * }} opts
 * @returns {() => void}
 */
export function bindKeyboardList({
  list,
  getItems,
  controllers,
  restoreFocus = null,
  onOpen = () => {},
}) {
  if (!list.getAttribute("role")) {
    list.setAttribute("role", "listbox");
  }
  if (!list.hasAttribute("tabindex")) {
    list.setAttribute("tabindex", "0");
  }

  let activeIndex = -1;

  const navigate = (delta) => {
    const items = getItems();
    if (items.length === 0) {
      activeIndex = -1;
      paintSelection(list, items, -1);
      return;
    }
    ensureOptions(items);
    if (activeIndex < 0) {
      activeIndex = delta > 0 ? 0 : items.length - 1;
    } else {
      activeIndex = (activeIndex + delta + items.length) % items.length;
    }
    paintSelection(list, items, activeIndex);
  };

  const handleNavKeys = (event) => {
    const key = event.key;
    const items = getItems();
    if (key === "ArrowDown") {
      event.preventDefault();
      navigate(1);
      return;
    }
    if (key === "ArrowUp") {
      event.preventDefault();
      navigate(-1);
      return;
    }
    if (key === "Enter") {
      if (activeIndex >= 0 && activeIndex < items.length) {
        event.preventDefault();
        onOpen(items[activeIndex], activeIndex);
      }
    }
  };

  /** @param {KeyboardEvent} event */
  const onListKeyDown = (event) => {
    if (event.key === "Escape") {
      event.preventDefault();
      const target = resolveRestoreFocus(restoreFocus);
      target?.focus();
      return;
    }
    handleNavKeys(event);
  };

  /** @param {KeyboardEvent} event */
  const onControllerKeyDown = (event) => {
    handleNavKeys(event);
  };

  const targets = controllers ?? [list];
  const cleanups = [];
  for (const node of targets) {
    if (node === list) {
      list.addEventListener("keydown", onListKeyDown);
      cleanups.push(() => list.removeEventListener("keydown", onListKeyDown));
    } else {
      node.addEventListener("keydown", onControllerKeyDown);
      cleanups.push(() => node.removeEventListener("keydown", onControllerKeyDown));
    }
  }

  return () => {
    for (const off of cleanups) {
      off();
    }
  };
}
