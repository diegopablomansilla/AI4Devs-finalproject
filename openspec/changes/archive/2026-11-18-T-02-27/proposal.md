# Proposal

## Ticket padre

- **Ticket:** [T-02-27](../../../docs/7-tickets.md) — Endpoint `GET /products` (listado admin)
- **Change:** `T-02-27`
- **HU:** [HU-02-10](../../../docs/6-historias.md#hu-02-10--listado-administrativo-de-productos)
- **Depende de:** T-02-21 (HTTP de ficha en `develop`), T-00-21 (helper de cursor opaco en `develop`)
- **Hace:** filtros `categoryId` (incluye la rama vía `categoria.ancestros`), `brandId` y `active`; orden por ID; respuesta `ProductSummaryPage`.
- **No hace:** búsqueda por texto.
- **Listo cuando:** tests de filtro por rama y por inactivos.

## Why

HU-02-10 exige listar el catálogo por rama de categoría, marca y estado (activos / dados de baja) para revisar y corregir productos por partes. T-02-21 dejó `POST`/`GET`/`PATCH`/`DELETE` de ficha y **prohibió** registrar `GET /api/v1/products`; T-00-21 dejó el helper de cursor. Sin el listado paginado, T-02-34 no tiene API que consumir.

## What Changes

- `GET /api/v1/products` → 200 `ProductSummaryPage` (`items` + `nextCursor`), ordenado por ID (ULID, orden de alta)
- Query: `categoryId` (categoría **y** descendientes vía `categoria.ancestros`, [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol)), `brandId`, `active` (default `true` = solo activos; `false` = solo baja), más `cursor`/`limit` del helper T-00-21
- DTO inglés `ProductSummary` ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id`, `sku`, `name`, `productType`, `brand` (nombre o `null`), `categoryPath`, `price`/`stock` string, `active`; `imageUrl` siempre `null` (T-02-28/T-02-30 son v2)
- Puerto `ProductRepo::list` + caso de uso en `catalog` + adaptador SurrealDB; el Gateway pagina con `paginate` (`limit+1` → `nextCursor`)
- Tests: filtro por rama (HU-02-10), filtro por inactivos, paginación, `invalid_input` de cursor/`limit`/ULID; `assert_conforms`
- `docs/openapi.yaml`: declarar `503` en el GET de listado (mismo patrón que ficha); MUST NOT agregar `q` ni otros params de búsqueda
- MUST NOT implementar `GET /search` ni ranking; MUST NOT hidratar imágenes

## Non-goals

Este change **no** cubre:

- Búsqueda por texto / facetas / sugerencias — **T-03-13** / **T-03-18** / **T-03-20** (No hace del ticket)
- UI listado de productos / "Cargar más" / baja desde la tabla — **T-02-34**
- Imágenes / `imageUrl` real — **T-02-28** … **T-02-32** (v2)
- Historial paginado `GET .../stock-movements` — **T-02-26** (v2)
- Reindexación Meilisearch al listar — **T-03-07**
- Cambiar el helper de cursor (ya está en T-00-21)
- Codegen OpenAPI, e2e Playwright, sidecars

## Capabilities

### New Capabilities

- (ninguna — el listado extiende producto / API / storage ya especificados)

### Modified Capabilities

- `domain-producto`: el puerto `ProductRepo` declara `list` (filtros categoría-rama / marca / activo, orden por ID, `after` + `take`); tipo `ResumenProducto` / `FiltroProducto`
- `catalog-producto`: caso de uso `list` + `ProductGateway::list`; sin HTTP ni SurrealDB
- `vortex-storage`: adaptador `list` con `categoria = $c OR categoria.ancestros CONTAINS $c` ([4.4.1](../../../docs/4-modelo.md#441-categoria--árbol))
- `vortex-api`: registrar `GET /api/v1/products`; DTO `ProductSummaryPage`; cursor/`limit` vía helper; tests HU-02-10; deja de prohibir el listado

## Impact

- **Repo:** `vortex-core/crates/domain/` (`producto.rs`: `list`, `FiltroProducto`, `ResumenProducto`); `catalog/` (`ProductService::list`, `ProductGateway`); `storage/` (`ProductoRepo::list` + tests de rama); `api/` (`products.rs` GET colección, `lib.rs` route, `tests/products.rs`); fakes en `api/tests/support/` y dummies de `catalog`
- **APIs / OpenAPI:** implementar `GET /api/v1/products` ya contratado; agregar `'503'` al GET de colección; MUST NOT tocar `GET /search` ni esquemas de `Product` ficha
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `openspec validate T-02-27`
- **Ticket:** cierra T-02-27 (Listo cuando: tests de filtro por rama y por inactivos)
- **Ámbito:** HTTP + `list` en puerto/caso de uso/adaptador; cabe en ~2 h. No se divide el ticket
- Sin **BREAKING**: el GET de colección hoy no está registrado (405/fallback); este change lo habilita
