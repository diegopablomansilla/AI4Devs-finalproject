# Proposal

## Ticket padre

- **Ticket:** [T-03-02](../../../docs/7-tickets.md) — Transacción "dato + outbox" y aviso al worker
- **Change:** `T-03-02`
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda)
- **Depende de:** T-03-01 (cerrado en `develop`; tablas `outbox`, `sync_cursor`, `outbox_fallo`)
- **Hace:** `OutboxEvent { entidad, registro, operacion, motivo }` en `domain`; `storage::Tx`, que acumula sentencias y parámetros, agrega un `CREATE outbox` por evento y ejecuta todo entre `BEGIN`/`COMMIT`; traducción de violaciones de índice `UNIQUE` y de `THROW` a `DomainError`; `OutboxNotifier` (`tokio::sync::Notify`) que se dispara después del *commit*.
- **No hace:** repositorios concretos.
- **Listo cuando:** tests verifican que una transacción fallida no deja ni el dato ni el evento, y que el aviso solo se emite tras un *commit* exitoso.

## Why

Toda escritura de catálogo debe persistir el dato maestro y su evento de outbox en la **misma** transacción (ADR-04, PC-02). T-03-01 dejó las tablas; sin `Tx`, mapeo a `DomainError` y aviso post-commit, T-02-03 / T-02-08 / T-02-11 y el worker (T-03-07) no tienen un helper atómico ni una señal para despertar.

## What Changes

- Tipo `OutboxEvent` en `domain` (`entidad`, `registro`, `operacion`, `motivo`) alineado a [4.8.1](../../../docs/4-modelo.md#481-outbox), sin SDK de SurrealDB
- `storage::Tx`: acumula sentencias + bindings, emite `CREATE outbox` por evento y ejecuta `BEGIN`/`COMMIT`
- Traducción de violaciones `UNIQUE` y de `THROW` (p. ej. `"stock_negativo"`) a `DomainError` existente (T-00-03); no se amplía la taxonomía
- `OutboxNotifier` (`tokio::sync::Notify`) en `storage`, disparado **solo** tras commit exitoso
- Allowlist de `storage`: path `domain` + `tokio` en `[dependencies]` (feature `sync`); `scripts/check-workspace.sh` y fixtures
- TDD: red = tests de `OutboxEvent`, atomicidad y notifier; green = `Tx` + mapper + `Notify`

## Non-goals

- Repositorios de catálogo (`CategoryRepo`, `BrandRepo`, `ProductRepo`, …) — **No hace**; **T-02-03**, **T-02-08**, **T-02-11**, **T-02-18**
- Worker Index Sync, fan-out, backoff, métricas `outbox_lag` / `outbox_failed_total` — **T-03-07**, **T-03-08**, **T-03-09**
- Purga del outbox — **T-03-10** (v2)
- Reindexación total — **T-03-11**
- Tablas de negocio (`categoria`, `marca`, `producto`, …) — **T-02-01** y siguientes
- Consumidor de vectores / embeddings — **T-05-08** (v2)
- Puerto `OutboxRepo`, avance de `sync_cursor`, escritura de `outbox_fallo`
- HTTP, `openapi.yaml`, `tracing`, UI
- Nuevas variantes o `code` de `DomainError` — **T-00-03** (solo mapeo)

## Capabilities

### New Capabilities

- `outbox-event`: tipo `OutboxEvent` en `domain` (entidad, registro ULID, operación, motivo opcional)

### Modified Capabilities

- `vortex-storage`: `storage::Tx` atómico dato + outbox; mapeo UNIQUE/THROW → `DomainError`; `OutboxNotifier` post-commit; se levanta el MUST NOT de T-03-01 sobre `OutboxEvent`/`Tx`
- `cargo-workspace`: `storage` MAY declarar path `domain` y `tokio` (`sync`) en `[dependencies]`; `cargo test --workspace` cubre Tx / notifier

## Impact

- **Repo:** `vortex-core/crates/domain/` (`OutboxEvent`), `vortex-core/crates/storage/` (`Tx`, notifier, mapper, `Cargo.toml`), `scripts/check-workspace.sh` y fixtures de allowlist
- **APIs / OpenAPI:** sin cambios
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `scripts/check-workspace.sh`; `openspec validate T-03-02`
- **CI:** job `test` (T-09-01) y `domain-deps` (T-00-02; `domain` no gana SDKs)
- **Ticket:** cierra T-03-02 (Listo cuando: rollback no deja dato ni evento; notify solo tras commit OK)
