# Proposal

## Ticket padre

- **Ticket:** [T-02-13](../../../docs/7-tickets.md) — UI de atributos técnicos
- **Change:** `T-02-13`
- **HU:** [HU-02-03](../../../docs/6-historias.md#hu-02-03--definición-de-atributos-técnicos)
- **Depende de:** T-02-12 (endpoints `/api/v1/attribute-definitions`, en `develop`); T-01-11 (shell admin con menú `#/admin/atributos` y `data-region="admin-content"`, en `develop`)
- **Hace:** listado ordenado por `order`; formulario con tipo, unidad, opciones (visible solo para `enum`), `facetable` y `searchable`.
- **No hace:** vista previa de facetas.
- **Listo cuando:** se crea "grado" (enum 2/5/8, facetable) desde la UI.

## Why

HU-02-03 pide que el administrador defina atributos técnicos tipados (medida, rosca, grado) sin cambiar software. T-02-12 ya expone `GET`/`POST /api/v1/attribute-definitions` y `PATCH`/`DELETE …/{attributeId}`; T-01-11 dejó `#/admin/atributos` con un título placeholder. Sin este change no hay ABM en la SPA: el Listo cuando (crear "grado" enum 2/5/8 facetable) no se puede ejecutar a mano.

## What Changes

- Pintar el listado de definiciones en `data-region="admin-content"` cuando el hash es `#/admin/atributos`, hidratado desde `{ items }` de `GET /api/v1/attribute-definitions?includeInactive=true`, **en el orden que devuelve la API** (`order` ascendente, empate por `key`).
- Formulario de alta/edición con `key`, `label`, `type` (`text` / `number` / `boolean` / `enum`), `unit`, `options` (visible **solo** si `type = enum`), `facetable`, `searchable` y `order`. Acciones: alta (`POST`), edición (`PATCH` de esos campos), baja (`DELETE`) y reactivación (`PATCH { active: true }`). Listado vacío ofrece crear.
- Errores de API (`invalid_input`, `conflict`, `not_found`, `unavailable`, `network`) mostrados por `code` vía `showError` / `messageFor` (T-01-07); sin interpretar `title`.
- TDD Node (`node --test`, sin jsdom ni npm): happy = listado por `order`, alta "grado" enum 2/5/8 facetable, opciones visibles solo en enum, edición, baja/reactivación; unhappy = enum sin opciones → `invalid_input`, clave duplicada → `conflict`, red, key/label vacíos, sin vista previa de facetas, secciones admin que no son categorías/marcas/atributos siguen sin ABM.
- `base.css`: clases del listado con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni UI de vista previa de facetas.

## Non-goals

- Vista previa de facetas (chips de filtro, mock de búsqueda, "así se verá en facetas") — **No hace** del ticket
- Escenario HU-02-03 «aparece como filtro en la búsqueda» — **T-03-18** / **T-03-19** (facetas; v2)
- Regla `attribute_in_use` al cambiar `key`/`type` — **T-02-23** (la UI muestra el `code` si llega; no consulta occupancy ni bloquea la edición)
- Asignar atributos a productos en la ficha — **T-02-36**
- Seed de atributos — **T-02-42**
- Endpoints nuevos o cambios a `docs/openapi.yaml` — contrato cerrado en **T-02-12**
- Playwright / e2e — **T-09-10** / **T-09-11**
- jsdom, `package.json`, dependencias npm, job nuevo de CI, crates Rust
- Autenticación o roles (MVP: red local de confianza)
- Extraer un *framework* de vistas admin (copia del patrón de T-02-10, no un router genérico)

## Capabilities

### New Capabilities

- Ninguna. La UI de atributos es contenido de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: montar el ABM de definiciones de atributo en `#/admin/atributos` (listado por `order`, formulario tipado con opciones solo para `enum`, `facetable`/`searchable`, baja/reactivación, errores por `code`, sin vista previa de facetas); exceptuar esa sección del «admin-content vacío» de T-01-11 (junto a `categorias` y `marcas`); ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/admin/attribute-definitions.js` (módulo nuevo); `vectra-web/js/router.js` y `vectra-web/js/app.js` (montar la sección); `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/attribute-definitions.test.mjs`; `scripts/check-vectra-web.sh` (incluir el test); ajuste de `router.test.mjs` para que el ABM vacío siga valiendo en secciones que no son `categorias`, `marcas` ni `atributos`
- **APIs / OpenAPI:** consumo de `/api/v1/attribute-definitions` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths ni DTOs
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-02-13`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando del ticket: crear "grado" (enum 2/5/8, facetable) a mano contra `vortex serve`
- **Ticket:** cierra T-02-13
- **Alcance:** cabe en ~2 h (espejo de T-02-10 + formulario tipado); no se divide
