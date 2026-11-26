# Design

## Context

`develop` ya tiene `vx-omnibox` (T-01-08): input, foco, Escape, sin selector ni `vx-query`. El harness Node (`dom-stub.mjs` + `omnibox.test.mjs`) no tiene `InputEvent`; `querySelector` no entiende `[data-mode]`. T-01-12 deshabilitará modos según `/health`; T-07-03 (v2) publicará `vision`. Ver proposal.md para el porqué; spec para modos, prefijo `?` y Listo cuando.

## Goals / Non-Goals

**Goals:**

- Contrato observable: `mode` + `vx-query { mode, text }` testeable en Node
- Prefijo `?` y selector equivalentes en el indicador
- Imagen ocultable con un atributo, sin hablar con `/health` todavía

**Non-Goals:**

- Listeners de `vx-query` en el *router* o en `app.js` (T-03-15)
- `disabled` según capacidades (T-01-12)
- Reusar `vx-button` / `vx-chip` dentro del shadow (anidamiento + stub)

## Decisions

### 1. Botones nativos en el shadow con `data-mode` y `aria-pressed`

Toolbar de `<button type="button">` (Texto, Voz, IA, Imagen) junto al input. `data-mode="text|voice|ai|image"`, etiqueta en español, `min-height`/`min-width: var(--vx-touch-min)`. El activo lleva `aria-pressed="true"` y estilo con `--vx-color-accent` (sin *tokens* nuevos).

No `vx-button`: el stub no simula bien CE anidados y el Omnibox no es un formulario. No `role="radio"`: el prefijo `?` puede forzar `ai` sin «elegir» el radio.

Tests: `shadowRoot.querySelectorAll("button")` + `getAttribute("data-mode")`. Si hace falta, extender `_matchSelector` para `[data-mode="…"]` (mismo patrón que `data-field`).

### 2. Modo efectivo = selector fijado ∪ prefijo `?`

Estado interno `_pinned`: `text` (default) | `voice` | `ai` | `image`.

| Condición | Modo efectivo |
|---|---|
| `value` empieza por `?` | `ai` (el prefijo gana) |
| si no, `_pinned` | ese valor |
| `voice`/`image` y el operador escribe | `_pinned = text`, luego la fila del prefijo |

Clic en **IA** pone `_pinned = ai` (sin `?`). Clic en **Texto** pone `_pinned = text`; si sigue habiendo `?`, el efectivo sigue `ai`. Escape vacía el value; si el `ai` venía solo del prefijo, efectivo `text`.

`observedAttributes`: `value`, `mode`, `vision`. Escribir `mode` en el host (salvo valores inválidos → `text`) actualiza `_pinned` y el indicador. `mode` desconocido: unhappy, fallback `text`, no throw.

### 3. `vx-query` en `input` y en clic de modo

`CustomEvent("vx-query", { bubbles: true, composed: true, detail: { mode, text } })` desde el host, igual que `vx-closed`. `text` = value crudo (T-05-20 puede recortar `?`).

Emitir también al cambiar de modo con el selector (aunque el texto no cambie) para que T-03-15 deje de buscar al pasar a `ai`/`voice`.

Sustituir `omnibox_does_not_emit_vx_query` por los tests de la spec. Escape que limpia MUST emitir `{ mode: "text", text: "" }` cuando el prefijo desaparece (o el `mode` efectivo actual si IA está fijada y el value queda vacío).

### 4. `vision` es atributo booleano; el *router* no lo setea

Sin `vision`: el botón Imagen tiene `hidden` (y `tabIndex = -1`). Con `vision`: se muestra. El `vx-omnibox` del *router* operador **no** lleva `vision` (MVP). T-01-12 / T-07-03 setearán el atributo después.

`dev/components.html`: un segundo `<vx-omnibox vision>` para ver Imagen a mano. El harness de catálogo no lo exige.

### 5. Stub: evento `input` y spy de `fetch`

Los tests setean `input.value` y despachan `new Event("input")` (ya hay `globalThis.Event`). No hace falta clase `InputEvent` salvo que el componente lea `event.inputType`.

Unhappy «no llama a search»: `globalThis.fetch = () => { calls += 1 }` alrededor del gesto; `calls === 0`. No se importa `api.js` en `vx-omnibox.js`.

## Risks / Trade-offs

- **[T-01-08 rompe `omnibox_does_not_emit_vx_query`]** → el test se reemplaza en el mismo PR; la spec maestra se actualiza al archivar.
- **[prefijo vs toggle]** → tabla del §2; tests `?hola`, clic IA sin `?`, `hola?`.
- **[T-01-12 necesitará `disabled`]** → los botones nativos ya aceptan `disabled`; este change no lo usa.
- **[macOS: `openspec new change T-01-09`]** → scaffold `t-01-09` y rename (igual que T-01-08).

## Migration Plan

N/A (SPA local). Rollback = revertir el change. Ningún consumidor de `vx-query` existe aún.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
