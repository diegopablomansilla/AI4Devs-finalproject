# Tasks

## 1. Tests (red)

- [x] 1.1 Agregar helper de seed (categoría + dos productos vía `ProductoRepo`) y test `relation_repo_creates_similar_a_to_b_and_b_to_a` en `vortex-core/crates/storage/tests/relacion_repo.rs` — `cargo test -p storage --test relacion_repo relation_repo_creates_similar_a_to_b_and_b_to_a` no compila o falla (escenario «A similar B crea A→B y B→A»; **Listo cuando** / HU-04-02)
- [x] 1.2 Agregar tests happy `relation_repo_creates_complement_perno_to_arandela`, `relation_repo_similar_explicit_false_is_unidirectional` y `relation_repo_complement_explicit_true_is_bidirectional` — `cargo test -p storage --test relacion_repo relation_repo_creates_complement relation_repo_similar_explicit_false relation_repo_complement_explicit_true` falla (escenarios «Complement Perno M8 a Arandela M8», flags explícitos; HU-04-01)
- [x] 1.3 Agregar tests unhappy `relation_repo_repeat_similar_is_duplicate_relation` y `relation_repo_repeat_complement_is_duplicate_relation` — `cargo test -p storage --test relacion_repo relation_repo_repeat` falla (escenarios «Repetir A similar B» **Listo cuando** y «Complement duplicado»; HU-04-01)
- [x] 1.4 Agregar tests unhappy `relation_repo_rejects_self_relation`, `relation_repo_missing_destino_is_not_found` y `relation_repo_update_delete_list_are_unavailable` — `cargo test -p storage --test relacion_repo relation_repo_rejects_self_relation relation_repo_missing_destino relation_repo_update_delete_list` falla (escenarios `self_relation`, destino inexistente, stubs T-04-04)
- [x] 1.5 Agregar test `tx_unique_relation_is_duplicate_relation` en `vortex-core/crates/storage/tests/tx.rs` — `cargo test -p storage --test tx tx_unique_relation_is_duplicate_relation` falla (escenario «UNIQUE de arista de relación»)

## 2. Implementación (green) — `RelacionRepo` y mapper

- [x] 2.1 Crear `vortex-core/crates/storage/src/relacion.rs` con `RelacionRepo { db: Arc<Db> }` que implementa `RelationRepo`: `create` aún no persiste (o falla); `update` / `delete` / `list_for_product` devuelven `unavailable`; reexportar desde `lib.rs` — `cargo test -p storage --test relacion_repo relation_repo_update_delete_list_are_unavailable` sale 0
- [x] 2.2 Extender `map_surreal_message` para UNIQUE de `uq_similar_a` / `uq_sustituible_por` / `uq_complementa` (o `RELATE` a esas tablas) → `duplicate_relation`, y `ENFORCED` / producto ausente → `not_found` — `cargo test -p storage --test tx tx_unique_relation_is_duplicate_relation` sale 0
- [x] 2.3 Implementar `create`: validar con `Relacion::try_new` antes de la `Tx`; `RELATE` simple o par bidireccional con `grupo_bidireccional` compartido; sin outbox; devolver la arista origen→destino — `cargo test -p storage --test relacion_repo` sale 0 (todos los escenarios de la spec, incl. **Listo cuando**)
- [x] 2.4 Confirmar non-goals: sin `update`/`delete`/`list_for_product` reales, sin HTTP, sin crate `catalog`/`recommend`, sin cambios en `openapi.yaml` ni `DomainError` ni `0007_relaciones.surql` — `git diff --name-only` sin `docs/openapi.yaml` ni `vortex-core/crates/api` ni `vortex-core/crates/catalog` ni `vortex-core/crates/recommend` ni `vortex-core/crates/domain` ni `vortex-core/crates/storage/migrations/`

## 3. Cierre (regresión)

- [x] 3.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 3.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 3.3 `openspec validate T-04-03` sale 0