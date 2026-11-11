# Tasks

## 1. Allowlist de `catalog`

- [x] 1.1 Extender `scripts/check-workspace.sh` con allowlist de `catalog` (`domain` path en `[dependencies]`; `tokio` en `[dev-dependencies]`) y ajustar `scripts/fixtures/workspace/run-script-tests.sh` (happy: `Cargo.toml` real; unhappy: `catalog` con crate fuera de allowlist, p. ej. `surrealdb`) — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0 (escenarios «catalog declara path domain» / «catalog declara una dependencia fuera de la allowlist»)

## 2. Puerto `set_activo` (red → green)

- [x] 2.1 Agregar test `category_repo_trait_exposes_set_activo` en `domain/src/categoria.rs` — `cargo test -p domain -- category_repo_trait_exposes_set_activo` no compila (escenario «Tests del puerto incluyen set_activo»)
- [x] 2.2 Añadir `CategoryRepo::set_activo` y actualizar `DummyCategoryRepo` — el test 2.1 pasa

## 3. Adaptador `set_activo` (red → green)

- [x] 3.1 Agregar `category_repo_set_activo_persists_inactive` y `category_repo_set_activo_unknown_id_is_not_found` en `storage/tests/categoria.rs` — `cargo test -p storage --test categoria -- category_repo_set_activo` falla o no compila (escenarios «Baja lógica persistida» / «set_activo de id desconocido»)
- [x] 3.2 Implementar `set_activo` en `storage/src/categoria.rs` (`Tx`, UPDATE `activo`, outbox `upsert`) — los tests 3.1 pasan

## 4. Tests `catalog` (red)

- [x] 4.1 Declarar deps de `catalog` (`domain`; `tokio` dev) y tests happy `catalog_create_child_sets_ruta`, `catalog_rename_keeps_padre`, `catalog_move_updates_padre`, `catalog_deactivate_leaf_sets_inactive`, `catalog_reactivate_sets_active` — `cargo test -p catalog` no compila o falla (Listo cuando / HU-02-01)
- [x] 4.2 Agregar tests unhappy `catalog_create_duplicate_slug_is_duplicate_slug`, `catalog_move_under_descendant_is_category_cycle`, `catalog_rename_unknown_id_is_not_found`, `catalog_deactivate_with_active_child_is_conflict`, `catalog_deactivate_with_active_product_is_conflict`, `catalog_deactivate_unknown_id_is_not_found` — siguen en rojo (H-10 / Listo cuando)

## 5. Casos de uso (green)

- [x] 5.1 Fake en memoria (`CategoryRepo` + occupancy configurable) que calcula `ruta`/`ancestros`/`slug` y `reject_cycle` — suficiente para los tests 4.x
- [x] 5.2 `CategoryService` (crear, editar, mover, baja H-10, reactivar, get/list) — `cargo test -p catalog` verde (escenarios de `catalog-categoria`)

## 6. Cierre

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`; `openspec validate T-02-05` — [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
