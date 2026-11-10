# Design

## Context

En `develop`, `api::build_router` registra `GET /health` y fallback 404; la spec vigente **prohíbe** `/metrics`. `crates/api/Cargo.toml` no declara `metrics` ni `metrics-exporter-prometheus` (pins ya en `[workspace.dependencies]`; `check-workspace.sh` las rechaza en `api`). El binario `vortex` reutiliza `build_router` y **no** puede declarar `metrics`. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: hexagonal (`domain` no conoce Prometheus; `vortex` no declara `metrics`); allowlist de `check-workspace.sh`; recorder global del crate `metrics`; ~2 h; MVP del curso; `docs/openapi.yaml` ya define `GET /metrics` como `text/plain`.

## Goals / Non-Goals

**Goals:**

- Recorder Prometheus en `api` + `GET /metrics` 200 `text/plain`
- Histograma `http_request_duration_seconds{route,method,status}` con `MatchedPath`; `unmatched` en 404
- Constantes de nombres reservados; allowlist y fixtures; tests que cierran el Listo cuando

**Non-Goals (diseño):**

- Buckets custom del histograma (defaults del exporter)
- Listener HTTP propio del exporter (feature `http-listener`)
- Reloj inyectable para la duración
- Auth / bind separado para scrape
- Cambiar `docs/openapi.yaml`

## Decisions

### 1. Recorder global en `api`; el binario no declara `metrics`

**Elección:** `metrics-exporter-prometheus` 0.18 (`PrometheusBuilder` + `install_recorder` / handle equivalente) se instala **una vez** (`OnceLock`) en el crate `api`. `GET /metrics` llama a `handle.render()`. `AppState` no lleva el handle. `bin/vortex` no declara `metrics`: el scrape sale por el router. Se **desactiva** (no se usa) cualquier listener HTTP del exporter.

**Por qué:** el crate `metrics` es un recorder de proceso; duplicarlo por `AppState` no aísla tests y pelea con `set_global_recorder`. El ticket nombra `metrics-exporter-prometheus`. Hex: HTTP vive en `api`; la allowlist de `vortex` sigue prohibiendo `metrics` (T-00-07 lo dejó como unhappy).

**Alternativa:** instalar el recorder en `vortex serve`. Se descartó: exigiría `metrics` en el binario y tests de `api` no verían el scrape. **Alternativa:** puerto aparte. Se descartó: OpenAPI pone `GET /metrics` en el mismo servidor.

### 2. Middleware `from_fn` con `MatchedPath`; skip del scrape

**Elección:** layer en `apply_gateway_layers` (o adyacente): guarda `Instant`, ejecuta `next`, lee `MatchedPath` de extensions, método y `status`. Observa `histogram!("http_request_duration_seconds")` en **segundos**. Si no hay `MatchedPath`, `route = "unmatched"`. Si el path es `/metrics` y el método es `GET`, **no** observa (el scrape no se mide a sí mismo). Orden: después de producir la respuesta (hace falta el status), cerca de `TraceLayer`.

**Por qué:** el Hace pide `MatchedPath`; el URI crudo explota cardinalidad (unhappy de la spec). Saltar el scrape evita que el Listo cuando dependa de auto-observación. Reutiliza el patrón de T-00-12 (`from_fn` para `no-store`).

**Alternativa:** `tower-http` metrics. Se descartó: el ticket nombra el crate `metrics`. **Alternativa:** observar también `/metrics`. Se descartó: ensucia el scrape y complica el test de «petición previa».

### 3. Constantes de nombres en `api`, sin emitir series de dominio

**Elección:** módulo `api` (p. ej. `metrics_names`) con `OUTBOX_LAG`, `OUTBOX_FAILED_TOTAL`, `BACKUP_FAILURES_TOTAL` y `fn operation_duration_seconds(op: &str) -> String` (`{op}_duration_seconds`). Tests de igualdad de strings. `GET /metrics` **no** hace `gauge!`/`counter!` de esas series. `domain` no depende de `metrics`.

**Por qué:** el Hace pide la convención ahora; el No hace prohíbe implementar las métricas. T-03-09 / T-04-07 copian los nombres (o reexportan) cuando declaren `metrics` en su crate.

**Alternativa:** constantes en `domain`. Se descartó: Prometheus no es dominio; inflaría la allowlist de `domain`. **Alternativa:** solo documentar en la spec. Se descartó: el config OpenSpec pide tests concretos; un string en código evita drift.

### 4. Allowlist: `metrics` + `metrics-exporter-prometheus` en `api`

**Elección:** `api_deps_allowed` acepta esas dos crates en `[dependencies]` (`workspace = true`). Fixture happy nuevo (`api con allowlist prometheus`). Unhappy `api con clap` y `vortex` con `metrics` no cambian. `domain` / `supervisor` igual.

**Por qué:** sin esto `check-workspace.sh` falla al implementar el Hace. Misma mecánica que T-00-12.

**Alternativa:** declarar las crates en `vortex`. Se descartó: decisión 1.

### 5. Tests: presencia de series, no conteos exactos

**Elección:** `crates/api/tests/metrics.rs` (y unitario de nombres en el crate): `GET /health` luego `GET /metrics` y asertar substrings (`# TYPE http_request_duration_seconds`, `route="/health"`, `_bucket`). 404 con path único → `route="unmatched"` y ausencia del URI. `POST /metrics` no es 200 Prometheus. `vortex serve` pega `/health` + `/metrics`. Los tests **no** asertan `_count` == 1: el recorder es global y `cargo test` es paralelo.

**Por qué:** el Listo cuando es texto válido + histograma de una petición previa. Contar observaciones exactas es frágil.

**Alternativa:** mutex global en tests. Se descartó: ~2 h y no aporta al contrato. **Alternativa:** solo test de binario. Se descartó: T-09-04 reutilizará el test de `api`.

## Risks / Trade-offs

- **[Recorder global acumula series entre tests]** → Mitigación: asserts de presencia/ausencia de labels; no de contadores absolutos.
- **[API de `metrics-exporter-prometheus` 0.18 (`install_recorder` vs handle)]** → Mitigación: usar la API del pin; si hay que clonar el `PrometheusHandle`, se guarda en el `OnceLock`.
- **[CompressionLayer gzip del scrape si el cliente manda `Accept-Encoding`]** → Aceptado: OpenAPI no lo prohíbe; los tests de `/metrics` no envían esa cabecera.
- **[Default `ok`/`disabled` hace que `/health` sea 200]** → El histograma usa `status="200"` en el happy path; el 503 queda cubierto si un test inyecta snapshot `starting` (opcional; no es el Listo cuando).
- **[Hex: `sync` no ve las constantes de `api`]** → Aceptado: T-03-09 declarará `metrics` en `sync` y reutilizará los strings de la spec; no se abre un crate nuevo aquí.

## Migration Plan

1. Red: tests de `/metrics` / histograma / nombres no compilan; `check-workspace.sh` rechaza `metrics` en `api` hasta relajar allowlist.
2. Green: allowlist + recorder + layer + ruta + fixtures.
3. Cierre: § 5 en `vortex-core/`, `scripts/check-workspace.sh`, `openspec validate T-00-14`.
4. Rollback: quitar ruta `/metrics`, layer, deps y fixtures. Sin datos de usuario; `GET /metrics` vuelve a 404 `not_found`.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre exacto del constructor 0.18 (`install_recorder` vs `build_recorder`) es detalle de `Cargo.toml` cubierto por la decisión 1.
