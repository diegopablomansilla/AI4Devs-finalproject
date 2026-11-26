# Proposal

## Ticket padre

- **Ticket:** [T-02-01](../../../docs/7-tickets.md) — Migración `categoria`
- **Change:** `T-02-01`
- **HU:** [HU-02-01](../../../docs/6-historias.md#hu-02-01--árbol-de-categorías)
- **Depende de:** T-00-09 (en `develop`; harness T-09-02 y outbox T-03-01 también en `develop`)
- **Hace:** tabla, campos e índices de [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol).
- **No hace:** lógica de árbol.
- **Listo cuando:** test de integración verifica el índice único `(padre, slug)`.

## Why

El catálogo (Fase 2) necesita la tabla maestra `categoria` antes de dominio (`T-02-02`) y repositorio (`T-02-03`). El motor de migraciones (T-00-09) ya aplica `0000` y `0001_outbox`; sin el esquema de [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol) no se puede persistir el árbol ni validar el índice único `(padre, slug)` que HU-02-01 exige para `duplicate_slug`.

## What Changes

- Migración embebida `0002_categoria.surql`: tabla `SCHEMAFULL` `categoria` según 4.4.1 (campos, `ASSERT` de `nombre`, índices `uq_categoria_slug` y `idx_categoria_ancestros`, `CHANGEFEED 3d`), con `DEFINE … IF NOT EXISTS` (4.10)
- Registro de la migración en `EMBEDDED` para que `Db::migrate()` / `storage::testing::test_db()` la apliquen
- Tests de integración en `crates/storage/tests/` (happy: esquema + mismo slug bajo padres distintos; unhappy: colisión `(padre, slug)` y `nombre` vacío)
- Ajuste de tests existentes (T-00-09 / T-09-02) que cuentan filas de `migracion` (`len() == 2`)
- TDD: red = tests de esquema sin `0002`; green = `.surql` + embed

## Non-goals

- Entidad `Categoria`, generación de *slug*, profundidad máxima 4, detección de ciclo, puerto `CategoryRepo` — **T-02-02**
- Persistencia `create`/`get`/`list_all`, cálculo de `ancestros`/`ruta`, mapeo UNIQUE → `duplicate_slug` — **T-02-03**
- Mover, renombrar, recálculo de rama, `category_cycle` — **T-02-04**
- Casos de uso, HTTP `/categories`, UI de árbol — **T-02-05**, **T-02-06**, **T-02-07**
- Tablas `marca`, `atributo_def`, `producto` — **T-02-08**, **T-02-11**, **T-02-14**
- Seed de categorías — **T-02-42**
- Worker / reindexación por cambio de categoría — **T-03-08**
- HTTP, `openapi.yaml`, `tracing`, UI
- Ampliar `DomainError` o `storage::Tx` — ya cubiertos por **T-00-03** / **T-03-02**

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage`)

### Modified Capabilities

- `vortex-storage`: migración `0002` con tabla `categoria`, campos e índices de 4.4.1; índice único `(padre, slug)` verificado por tests de integración

## Impact

- **Repo:** `vortex-core/crates/storage/migrations/0002_categoria.surql`, `src/migrate.rs` (`EMBEDDED`), `src/lib.rs` (re-export de `MIGRATION_0002_SQL`), `tests/` (nuevo + ajuste de `migrate.rs` / `harness.rs`)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `openspec validate T-02-01`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml`
