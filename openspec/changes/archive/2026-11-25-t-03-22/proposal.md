# Proposal

## Ticket padre

- **Ticket:** [T-03-22](../../../docs/7-tickets.md) — Búsqueda por código en SurrealDB
- **Change:** `t-03-22` (rama Git `T-03-22`)
- **HU:** [HU-03-08](../../../docs/6-historias.md#hu-03-08--búsqueda-por-código)
- **Depende de:** T-02-19 (en `develop`: `ProductRepo`, índices `uq_producto_sku` e `idx_producto_codigos`)
- **Hace:** `GET /api/v1/search/by-code/{code}` con coincidencia exacta sobre `sku` y `codigos[*].valor`; solo productos activos.
- **No hace:** coincidencia parcial.
- **Listo cuando:** tests: OEM compartido devuelve ambas piezas; `"HX-0841Z"` no devuelve `"HX-0840Z"`.

## Why

HU-03-08 exige encontrar repuestos por SKU u OEM con **exactitud** (H-05). Meilisearch tolera prefijos y typos en atributos de código; la fuente de verdad para `by-code` es SurrealDB. Sin endpoint ni consulta en el repositorio, T-03-23 no puede fijar coincidencias exactas arriba en el Omnibox.

## What Changes

- **BREAKING** (puerto): `ProductRepo::find_by_code` + `ProductGateway::find_by_code`
- Adaptador `storage`: consulta exacta `sku = $code` o valor en `codigos`, `activo = true`, orden estable
- Caso de uso `catalog::ProductService::find_by_code`
- `GET /api/v1/search/by-code/{code}` en `api`, DTO `{ items: ProductSummary[] }`, validación `code` (1..=64), `assert_conforms`
- `docs/openapi.yaml`: respuestas `400` y `503` en la ruta (T-09-04)
- Levantar `MUST NOT implementar by-code` en spec consolidada `vortex-api`
- Tests storage, catalog (dummy), api (happy + unhappy, incl. sin `SearchGateway`)

## Non-goals

- UI Omnibox / etiqueta «Código exacto» — **T-03-23**
- Prefijo, typo, Meilisearch — **No hace**
- Búsqueda de texto degradada — **T-03-30** (v2)
- Lector de barras — T-03-23 **No hace**
- Facetas, sugerencias, sinónimos, `vectra-web`
- Timeout 2 s / histograma `search_duration_seconds` (pertenecen a `/search`)

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `domain-producto`: puerto `ProductRepo::find_by_code`
- `vortex-storage`: implementación SurrealDB de búsqueda exacta por código
- `catalog-producto`: caso de uso y `ProductGateway::find_by_code`
- `vortex-api`: endpoint `GET /api/v1/search/by-code/{code}`

## Impact

- `vortex-core/crates/domain/src/producto.rs`
- `vortex-core/crates/storage/src/producto.rs`, `tests/producto.rs`
- `vortex-core/crates/catalog/src/producto.rs`
- `vortex-core/crates/api/src/lib.rs`, `search.rs` o módulo dedicado, `tests/search.rs`, `tests/support/fake_catalog.rs`
- `docs/openapi.yaml`
- `openspec/specs/domain-producto`, `vortex-storage`, `catalog-producto`, `vortex-api`