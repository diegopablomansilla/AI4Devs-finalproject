# Proposal

## Ticket padre

- **Ticket:** [T-00-27](../../../docs/7-tickets.md) — Arranque ordenado (paso 2): Meilisearch y worker
- **Change / rama:** `T-00-27` (CLI OpenSpec: `t-00-27` → carpeta `T-00-27`)
- **HU:** [HU-00-06](../../../docs/6-historias.md#hu-00-06--servicio-del-sistema-y-arranque-ordenado) (MVP: arranque degradado con Meilisearch externo; `launchd` y arranque en frío completo quedan en v2)
- **Depende de:** T-00-20 (paso 1: config → logs → Storage → API), T-03-07 (Index Sync Worker; T-03-08/T-03-09 ya en `develop`)
- **Hace:** secuencia configuración → Storage → verificación de Meilisearch externo por la URL configurada (espera `up` con límite de 20 s; si no, sigue en modo degradado) → Index Sync Worker → API
- **No hace:** levantar Meilisearch como proceso hijo (supervisor T-00-25, v2) ni carga de modelos de IA (T-05-10, T-05-13 y T-06-08)
- **Listo cuando:** con Meilisearch detenido, la API igual arranca y `/health` reporta `meilisearch: down`

## Why

`vortex serve` ya abre Storage y escucha HTTP (T-00-20), y el Index Sync Worker existe en `sync` (T-03-07+), pero **el composition root no verifica Meilisearch ni arranca el worker**. `/health` deja `meilisearch` y `sync` en `disabled`; el outbox no se drena en el proceso real. HU-00-06 (escenario degradado) y el recableo MVP ([7.5.1](../../../docs/7-tickets.md): Meilisearch a mano, sin supervisor) exigen esperar hasta 20 s por la URL configurada y, si no hay `up`, seguir con la API arriba y `meilisearch: down`.

## What Changes

- Tras Storage OK, `vortex serve` instancia `MeiliTextSearch` con `meilisearch.url` / `api_key`, publica `meilisearch = starting`, espera `TextSearch::health() == up` hasta 20 s, y publica `up` o `down` (modo degradado; **no** aborta el proceso)
- Después de esa verificación, arranca `MeiliProductWorker` con índice `productos`, `OutboxNotifier` de `Db`, repos de `storage` y `PrometheusSyncTelemetry`; publica `sync = up`
- Recién entonces bindea HTTP (el orden del ticket: Meilisearch → worker → API). El primer `GET /health` observable ya incluye el resultado de la espera
- Allowlist de `bin/vortex`: path `search`, `sync` y `domain` (composition root)
- TDD: happy = Meilisearch `up` → `/health` `ok` + `meilisearch: up` + `sync: up`; unhappy / Listo cuando = Meilisearch detenido → API arranca, HTTP 200, agregado `degraded`, `meilisearch: down`
- **BREAKING** (observabilidad): con Meilisearch caído, el agregado de `/health` pasa de `ok` (componentes `disabled`) a `degraded`; HTTP sigue 200. Los tests de binario que esperan `status == "ok"` se actualizan
- **BREAKING** (harness): `check-workspace.sh` deja de fallar si `vortex` declara path `search`, `sync` o `domain`

## Non-goals

Este change **no** cubre:

- Levantar, reiniciar o apagar Meilisearch como proceso hijo — **No hace**; **T-00-25** / **T-00-26** (v2)
- HTTPS, `rustls`, `vectra.local`, `launchd` — **T-00-16**, **T-00-17**, **T-00-30** (v2)
- Carga de modelos de IA / salud `textEmbeddings`, `llm`, `speech` — **T-05-10**, **T-05-13**, **T-06-08** (v2)
- Canal Unix socket, `vortex reindex`, `GET /search`, facetas, sinónimos — **T-00-22**, **T-03-11**, **T-03-12+**, **T-03-18**, **T-03-27**
- Búsqueda de respaldo en SurrealDB cuando Meilisearch está `down` — **T-03-30** (v2)
- Re-sonda periódica de Meilisearch tras el boot (el worker ya reintenta con *backoff*; el supervisor actualiza `/health` en v2) — **T-00-25**
- Cambiar `docs/openapi.yaml` (`GET /health` ya declara `meilisearch` / `sync`) ni el workflow CI
- Crates nuevas en `[workspace.dependencies]`

## Capabilities

### New Capabilities

- (ninguna — el paso 2 vive en el composition root de `vortex serve`, ya especificado)

### Modified Capabilities

- `vortex-api`: `vortex serve` verifica Meilisearch externo (espera 20 s), arranca el Index Sync Worker con uid `productos` y **después** escucha HTTP; Meilisearch caído no aborta; `/health` publica `meilisearch` `up`/`down` y `sync` `up`
- `vortex-sync`: se levanta el MUST NOT de enganchar el worker en `bin/vortex`; el proceso de `serve` usa el índice `productos`
- `cargo-workspace`: `bin/vortex` MAY declarar path `search`, `sync` y `domain`; `cargo test --workspace` MUST cubrir boot degradado (`meilisearch: down`) y boot con sidecar (`meilisearch: up`)

## Impact

- **Repo:** `vortex-core/bin/vortex/` (`Cargo.toml`, `serve.rs`, `tests/serve.rs`), `scripts/check-workspace.sh` (+ fixtures de allowlist)
- **APIs / OpenAPI:** sin paths nuevos; `GET /health` en el proceso real pasa a emitir `meilisearch` y `sync` distintos de `disabled`
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); el happy con sidecar es `#[ignore = "meilisearch"]` (filtro ya usado por T-09-05); `openspec validate T-00-27`
- **Ticket:** cierra T-00-27 (Listo cuando: Meilisearch detenido → API + `meilisearch: down`); desbloquea T-00-30 (v2) y el uso real de búsqueda (T-03-12+)
- **Ámbito:** cabe en ~2 h: el worker, el adaptador `health()` y el registro ya existen; este ticket solo cablea y espera. No se divide el ticket
