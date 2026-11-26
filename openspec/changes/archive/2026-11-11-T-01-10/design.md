# Design

## Context

`develop` tiene `vx-omnibox` (foco, Escape limpia, modos, `vx-query`), `vx-product-card` y el layout operador con `data-region="results"` vacío. El harness Node (`dom-stub.mjs`) ya despacha `KeyboardEvent` y `focus()`. `querySelector` del stub es mínimo (`input`, `button`, `[data-mode]`); no hay `querySelectorAll('[role="option"]')`. Ver proposal.md para el porqué; spec para listbox, flechas, Enter, Escape y Listo cuando.

## Goals / Non-Goals

**Goals:**

- API de módulo testeable en Node: `getItems()` + callbacks, sin CE nuevo
- Mismo helper usable desde el input del Omnibox (T-03-15 / T-03-21) y con la lista enfocada
- Demo en `components.html` que cierra el Listo cuando humano

**Non-Goals:**

- Cablear `data-region="results"` del *router* (T-03-15)
- Patrón combobox completo (`aria-controls` / `aria-expanded` en el input) — T-03-21 puede sumarlo
- Extender `vx-omnibox.js` (el demo toma el input del shadow)

## Decisions

### 1. Función `bindKeyboardList`, no custom element

```js
export function bindKeyboardList({
  list,
  getItems,
  controllers = [list],
  restoreFocus = null,
  onOpen = () => {},
})
```

Devuelve `unbind()`. `getItems()` se evalúa en cada `keydown`. El caller pone `role="listbox"` o el binder lo asegura junto con `tabindex="0"`. Cada ítem recibe `role="option"`, `id` si falta (`vx-opt-${i}`), `aria-selected`. El listbox lleva `aria-activedescendant`.

No CE: no hay upgrade, no pelea con el stub, y T-03-15 puede re-bindear al re-renderizar.

Alternativa descartada: `vx-keyboard-list` — obliga a envolver resultados y sugerencias en otro host; el ticket pide un módulo.

### 2. Listbox en el contenedor; controladores extra sin Escape

Patrón **listbox** (foco en la lista, `aria-activedescendant` en el listbox). `controllers` extra (el `<input>` del Omnibox) reciben ArrowDown / ArrowUp / Enter con el mismo efecto. **Escape solo se maneja si `event.currentTarget === list`**, para no pelear con T-01-08 (Escape en el input vacía la consulta).

`restoreFocus`: `HTMLElement` o `() => HTMLElement | null`. Si el nodo tiene `shadowRoot` y un `input`, se enfoca ese input; si no, `element.focus()`. Ausente o `null`: no-op, no throw.

Alternativa descartada: combobox con `aria-activedescendant` en el input — correcto para sugerencias, pero el ticket dice «volver al Omnibox», lo que implica que el foco puede estar en la lista. Los `controllers` cubren el caso «flechas con el Omnibox enfocado» sin mudar el patrón.

### 3. Ciclo en flechas; Enter sin selección es no-op

Índice interno `-1` (nada activo). ArrowDown desde `-1` → `0`; desde el último → `0`. ArrowUp desde `-1` → último; desde `0` → último. Enter con índice `-1` no llama `onOpen`. Lista vacía: índice `-1`, teclas no throw, se quita `aria-activedescendant`.

`preventDefault` en ArrowDown, ArrowUp, Enter consumidos, y Escape en la lista.

Home/End, PageUp/PageDown y clic de mouse: fuera. T-03-15 puede sumar click → `onOpen` en el caller.

### 4. Demo solo en `dev/components.html`

Sección «Lista por teclado»: contenedor `#demo-keyboard-list` con ≥2 `vx-product-card` de ejemplo (datos distintos de la sección de tarjetas estáticas, o los mismos campos). Script de página: `bindKeyboardList` con `controllers: [list, omniboxInput]`, `onOpen` → `demo-panel.show(...)` y texto del `name`/`sku`. Escape en la lista → input del Omnibox. Escape con el panel abierto sigue siendo T-01-04 + foco Omnibox (T-01-08).

El *router* no se toca: `data-region="results"` vacío.

### 5. Tests Node con `getItems` explícito; stub mínimo

`keyboard-list.test.mjs` instala el stub, crea `list` + nodos `div` (reuso) y, en el happy del catálogo, lee `components.html`. Pasa `getItems: () => items` para no depender de `querySelectorAll`. Spy de `onOpen` y de `globalThis.fetch`.

Extender `dom-stub.mjs` **solo** si un assert no se puede expresar (p. ej. `tabIndex`). No jsdom.

Harness: `[ -f vectra-web/js/keyboard-list.js ]` y `node --test … keyboard-list.test.mjs`.

## Risks / Trade-offs

- **[Escape en Omnibox vs lista]** → Escape en extra-controllers no se intercepta; solo en `list`. Tests: consulta intacta al volver; `omnibox_escape_clears_query` sigue verde.
- **[stub sin querySelectorAll]** → tests inyectan `getItems`; el demo del navegador usa DOM real.
- **[macOS: `openspec new change T-01-10`]** → scaffold `t-01-10` y rename (igual que T-01-09).
- **[T-03-15 re-renderiza resultados]** → `getItems()` por tecla + `unbind()`; no se cachean nodos muertos.

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Ningún consumidor en el *router* todavía.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
