# Tasks

## 1. Tests (red) — dominio

- [x] 1.1 Agregar tipos `DestinoResumen`, `RelacionConDestino`, `RelacionesDeProducto` (pueden no compilar aún) y tests `destino_resumen_exposes_sku_nombre_activo` y `relaciones_de_producto_groups_three_types` en `vortex-core/crates/domain/src/relacion.rs` — `cargo test -p domain destino_resumen_exposes relaciones_de_producto_groups` no compila o falla (escenarios «DestinoResumen cubre identidad administrativa» y «RelacionesDeProducto agrupa los tres tipos»; HU-04-03)
- [x] 1.2 Cambiar la firma de `RelationRepo::list_for_product` a `RelacionesDeProducto` y ajustar el dummy de `relation_repo_trait_exposes_crud` — `cargo test -p domain relation_repo_trait_exposes_crud` falla o no compila (escenario «El puerto declara create, update, delete y list_for_product»)

## 2. Implementación (green) — dominio

- [x] 2.1 Implementar los tipos de listado, reexportarlos en `domain::lib` y dejar el dummy devolviendo `RelacionesDeProducto` vacío — `cargo test -p domain relacion` sale 0 (incluye 1.1 y 1.2)
- [x] 2.2 Ajustar la firma de `list_for_product` en `storage/src/relacion.rs` (el cuerpo puede seguir en `unavailable` un momento) para que el workspace compile — `cargo test -p storage --test relacion_repo relation_repo_creates_similar` sale 0

## 3. Tests (red) — storage

- [x] 3.1 Reemplazar `relation_repo_update_delete_list_are_unavailable` por `relation_repo_update_priority_from_b_updates_both_edges` en `vortex-core/crates/storage/tests/relacion_repo.rs` — `cargo test -p storage --test relacion_repo relation_repo_update_priority_from_b_updates_both_edges` falla (escenario «Cambiar prioridad desde B modifica ambas aristas»; **Listo cuando** / HU-04-02)
- [x] 3.2 Agregar tests happy `relation_repo_update_unidirectional_nota_leaves_no_inverse`, `relation_repo_delete_from_b_removes_both_edges`, `relation_repo_delete_unidirectional_leaves_other_types`, `relation_repo_list_for_product_groups_by_type_with_destino_resumen` y `relation_repo_list_for_product_empty_when_no_edges` — `cargo test -p storage --test relacion_repo relation_repo_update_unidirectional relation_repo_delete_from_b relation_repo_delete_unidirectional relation_repo_list_for_product` falla (escenarios de edición/borrado unidireccional, grupo, listado HU-04-03 y grupos vacíos)
- [x] 3.3 Agregar tests unhappy `relation_repo_update_unknown_id_is_not_found`, `relation_repo_delete_unknown_id_is_not_found`, `relation_repo_update_rejects_prioridad_below_one` y `relation_repo_update_rejects_nota_over_500` — `cargo test -p storage --test relacion_repo relation_repo_update_unknown relation_repo_delete_unknown relation_repo_update_rejects` falla (escenarios `not_found` e `invalid_input`)

## 4. Implementación (green) — storage

- [x] 4.1 Implementar `update`: SELECT arista, validar parche con `Relacion::try_new`, `Tx` por `grupo_bidireccional` o por id, re-fetch de la arista del `id` — `cargo test -p storage --test relacion_repo relation_repo_update_priority_from_b_updates_both_edges relation_repo_update_unidirectional relation_repo_update_unknown relation_repo_update_rejects` sale 0 (Listo cuando + unhappy de update)
- [x] 4.2 Implementar `delete`: SELECT arista, `Tx` borra el grupo o solo esa arista; id ausente → `not_found` — `cargo test -p storage --test relacion_repo relation_repo_delete_from_b_removes_both_edges relation_repo_delete_unidirectional relation_repo_delete_unknown` sale 0
- [x] 4.3 Implementar `list_for_product`: aristas salientes de las tres tablas con `DestinoResumen`, agrupadas y ordenadas por `prioridad`; producto sin aristas → grupos vacíos; incluye destino inactivo — `cargo test -p storage --test relacion_repo` sale 0 (todos los escenarios de la spec, incl. **Listo cuando**)
- [x] 4.4 Confirmar non-goals: sin HTTP, sin crate `catalog`/`recommend`, sin cambios en `openapi.yaml` ni `0007_relaciones.surql` ni `create` — `git diff --name-only` sin `docs/openapi.yaml` ni `vortex-core/crates/api` ni `vortex-core/crates/catalog` ni `vortex-core/crates/recommend` ni `vortex-core/crates/storage/migrations/`

## 5. Cierre (regresión)

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 5.3 `openspec validate T-04-04` sale 0
