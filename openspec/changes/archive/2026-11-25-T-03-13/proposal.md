# Proposal

## Ticket padre

- **Ticket:** [T-03-13](../../../docs/7-tickets.md) — Endpoint `GET /search`
- **Change:** `T-03-13`
- **HU:** [HU-03-03](../../../docs/6-historias.md#hu-03-03--búsqueda-mientras-se-escribe-tolerante-a-errores), [HU-03-04](../../../docs/6-historias.md#hu-03-04--ranking-de-relevancia)
- **Depende de:** T-03-12 (en `develop`: `SearchService` + `SearchGateway`, filtros/sort/`_formatted` → `SearchHit.resaltado`); T-09-04 (`assert_conforms` en `crates/api/tests/support/openapi.rs`)
- **Hace:** `GET /api/v1/search` con parámetros `attr[clave]` (*deepObject*), *timeout* de 2 s y respuesta `SearchResponse` (`highlight` desde `_formatted`); si el cliente corta la conexión se descarta el trabajo; métrica `search_duration_seconds`.
- **No hace:** facetas.
- **Listo cuando:** respuestas validadas contra `openapi.yaml` y un test de cancelación.

## Why

HU-03-03 exige resultados en cada tecla y que una petición abortada no deje trabajo colgado. T-03-12 ya arma la consulta y pide `_formatted`; el router de `api` aún no registra `/api/v1/search`. Sin el mapper HTTP (query OpenAPI → dominio, `SearchPage` → `SearchResponse`, timeout 2 s, cancelación, histograma), T-03-15 no puede llamar al Omnibox y T-03-14 no tiene un endpoint contra el que medir ranking.

## What Changes

- Registrar `GET /api/v1/search` en el router de `api`; `AppState` inyecta `Option<Arc<dyn SearchGateway>>` (sin servicio → `503 unavailable`)
- Parsear query OpenAPI: `q`, `categoryId`, `brandId`, `inStock`, `attr[clave]` (*deepObject*), `sort`, `offset`, `limit`; `facets` se acepta y se ignora (`facets` en la respuesta = `null`)
- Mapear DTO inglés ↔ dominio: `categoryId` → `categoria_ids` (un id; la rama vive en el documento 4.11.1), `brandId` → `marca_id`, `inStock` → `con_stock`, `attr[clave]` → `atributos`, `sort` (`relevance`/`price_asc`/`price_desc`/`stock_desc`) → `SearchOrden`; `highlight` desde `SearchHit.resaltado` (`<mark>` ya pedido al motor)
- Timeout de ruta **2 s** → `504 timeout`; el default del Gateway (5 s) no cambia para el resto
- Si el cliente cierra la conexión, Axum descarta el *future* y el trabajo del Gateway se cancela (Drop); test de cancelación (Listo cuando)
- Histograma Prometheus `search_duration_seconds` (nombre reservado en T-00-14)
- `docs/openapi.yaml`: declarar `503` en `GET /api/v1/search` (mismo patrón que el resto de `/api/v1`; T-09-04 exige schema por estado)
- Tests de `api` con `SearchGateway` falso + `assert_conforms` (200/400/504/503)
- Allowlist: `api` MAY depender de `search` (path), espejo de `catalog`
- Composition root: `vortex serve` inyecta `SearchService` sobre el `MeiliTextSearch` ya construido, uid `productos`

## Non-goals

- Facetas (`facets=true`, `SearchFacets`, distribución `categoria.lvl*`) — **T-03-18** (v2)
- Búsqueda de respaldo en SurrealDB / `degraded: true` — **T-03-30** (v2). Motor caído → `503 unavailable`, no `200` degradado
- Tests de ranking contra Meilisearch real ("tornilo", "Perno", desempate por stock) — **T-03-14**
- UI Omnibox / `AbortController` en el cliente / resaltado seguro / tarjetas — **T-03-15**, **T-03-16**, **T-03-17**
- Sugerencias, búsqueda por código, sinónimos — **T-03-20**, **T-03-22**, **T-03-24**
- Reabrir `SearchService`, traductor Meilisearch, settings de índice o worker — ya cerrados en T-03-12 / T-03-05 / T-03-09
- Acortar el timeout global de 5 s; SSE, voz, recomendaciones
- `serde_qs` u otra crate nueva para *deepObject* (parseo de `attr[clave]` en el extractor, sin ampliar allowlist de crates.io)
- Codegen desde OpenAPI (PAPI-04)

## Capabilities

### New Capabilities

- (ninguna; el endpoint vive en el Gateway, ya cubierto por `vortex-api`)

### Modified Capabilities

- `vortex-api`: registrar `GET /api/v1/search`; DTO 5.3; timeout 2 s; cancelación al drop; `AppState` + `vortex serve` inyectan `SearchGateway`; tests `assert_conforms`; OpenAPI declara `503`
- `prometheus-metrics`: emitir el histograma `search_duration_seconds` (T-00-14 lo reservó; este ticket lo observa)
- `cargo-workspace`: `api` MAY declarar path `search` en `[dependencies]` (junto a `catalog`)

## Impact

- `vortex-core/crates/api/` (handler, DTO, `AppState`, métrica, tests `tests/search.rs`)
- `vortex-core/crates/api/Cargo.toml` + `scripts/check-workspace.sh` + fixture de allowlist (`search` path)
- `vortex-core/bin/vortex/src/serve.rs` (composition root)
- `docs/openapi.yaml` (`503` en `GET /api/v1/search`)
- `openspec/specs/vortex-api`, `prometheus-metrics`, `cargo-workspace`
- Sin sidecar obligatorio en tests de `api` (fake de `SearchGateway`); sin UI
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-03-13 (Listo cuando: `assert_conforms` + test de cancelación)
- Ámbito: mapper HTTP delgado sobre T-03-12; cabe en ~2 h. No se divide el ticket
