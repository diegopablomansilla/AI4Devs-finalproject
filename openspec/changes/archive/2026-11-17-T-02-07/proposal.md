# Proposal

## Ticket padre

- **Ticket:** [T-02-07](../../../docs/7-tickets.md) — UI de categorías
- **Change:** `T-02-07`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-02-06 (endpoints `/api/v1/categories`, en `develop`); T-01-11 (shell admin con menú y `data-region="admin-content"` vacío, en `develop`)
- **Hace:** árbol en admin con crear subcategoría, renombrar, mover (selector de padre) y dar de baja/reactivar; errores por `code`.
- **No hace:** arrastrar y soltar.
- **Listo cuando:** se ejecutan a mano los escenarios de HU-02-01 excepto el de búsqueda (se valida en T-03-08).

## Why

HU-02-01 pide que el administrador cree, edite, mueva y dé de baja categorías jerárquicas. T-02-06 ya expone `GET`/`POST /api/v1/categories` y `PATCH`/`DELETE /api/v1/categories/{categoryId}`; T-01-11 dejó `#/admin/categorias` con un título placeholder. Sin este change no hay ABM en la SPA: el Listo cuando de la HU (crear subcategoría, ciclo, rename de rama, `duplicate_slug`) no se puede ejecutar a mano.

## What Changes

- Pintar el árbol de categorías en `data-region="admin-content"` cuando el hash es `#/admin/categorias`, armado en el cliente desde la lista plana `{ items }` y `parentId`.
- Acciones por nodo: crear subcategoría, renombrar, mover (selector de padre, no *drag and drop*), dar de baja y reactivar. Alta de raíz para un árbol vacío.
- Errores de API (`category_cycle`, `duplicate_slug`, `conflict`, `invalid_input`, `not_found`, `unavailable`, `network`) mostrados por `code` vía `showError` / `messageFor` (T-01-07); sin interpretar `title`.
- TDD Node (`node --test`, sin jsdom ni npm): happy = árbol, alta, rename, move, baja/reactivación; unhappy = ciclo, slug duplicado, conflicto H-10, red, secciones admin que no son categorías siguen sin ABM.
- `base.css`: clases del árbol con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni arrastrar y soltar.

## Non-goals

- Arrastrar y soltar — **No hace** del ticket
- Reflejo en búsqueda / worker *fan-out* — **T-03-08** (el escenario «la búsqueda refleja el cambio» de HU-02-01)
- ABM de marcas, atributos, productos, stock — **T-02-10**, **T-02-13**, **T-02-34**, **T-02-38**
- Campo de `slug` editable; occupancy real de productos — **T-02-18** (H-10 de productos no rige en HTTP)
- Endpoints nuevos o cambios a `docs/openapi.yaml` — contrato cerrado en **T-02-06**
- Playwright / e2e — **T-09-10** / **T-09-11** (H1: búsqueda y alta de producto)
- jsdom, `package.json`, dependencias npm, job nuevo de CI, crates Rust
- Autenticación o roles (MVP: red local de confianza)

## Capabilities

### New Capabilities

- Ninguna. La UI de categorías es contenido de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: montar el ABM de categorías en `#/admin/categorias` (árbol, selector de padre, baja/reactivación, errores por `code`); exceptuar esa sección del «admin-content vacío» de T-01-11; ampliar la suite Node (happy + unhappy).

## Impact

- **Repo:** `vectra-web/js/admin/categories.js` (módulo nuevo); `vectra-web/js/router.js` y `vectra-web/js/app.js` (montar la sección); `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/categories.test.mjs`; `scripts/check-vectra-web.sh` (incluir el test); ajuste de `router.test.mjs` para que el ABM vacío siga valiendo en secciones que no son `categorias`
- **APIs / OpenAPI:** consumo de `/api/v1/categories` según [docs/openapi.yaml](../../../docs/openapi.yaml); MUST NOT agregar paths ni DTOs
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-02-07`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5). Listo cuando del ticket: ejecución manual de HU-02-01 salvo búsqueda
- **Ticket:** cierra T-02-07
- **Alcance:** cabe en ~2 h (módulo de árbol + 4 acciones HTTP + toasts por `code` + tests Node); no se divide
