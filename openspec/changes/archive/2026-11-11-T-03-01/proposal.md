# Proposal

## Ticket padre

- **Ticket:** [T-03-01](../../../docs/7-tickets.md) — Migración `outbox`, `sync_cursor`, `outbox_fallo`
- **Change:** `T-03-01`
- **HU:** [HU-03-01](../../../docs/6-historias.md#hu-03-01--sincronización-del-índice-de-búsqueda)
- **Depende de:** T-00-09 (en `develop`; harness T-09-02 también en `develop`)
- **Hace:** migración con las tres tablas de [4.8](../../../docs/4-modelo.md#48-sincronización-outbox-y-cursores) y los registros iniciales `sync_cursor:meilisearch` y `sync_cursor:vectores`. Va primero porque toda escritura del catálogo emite un evento.
- **No hace:** el worker (Fase 3).
- **Listo cuando:** test de integración inserta un evento y verifica los `ASSERT` de `entidad` y `operacion`.

## Why

Toda escritura del catálogo (Fase 2) debe emitir un evento de outbox en la misma transacción (ADR-04, PC-02). El motor de migraciones (T-00-09) solo tiene `0000_migracion`; sin las tablas técnicas de [4.8](../../../docs/4-modelo.md#48-sincronización-outbox-y-cursores) no se puede empezar T-03-02 ni los repositorios de catálogo.

## What Changes

- Migración embebida `0001_outbox.surql`: tablas `SCHEMAFULL` `outbox`, `sync_cursor` y `outbox_fallo` según 4.8, con `DEFINE … IF NOT EXISTS` (4.10)
- Seed de `sync_cursor:meilisearch` y `sync_cursor:vectores` (`ultimo_evento` vacío)
- Registro de la migración en `EMBEDDED` para que `Db::migrate()` / `storage::testing::test_db()` la apliquen
- Tests de integración en `crates/storage/tests/` (happy: evento válido + cursores sembrados; unhappy: `ASSERT` de `entidad` y `operacion`)
- Ajuste de tests existentes (T-00-09 / T-09-02) que asumen un único registro en `migracion`
- TDD: red = tests de esquema sin `0001`; green = `.surql` + embed

## Non-goals

- Transacción "dato + outbox", `OutboxEvent`, `storage::Tx`, `OutboxNotifier` — **T-03-02**
- Worker Index Sync, *backoff*, fan-out, métricas `outbox_lag` / `outbox_failed_total` — **T-03-07**, **T-03-08**, **T-03-09**
- Purga del outbox — **T-03-10** (v2)
- Reindexación total — **T-03-11**
- Tablas de catálogo (`categoria`, `marca`, `producto`, …) — **T-02-01** y siguientes
- Consumidor de vectores / embeddings — **T-05-08** (v2); este ticket solo siembra el cursor
- HTTP, `openapi.yaml`, `tracing`, UI
- Ampliar `DomainError` — **T-00-03** / mapeo Surreal → dominio en **T-03-02**

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: migración `0001` con `outbox`, `sync_cursor`, `outbox_fallo` y seed de cursores; `ASSERT` de `entidad`/`operacion` verificados por tests de integración

## Impact

- **Repo:** `vortex-core/crates/storage/migrations/0001_outbox.surql`, `src/migrate.rs` (`EMBEDDED`), `tests/` (nuevo + ajuste de `migrate.rs` / `harness.rs`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `openspec validate T-03-01`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
