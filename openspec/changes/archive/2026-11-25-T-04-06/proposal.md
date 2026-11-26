# Proposal

## Ticket padre

- **Ticket:** [T-04-06](../../../docs/7-tickets.md) — Consulta de recomendaciones
- **Change:** `T-04-06`
- **HU:** [HU-04-05](../../../docs/6-historias.md#hu-04-05--panel-de-recomendaciones), [HU-04-06](../../../docs/6-historias.md#hu-04-06--sustitutos-destacados-sin-stock)
- **Depende de:** T-04-01 (tablas `similar_a` / `sustituible_por` / `complementa`; en `develop` también T-04-03…T-04-05: puerto, persistencia y ABM)
- **Hace:** consulta única de [4.6.2](../../../docs/4-modelo.md#462-consulta-de-recomendaciones) (con `sustituible_por`) más ubicación; `imageUrl` en `null` (imagen principal en v2 con T-02-28); destinos inactivos excluidos; sustitutos con `stock > 0`; `destacar_sustitutos` si el producto tiene stock 0.
- **No hace:** HTTP.
- **Listo cuando:** tests de los escenarios de HU-04-05 y HU-04-06 (incluido el producto dado de baja).

## Why

HU-04-05 y HU-04-06 exigen el panel post-búsqueda: similares, sustitutos (destacados y filtrados por stock) y complementarios, sin destinos dados de baja. El grafo ya se cura por HTTP (T-04-05) y `list_for_product` no filtra `activo` ni `stock`. Sin la consulta 4.6.2, T-04-07 no tiene caso de uso que mapear a `GET /recommendations`.

## What Changes

- Tipos de dominio `Recomendaciones` / `ProductoRecomendado` (nombres en español, alineados a 4.6.2 y al DTO `Recommendations` / `RecommendedProduct` de [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) / `docs/openapi.yaml`)
- `RelationRepo::recommendations_for_product`: una ida a SurrealDB con los tres bloques, `destacar_sustitutos`, stock/precio reales y ubicación formateada; `not_found` si el producto no existe
- Adaptador en `storage` (SQL 4.6.2 + `out.ubicacion` + id de arista); filtros `out.activo = true` y, en sustitutos, `out.stock > 0`; sin cambiar el esquema `0007`
- Caso de uso en `recommend` (`RecommendationService` / `RecommendationGateway` object-safe) que delega en el puerto; sin HTTP ni SurrealDB
- `imageUrl` / imagen principal ausente en el tipo de dominio (siempre `null` en T-04-07; T-02-28 es v2)
- Tests TDD: happy (tres bloques, orden por prioridad, ubicación, HU-04-06) y unhappy (destino inactivo excluido, sustituto sin stock excluido, producto inexistente)

## Non-goals

- Endpoint `GET /api/v1/products/{productId}/recommendations`, métrica `recommendations_duration_seconds`, DTO JSON inglés — **T-04-07** (No hace del ticket)
- UI del panel (`vx-side-panel`, bloques destacados) — **T-04-08**
- UI de relaciones en ficha / carga rápida / `vortex seed` de aristas — **T-04-09**, **T-04-10** (v2), **T-04-11**
- Imagen principal / `imageUrl` hidratado — **T-02-28** (v2; recableo MVP § 7.5.1)
- Latencia p95 < 20 ms — escenario HU-04-05 «Latencia»; [6.3.1](../../../docs/6-historias.md#631-alcance-por-historia) lo deja para *benchmarks* v2 (**T-09-07**)
- Recuperación híbrida RRF — **T-05-11** (v2)
- Reabrir ABM de relaciones, `list_for_product` (sigue incluyendo inactivos y stock 0), esquema `0007`, outbox, Meilisearch, crate `catalog`

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `domain-relacion`: tipos `Recomendaciones` / `ProductoRecomendado`; el puerto gana `recommendations_for_product`
- `vortex-storage`: una consulta Surreal 4.6.2 + ubicación; excluye destinos inactivos y sustitutos sin stock; `not_found` si el producto no existe
- `vortex-recommend`: caso de uso de consulta de recomendaciones (HU-04-05 / HU-04-06) sobre el puerto; fachada object-safe para T-04-07; MUST NOT HTTP

## Impact

- `vortex-core/crates/domain/src/relacion.rs` (tipos + método del puerto; dummy de test)
- `vortex-core/crates/domain/src/search_document.rs` o `producto.rs` (promover el formateo de `ubicacion` ya usado por el documento de búsqueda)
- `vortex-core/crates/storage/src/relacion.rs` + `tests/relacion_repo.rs`
- `vortex-core/crates/recommend/` (`RecommendationService` / `RecommendationGateway`, tests con fake del puerto)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`)
- Ticket: cierra T-04-06 (Listo cuando: tests de HU-04-05 y HU-04-06, incluido destino dado de baja)
- Ámbito: un método de puerto + consulta + servicio delgado; cabe en ~2 h. No se divide el ticket.
