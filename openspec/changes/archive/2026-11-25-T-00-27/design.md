# Design

## Context

En `develop`, `vortex serve` (T-00-20) carga `Config`, instala logs, abre `<datos>/db`, publica `storage = up` y bindea HTTP. `HealthRegistry` deja `meilisearch` y `sync` en `disabled` → agregado `ok`. El worker `MeiliProductWorker` (T-03-07/08/09) existe con `run_while`, *backoff* y `PrometheusSyncTelemetry` en `api`, pero **nadie lo instancia**. `MeiliTextSearch::health()` ya mapea `available` → `up`. `check-workspace.sh` no permite `search`/`sync`/`domain` en `bin/vortex`. Tests de binario en `tests/serve.rs` esperan `status == "ok"` y usan `meilisearch.url = http://127.0.0.1:7700` con `BOOT_WAIT` de 15 s. Ver `proposal.md` para motivación y `specs/` para el contrato.

Constraints: HTTP `localhost` (MVP); Meilisearch a mano ([2.7](../../../docs/2-arquitectura.md#27-red-local-y-seguridad-mínima)); sin supervisor (T-00-25); ~2 h; sin crates nuevas en `[workspace.dependencies]`.

## Goals / Non-Goals

**Goals:**

- Secuenciador paso 2 en `serve.rs`: sonda Meilisearch (tope 20 s) → worker `productos` → bind
- Tests que cierran el Listo cuando (`meilisearch: down`, API arriba) y el happy con sidecar
- Allowlist `vortex` → `search` + `sync` + `domain`

**Non-Goals (diseño):**

- Extraer un crate `boot` o un supervisor de procesos
- Re-sonda periódica de Meilisearch para flippear `/health` tras el boot (T-00-25)
- Crear/aplicar settings del índice en el arranque (el worker / T-03-11)
- Reloj inyectable de Tokio pause; basta un timeout parámetro + env de harness
- `kv-mem` en `serve` (sigue RocksDB)

## Decisions

### 1. No bindear HTTP hasta terminar la espera; `starting` solo en el registro

**Elección:** tras `storage = up`, publicar `meilisearch = starting`, sondear `TextSearch::health()` en bucle (intervalo ~200 ms) hasta `up` o 20 s, publicar `up` o `down`, spawn del worker, `sync = up`, **entonces** `TcpListener::bind`. El primer GET observable ya trae el resultado (Listo cuando: `down` + HTTP 200 `degraded`).

**Por qué:** el Hace ordena Meilisearch → worker → API. T-00-20 ya decidió no exponer la API a medias. 503 `starting` de Meilisearch no hace falta observable (mismo argumento que Storage).

**Alternativa:** bindear al inicio y esperar en background. Se descartó: un cliente vería `starting`/`disabled` y el ticket pide degradado *después* de la espera. **Alternativa:** abortar si Meilisearch no sube. Contradice el Listo cuando.

### 2. Timeout de 20 s en producción; harness corto para no inflar la suite

**Elección:** constante `MEILI_BOOT_WAIT = 20s` en `serve`. Función `wait_meilisearch_up(probe, timeout, interval) -> bool` testeada en unidad con sonda falsa y timeout de milisegundos. El binario lee `VORTEX_BOOT_MEILI_TIMEOUT_MS` **solo si está set** (harness); si no, 20 s. No es clave TOML (T-00-05 cerrado).

Tests de `serve.rs` que no cubren el wait: `spawn_serve_production` exporta `VORTEX_BOOT_MEILI_TIMEOUT_MS=0` (una sonda) y el TOML usa URL inalcanzable `http://127.0.0.1:1` para no depender de si el desarrollador tiene Meilisearch en 7700. El Listo cuando usa el mismo patrón (determinista, < 1 s). El happy `#[ignore = "meilisearch"]` **no** setea el env (espera real ≤ 20 s; el sidecar ya está `available` → un probe).

**Por qué:** 20 s × N tests de binario rompería CI. URL `7700` en tests actuales haría *flaky* el agregado (`ok` vs `degraded` según el sidecar local).

**Alternativa:** mock HTTP de `/health` de Meilisearch en cada test. Pesado y el worker pegaría al mock. **Alternativa:** fallar rápido en connection refused sin esperar 20 s. Se descartó: Meilisearch levantado a mano puede tardar en `available`; el Hace pide el límite de 20 s.

### 3. Worker siempre se spawnea; índice `productos`; abort en shutdown

**Elección:** `MeiliProductWorker::new(OutboxStore, ProductoRepo, AtributoDefRepo, MeiliTextSearch, PrometheusSyncTelemetry, db.outbox_notifier(), IndexUid::try_new("productos"), DEFAULT_POLL_INTERVAL)` y `tokio::spawn(worker.run_while(|| stop.load(...)))`. `MeiliTextSearch` en `Arc` (Clone). Si Meilisearch está `down`, el worker entra en *backoff* T-03-09. En `shutdown_signal`, `stop = true` y `JoinHandle::abort()` (no esperar el *backoff* de 30 s). SIGTERM sigue saliendo 0 (T-00-10).

Telemetría: inyectar `PrometheusSyncTelemetry`; un wrapper fino en `serve.rs` que además llama `HealthRegistry::set_sync_stats` cuando el worker publica lag/fallidos (el spec de T-03-09 ya lo espera del composition root).

**Por qué:** el Hace es «→ Index Sync Worker → API» sin condicional. `sync = up` significa “la tarea corre”, no “Meilisearch está sano”.

**Alternativa:** no arrancar el worker si Meilisearch está `down`. Se descartó: perdería el *backoff* y el catch-up al volver el sidecar. **Alternativa:** `run_while` hasta que el notifier despierte en shutdown. El *backoff* de 30 s retrasaría SIGTERM.

### 4. Allowlist `search` + `sync` + `domain`; no `meilisearch-sdk` en vortex

**Elección:** `bin/vortex` declara path `search`, `sync`, `domain`. `MeiliTextSearch` y `IndexUid` se usan desde ahí. `vortex_deps_allowed` y fixtures se actualizan. Hexagonal: el binario es el único que instancia el adaptador.

**Por qué:** 3.2 — solo el composition root conoce implementaciones concretas. `domain` hace falta para `IndexUid` (y los traits se resuelven por los repos).

**Alternativa:** reexportar `IndexUid` desde `sync` para no declarar `domain`. Cosmético; el binario ya depende de puertos de dominio vía `storage`/`api`.

### 5. Tests existentes de `/health` `ok` se relajan a “Storage up + HTTP 200”

**Elección:** `wait_for_health_ok_storage_up` pasa a aceptar `status` `ok` **o** `degraded` mientras `storage` sea `up` y HTTP 200. El Listo cuando y el happy ignore asertan el par `meilisearch`/`status` completo. `BOOT_WAIT` de 15 s se mantiene (con timeout de Meilisearch 0 no hace falta subirlo).

**Por qué:** el agregado `ok` con componentes `disabled` deja de ser el estado real. Relajar el helper evita reescribir cada test de catálogo/estáticos.

**Alternativa:** exigir sidecar en todos los tests de binario. Inaceptable para `cargo test --workspace` sin ignore.

## Risks / Trade-offs

- **[Espera de 20 s en un `vortex serve` de producción sin Meilisearch]** → Aceptado: es el Hace. El operador levanta Meilisearch a mano antes o convive 20 s + degradado.
- **[`/health` no flippea si Meilisearch cae o vuelve después del boot]** → Aceptado: T-00-25. El worker reintenta igual; la UI verá `down`/`up` “congelado” hasta reiniciar. Queda documentado en Non-goals.
- **[Worker abortado a mitad de *task* Meilisearch]** → Idempotencia T-03-07: el cursor no avanza sin confirmar; el próximo `serve` relee.
- **[Env `VORTEX_BOOT_MEILI_TIMEOUT_MS` mal usado en producción]** → Mitigación: no documentar en `config.example.toml`; solo harness. Un valor 0 en prod sondea una vez (sigue degradado, no aborta).
- **[macOS: `openspec new change` exige `t-00-27`]** → Ya renombrado a `T-00-27`.

## Migration Plan

1. Red: tests nuevos de wait + Listo cuando no compilan o fallan; tests viejos de `status == "ok"` fallan si se cablea Meilisearch `down` antes de relajar el helper; `check-workspace.sh` rechaza `search`/`sync`/`domain` hasta la allowlist.
2. Green: allowlist + deps + `wait_meilisearch_up` + spawn worker + helper de health + TOML de tests a URL inalcanzable + env de timeout 0.
3. Cierre: § 5 en `vortex-core/`; `openspec validate T-00-27`.
4. Rollback: revertir `serve.rs`, deps, allowlist y tests. Sin migración de datos ni cambio de OpenAPI. Un índice `productos` creado por el worker en un sidecar de dev se puede borrar a mano.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre exacto del env (`VORTEX_BOOT_MEILI_TIMEOUT_MS`) se fija al escribir el helper.
