# Design

## Context

`#/admin/productos` pinta el shell de T-01-11 (`data-region="admin-content"` con `<h1>Productos</h1>`). T-02-07 / T-02-10 / T-02-13 ya dejaron el patrón: `startRouter` acepta `request` / `showError` / `mount*` y monta solo si `section` coincide. Hoy monta `categorias`, `marcas` y `atributos`; `productos` sigue siendo el placeholder que usan `router_admin_content_region_has_no_business_ui` y `router_admin_does_not_call_api`.

T-02-27 expone `GET /api/v1/products` (`categoryId` incluye rama, `brandId`, `active` default `true`, `cursor`/`limit`, `ProductSummaryPage`). T-02-22 expone `DELETE` (204) y `PATCH { version, active: true }`. `ProductSummary` no trae `version`; `ProductUpdate` la exige. El cliente `api.js` y `errors.js` ya mapean `invalid_input`, `not_found`, `version_conflict`, `unavailable` y `network`. Harness: `node --test` con stubs HTML (sin jsdom). Ver proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Módulo de productos testeable en Node con `request` / `showError` inyectados
- Tabla + filtros + "Cargar más" (append) + baja/reactivación + toasts por `code`
- `startRouter` monta la vista en `productos`; el resto del admin (salvo categorias/marcas/atributos) sigue sin ABM
- Listo cuando: HU-02-10 a mano (filtro rama «Fijaciones» y filtro inactivos con reactivar)

**Non-Goals:**

- Extraer un *framework* de vistas admin
- jsdom, Playwright, Web Component `vx-product-table`
- Alta/ficha (T-02-35), edición en línea, diff de `version_conflict` (T-02-37)
- Cambiar el contrato HTTP o el default `limit=50`

## Decisions

### 1. Módulo `vectra-web/js/admin/products.js`; el router solo monta

Exportar `mountProducts(el, { request, showError, document })` → lookups + listado, pintar, bind de filtros/acciones.

`startRouter` acepta `mountProducts` opcional. Tras pintar admin, si `section === "productos"` y `typeof request === "function"`, llama `mountProducts` sobre `[data-region="admin-content"]`. Sin `request` (tests actuales de `router.test.mjs`) no hay fetch.

`router_admin_content_region_has_no_business_ui` y `router_admin_does_not_call_api` pasan de `#/admin/productos` a `#/admin/stock` (el placeholder que queda). `app.js` inyecta `mountProducts` junto a los otros `mount*`.

Alternativa descartada: un `mountAdminSection` genérico — T-02-13 lo dejó explícitamente como copia del patrón.

### 2. HTML: `<table>` + `<select>` nativos; acciones por `data-*`

El test lee `el.innerHTML` y dispara `change`/`click` sin custom elements reales.

```html
<div class="vx-product-admin">
  <h1>Productos</h1>
  <div class="vx-product-filters">
    <label>Categoría <select data-filter="category">…</select></label>
    <label>Marca <select data-filter="brand">…</select></label>
    <label>Estado <select data-filter="active">…</select></label>
  </div>
  <table class="vx-product-table">…</table>
  <button type="button" data-action="load-more">Cargar más</button>
</div>
```

Fila: `data-product-id`, `data-active`, celdas de summary, `data-action="deactivate"|"reactivate"`. Encabezados en español. Option vacía «Todas» / «Todas» en categoría y marca (omite el query). Estado: `true` (Activos, default) / `false` (Inactivos).

Sin `<form>` de alta/edición. Alternativa descartada: `vx-product-card` compacto — es la tarjeta del operador (T-03-17), no el listado admin.

### 3. Query string y paginación

Helper local `productsQuery({ categoryId, brandId, active, cursor })` → `/api/v1/products?...`. Siempre envía `active=true|false`. Omite `categoryId`/`brandId`/`cursor` si están vacíos. Omite `limit` (default 50 del contrato).

Estado en el módulo: `filters`, `items[]`, `nextCursor`. «Cargar más» hace GET con cursor y `items = items.concat(page.items)`. Cambio de filtro o mutación exitosa: `items = []`, GET sin cursor, reemplazo.

Lookups en paralelo al primer GET (`Promise.all` de categories, brands, products). Fallo de lookup → `showError`, selects vacíos; fallo de products → `showError` y tabla vacía/previa.

### 4. Reactivar: GET ficha + PATCH `{ version, active: true }`

`ProductSummary` no tiene `version`. Flujo: `GET /api/v1/products/{id}` → `PATCH` solo esos dos campos. `DELETE` para baja (no `PATCH { active: false }`).

Si el GET de ficha falla, no se envía PATCH. `version_conflict` → `showError`; no hay recarga automática de ficha ni diff (T-02-37).

Alternativa descartada: pedir un cambio de API para `version` en el summary — fuera de alcance (contrato cerrado).

### 5. CSS `.vx-product-*` con tokens existentes

Tabla, filtros, fila inactiva con `--vx-color-text-muted`. Botones ya tienen `--vx-touch-min`. Sin *tokens* nuevos. Sin estilos de imagen.

### 6. Tests en `products.test.mjs`; harness del archivo

Fake `request(method, path, opts)` que graba llamadas y resuelve páginas `{ items, nextCursor }` o lanza `{ code, name: "ApiError" }`. Fake `showError` que acumula `code`. Stub mínimo de `innerHTML` / `querySelector` / `addEventListener` (mismo estilo que `brands.test.mjs`).

`check-vectra-web.sh`: exigir `vectra-web/js/admin/products.js` y añadir el archivo a `node --test`.

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-34` y rename a `T-02-34` (igual que T-02-10 / T-02-13).

## Risks / Trade-offs

- [GET extra por reactivación] → un round-trip más por fila. Mitigación: solo al reactivar; el listado no pide fichas.
- [Append + recarga tras mutación] → «Cargar más» se pierde al dar de baja. Mitigación: aceptable en MVP; evita estado divergente.
- [HU-02-06 «desaparece de la búsqueda»] → no hay Meilisearch en esta UI. Mitigación: T-03-07 consume el outbox; Listo cuando de este ticket es HU-02-10 a mano.
- [Stub sin shadow DOM] → los tests no cubren upgrade de `vx-toast`. Mitigación: igual que T-02-13; DoD de teclado/contraste a mano.
- [Lookups `includeInactive`] → aparecen categorías/marcas dadas de baja en el filtro. Mitigación: el admin necesita filtrar productos huérfanos de un maestro inactivo.

## Migration Plan

Sin migración Surreal ni cambio de OpenAPI. Archivos nuevos bajo `vectra-web/js/admin/` y un test Node. Rollback: revertir el change; el shell admin vuelve al placeholder de productos.

## Open Questions

Ninguna que altere specs o tareas.
