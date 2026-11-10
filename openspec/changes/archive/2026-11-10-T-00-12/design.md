# Design

## Context

En `develop`, `api::build_router` es un `Router` con fallback 404 `problem+json` y `AppJson`; no hay layers de `tower-http`. `ApiError` cubre `DomainError`, JSON inválido, 404 y 500 `internal`; **no** emite `payload_too_large`. `check-workspace.sh` **prohíbe** `tower-http` en `api` (T-00-11 lo dejó para este ticket). Axum aplica `DefaultBodyLimit` de 2 MiB. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: defaults de [5.10](../../../docs/5-contract-api.md#510-límites-timeouts-y-comportamiento-del-gateway) (256 KiB / 5 s) en todo el router; sin límites por ruta; ~2 h; sin subscriber global en `vortex serve`.

## Goals / Non-Goals

**Goals:**

- Layers transversales en `build_router` con `tower-http` + `ApiError` 413/504
- `Cache-Control: no-store` solo bajo `/api`; compresión JSON y no SSE; `TraceLayer`
- Allowlist y fixtures alineados; tests happy/unhappy que cierren el Listo cuando

**Non-Goals (diseño):**

- Mapa de límites por path (T-02-30, T-03-13, T-06-04, …)
- `metrics` / `MatchedPath` histogram (T-00-14 reutiliza el span/ruta)
- Parser chunked de cuerpo sin `Content-Length` más allá de lo que dé `RequestBodyLimitLayer`
- Crate `tower` en `[workspace.dependencies]` salvo que el compilador lo exija para `Elapsed` (ver decisión 2)

## Decisions

### 1. Defaults globales en `build_router`, constantes exportadas

**Elección:** `build_router` instala los layers sobre el router completo. Se exportan `DEFAULT_BODY_LIMIT: usize = 256 * 1024` y `DEFAULT_REQUEST_TIMEOUT: Duration = 5s` para que tickets posteriores aniden sub-routers (p. ej. imágenes 10 MiB / 15 s) **fuera** de estas layers sin redefinir magia.

**Por qué:** el ticket pide defaults del Gateway, no una tabla por ruta. T-00-11 ya centraliza HTTP en `build_router`; `vortex serve` lo reutiliza.

**Alternativa:** nest solo `/api` con el límite. Se descartó: uploads y SSE también viven bajo `/api/v1`, así que no evita el techo. **Alternativa:** no poner timeout global y dejarlo a cada handler. Se descartó: el Hace del ticket es el default 5 s ahora.

Extensión (fuera de este change): `Router::new().merge(abm.layer(limit+timeout)).merge(uploads.layer(...)).layer(trace+compression+no-store)`.

### 2. Errores de layer → `ApiError`, no el texto de tower-http

**Elección:** `RequestBodyLimitLayer::new(DEFAULT_BODY_LIMIT)` y `TimeoutLayer::with_status_code` / `HandleErrorLayer` según lo que exponga `tower-http` 0.7. El 413 y el 504 MUST ser `ApiError` (`payload_too_large` / `timeout`) con `instance` = path. Se añade `ApiError::payload_too_large(instance)` y `ApiError::timeout(instance)` en `api` (Gateway-only; **no** pasar por `DomainError`). `title_for_code` gana `payload_too_large` → "El cuerpo es demasiado grande". El timeout de middleware **no** usa el 408 por defecto de tower.

También se baja `axum::extract::DefaultBodyLimit` a 256 KiB para que un extractor futuro no acepte 2 MiB si el layer no cortó.

**Por qué:** T-00-11 dejó `payload_too_large` fuera a propósito; 5.4 y `openapi.yaml` ya definen 413/504 `problem+json`. HU-01-05 necesita `code`, no el body de tower.

**Alternativa:** dejar 408/texto de tower-http. Se descartó: rompe el contrato. **Alternativa:** `tokio::time::timeout` en un `from_fn` sin `TimeoutLayer`. Se descartó: el ticket nombra `tower-http`. Si `Elapsed` exige el crate `tower` como dependencia directa, se declara `tower` con `workspace = true` y pin `=` de la versión que ya trae `tower-http` 0.7; si no hace falta, no se agrega.

### 3. `no-store` por path; compresión con predicado SSE

**Elección:** middleware `from_fn` que escribe `Cache-Control: no-store` si `path == "/api" || path.starts_with("/api/")`. `CompressionLayer` gzip + brotli con predicado que **excluye** `text/event-stream` (y no comprime si no hay `Accept-Encoding`).

**Por qué:** `SetResponseHeaderLayer` es global y T-00-19 necesita caché larga en estáticos. El predicado SSE adelanta `POST /api/v1/ai/ask` sin implementar la ruta.

**Alternativa:** nest `/api` solo para el header. Equivale en efecto; `from_fn` es menos estructura ahora que no hay rutas `/api`. **Alternativa:** comprimir todo. Se descartó: 5.10 prohíbe comprimir SSE.

Orden (afuera → adentro): `TraceLayer` → `CompressionLayer` → `no-store` → handle timeout/limit → router. Trace por fuera para incluir compresión en la duración.

### 4. Sondas solo en tests; tiempo pausado; subscriber de test

**Elección:** igual que `json_rejection.rs`: el test hace `build_router(state).route("/_probe/slow", …).route("/_probe/large", …).route("/_probe/sse", …)`. Producción no registra esas rutas. Timeout: `#[tokio::test(start_paused = true)]` + `advance` para no esperar 5 s de pared (`tokio` test ya activa `time`). Span: `tracing-subscriber` en `[dev-dependencies]` con un `MakeWriter` a buffer; se aserta método, ruta, estado y duración. Cuerpo 413: `POST` con `Content-Length: 262145` (no hace falta sonda). Cuerpo en el límite: `Content-Length: 262144` a ruta inexistente → no 413.

**Por qué:** `cfg(test)` en `lib.rs` no aplica a tests de integración. Listo cuando pide cuerpo, timeout y cabeceras; el span está en el Hace.

**Alternativa:** rutas `/_probe/*` en el router de producción. Se descartó: contaminan el Gateway. **Alternativa:** no testear el span (el Listo cuando no lo nombra). Se descartó: el Hace lo exige y HU-00-09 habla de spans por operación.

### 5. Allowlist: `tower-http` + `tracing`; unhappy sigue siendo `clap`

**Elección:** `api_deps_allowed` acepta `tower-http` y `tracing` en `[dependencies]` (`workspace = true`; features de `tower-http`: `limit`, `timeout`, `compression-gzip`, `compression-br`, `trace`) y `tracing-subscriber` en `[dev-dependencies]`. El fixture happy T-00-11 (allowlist problem+json **sin** tower-http) sigue MAY. Se agrega un happy con la allowlist de middlewares. El unhappy `api con clap` no cambia.

**Por qué:** el ticket nombra `tower-http`; `TraceLayer` y el test del span usan `tracing` / `tracing-subscriber`, ya pineados. Usar `tower-http` como ejemplo de crate **prohibida** dejaría el harness en rojo al implementar.

**Alternativa:** no declarar `tracing` (solo transitiva). Se descartó: un `info_span!` propio o asserts del subscriber lo requieren como directa.

## Risks / Trade-offs

- **[El timeout/límite global tapa uploads 10 MiB y SSE 90 s]** → Mitigación: constantes exportadas + nota para anidar sub-routers en T-02-30 / T-06-06; no se implementa el mapa aquí.
- **[CompressionLayer no comprime cuerpos chicos (predicado por tamaño)]** → Mitigación: sonda `/_probe/large` ≥ 1 KiB JSON; no se exige gzip en el 404 corto.
- **[TimeoutLayer 408 por defecto]** → Mitigación: mapear siempre a `ApiError::timeout`; test `slow_handler_returns_504_timeout` falla si queda 408.
- **[Tests de timeout lentos]** → Mitigación: reloj pausado de Tokio; si una layer no respeta `tokio::time`, caer a un timeout de test corto documentado — no cambiar el default de producción.
- **[Fixture `api con allowlist problem+json` sigue verde sin tower-http]** → Aceptado: MAY; el Cargo.toml real de `api` sí las declara.

## Migration Plan

1. Red: tests de 413 / 504 / cabeceras / span no compilan o fallan; `check-workspace.sh` rechaza `tower-http` hasta relajar allowlist.
2. Green: allowlist + layers + `ApiError` 413/504 + fixtures.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-12`.
4. Rollback: quitar layers, constructores, deps y fixtures. Sin datos de usuario; el Gateway vuelve a 404 sin tope de cuerpo.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El pin extra de `tower` (si aparece) es detalle de `Cargo.toml` cubierto por la decisión 2.
