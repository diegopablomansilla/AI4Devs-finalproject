# Proposal

## Ticket padre

- **Ticket:** [T-02-34](../../../docs/7-tickets.md) — UI listado de productos
- **Change:** `T-02-34`
- **HU:** [HU-02-10](../../../docs/6-historias.md#hu-02-10--listado-administrativo-de-productos), [HU-02-06](../../../docs/6-historias.md#hu-02-06--baja-y-reactivación-de-producto)
- **Depende de:** T-02-27 (`GET /api/v1/products` con filtros y cursor, en `develop`); T-02-22 (`DELETE` y `PATCH { active: true }`, en `develop`); T-01-11 (shell admin con menú `#/admin/productos` y `data-region="admin-content"`, en `develop`)
- **Hace:** tabla con filtros de categoría, marca y estado; "Cargar más" con cursor; acciones de baja y reactivación.
- **No hace:** edición en línea.
- **Listo cuando:** escenarios de HU-02-10 ejecutados a mano.

## Why

HU-02-10 pide revisar el catálogo por rama de categoría, marca y estado; HU-02-06 pide dar de baja y reactivar desde esa misma superficie. T-02-27 ya expone `GET /api/v1/products` y T-02-22 ya expone `DELETE` / reactivación; T-01-11 dejó `#/admin/productos` con un título placeholder. Sin este change el administrador no puede ejecutar a mano el listado ni las acciones de baja.

## What Changes

- Pintar la tabla de productos en `data-region="admin-content"` cuando el hash es `#/admin/productos`, hidratada desde `{ items, nextCursor }` de `GET /api/v1/products` (`active=true` por defecto).
- Filtros en español: categoría (`categoryId`, la API incluye la rama), marca (`brandId`) y estado (`active=true` activos / `active=false` dados de baja). Las opciones de categoría y marca se cargan de `GET /api/v1/categories?includeInactive=true` y `GET /api/v1/brands?includeInactive=true`.
- "Cargar más" si `nextCursor` no es `null`: siguiente `GET` con `cursor` y los mismos filtros, **append** de filas.
- Acciones por fila: baja (`DELETE /api/v1/products/{productId}`) si está activo; reactivar (`GET` ficha para `version` + `PATCH { version, active: true }`) si está inactivo.
- Errores de API (`invalid_input`, `not_found`, `version_conflict`, `unavailable`, `network`) mostrados por `code` vía `showError` / `messageFor` (T-01-07); sin interpretar `title`.
- TDD Node (`node --test`, sin jsdom ni npm): happy = listado default activos, filtro rama "Fijaciones", filtro inactivos + reactivar, "Cargar más", baja; unhappy = red, `not_found`, `invalid_input`, `version_conflict`, sin edición en línea, secciones admin restantes siguen sin ABM.
- `base.css`: clases de tabla/filtros con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni Web Component nuevo.

## Non-goals

- Edición en línea de celdas (SKU, nombre, precio, stock, …) — **No hace** del ticket
- Alta de producto / ficha con pestañas — **T-02-35**
- Atributos y códigos en la ficha — **T-02-36**
- Edición con conflicto de versión (diff de campos) — **T-02-37** (este change solo muestra `version_conflict` por toast)
- UI de movimientos de stock — **T-02-38**
- Imágenes / `imageUrl` — **T-02-39** (v2; `imageUrl` del summary sigue `null`)
- Quitar de búsqueda / recomendaciones al dar de baja — **T-03-07** / **T-04-06** (el backend ya emite outbox; la UI no llama a Meilisearch)
- Búsqueda por texto en este listado — **T-03-15** (No hace de T-02-27)
- Facetas de búsqueda — **T-03-18** / **T-03-19** (v2)
- Endpoints nuevos o cambios a `docs/openapi.yaml` — contrato cerrado en **T-02-27** / **T-02-22**
- Playwright / e2e — **T-09-09** / **T-09-10** / **T-09-11**
- jsdom, `package.json`, dependencias npm, job nuevo de CI, crates Rust
- Autenticación o roles (MVP: red local de confianza)
- Extraer un *framework* de vistas admin (copia del patrón de T-02-13, no un router genérico)

## Capabilities

### New Capabilities

- Ninguna. La UI de listado de productos es contenido de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: montar el listado admin de productos en `#/admin/productos` (tabla, filtros categoría/marca/estado, "Cargar más" con cursor, baja/reactivación, errores por `code`, sin edición en línea); exceptuar esa sección del «admin-content vacío» de T-01-11 (junto a `categorias`, `marcas` y `atributos`); ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/admin/products.js` (módulo nuevo); `vectra-web/js/router.js` y `vectra-web/js/app.js` (montar la sección); `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/products.test.mjs`; `scripts/check-vectra-web.sh` (incluir el test); ajuste de `router.test.mjs` para que el ABM vacío siga valiendo en secciones que no son `categorias`, `marcas`, `atributos` ni `productos` (p. ej. `#/admin/stock`)
- **APIs / OpenAPI:** consumo de `GET /api/v1/products`, `GET /api/v1/categories`, `GET /api/v1/brands`, `DELETE /api/v1/products/{productId}`, `GET /api/v1/products/{productId}` y `PATCH /api/v1/products/{productId}` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths ni DTOs
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-02-34`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando del ticket: ejecución manual de HU-02-10 (filtro por rama "Fijaciones" y filtro por inactivos con reactivar)
- **Ticket:** cierra T-02-34
- **Alcance:** cabe en ~2 h (espejo de T-02-13 + paginación cursor y lookups de filtros); no se divide
