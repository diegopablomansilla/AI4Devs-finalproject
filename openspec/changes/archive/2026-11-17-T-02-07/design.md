# Design

## Context

`#/admin/categorias` pinta el shell de T-01-11 (`data-region="admin-content"` con `<h1>Categorías</h1>`). No hay `js/admin.js`; T-01-11 dejó el hueco explícito para este ticket. El cliente `api.js` (`request`) y `errors.js` (`showError` / `messageFor`) ya mapean `category_cycle` y `duplicate_slug`. T-02-06 expone lista plana, alta, PATCH (`name` / `parentId` / `active`) y DELETE. El harness de `vectra-web` usa `node --test` con stubs de HTML (sin jsdom ni upgrade de custom elements). Ver proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Módulo de categorías testeable en Node con `request` / `showError` inyectados
- Árbol + cuatro acciones HTTP + toasts por `code`
- `startRouter` monta la vista solo en `categorias`; el resto del admin sigue sin ABM
- Listo cuando: HU-02-01 a mano salvo búsqueda (T-03-08)

**Non-Goals:**

- Extraer un *framework* de vistas admin (T-02-10 / T-02-13 reutilizarán el patrón, no un router nuevo)
- jsdom, Playwright, Web Component `vx-tree`
- Campo `slug`, occupancy real, reindex

## Decisions

### 1. Módulo `vectra-web/js/admin/categories.js`; el router solo monta

Exportar funciones puras + un `mountCategories(el, deps)`:

- `buildTree(items)` → nodos `{ category, children }[]` ordenados por `name`
- `parentOptions(items, movingId)` → candidatos de padre (raíz + todos menos `movingId` y descendientes)
- `mountCategories(el, { request, showError, document })` → GET, pintar, bind de acciones

`startRouter` acepta `request` y `showError` opcionales. Tras pintar admin, si `section === "categorias"` y `typeof request === "function"`, llama `mountCategories` sobre `[data-region="admin-content"]`. Sin `request` (tests actuales de `router.test.mjs`) no hay fetch: `router_admin_does_not_call_api` y `router_admin_content_region_has_no_business_ui` siguen verdes en `#/admin/productos` / `#/admin/stock`.

`app.js` pasa `request` de `api.js` y `showError` de `errors.js`. Alternativa descartada: `fetch` global desde el módulo — rompe el stub del *router* y el encabezado de terminal.

### 2. HTML del árbol como string; acciones por `data-*`

Misma estrategia que T-01-11: el test lee `el.innerHTML` y dispara handlers sin custom elements reales. Cada nodo:

```html
<li data-category-id="…" data-active="true|false">
  <span data-name>…</span>
  <span data-path>Fijaciones > Pernos</span>
  <vx-button data-action="create-child">Crear subcategoría</vx-button>
  …
</li>
```

Formularios en `vx-modal` (nombre) o `<select name="parentId">` (mover). `vx-field name="name"` para que `showError` pueda asignar `errors[].field === "name"`. Sin campo slug.

Alternativa descartada: editar *inline* sin modal — más estado y peor de testear en el stub.

### 3. Cliente HTTP: un `request` por operación; recargar con GET

Tras 2xx, `GET ?includeInactive=true` de nuevo (árbol chico). No se hace *optimistic UI*. `DELETE` devuelve 204 (`request` → `null`): no se parsea cuerpo. Errores: `catch` → `showError(err)` y se conserva el árbol previo.

Selector de padre omite self + descendientes para no ofrecer el ciclo en UI; el unhappy `category_cycle` se cubre inyectando el `ApiError` (el servidor sigue siendo la fuente de verdad).

### 4. CSS `.vx-category-*` con tokens existentes

Lista anidada (`ul`/`li`), indentación `--vx-space-*`, inactiva con `--vx-color-text-muted`. Botones ya tienen `--vx-touch-min`. Sin *tokens* nuevos. Sin `draggable`.

### 5. Tests en `categories.test.mjs`; harness del archivo

Fake `request(method, path, opts)` que graba llamadas y resuelve `{ items }` o lanza `{ code, name: "ApiError" }`. Fake `showError` que acumula `code`. El árbol se monta en un stub `{ innerHTML, querySelector, querySelectorAll, addEventListener }` mínimo (mismo estilo que `admin.test.mjs`).

`check-vectra-web.sh`: exigir `vectra-web/js/admin/categories.js` y añadir el archivo a `node --test`.

`router_admin_content_region_has_no_business_ui` MUST seguir usando `#/admin/productos` (no `categorias`).

### 6. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-07` y rename a `T-02-07` (igual que T-02-06).

## Risks / Trade-offs

- [Stub sin shadow DOM] → los tests no cubren el *upgrade* de `vx-modal` / `vx-field`. Mitigación: igual que T-01-11; DoD de teclado/contraste se valida a mano en el Listo cuando.
- [GET tras cada mutación] → un round-trip extra. Mitigación: catálogo de categorías es pequeño; evita estado divergente.
- [H-10 de productos stub `false`] → se puede dar de baja una categoría con productos hasta T-02-18. Mitigación: ya documentado en T-02-06; la UI solo muestra `conflict` si el API lo envía (hijas activas).
- [Rename de rama y búsqueda] → el path en UI se actualiza; Meilisearch no (T-03-08). Mitigación: Non-goal explícito.

## Migration Plan

Sin migración Surreal ni cambio de OpenAPI. Archivos nuevos bajo `vectra-web/js/admin/` y un test Node. Rollback: revertir el change; el shell admin vuelve al placeholder.

## Open Questions

Ninguna que altere specs o tareas.
