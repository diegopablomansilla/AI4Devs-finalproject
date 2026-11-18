# Proposal

## Ticket padre

- **Ticket:** [T-02-35](../../../docs/7-tickets.md) — UI alta de producto
- **Change:** `T-02-35`
- **HU:** [HU-02-04](../../../docs/6-historias.md#hu-02-04--alta-de-producto)
- **Depende de:** T-02-34 (listado admin de productos en `#/admin/productos`, en `develop`); T-02-21 (`POST /api/v1/products`, en `develop`)
- **Hace:** ficha con pestañas; pestaña "Datos" con SKU, nombre, tipo, descripción, categoría, marca, unidad, precio, ubicación y stock inicial; errores por campo.
- **No hace:** atributos ni códigos (T-02-36).
- **Listo cuando:** se crea un producto desde la UI y aparece en el listado.

## Why

HU-02-04 pide dar de alta un producto con ficha y stock inicial desde la administración. T-02-21 ya expone `POST /api/v1/products` y T-02-34 ya lista el catálogo, pero el listado **no** ofrece crear (lo dejó explícitamente a este ticket). Sin la ficha de alta el administrador no puede ejecutar a mano el escenario «Alta completa» ni ver el producto nuevo en la tabla.

## What Changes

- Añadir la acción **Nuevo producto** al listado de `#/admin/productos` (también con lista vacía). Al activarla, `data-region="admin-content"` MUST reemplazar la tabla por la **ficha** de alta, sin cambiar el hash (`#/admin/productos`).
- Ficha con **pestañas**: pestaña **Datos** activa y operable; las demás pestañas del MVP posterior (Stock, Relaciones) visibles pero no operables. MUST NOT pintar pestaña Imágenes (v2, T-02-39).
- Pestaña Datos: `vx-field` / `<select>` para SKU, nombre, tipo (`productType`, texto libre), descripción, categoría, marca (opcional), unidad (`unitOfMeasure`, default `unit`), precio, ubicación (pasillo / estante / posición) y stock inicial. Guardar envía `POST /api/v1/products` con `ProductCreate` (sin `attributes` ni `codes`, o arrays vacíos).
- Tras `201`, volver al listado y recargar la primera página: el producto creado MUST aparecer (Listo cuando).
- Errores: toast por `code` vía `showError` / `messageFor` (T-01-07); `invalid_input` pinta `errors[].field` sobre el `vx-field` cuyo `name` coincide (`sku`, `name`, `productType`, `categoryId`, `price`, `initialStock`, …); `duplicate_sku` (HU-02-04) por toast.
- TDD Node (`node --test`, sin jsdom ni npm): happy = abrir ficha, POST del perno M8×40, producto en el listado, stock inicial; unhappy = `duplicate_sku`, `invalid_input` por campo, red, sin atributos/códigos.
- `base.css`: clases de ficha/pestañas con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni Web Component nuevo.

## Non-goals

- Editor de atributos ni códigos (fabricante, OEM, barcode) — **No hace** del ticket; **T-02-36**
- Edición de ficha existente / `PATCH` / conflicto de `version` — **T-02-37**
- Pestaña Stock (ingresos/ajustes) — **T-02-38**
- Pestaña Imágenes / `multipart` — **T-02-39** (v2)
- Pestaña Relaciones — **T-04-09**
- Hash profundo `#/admin/productos/nuevo` o `#/admin/productos/{id}` — `parseHash` sigue en dos segmentos (T-01-01); la ficha es estado interno de la sección
- Quitar «sin acción de crear» del listado vacío de T-02-34 queda **superseded** aquí (ahora sí hay alta)
- Indexación en Meilisearch / «aparece en la búsqueda en < 1 s» — **T-03-07** / **T-03-15** (HU-02-04 búsqueda; Listo cuando de este ticket es el listado admin)
- Endpoints nuevos o cambios a `docs/openapi.yaml` — contrato cerrado en **T-02-21**
- Playwright / e2e de alta — **T-09-09** / **T-09-11**
- jsdom, `package.json`, dependencias npm, job nuevo de CI, crates Rust
- Autenticación o roles (MVP: red local de confianza)

## Capabilities

### New Capabilities

- Ninguna. La UI de alta de producto es contenido de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: ficha de alta con pestaña Datos en `#/admin/productos` (`POST /api/v1/products`, errores por campo, vuelta al listado); el listado ofrece «Nuevo producto» (también vacío); ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/admin/products.js` (acción de alta + modo ficha); `vectra-web/js/admin/product-sheet.js` (módulo nuevo de ficha); `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/product-sheet.test.mjs`; ajuste de `products.test.mjs` (el listado vacío ahora ofrece crear; la tabla sigue sin `<form>` ni edición en línea); `scripts/check-vectra-web.sh` (incluir el test y exigir el módulo)
- **APIs / OpenAPI:** consumo de `POST /api/v1/products` (`ProductCreate`) y lookups activos de `GET /api/v1/categories` y `GET /api/v1/brands` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths ni DTOs
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-02-35`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando del ticket: crear un producto desde la UI y verlo en el listado (contra `vortex serve`)
- **Ticket:** cierra T-02-35
- **Alcance:** cabe en ~2 h (espejo del formulario de marcas + pestaña Datos y POST); no se divide
