# Design

## Context

En `develop`, T-03-12 dejó `SearchService` + `SearchGateway` (`Arc<dyn …>` listo), `SearchHit` 4.11.1 + `resaltado` (`<mark>`), y traducción de filtros/sort. El crate `api` registra catálogo/stock pero no `/api/v1/search`; `AppState` no tiene búsqueda; allowlist de `api` es `domain`+`catalog` (sin `search`). Timeout global = 5 s (`TimeoutLayer`). T-00-14 reservó `search_duration_seconds` sin emitirlo. OpenAPI de `GET /api/v1/search` existe (200/400/504; sin 503). Tests de `vortex serve` apuntan Meilisearch a `127.0.0.1:1` (caído a propósito). Ver `proposal.md` para motivación.

Constraints: MVP del curso; ~2 h; sin facetas (T-03-18); sin respaldo (T-03-30); Listo cuando = `assert_conforms` + test de cancelación; sin sidecar en tests de `api`.

## Goals / Non-Goals

**Goals:**

- Handler HTTP delgado: query OpenAPI → `SearchGateway::search` → `SearchResponse`
- Timeout de ruta 2 s y Drop al abortar el cliente
- Histograma `search_duration_seconds`
- Fake de `SearchGateway` en tests de `api` (mismo molde que `fake_catalog`)
- Allowlist `api` → `search`; OpenAPI `503`; `vortex serve` inyecta el servicio

**Non-Goals (diseño):**

- Reabrir `SearchService` / `meili_filter` / shape de `SearchHit`
- `serde_qs` / `serde_html_form` (crates.io nuevas)
- `async-trait`
- Distinguir en `vortex serve` «sin servicio» vs «Meilisearch caído» (ambos 503 con la config de test)
- Tests `#[ignore = "meilisearch"]` (T-03-14)

## Decisions

### 1. `api` depende de `search` (path), no se mueve el Gateway

**Elección:** `api` declara `search = { path = "../search" }`. El handler usa `search::SearchGateway`. Se actualizan `openspec/specs/cargo-workspace`, `scripts/check-workspace.sh` (`api_deps_allowed` acepta `search` path) y el fixture `scripts/fixtures/workspace/run-script-tests.sh` (caso happy `api declara path search`).

**Por qué:** T-03-12 puso la fachada en `search` «mismo molde que `ProductGateway`». `api` ya depende de `catalog` por esa razón. Duplicar el trait en `api` o bajarlo a `domain` rompería el hexágono.

**Alternativa:** trait HTTP en `api` que envuelve tipos de `domain`. Se descartó: dos fachadas. **Alternativa:** `api` llama a `MeiliTextSearch` directo. Se descartó: tests de contrato acoplados al sidecar.

### 2. Extractor de query: serde + scan de `attr[…]`

**Elección:** struct `SearchQueryParams` con `q`, `categoryId`, `brandId`, `inStock`, `sort`, `offset`, `limit`, `facets` vía `axum::extract::Query` (serde). Un extractor (o paso posterior sobre `uri().query()`) recorre pares `key=value` y acumula claves `attr[<clave>]` en un `BTreeMap`. Sin crate nueva.

**Por qué:** `serde_urlencoded` no modela *deepObject*. El ticket exige `attr[clave]`. La allowlist de `api` no incluye `serde_qs`.

**Alternativa:** `serde_qs`. Se descartó: crate nueva + pin en workspace + allowlist. **Alternativa:** `HashMap<String, String>` de todo el query. Se descartó: pierde defaults/tipos de `limit`/`offset`.

### 3. Timeout 2 s en el handler, no en el layer global

**Elección:** `tokio::time::timeout(Duration::from_secs(2), gateway.search(...))`. Agotado → `DomainError::timeout()` → `504` (mapeo T-00-11). El `TimeoutLayer` de 5 s permanece para el resto.

**Por qué:** Axum no tiene timeout por ruta en el layer global sin un sub-router. El timeout en el `Future` del handler se cancela si el cliente aborta (se dropea junto con el search). Un test de 504 no necesita esperar 5 s.

**Alternativa:** `.route(..., get(search).layer(TimeoutLayer::2s))`. Equivalente; se prefiere `tokio::time::timeout` para mapear a `problem+json` via `ApiError` sin depender del middleware que traduce 504 genérico (ya existe, pero el handler queda explícito). **Alternativa:** acortar el global a 2 s. Se descartó: rompe ABM (5 s, [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway)).

### 4. Cancelación = Drop del future (test TCP)

**Elección:** fake `SearchGateway` cuyo `search` cuelga (`pending`) y un `Drop` pone un `AtomicBool`. El test abre `TcpStream`, escribe el GET y **cierra sin leer**. Espera el flag. Mismo patrón `http_send` / `TcpListener` que `api/tests/products.rs`.

**Por qué:** el Listo cuando pide «un test de cancelación». HU-03-03 / [5.7](../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación): Axum descarta el future al disconnect. `oneshot` de tower no simula abort de cliente.

**Alternativa:** `reqwest` + `AbortHandle`. Se descartó: no está en dev-deps de `api`. **Alternativa:** `select!` en el handler sobre `axum::extract::ConnectInfo`. Innecesario si el runtime ya dropea.

### 5. Mapeo DTO y números

**Elección:** módulo `api/src/search.rs`. `SearchHit` dominio → JSON OpenAPI: `nombre`→`name`, `tipo`→`productType`, `marca`→`brand`, `codigos`→`codes`, `atributos_texto`→`attributes`, `ubicacion`→`location`, `imagen`→`imageUrl`, `con_stock`→`inStock`, `resaltado`→`highlight`. `precio`/`stock_disponible` (`f64` del índice) → `domain::Decimal` string (`NonNegativeDecimal`); si el parseo falla, `"0"` para no tumbar la página. `unidad_medida` (wire español del documento) → `UnidadMedida::parse_wire` + el mismo `unit_to_api` que productos; desconocido → `"unit"`. `categoryPath` se omite o `[]`. `degraded: false`. `facets: null`. `query` = `q` pedido.

`sort` OpenAPI → `SearchOrden` en el handler (`relevance`→`Relevancia`, etc.); no usar `SearchOrden::try_from_str` (wire español). `remap_api_field`: `clave`→`key` ya existe; `orden` no aplica (el field HTTP es `sort`, construido en el handler). ULID inválido → `categoryId` / `brandId` como en productos.

**Por qué:** 5.3 + schema required. T-03-12 hidrató el hit para que este ticket no reconsulte Surreal.

**Alternativa:** devolver `f64` en JSON. Se descartó: OpenAPI `NonNegativeDecimal` es string; `assert_conforms` falla.

### 6. `facets=true` se ignora; Meilisearch caído es 503

**Elección:** el query `facets` se parsea y se descarta. Respuesta siempre `facets: null`, `degraded: false`. `DomainError::Unavailable` → 503 (no 200 degradado). Se agrega `503` al path en `openapi.yaml`.

**Por qué:** No hace = facetas. T-03-30 es v2. T-09-04 exige schema por estado; sin `503` el unhappy de motor caído no puede `assert_conforms`.

**Alternativa:** 400 si `facets=true`. Se descartó: el parámetro está en el contrato; ignorarlo cumple el no-goal sin pelear con la SPA futura.

### 7. Métrica y tracing

**Elección:** `describe_histogram!("search_duration_seconds", …)` junto al recorder; `histogram!(operation_duration_seconds("search")).record(elapsed)` al completar el handler (200/4xx/5xx). Span del `TraceLayer` global ya cubre la petición; el handler MAY añadir `tracing::debug` con `q` truncada, sin loguear el query completo en `info`.

**Por qué:** el ticket nombra la métrica. T-00-14 ya fijó el nombre. Labels de `q` explotarían cardinalidad.

**Alternativa:** reusar solo `http_request_duration_seconds`. Se descartó: el Hace pide `search_duration_seconds` (p95 de negocio distinto del HTTP genérico).

### 8. Composition root y test de serve

**Elección:** en `serve.rs`, `SearchService::new(text_search.clone(), IndexUid::try_new("productos")?)` — `MeiliTextSearch` ya es `Clone`. `AppState::with_search`. Test binario `vortex_serve_get_search_is_not_404`: con Meilisearch en `:1` el status será 503, no 404. La inyección se verifica además con `with_search` en el source (tarea de cierre). Tests de contrato viven en `api` con fake.

**Por qué:** los tests de serve no levantan Meilisearch; exigir 200 ahí es flaky. 404 vs no-404 prueba que la ruta está en el router de producción.

**Alternativa:** stub in-process de Meilisearch en serve tests. Se descartó: fuera de ~2 h.

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-03-13` y rename a `T-03-13`.

## Risks / Trade-offs

- **[Drop no se observa si hyper bufferiza]** → Mitigación: el test espera el flag con timeout corto (p. ej. 500 ms) y falla explícito si el future sigue vivo; usar `Connection: close` y dropear el `TcpStream`.
- **[f64 → Decimal pierde precisión]** → Mitigación: el índice ya serializa precio/stock del dominio; el formatter de `Decimal` recorta a 4 decimales. T-03-14 no aserta el string decimal.
- **[api → search tira de meilisearch-sdk en el grafo de api]** → Mitigación: aceptable (path dep); `api` no llama al SDK. `check-domain-deps` no aplica a `api`.
- **[~2 h]** → Mitigación: fake HTTP, sin sidecar, sin facetas. Si el parseo *deepObject* se alarga, el mínimo Listo cuando es `q` + un `attr[clave]` + cancelación + `assert_conforms`.

## Migration Plan

Sin migración `.surql`. Despliegue = merge a `develop`. Rollback = revertir handler, `AppState`, allowlist y el `503` de OpenAPI. El índice y `SearchService` no cambian.

## Open Questions

Ninguna que altere spec, enfoque o tasks.
