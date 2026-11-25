# Design

## Context

En `develop`, `search.js` pinta filas con nombre (slot sanitizado) + SKU + `highlight.description` opcional. `vx-product-card` (T-01-05) ya implementa variante `full`, atributos de `SearchHit`, slot `name`, `data-out-of-stock` y *placeholder* «Sin imagen». El stub Node (`dom-stub.mjs`) registra custom elements vía `customElements._create`, pero `document.createElement` no los instancia y `appendChild` no dispara `connectedCallback`. `querySelector("vx-product-card")` no está cableado (el test T-03-16 que aserta `null` pasa aunque el selector no matchee). Ver `proposal.md` para el porqué; spec para mapeo, sin stock y unhappy path.

Constraints: MVP del curso; ~2 h; HU-03-05 parcial sin foto principal; `openspec new change` exige kebab-case (`t-03-17` → rename `T-03-17`).

## Goals / Non-Goals

**Goals:**

- Sustituir las filas de texto por `vx-product-card` `full` sin cambiar el contrato HTTP ni el listbox
- Nombre resaltado seguro en el slot; resto de campos por atributos
- Harness Node capaz de crear, conectar y seleccionar las tarjetas

**Non-Goals (diseño):**

- Reescribir `vx-product-card` (salvo un ajuste mínimo del slot si el stub no serializa `innerHTML`)
- Native `<slot>` projection como refactor amplio
- `onOpen` → ficha o recomendaciones (T-04-08)
- Debounce, facetas, compact

## Decisions

### 1. `paintHits` crea `vx-product-card`, no `div` de texto

Por cada hit válido: `document.createElement("vx-product-card")`, atributos de la tabla de la spec, hijo `span[slot="name"]` alimentado con `appendSanitizedHtml` si hay `highlight.name`; si no, atributo `name` (y MAY el mismo texto en el slot). `variant` omitido o `"full"`. Los ítems de `bindKeyboardList` son las tarjetas (`role="option"` lo pone T-01-10). `onOpen` sigue `() => {}`.

**Por qué:** el ticket Hace nombra `vx-product-card` completa; compact es T-04-08. Reusar el componente evita duplicar markup y el estado «Sin stock».

**Alternativa:** seguir con `div` y copiar el layout. Se descartó: duplica T-01-05. **Alternativa:** variante compact. Se descartó: oculta ubicación e imagen.

### 2. Mapeo plano de `SearchHit`; sin `highlight.description`

`brand`/`location`/`imageUrl` `null` → no setear el atributo (o string vacío). `price` y `stock` ya son strings `NonNegativeDecimal`. No formatear moneda extra: la tarjeta ya prefija `$ `.

No pintar `highlight.description`: el componente no tiene campo descripción y el Hace no lo lista. El test `search_highlight_description_paints_mark` se reemplaza por `search_highlight_description_does_not_paint_on_card`.

**Por qué:** HU-03-05 + T-01-05. **Alternativa:** línea extra de descripción bajo la tarjeta. Se descartó: fuera del ticket y ensucia la lista.

### 3. Imagen: *placeholder* en MVP, `image-url` si viene

HU-03-05 es MVP parcial (sin imagen principal; recableo T-03-06). `imageUrl` en el hit suele ser `null` → la tarjeta ya muestra «Sin imagen». Si el backend manda URL, se pasa; no se inventa `src`. No sanitizar `javascript:` en este ticket (el componente setea `src` como atributo; no hay fotos en el dataset MVP).

**Por qué:** el Hace pide el área de imagen; el recableo pide no depender de fotos. **Alternativa:** ocultar el bloque media. Se descartó: rompería la variante `full` ya especificada.

### 4. Hit malformado: no lanzar

`Array.isArray(hits)` se mantiene. Por cada elemento: si no es un objeto, `continue`. Campos faltantes → atributos vacíos; la tarjeta no lanza (T-01-05). Un único hit podrido no debe tumbar el listado.

**Por qué:** unhappy path obligatorio; la API 200 no garantiza hits perfectos en tests ni en degradación futura.

### 5. Stub: upgrade en `createElement` + `connectedCallback` al insertar

- `DocumentStub.createElement(tag)`: si `customElements.get(tag)` existe, `customElements._create(tag)`; si no, `FakeNode` como hoy.
- `FakeNode.appendChild`: si `typeof node.connectedCallback === "function"` y aún no hay `shadowRoot`, invocar `connectedCallback()` (fire-and-forget; la tarjeta es `async`).
- `querySelector("vx-product-card")` / `localName === "vx-product-card"` en `FakeNode` y `DocumentStub`, espejo de `vx-omnibox`.
- Tests: importar `vx-product-card.js` (o `components/index.js`) y `setSharedStyleLoader` como el omnibox; `waitTick` tras pintar para el `_render` async.

**Por qué:** en el navegador `createElement("vx-product-card")` + append dispara el ciclo. Sin esto, los tests no ven `data-out-of-stock` ni `[data-field]`.

**Alternativa:** `customElements._create` solo en tests inyectando un factory en `mountSearch`. Se descartó: el código de producción debe ser el DOM estándar. **Alternativa:** asertar solo atributos del host sin `connectedCallback`. Se descartó: no cubre placeholder ni «Sin stock» (ambos se aplican en `_render`).

El slot `name`: `appendSanitizedHtml` crea nodos `mark`; la tarjeta copia `slotChild.innerHTML` al `data-field="name"`. Si el stub no serializa `innerHTML`, extender getter/setter lo mínimo **o** hacer que `_fillNameEl` mueva/clone hijos en vez de `innerHTML`. Preferir el menor cambio que deje un `mark` observable en el nombre (slot o shadow). MUST NOT asignar el string crudo de `highlight.name` a `innerHTML`.

### 6. Teclado y vaciado

Mismo `bindKeyboardList` con `getItems: () => cards`. Consulta vacía / `hits: []`: quitar listbox y tarjetas (el test de empty query ya aserta listbox `null`).

## Risks / Trade-offs

- **[Stub no dispara `connectedCallback`]** → decisión 5; tests `search_zero_stock_*` y placeholder fallan en rojo hasta el stub.
- **[Copia `innerHTML` del slot reparsea HTML]** → solo nodos ya sanitizados por `highlight.js`; test `search_highlight_img_onerror_no_img_element` sigue siendo la red de seguridad.
- **[Lista más alta, scroll]** → `base.css` MAY `gap` con tokens; no tokens nuevos.
- **[kebab-case del CLI]** → scaffold `t-03-17` y rename (igual que T-03-16).
- **[Listo cuando humano]** → no se automatiza Playwright (T-09-10); la suite Node cubre los escenarios de HU-03-05.

## Migration Plan

N/A (SPA local). Rollback = revertir el change. `mountSearch` no cambia de firma.

## Open Questions

Ninguna que altere spec, enfoque o tareas.
