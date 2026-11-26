# Tasks

## 1. Tests (red) — happy

- [x] 1.1 Agregar `vortex-core/crates/storage/tests/producto.rs` con `migrate_applies_0005_producto` (escenario «Migración aplica esquema producto») — `cargo test -p storage --test producto migrate_applies_0005` falla o no compila
- [x] 1.2 Tests `migrate_rerun_keeps_single_0005_record` y `producto_creates_minimal_sku` (escenarios «Re-arranque no duplica el registro de migración» y «Alta mínima de producto») — `cargo test -p storage --test producto migrate_rerun producto_creates_minimal` sigue en rojo
- [x] 1.3 Tests `producto_accepts_zero_precio` y `producto_allows_duplicate_oem_code` (escenarios «precio cero aceptado» y «mismo código OEM en dos productos») — `cargo test -p storage --test producto producto_accepts_zero producto_allows_duplicate_oem` falla

## 2. Tests (red) — unhappy

- [x] 2.1 Test `producto_rejects_duplicate_sku` (escenario «Índice único sku rechaza duplicado»; **Listo cuando**) — `cargo test -p storage --test producto producto_rejects_duplicate_sku` falla
- [x] 2.2 Tests `producto_rejects_negative_precio` y `producto_rejects_invalid_codigo_tipo` (escenarios «precio negativo rechazado» y «tipo de código fuera del enum rechazado»; **Listo cuando**) — `cargo test -p storage --test producto producto_rejects_negative_precio producto_rejects_invalid_codigo_tipo` falla
- [x] 2.3 Test `producto_rejects_empty_sku` (escenario «sku vacío rechazado») — `cargo test -p storage --test producto producto_rejects_empty_sku` falla

## 3. Implementación (green)

- [x] 3.1 Crear `migrations/0005_producto.surql` (tabla 4.4.4, `DEFINE … IF NOT EXISTS`, `CHANGEFEED 3d`, analizador `an_respaldo`, índices `uq_producto_sku`, `idx_producto_categoria`, `idx_producto_marca`, `ft_producto_nombre` e `idx_producto_codigos` no UNIQUE) y registrar `MIGRATION_0005_SQL` en `EMBEDDED` — `cargo test -p storage --test producto` en verde
- [x] 3.2 Documentar `idx_producto_codigos` sobre `codigos[*].valor` (H-05, no UNIQUE) en `docs/4-modelo.md` § 4.4.4, alineado con el path que acepte SurrealDB 2.7 — el snippet de 4.4.4 incluye el índice y la nota de OEM compartible
- [x] 3.3 Ajustar `tests/migrate.rs` y `tests/harness.rs` para que `list_migrations` exija `version` 0–5 (`len() == 6`, checksum de `0005`) — `cargo test -p storage --test migrate --test harness` en verde

## 4. Cierre (regresión)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 Confirmar non-goals: sin entidad `Producto`, sin `ProductRepo`, sin HTTP ni cambios en `openapi.yaml` — `git diff --name-only` sin `vortex-core/crates/domain`, `vortex-core/crates/catalog`, `vortex-core/crates/api` ni `docs/openapi.yaml`
- [x] 4.3 `openspec validate T-02-14` sale 0