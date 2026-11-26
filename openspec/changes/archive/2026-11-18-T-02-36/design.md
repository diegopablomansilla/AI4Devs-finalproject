# Design

## Context

La ficha de alta (`product-sheet.js` + `mode: "create"` en `products.js`) ya pinta Datos y hace `POST ProductCreate` **sin** `attributes` ni `codes`. `showError` asigna `errors[].field` al `vx-field` de `name` exacto (T-01-07); el Gateway mapea JSON string → `ValorAtributo::Texto` y JSON number → `Numero`, y un number con texto (`"cuarenta"`) es `400` `attributes[n].value`. `GET /api/v1/attribute-definitions` (T-02-12) lista el maestro global, ordenado por `order`. Harness Node sin jsdom, igual que T-02-35. Ver proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Extender `renderCreateProductSheet` / `readProductCreate` con atributos tipados + códigos fabricante/OEM
- Lookup de definiciones al abrir la ficha, sin ensuciar el GET del listado
- Pintar `attributes[n].value` sobre el control (Listo cuando)
- Sustituir `sheet_no_attributes_or_codes` por cobertura happy/unhappy de este ticket

**Non-Goals:**

- Cambiar `parseHash`, OpenAPI, o `showError`
- Web Component `vx-attribute-editor` / `vx-tabs`
- jsdom, Playwright, PATCH/edición (T-02-37)

## Decisions

### 1. Lookup de definiciones al entrar en `mode: "create"`

`mountProducts` sigue pidiendo categorías/marcas con `includeInactive=true` para filtros. Al pasar a `create`, pide `GET /api/v1/attribute-definitions` **sin** query (activas). Si falla, `showError` y `attributeDefs = []`; la ficha se pinta igual.

Alternativa descartada: pedir definiciones en el mount del listado — el listado no las usa y todos los tests de tabla tendrían una llamada extra.

### 2. Controles por `type` en `product-sheet.js`; sin `input type="number"`

`number` usa `vx-field` de texto para que `"cuarenta"` sea tipeable (Listo cuando). `boolean` y `enum` son `<select>` + `vx-field` receptor (mismo patrón que `categoryId`). Unidad en la etiqueta: `Largo (mm)`.

`readProductCreate` arma `attributes` solo con valores no vacíos, en orden de `items`. `value`:

- `text` / `enum` → string
- `boolean` → `true` / `false` JSON
- `number` → `Number` si `/^-?\d+(\.\d+)?$/`; si no, string (`"cuarenta"`)

Códigos: dos `vx-field` (`codes.manufacturer`, `codes.oem`). Vacío → omitir. MUST NOT `barcode`.

Alternativa descartada: un `<select>` de tipo de código + lista dinámica — el ticket pide fabricante y OEM, no N códigos ni barras.

### 3. `name="attributes[${i}].value"` = índice entre definiciones **renderizadas**

`i` es la posición en el array activo pintado (orden API). El POST omite vacíos, así que el `n` del `problem+json` es el índice en el array **enviado**. Tras el POST fallido se guarda ese array y se remapea `attributes[n].value` → `definitionId` → `i` de render; `fieldErrors["attributes[" + i + "].value"]` pinta el `vx-field`. Si el conjunto enviado es un prefijo (caso Listo cuando: solo `largo`), `n === i` y `showError` también acierta por nombre exacto.

Alternativa descartada: enviar todos los defs con `value: ""` — un number vacío sería `Texto("")` y `invalid_input` espurio.

### 4. Tests: extender `product-sheet.test.mjs`; harness de mount

`openCreateSheet` / `defaultMountResponses` MUST aceptar un 4º paso (GET definiciones) después del click «Nuevo producto». `sheet_create_perno_posts_product_create` deja de exigir `attributes === undefined`; el POST con medida/largo/fabricante vive en `sheet_create_perno_posts_attributes_and_codes`. Se elimina `sheet_no_attributes_or_codes`.

`queryStub` se extiende lo mínimo para `select[data-attr-id]` / `vx-field[name="attributes[0].value"]` (mismo estilo que `categoryId`).

### 5. CSS con tokens existentes

Clases `.vx-product-attrs` / `.vx-product-codes` con `--vx-touch-min` y `--vx-color-*`. Sin tokens nuevos.

### 6. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-36` y rename a `T-02-36` (igual que T-02-35).

## Risks / Trade-offs

- [Índice `n` ≠ `i` si hay huecos] → remap por `definitionId` del último POST. Mitigación: el Listo cuando usa una sola def; el happy path llena medida y largo consecutivos.
- [JSON number vs string `"40"`] → `"40"` sería `Texto` y fallaría igual que cuarenta. Mitigación: parsear decimales a number en `readProductCreate`.
- [Repaint pierde valores del form] → igual que T-02-35; el Listo cuando solo exige el error sobre el campo, no el valor persistido. Mitigación: aceptable en MVP.
- [Stub sin upgrade de custom elements] → no se cubre shadow DOM de `vx-field`. Mitigación: igual que T-02-35; DoD de teclado/contraste a mano.

## Migration Plan

Sin migración Surreal ni cambio de OpenAPI. Rollback: revertir el change; la ficha vuelve a T-02-35 (alta sin atributos/códigos).

## Open Questions

Ninguna que altere specs o tareas.
