# Proposal

## Ticket padre

- **Ticket:** [T-00-21](../../../docs/7-tickets.md) — Paginación por cursor
- **Change:** `T-00-21`
- **HU:** [HU-02-08](../../../docs/6-historias.md#hu-02-08--historial-de-movimientos), [HU-02-10](../../../docs/6-historias.md#hu-02-10--listado-administrativo-de-productos)
- **Depende de:** T-00-11 (`ApiError` / `invalid_input` en `develop`)
- **Hace:** helper en `api` con cursor opaco (último ULID visto, en base64url), `limit` ≤ 100 y `nextCursor = null` al final; cursor inválido → `invalid_input`
- **No hace:** paginación por *offset* (la maneja la búsqueda)
- **Listo cuando:** tests unitarios de codificación y límites

## Why

Los listados de administración (`GET /products` en T-02-27, historial de stock en T-02-26 v2) necesitan paginación por cursor opaco según [5.6](../../../docs/5-contract-api.md#56-paginación) y `docs/openapi.yaml` (`Cursor`, `Limit` ≤ 100, `nextCursor`). Hoy el crate `api` no tiene helper de cursor ni validación de `limit`. Sin esto, cada endpoint reimplementaría encode/decode y límites, y T-02-27 no puede arrancar.

## What Changes

- Helper en `crates/api` que codifica el último ULID visto como cursor **opaco** en **base64url** (sin padding) y lo decodifica de vuelta a un ULID canónico de 26 caracteres
- `limit` ausente → 50 (default OpenAPI); `limit` en 1..=100 se acepta; fuera de rango → `invalid_input`
- Armado de página: si hay más ítems que `limit`, `nextCursor` es el cursor del último ítem devuelto; si no, `nextCursor = null`
- Cursor inválido (no base64url, no ULID, vacío) → `invalid_input` con `errors[].field` `cursor`
- TDD: happy = roundtrip ULID↔cursor, `limit` default/máximo, última página con `nextCursor` nulo; unhappy = cursor inválido, `limit` 0 y 101
- MUST NOT registrar `GET /products` ni `GET .../stock-movements`; MUST NOT implementar offset (`offset` / `estimatedTotalHits` de búsqueda)

## Non-goals

Este change **no** cubre:

- Endpoint `GET /api/v1/products` (listado admin) — **T-02-27** (MVP; consume este helper)
- Endpoint `GET .../stock-movements` (historial) — **T-02-26** (v2; No hace del alcance MVP de HU-02-08 en este ticket)
- UI listado de productos / "Cargar más" — **T-02-34**
- Paginación por *offset* de `GET /search` (`offset` ≤ 1000, `limit` ≤ 50, `estimatedTotalHits`) — **T-03-13** (No hace del ticket)
- Extraer cursores de outbox / `sync_cursor` — **T-03-01** / **T-03-07** (otro tipo de cursor)
- Cambiar `docs/openapi.yaml` (`Cursor`, `Limit`, `ProductSummaryPage.nextCursor` ya existen)
- Nueva crate en la allowlist de `api` (`base64` u otra) — se codifica en el módulo
- Codegen OpenAPI, e2e Playwright, sidecars

## Capabilities

### New Capabilities

- (ninguna — el helper vive en el Gateway HTTP ya especificado)

### Modified Capabilities

- `vortex-api`: el crate `api` expone un helper de paginación por cursor opaco (ULID en base64url), `limit` ≤ 100 con default 50, `nextCursor = null` al final de la colección, y mapeo de cursor/`limit` inválidos a `invalid_input`

## Impact

- **Repo:** `vortex-core/crates/api/` (`src/pagination.rs` nuevo, `src/lib.rs` módulo + reexport, tests unitarios del helper; MAY `src/error.rs` con constructores `invalid_cursor` / `invalid_limit`)
- **APIs / OpenAPI:** sin paths nuevos; el helper se alinea a parámetros `Cursor`/`Limit` y a `nextCursor` ya definidos; MUST NOT tocar `GET /search`
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `openspec validate T-00-21`
- **Ticket:** cierra T-00-21 (Listo cuando: tests unitarios de codificación y límites)
- **Ámbito:** helper + tests unitarios; cabe en ~2 h. No se divide el ticket
- Sin **BREAKING**: no hay listados paginados en producción todavía
