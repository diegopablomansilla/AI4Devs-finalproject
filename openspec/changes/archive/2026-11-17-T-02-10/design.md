# Design

## Context

`#/admin/marcas` pinta el shell de T-01-11 (`data-region="admin-content"` con `<h1>Marcas</h1>`). T-02-07 ya dejó el patrón: `startRouter` acepta `request` / `showError` / `mountCategories` y monta solo si `section === "categorias"`. El cliente `api.js` (`request`) y `errors.js` (`showError` / `messageFor`) ya mapean `duplicate_slug`, `invalid_input`, `not_found`, `unavailable` y `network`. T-02-09 expone lista plana, alta (`name`; `slug` opcional), PATCH (`name` / `slug` / `active`) y DELETE (baja lógica sin occupancy). El harness de `vectra-web` usa `node --test` con stubs de HTML (sin jsdom ni upgrade de custom elements). Ver proposal.md — Why.

## Goals / Non-Goals

**Goals:**

- Módulo de marcas testeable en Node con `request` / `showError` inyectados
- Listado plano + alta / edición / baja / reactivación + toasts por `code`
- `startRouter` monta la vista en `marcas`; el resto del admin (salvo `categorias`) sigue sin ABM
- Listo cuando: HU-02-02 a mano (alta "Fixser" y baja lógica; referencias en producto = T-02-14)

**Non-Goals:**

- Extraer un *framework* de vistas admin (T-02-13 reutilizará el patrón, no un router nuevo)
- jsdom, Playwright, Web Component `vx-brand-list`
- Campo `slug` editable, query `q`, logos, occupancy, reindex

## Decisions

### 1. Módulo `vectra-web/js/admin/brands.js`; el router solo monta

Exportar `mountBrands(el, { request, showError, document })` → GET, pintar, bind de acciones. Sin árbol: no hay `buildTree` ni `parentOptions`.

`startRouter` acepta `mountBrands` opcional. Tras pintar admin, si `section === "marcas"` y `typeof request === "function"`, llama `mountBrands` sobre `[data-region="admin-content"]`. `categorias` sigue usando `mountCategories`. Sin `request` (tests actuales de `router.test.mjs`) no hay fetch: `router_admin_does_not_call_api` y `router_admin_content_region_has_no_business_ui` siguen verdes en `#/admin/productos` / `#/admin/stock`.

`app.js` pasa `request` de `api.js`, `showError` de `errors.js` y ambos `mount*`. Alternativa descartada: un `mountAdminSection` genérico — T-02-07 lo dejó explícitamente para este ticket como copia del patrón, no un framework.

### 2. HTML del listado como string; acciones por `data-*`

Misma estrategia que T-02-07: el test lee `el.innerHTML` y dispara handlers sin custom elements reales. Cada fila:

```html
<li data-brand-id="…" data-active="true|false">
  <span data-name>Fixser</span>
  <span data-slug>fixser</span>
  <button type="button" data-action="rename">Editar</button>
  <button type="button" data-action="deactivate">Dar de baja</button>
</li>
```

Formulario inline o en `vx-modal` con `vx-field name="name"` para que `showError` pueda asignar `errors[].field === "name"`. Sin campo slug. Sin `input type="file"`.

Alternativa descartada: editar *inline* en la fila sin formulario — más estado y peor de testear en el stub.

### 3. Cliente HTTP: un `request` por operación; recargar con GET

Tras 2xx, `GET ?includeInactive=true` de nuevo (listado chico). No se hace *optimistic UI*. `DELETE` devuelve 204 (`request` → `null`): no se parsea cuerpo. Errores: `catch` → `showError(err)` y se conserva el listado previo. `POST`/`PATCH` de nombre envían solo `{ name }`.

### 4. CSS `.vx-brand-*` con tokens existentes

Lista (`ul`/`li`), inactiva con `--vx-color-text-muted`. Botones ya tienen `--vx-touch-min`. Sin *tokens* nuevos. Sin estilos de logo.

### 5. Tests en `brands.test.mjs`; harness del archivo

Fake `request(method, path, opts)` que graba llamadas y resuelve `{ items }` o lanza `{ code, name: "ApiError" }`. Fake `showError` que acumula `code`. El listado se monta en un stub `{ innerHTML, querySelector, querySelectorAll, addEventListener }` mínimo (mismo estilo que `categories.test.mjs`).

`check-vectra-web.sh`: exigir `vectra-web/js/admin/brands.js` y añadir el archivo a `node --test`.

`router_admin_content_region_has_no_business_ui` MUST seguir usando `#/admin/productos` (no `marcas` ni `categorias`).

### 6. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-10` y rename a `T-02-10` (igual que T-02-07 / T-02-09).

## Risks / Trade-offs

- [Stub sin shadow DOM] → los tests no cubren el *upgrade* de `vx-modal` / `vx-field`. Mitigación: igual que T-02-07; DoD de teclado/contraste se valida a mano en el Listo cuando.
- [GET tras cada mutación] → un round-trip extra. Mitigación: catálogo de marcas es pequeño; evita estado divergente.
- [HU-02-02 «productos conservan la referencia»] → no hay tabla `producto` todavía. Mitigación: T-02-09 ya permite la baja; la UI no consulta occupancy; Listo cuando manual cubre alta + baja inactiva.
- [Slug visible vs editable] → mostrarlo como texto secundario no implica enviarlo. Mitigación: test `brands_does_not_send_slug`.

## Migration Plan

Sin migración Surreal ni cambio de OpenAPI. Archivos nuevos bajo `vectra-web/js/admin/` y un test Node. Rollback: revertir el change; el shell admin vuelve al placeholder de marcas.

## Open Questions

Ninguna que altere specs o tareas.
