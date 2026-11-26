# Design

## Context

En `develop`, T-04-06 dejó `RecommendationService` + `RecommendationGateway` (`Arc<dyn …>` listo) y tipos `Recomendaciones` / `ProductoRecomendado`. T-04-05 registró el ABM HTTP de relaciones y dejó `GET .../recommendations` fuera del router. `AppState` tiene `with_relations` pero no recomendaciones. T-00-14 reservó `recommendations_duration_seconds` sin emitirlo. OpenAPI de `GET .../recommendations` existe (200/404; sin 400/503). `api` ya depende de `recommend` (path). Ver `proposal.md` para motivación.

Constraints: MVP del curso; ~2 h; sin caché (No hace); Listo cuando = `assert_conforms`; sin sidecar ni SurrealDB en tests de `api`; latencia p95 es T-09-07.

## Goals / Non-Goals

**Goals:**

- Handler HTTP delgado: `productId` → `RecommendationGateway::get` → `Recommendations`
- Histograma `recommendations_duration_seconds`
- Fake de `RecommendationGateway` en tests de `api`
- OpenAPI `400`/`503`; `vortex serve` inyecta el servicio

**Non-Goals (diseño):**

- Reabrir `RecommendationService` / SQL 4.6.2 / filtros `activo`/`stock`
- Timeout de ruta distinto de 5 s; test de abort de cliente
- Caché in-process o `Cache-Control: max-age`
- `async-trait`; crates.io nuevas
- Tests `#[ignore]` contra SurrealDB real
- Unificar `RelationGateway` y `RecommendationGateway` en un solo trait

## Decisions

### 1. `AppState` gana `recommendations` aparte de `relations`

**Elección:** `Option<Arc<dyn RecommendationGateway>>` con `with_recommendations`. El handler de recomendaciones no llama a `RelationGateway`. En `serve.rs`, ambos servicios se construyen sobre el mismo `RelacionRepo` (`Arc<Db>` ya se clona).

**Por qué:** T-04-06 diseñó la fachada object-safe para este ticket. Mezclar ABM y consulta 4.6.2 en un trait hincharía el fake de relaciones y acoplaría 404 de listado admin con el panel.

**Alternativa:** añadir `get_recommendations` a `RelationGateway`. Se descartó: dos casos de uso, dos fachadas. **Alternativa:** el handler llama a `storage::RelacionRepo` directo. Se descartó: tests de contrato acoplados a SurrealDB.

### 2. Módulo `api/src/recommendations.rs`, no extender `relations.rs`

**Elección:** handler y DTO en un módulo propio; ruta `GET /api/v1/products/{product_id}/recommendations` junto a las de relaciones en `router_shell`. Tests en `crates/api/tests/recommendations.rs`. Fake en `tests/support/fake_recommendations.rs`.

**Por qué:** el ABM admin (`/relations`) incluye inactivos y stock 0; el panel no. Un solo archivo mezclaría DTO `Relation` y `RecommendedProduct`.

**Alternativa:** colgar el GET del mismo `relations.rs`. Se descartó: el archivo ya cubre POST/PATCH/DELETE/list.

### 3. Fake canned sobre `RecommendationGateway`

**Elección:** fake que implementa `RecommendationGateway` con un mapa `Id<Producto>` → `Recomendaciones` (más modos `NotFound` / `Unavailable`). Los tests HTTP no re-ejecutan SQL 4.6.2.

**Por qué:** T-04-06 ya cubre filtros y orden en `storage`/`recommend`. Este ticket valida mapper + status + OpenAPI. Un fake canned hace determinista `highlightSubstitutes` e `imageUrl`.

**Alternativa:** reusar `InMemoryRelationRepo` de `fake_relations.rs` + `RecommendationService`. Viable; se prefiere canned para no arrastrar el ABM al panel. Si el fake canned se alarga, MAY delegar en el servicio in-memory.

### 4. OpenAPI 400 y 503

**Elección:** agregar `'400'` y `'503'` al path (mismo patrón que relaciones y `GET /search`). T-09-04 exige schema por estado.

**Por qué:** ULID inválido ya es 400 en `{productId}` de relaciones; sin servicio es 503 en el resto de `/api/v1`.

**Alternativa:** dejar solo 200/404 y no devolver 400/503. Se descartó: rompería el molde del Gateway y `assert_conforms`.

### 5. Mapeo DTO y números

**Elección:** `Recomendaciones` → JSON OpenAPI: `producto_id`→`productId`, `destacar_sustitutos`→`highlightSubstitutes`, `similares`→`similar`, `sustitutos`→`substitutes`, `complementarios`→`complements`. Ítem: `relacion_id`→`relationId`, `nombre`→`name`, `precio`/`stock` (`Decimal`) → string, `ubicacion`→`location`, `prioridad`→`priority`, `nota`→`note`, `imageUrl: null`. Completar [5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos) con esas filas (`location` del panel es string formateado, no el objeto `location.aisle` de la ficha).

**Por qué:** 5.3 + schema required. T-04-06 hidrató stock/precio/ubicación para que este ticket no reconstruya el grafo.

**Alternativa:** devolver `Decimal` como número JSON. Se descartó: `NonNegativeDecimal` es string; `assert_conforms` falla.

### 6. Métrica y tracing

**Elección:** `describe_histogram!(operation_duration_seconds("recommendations"), …)` junto al recorder; `histogram!(...).record(elapsed)` al completar el handler (200/4xx/5xx), mismo molde que search. Sin labels. Span del `TraceLayer` global cubre la petición; el handler MAY `tracing::debug` con el ULID.

**Por qué:** el ticket nombra la métrica. Labels de `productId` explotarían cardinalidad.

**Alternativa:** reusar solo `http_request_duration_seconds`. Se descartó: el Hace pide `recommendations_duration_seconds`.

### 7. Composition root y test de serve

**Elección:** `RecommendationService::new(RelacionRepo::new(Arc::clone(&db)))` y `AppState::with_recommendations`. Test `vortex_serve_get_recommendations_is_not_404`: ULID válido inexistente → 404 (ruta registrada), nunca 404 de fallback `not_found` de router… espera: producto inexistente ES 404 de negocio. Distinguir: el fallback de ruta usa `instance` del path crudo; el 404 de dominio es `not_found` de producto. Mejor: ULID bien formado → 404 `not_found` de producto (no 404 de ruta no registrada si el body/`instance` coinciden). Como T-03-13: **no 404 de unmatched**; 404 de dominio o 503 aceptable. Aserción: status ≠ unmatched fallback; `code` es `not_found` (producto) o la ruta respondió 200. Con seed vacío, 404 de producto prueba que la ruta está cableada.

**Por qué:** los tests de serve no cargan el dataset de relaciones. 404 de producto ≠ 404 de router.

**Alternativa:** exigir 200 en serve. Se descartó: flaky sin seed.

### 8. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-04-07` y rename a `T-04-07`.

## Risks / Trade-offs

- **[Recorder Prometheus global]** → Mitigación: el unhappy «sin consultas no exige la serie» no aserta ausencia si otro test del proceso ya observó una muestra (igual que search).
- **[404 de producto vs 404 de router en serve]** → Mitigación: asertar `application/problem+json` con `code` `not_found` **y** que el path coincida con `/recommendations` (no fallback genérico de ruta desconocida si se puede distinguir); nunca 404 con body de «ruta no registrada» para un ULID válido.
- **[~2 h]** → Mitigación: fake canned, sin Surreal, sin UI, sin timeout extra. Listo cuando mínimo = 200 conforme + 404 + histograma.

## Migration Plan

Sin migración `.surql`. Despliegue = merge a `develop`. Rollback = revertir handler, `AppState`, OpenAPI 400/503 y la métrica. El puerto y `RecommendationService` no cambian.

## Open Questions

Ninguna que altere spec, enfoque o tasks.
