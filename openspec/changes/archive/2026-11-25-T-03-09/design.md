# Design

## Context

En `develop` (tras T-03-08): `MeiliProductWorker::drain_once` hace `process_row?` + `advance` por evento; cualquier `DomainError` (incluido `invalid_input`) aborta el lote sin cursor ni `outbox_fallo`. `run_while` ignora el error y vuelve a `select!` notifier / 1 s. `OutboxRepo` solo lista/avanza el cursor `meilisearch`; la tabla `outbox_fallo` existe (T-03-01) y no se escribe. `GET /health` serializa `components.sync.status` sin `outboxLag`/`failedEvents`. Constantes Prometheus `OUTBOX_LAG` / `OUTBOX_FAILED_TOTAL` viven en `api` y **no** se emiten. Allowlist de `sync`: `domain` + `storage` + `tokio` (sin `metrics`). Worker **no** está en `bin/vortex` (T-00-27). Ver `proposal.md` para motivación.

Constraints: reproceso manual = No hace; purga = T-03-10; vectores = T-05-08; sin `openapi.yaml`; MVP del curso; ~2 h. CLI OpenSpec kebab-case (`t-03-09` → carpeta `T-03-09`).

## Goals / Non-Goals

**Goals:**

- Clasificar transitorio vs permanente en el worker existente
- *Backoff* puro (función + loop), tests sin sleep de 30 s
- Extender `OutboxRepo` + adaptador (sin migración nueva)
- Snapshot / JSON de `/health` con extras; telemetría Prometheus en `api`

**Non-Goals (diseño):**

- Añadir `metrics` a la allowlist de `sync`
- `dyn TextSearch` / `async-trait` en repos
- Wiring del loop en `vortex serve`
- Filas `reintentando`
- Incluir `vectores` en lag (el cursor existe y nunca avanza: inflaría `/health` para siempre)

## Decisions

### 1. Transitorio = cursor quieto; permanente = fallo + avance + seguir el lote

**Elección:** `unavailable`/`timeout` → `return Err` desde `drain_once` (sin avance, sin `outbox_fallo`). `invalid_input` → `record_fallido` + `advance` + siguiente evento del lote (`drain_once` sigue `Ok` si el resto sale bien). Otros códigos (p. ej. `not_found` de Storage al avanzar) no se clasifican como permanentes.

**Por qué:** [4.8.2](../../../docs/4-modelo.md#482-sync_cursor-y-outbox_fallo) y HU-03-01. Atascar el cursor en un documento rechazado contradice el Listo cuando de error permanente.

**Alternativa:** abortar el lote también en `invalid_input`. Se descartó: un evento podrido bloquearía los posteriores aunque el cursor avanzara.

### 2. Backoff `min(30s, 1s × 2^n)` y notifier ignorado durante la espera

**Elección:** `n` = fallos transitorios consecutivos (`n = 0` → 1 s). Tope 30 s. Función pura `transient_backoff(n) -> Duration` para el test del tope (sin `sleep(30s)`). En `run_while`, si el último `drain_once` fue transitorio, esperar **solo** ese delay (sin `select!` con notifier). `Ok` reinicia `n` y vuelve a notifier + 1 s.

**Por qué:** el ticket pide tope 30 s; despertar por notifier con Meilisearch caído martillaría el sidecar.

**Alternativa:** jitter. Se pospuso: un nodo, un worker; no aporta al Listo cuando.

### 3. No escribir `reintentando`

**Elección:** transitorio = solo cursor + backoff. `outbox_fallo` solo `fallido`.

**Por qué:** el ticket nombra fila `fallido` para `InvalidInput`; el schema admite `reintentando` para un futuro, no para este alcance.

**Alternativa:** upsert `reintentando` en cada timeout. Se descartó: más I/O y no hay lector MVP.

### 4. Puerto `SyncTelemetry` en `domain`; impl Prometheus en `api`

**Elección:** trait síncrono (`set_outbox_lag(consumidor, lag)`, `inc_outbox_failed()`, `observe_sync_propagation(seconds)`). Worker genérico o `Arc<dyn SyncTelemetry>`. `Noop` / recording en tests de `sync`. `api` implementa con el crate `metrics` (recorder ya instalado). `sync` **no** declara `metrics`.

**Por qué:** allowlist de `sync` intacta; `api` no depende de `sync`. T-00-27 inyecta la impl al worker.

**Alternativa:** `metrics` en `sync`. Se descartó: delta `cargo-workspace` y acople al recorder.

### 5. Lag y fallidos: solo consumidor `meilisearch`

**Elección:** `outbox_lag{consumidor="meilisearch"}` = count `outbox.id > cursor meilisearch`. `failedEvents` = count `outbox_fallo` meilisearch/`fallido`. **No** `max(meilisearch, vectores)`.

**Por qué:** OpenAPI dice “consumidor más atrasado”, pero `sync_cursor:vectores` está en origen y T-05-08 es v2: incluirlo haría `outboxLag` crecer con cada alta de catálogo.

**Alternativa:** lag = max de ambos cursores. Se descartó para el MVP del curso.

### 6. `/health` extras desde el snapshot, no un probe en el handler

**Elección:** `ComponentHealth` de `sync` gana `outbox_lag: u64` y `failed_events: u64` (default 0). `HealthRegistry` los publica. `GET /health` los serializa (camelCase). El worker actualiza `SyncTelemetry` (Prometheus) y, cuando T-00-27 cablee el mismo `Arc` de stats al registry, `/health` dejará de ser 0. En este ticket los tests de API inyectan un snapshot con valores.

**Por qué:** T-00-13: `HealthReport` no sondea sidecars ni abre Storage. `api` no tiene `OutboxRepo` (RPITIT, no `dyn`). T-00-27 es el composition root.

**Alternativa:** `GET /health` consulta Surreal. Se descartó: rompe el puerto y la allowlist de `api`.

### 7. `creado_en` en `OutboxRow` para el histograma

**Elección:** ampliar el SELECT del adaptador; `OutboxEvent` no cambia. Observar `sync_propagation_seconds` solo al avanzar por **éxito** (no al marcar `fallido`).

**Por qué:** el ticket pide histograma de propagación; el outbox ya tiene `creado_en` READONLY.

**Alternativa:** `Instant` desde que el worker vio el evento. Se descartó: no mide commit → índice.

### 8. Upsert de `outbox_fallo` por `(evento, consumidor)`

**Elección:** `CREATE ...` o `UPDATE` según exista; unique index T-03-01. Registrar sobre id ausente → `not_found`.

**Por qué:** reintentar un permanente (p. ej. fallo de `advance` a mitad) no debe duplicar filas.

### 9. Listo cuando sin sidecar

**Elección:** `worker_retries_after_unavailable_and_does_not_lose_events`: `TextSearch` falso que falla `unavailable` una vez y luego upserta; dos `drain_once`. `worker_records_fallido_and_advances_on_invalid_input`: fake `invalid_input`, assert fallo + cursor. El ignore Meilisearch de T-03-07/08 no se toca.

**Por qué:** ~2 h; HU-03-01 no exige tumbar el proceso Meilisearch.

## Risks / Trade-offs

- [`/health` en `vortex serve` queda `outboxLag: 0` hasta T-00-27] → Mitigación: documentado; tests de API cubren el JSON; T-00-27 inyecta stats.
- [`invalid_input` a mitad de un fan-out grande salta el resto de productos de ese evento] → Mitigación: `vortex reindex` (T-03-11) reconstruye; mismo trade-off que “avanzar sí o sí”.
- [Tests Prometheus globales: el recorder es process-wide] → Mitigación: asertar presencia de series y labels, no valores absolutos frágiles entre tests.
- [Ámbito ~2 h] → Mitigación: sin migración, sin wiring, sin `metrics` en `sync`, Listo cuando en unidad.

## Migration Plan

Ninguna migración `.surql` (tablas T-03-01). Rollback: revertir el worker al `?` + 1 s de T-03-08; las filas `outbox_fallo` quedan huérfanas e inofensivas (T-03-10 las purgará en v2). Tests de unidad no dejan estado en disco.

## Open Questions

Ninguna que altere la spec. El nombre exacto del trait (`SyncTelemetry` vs `OutboxMetrics`) se fija en apply.
