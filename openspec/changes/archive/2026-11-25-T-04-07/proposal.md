# Proposal

## Ticket padre

- **Ticket:** [T-04-07](../../../docs/7-tickets.md) — Endpoint de recomendaciones
- **Change:** `T-04-07`
- **HU:** [HU-04-05](../../../docs/6-historias.md#hu-04-05--panel-de-recomendaciones)
- **Depende de:** T-04-06 (en `develop`: `RecommendationService` + `RecommendationGateway`, consulta 4.6.2, `imageUrl` ausente)
- **Hace:** `GET /api/v1/products/{productId}/recommendations` con métrica `recommendations_duration_seconds`.
- **No hace:** caché.
- **Listo cuando:** respuesta validada contra `openapi.yaml`.

## Why

HU-04-05 exige el panel post-búsqueda (similares, sustitutos, complementarios) vía `GET /api/v1/products/{productId}/recommendations`. T-04-06 ya resuelve la consulta 4.6.2 y deja `RecommendationGateway` listo para inyectar; el router de `api` aún no registra esa ruta (T-04-05 lo dejó explícitamente fuera). Sin el mapper HTTP (DTO inglés, `assert_conforms`, histograma), T-04-08 no puede pintar el `vx-side-panel`.

## What Changes

- Registrar `GET /api/v1/products/{productId}/recommendations` en el router de `api`; `AppState` inyecta `Option<Arc<dyn RecommendationGateway>>` (sin servicio → `503 unavailable`)
- Mapear DTO inglés ↔ dominio según [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) / `docs/openapi.yaml`: `productId`, `highlightSubstitutes`, `similar` / `substitutes` / `complements`; cada ítem `RecommendedProduct` (`relationId`, `id`, `sku`, `name`, `stock`/`price` string, `location`, `imageUrl` siempre `null`, `priority`, `note`)
- Producto inexistente → `404 not_found`; `{productId}` que no parsea como ULID → `400 invalid_input` (`errors[].field` `productId`); producto existente sin aristas → `200` con vectores vacíos
- Histograma Prometheus `recommendations_duration_seconds` (nombre reservado en T-00-14; mismo molde que `search_duration_seconds`)
- `docs/openapi.yaml`: declarar `400` y `503` en `GET .../recommendations` (T-09-04 exige schema por estado; hoy solo hay `200`/`404`)
- Tests de `api` con fake de `RecommendationGateway` + `assert_conforms` (200/400/404/503)
- Composition root: `vortex serve` inyecta `RecommendationService` sobre el `RelacionRepo` ya construido
- Invertir el non-goal de T-04-05: la ruta deja de ser 404 genérico

## Non-goals

- Caché de recomendaciones (HTTP `max-age`, capa in-process, invalidación) — **No hace** del ticket; el `Cache-Control: no-store` de `/api/*` no se toca
- UI del panel (`vx-side-panel`, bloques destacados, foco al Omnibox) — **T-04-08**
- UI de relaciones en ficha / carga rápida / `vortex seed` de aristas — **T-04-09**, **T-04-10** (v2), **T-04-11**
- Imagen principal / `imageUrl` hidratado — **T-02-28** (v2; recableo MVP § 7.5.1; el DTO serializa `null`)
- Latencia p95 < 20 ms — escenario HU-04-05 «Latencia»; [6.3.1](../../../docs/6-historias.md#631-alcance-por-historia) lo deja para *benchmarks* v2 (**T-09-07**)
- Timeout de ruta dedicado (el 2 s de búsqueda no aplica); cancelación al abortar el cliente
- Reabrir consulta 4.6.2, filtros `activo`/`stock > 0`, `RecommendationService`, esquema `0007`, ABM de relaciones
- Recuperación híbrida RRF — **T-05-11** (v2)
- Codegen desde OpenAPI (PAPI-04)

## Capabilities

### New Capabilities

- (ninguna; el endpoint vive en el Gateway, ya cubierto por `vortex-api`)

### Modified Capabilities

- `vortex-api`: registrar `GET /api/v1/products/{productId}/recommendations`; DTO 5.3; `AppState` + `vortex serve` inyectan `RecommendationGateway`; tests `assert_conforms`; OpenAPI declara `400`/`503`; deja de exigir que la ruta sea 404
- `prometheus-metrics`: emitir el histograma `recommendations_duration_seconds` (T-00-14 lo reservó; este ticket lo observa)
- `vortex-recommend`: la fachada `RecommendationGateway` queda consumida por el Gateway HTTP; el crate `recommend` sigue sin HTTP ni DTO JSON

## Impact

- `vortex-core/crates/api/` (handler, DTO, `AppState`, métrica, tests `tests/recommendations.rs`, fake de `RecommendationGateway`)
- `vortex-core/bin/vortex/src/serve.rs` (composition root)
- `docs/openapi.yaml` (`400`/`503` en `GET .../recommendations`)
- `docs/5-contract-api.md` § 5.3 (filas del DTO de recomendaciones si faltan)
- `openspec/specs/vortex-api`, `prometheus-metrics`, `vortex-recommend`
- Sin sidecar ni SurrealDB en tests de `api` (fake del Gateway); sin UI
- Allowlist `api` → `recommend` ya existe (T-04-05); no se toca `cargo-workspace`
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-04-07 (Listo cuando: respuesta validada contra `openapi.yaml`)
- Ámbito: mapper HTTP delgado sobre T-04-06; cabe en ~2 h. No se divide el ticket
