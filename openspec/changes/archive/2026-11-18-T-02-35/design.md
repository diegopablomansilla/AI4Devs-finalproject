# Design

## Context

`#/admin/productos` monta `mountProducts` (T-02-34): tabla, filtros, baja/reactivación, **sin** alta. `parseHash` solo acepta `#/admin/<sección>` (dos segmentos); `#/admin/productos/nuevo` sería `not-found`. T-02-21 expone `POST /api/v1/products` (`ProductCreate`: `sku`, `name`, `productType`, `categoryId`, `price` obligatorios; `initialStock` dispara movimiento `initial`; UNIQUE → `duplicate_sku`; `invalid_input` con `errors[].field` ya remapeado a camelCase: `categoryId`, `productType`, `initialStock`, …). `showError` (T-01-07) pinta toast por `code` y asigna `errors[].field` a `vx-field[name]`. Marcas/categorías ya usan formulario in-view (`data-action="create"` + `<form>`) sin cambiar el hash. Harness: `node --test` con stubs HTML (sin jsdom). Ver proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Módulo de ficha testeable en Node (`request` / `showError` inyectados)
- Listado ofrece «Nuevo producto»; ficha pestaña Datos; `POST ProductCreate`; vuelta al listado con el SKU visible
- Errores por `code` y por campo (`vx-field[name]`); `duplicate_sku` (HU-02-04)
- Pestañas Stock/Relaciones presentes y no operables (hueco para T-02-38 / T-04-09)

**Non-Goals:**

- Extender `parseHash` a tres segmentos
- Extraer un *framework* de fichas o un Web Component `vx-tabs`
- jsdom, Playwright, cambios de OpenAPI
- Atributos/códigos (T-02-36), PATCH/edición (T-02-37)

## Decisions

### 1. Ficha como estado interno de `#/admin/productos`; hash intacto

`mountProducts` guarda `mode: "list" | "create"`. «Nuevo producto» → `create` y pinta la ficha. Cancelar o `201` → `list` y recarga la primera página. `startRouter` no cambia.

Alternativa descartada: `#/admin/productos/nuevo` — exige cambiar T-01-01 (`parts.length !== 2` → `not-found`) y tests de router; el Listo cuando no pide deep-link. T-02-37 MAY reabrir este debate para edición por id.

### 2. Módulo `vectra-web/js/admin/product-sheet.js`

Exportar `renderProductSheet({ categories, brands, fieldErrors })` y `readProductCreate(form)` (o equivalente: HTML + parseo del `ProductCreate`). `mountProducts` posee el ciclo de vida (click, submit, `request`, `showError`) para no duplicar lookups.

`check-vectra-web.sh` exige el archivo (como `products.js`). Alternativa descartada: todo en `products.js` — el archivo ya cubre listado; T-02-36/38 extenderán la ficha.

### 3. HTML: `role="tablist"` nativo + `<form data-form="create-product">`

```html
<div class="vx-product-sheet">
  <h1>Nuevo producto</h1>
  <div role="tablist" aria-label="Ficha de producto">
    <button type="button" role="tab" aria-selected="true" data-tab="datos">Datos</button>
    <button type="button" role="tab" aria-selected="false" aria-disabled="true" data-tab="stock">Stock</button>
    <button type="button" role="tab" aria-selected="false" aria-disabled="true" data-tab="relaciones">Relaciones</button>
  </div>
  <form data-form="create-product">
    <vx-field name="sku" label="SKU" required></vx-field>
    …
    <label>Categoría
      <select name="categoryId">…</select>
    </label>
    <vx-field name="categoryId"></vx-field>
    …
    <button type="submit">Guardar</button>
    <button type="button" data-action="cancel-sheet">Cancelar</button>
  </form>
</div>
```

Los `<select>` de categoría/marca/unidad no pueden ser `vx-field` (el componente es `<input>`). Un `vx-field` hermano con el mismo `name` recibe el `error` de `showError` (T-01-07). En tests Node el stub de `querySelector('vx-field[name="sku"]')` sigue el patrón de `brands.test.mjs`.

Lookups del create: filtrar `active !== false` sobre las listas ya cargadas con `includeInactive=true` (el admin no debe dar de alta contra un maestro inactivo). Default unidad `unit`. Marca: option vacía «Sin marca».

Alternativa descartada: `vx-side-panel` — la ficha con pestañas es una vista completa, no un overlay.

### 4. Cuerpo `ProductCreate`

`readProductCreate` arma el JSON camelCase. `price` e `initialStock` salen como **string** (`NonNegativeDecimal`). Omite `description` / `brandId` / `location` / `initialStock` si están vacíos. **No** manda `attributes` ni `codes` (el default del API es `[]`). Guardia local: si falta un obligatorio, no llama a `request` (igual que marcas con nombre vacío).

`X-Vectra-Terminal` lo pone `api.js`; el fake `request` de tests no la ve y no hace falta.

### 5. Errores

`catch` → `showError(err)` y `paint()` de la ficha (sigue abierta). `duplicate_sku` no trae `errors[]`; solo toast. `invalid_input` con `field: "sku"` / `"categoryId"` / `"price"` / `"initialStock"` pinta el `vx-field` homónimo. No se interpreta `title`.

### 6. CSS `.vx-product-sheet*` / `.vx-product-tabs` con tokens existentes

Sin *tokens* nuevos. Tabs y campos usan `--vx-touch-min`, `--vx-color-*`. El `vx-field` receptor junto al `<select>` se puede dejar sin `label` y con input no prominente (el error `role="alert"` del componente basta).

### 7. Tests en `product-sheet.test.mjs` + un caso en `products.test.mjs`

Fake `request` / `showError` como en `products.test.mjs`. El test de Listo cuando: POST 201 con `{ sku: "PER-M8X40-ZN", … }` y el GET de listado siguiente incluye esa fila.

`products_empty_list_does_not_post` se conserva (no hay POST al montar). Se agrega `products_empty_list_offers_create`. `products_no_inline_edit` sigue inspeccionando el listado **sin** abrir la ficha.

### 8. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-35` y rename a `T-02-35` (igual que T-02-34).

## Risks / Trade-offs

- [Sin deep-link de ficha] → no se puede recargar «Nuevo producto» por URL. Mitigación: aceptable en MVP; T-02-37 puede introducir `#/admin/productos/{id}` si hace falta.
- [vx-field duplicado junto al select] → un input extra en el DOM. Mitigación: es el único gancho que `showError` entiende; no se cambia T-01-07.
- [HU-02-04 «aparece en la búsqueda»] → no hay Meilisearch en esta UI. Mitigación: Listo cuando del ticket es el listado admin; T-03-07 consume el outbox.
- [Filtro inactivos + alta] → el producto nuevo es activo y no aparece si se recarga con `active=false`. Mitigación: el happy path y el Listo cuando usan el default activos; no se fuerzan filtros.
- [Stub sin upgrade de custom elements] → los tests no cubren shadow DOM de `vx-field`. Mitigación: igual que T-02-10; DoD de teclado/contraste a mano.

## Migration Plan

Sin migración Surreal ni cambio de OpenAPI. Archivos nuevos bajo `vectra-web/js/admin/` y un test Node. Rollback: revertir el change; el listado vuelve a T-02-34 (sin alta).

## Open Questions

Ninguna que altere specs o tareas.
