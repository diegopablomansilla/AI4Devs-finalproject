# Proposal

## Ticket padre

- **Ticket:** [T-03-07](../../../docs/7-tickets.md) — Worker Meilisearch: productos
- **Change / rama:** `T-03-07`
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda) (escenarios Propagación e Idempotencia; Meilisearch caído y Error permanente → T-03-09)
- **Depende de:** T-03-05 (índice `productos`), T-03-06 (`build_search_document`), T-03-02 (`Tx` + `OutboxNotifier`)
- **Hace:** tarea en `sync` que lee `outbox` desde `sync_cursor:meilisearch` en lotes de 200, se despierta con `OutboxNotifier` (y cada 1 s como respaldo), relee cada producto, hace *upsert* (activo) o *delete* (inactivo o borrado) y avanza el cursor después de confirmar la *task*; procesa también eventos `imagen_producto` (MVP: no muta `imagen`, sigue `null` hasta T-02-28)
- **No hace:** *fan-out*, reintentos ni purga (tickets siguientes)
- **Listo cuando:** test de integración: alta de producto → documento en Meilisearch en menos de 1 s; procesar dos veces el mismo evento deja el índice igual

## Why

El catálogo ya escribe `outbox` en la misma transacción (T-03-02) y el índice `productos` más el builder 4.11.1 existen (T-03-05, T-03-06), pero **nadie consume el outbox**. La búsqueda no refleja altas, ediciones ni bajas. HU-03-01 exige propagación < 1 s e idempotencia. T-03-02 dejó explícito el puerto `OutboxRepo` y el avance de `sync_cursor` para este ticket; el adaptador T-03-04 sigue enviando solo `{ id }`.

## What Changes

- Puerto `OutboxRepo` en `domain` (leer lote `id > cursor`, LIMIT 200; leer/avanzar `sync_cursor:meilisearch` tras confirmar la *task*)
- Adaptador SurrealDB en `storage` (T-03-02 lo difería)
- Worker en `sync`: notify + poll 1 s; `producto` activo → `build_search_document` + `upsert_documents`; inactivo o `NotFound` → `delete_documents`; cursor adelante solo si la *task* confirma
- `MeiliTextSearch::upsert_documents` serializa el `SearchDocument` 4.11.1 completo (ya `Serialize` en domain)
- Allowlist de `sync` en `check-workspace.sh` (hoy el crate es placeholder sin deps)
- Tests TDD happy + unhappy; Listo cuando = integración alta → hit en < 1 s e idempotencia

## Non-goals

- *Fan-out* de `categoria` / `marca` / `atributo_def` → **T-03-08** (este ticket avanza el cursor sin reindexar afectados)
- Reintentos, `outbox_fallo`, métricas, `sync` en `/health` → **T-03-09**
- Purga del outbox → **T-03-10** (v2)
- `vortex reindex` / `swapIndexes` → **T-03-11**
- Wiring en `bin/vortex` / arranque ordenado → **T-00-27**
- `GET /search`, ranking, resaltado → **T-03-12+**
- Imagen principal en el documento / `ImageRepo` → **T-02-28** (v2; HU-03-05 MVP parcial)
- Sinónimos, vectores, facetas HTTP → **T-03-27**, **T-05-08**, **T-03-18** (v2)
- Cambiar `openapi.yaml`, CI YAML, o el probe T-09-05

## Capabilities

### New Capabilities

- `vortex-sync`: Index Sync Worker de productos (lote 200, notify+1 s, upsert/delete, cursor tras *task*, idempotencia)

### Modified Capabilities

- `outbox-event`: puerto `OutboxRepo` (lote desde cursor, `OutboxRow` con id de fila, avance de `sync_cursor:meilisearch`)
- `vortex-storage`: adaptador de `OutboxRepo` sobre tablas 4.8
- `vortex-search`: `upsert_documents` envía el JSON 4.11.1 (no solo `id`); levanta el MUST NOT de T-03-04/T-03-06
- `cargo-workspace`: allowlist de `sync` (`domain`, `storage`, `tokio`; `search` solo en dev-dependencies)

## Impact

- **Repo:** `vortex-core/crates/domain/` (`OutboxRepo`); `vortex-core/crates/storage/` (adaptador); `vortex-core/crates/sync/` (worker); `vortex-core/crates/search/src/meilisearch.rs` (cuerpo completo); `scripts/check-workspace.sh` + fixtures
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain`; `cargo test -p storage`; `cargo test -p search`; `cargo test -p sync`; `cargo test --workspace -- --include-ignored meilisearch`; `openspec validate T-03-07`
- **Ticket:** cierra T-03-07 (Listo cuando: alta < 1 s + idempotencia); desbloquea T-03-08, T-03-09, T-03-11, T-00-27
- **Ámbito:** cabe en ~2 h: el backlog ya partió fan-out, reintentos, arranque y reindex. No se divide el ticket
