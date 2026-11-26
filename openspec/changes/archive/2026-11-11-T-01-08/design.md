# Design

## Context

`develop` ya tiene el *router* hash (T-01-01: `#/` pinta un `<h1>` placeholder), el kit de componentes con Shadow DOM / `adoptedStyleSheets` (T-01-03), overlays que emiten `vx-closed` y restauran el abridor (T-01-04), y el harness `check-vectra-web.sh` + `dom-stub.mjs` (sin jsdom ni npm). `overlay.js` despacha `vx-closed` y **después** hace `savedOpener.focus()`. El stub de `document` no es un `EventTarget` y el burbujeo no cruza el light DOM. Ver proposal.md para el porqué; la spec para layout, contrato de `vx-omnibox` y Listo cuando.

## Goals / Non-Goals

**Goals:**

- Shell de mostrador testeable en Node: Omnibox + regiones vacías
- Listo cuando automatizado: foco al conectar y tras cerrar un `vx-side-panel`
- Escape limpia solo cuando el Omnibox tiene el foco (no pelea con el overlay)

**Non-Goals:**

- Cambiar el contrato T-01-04 «el panel restaura el abridor» en tests aislados (sin Omnibox)
- Reordenar `overlay.js` para emitir `vx-closed` después del restore (el Omnibox gana con *microtask*)
- Columna CSS permanente del panel (sigue siendo overlay `position: fixed`)

## Decisions

### 1. Tres regiones en el HTML del *router*, panel como overlay

`viewHtml({ view: "operator" })` deja de ser un heading: `main.vx-operator[data-view=operator]` con `data-region="omnibox"` (`<vx-omnibox>`), `data-region="results"` (vacío) y `data-region="side-panel"` (`<vx-side-panel>` sin `open`). El enlace a `#/admin/productos` queda en el encabezado junto al Omnibox.

`vx-side-panel` ya cubre desde la derecha; el «lugar» del ticket es el nodo en el árbol, no un aside visible. T-04-08 abrirá ese mismo panel.

`base.css` añade `.vx-operator` (grid o flex: Omnibox arriba, resultados crecen). Sin *tokens* nuevos.

Admin / not-found no incluyen `vx-omnibox` para no robar foco.

Alternativa descartada: columna vacía siempre visible — contradice el overlay de T-01-04 y se vería como un hueco.

### 2. `vx-omnibox` escucha `vx-closed` en `document` y enfoca en *microtask*

Al `connectedCallback`: construye shadow + input, llama `input.focus()`, y registra `document.addEventListener("vx-closed", this._onClosed)`. Al `disconnectedCallback`: `removeEventListener`. `_onClosed` hace `queueMicrotask(() => { if (this.isConnected) this._input.focus(); })`.

Así el Listo cuando gana sobre `savedOpener.focus()` síncrono de `overlay.js` sin tocar T-01-04. Tests de panel **sin** Omnibox siguen restaurando el abridor.

Alternativa descartada: escuchar solo en el host — el panel es hermano, el evento no llega. Alternativa: cambiar el orden en `overlay.js` — efecto colateral en todos los overlays.

### 3. Escape en el input del Omnibox, no en `window`

`keydown` en el input (o el host con el input enfocado): si `key === "Escape"`, `value = ""` y `preventDefault`. Con el panel abierto el foco está atrapado ahí: Escape cierra el panel (T-01-04) y **no** vacía la consulta. Consulta ya vacía: no-op, no lanza.

No se emite `vx-query` (T-01-09). No hay `fetch`.

El host sincroniza atributo `value` ↔ input (igual que `vx-field`) para que los tests y T-01-09/T-03-15 lean/escriban la consulta.

### 4. Landmark `role="search"` + `aria-label="Buscar"`

El host tiene `role="search"` y `aria-label="Buscar"` (español, HU-01-03). Un `<input type="search">` en el shadow con `min-height`/`min-width: var(--vx-touch-min)`. No se reutiliza `vx-field`: el Omnibox no es un campo de formulario con error/help, y T-01-09 le agregará modos.

### 5. Stub: `document` como EventTarget y burbujeo light DOM

Extender `dom-stub.mjs` lo mínimo para este ticket:

- `document` con `addEventListener` / `removeEventListener` / `dispatchEvent`
- `dispatchEvent` con `bubbles` recorre `parentNode` en light DOM y, si `composed`, sale del shadow hacia el host y sigue hasta `document`

Sin eso no se puede testear «cerrar el panel → foco en Omnibox» en Node. Los tests de overlay aislados no cambian.

`omnibox.test.mjs` usa el stub (como `components.test.mjs`). El layout del *router* se aserta en `router.test.mjs` sobre el HTML string (`<vx-omnibox>`, `data-region`, admin sin tag). Foco **no** se aserta en el stub del *router* (sigue siendo HTML, no nodos reales).

### 6. Catálogo: `vx-omnibox` en `components.html`

El tag entra al catálogo para el harness. Con Omnibox conectado, cerrar el panel de muestra deja el foco en el Omnibox (spec), no en el botón abridor. El test `vx_side_panel_close_restores_focus_to_opener` sigue montando panel + abridor **sin** Omnibox.

## Risks / Trade-offs

- **[overlay restaura abridor después de `vx-closed`]** → *microtask* en el Omnibox. Si un test no espera el turno, verá el abridor; los tests de T-01-08 `await queueMicrotask` (o `setImmediate`).
- **[stub incompleto vs jsdom]** → solo EventTarget en `document` + burbujeo; no se agrega jsdom ni npm.
- **[catálogo vs T-01-04 Listo cuando humano]** → en `components.html` el foco post-cierre va al Omnibox; el Listo cuando de T-01-04 queda cubierto por el test aislado.
- **[innerHTML del *router* y upgrade]** → en el navegador los custom elements upgredean al pintar; el foco al conectar cubre carga y vuelta desde admin. El stub del *router* no simula upgrade; el foco vive en `omnibox.test.mjs`.

## Migration Plan

N/A (SPA local, sin persistencia ni API). Rollback = revertir el change.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
