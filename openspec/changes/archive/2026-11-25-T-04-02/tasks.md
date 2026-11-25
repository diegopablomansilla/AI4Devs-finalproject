# Tasks

## 1. Tests (red) — entidad, tipos y default bidireccional

- [x] 1.1 Agregar test `relacion_constructs_complement_perno_to_arandela` en `vortex-core/crates/domain` — `cargo test -p domain -- relacion_constructs_complement_perno_to_arandela` no compila o falla (escenario «Construcción válida complement Perno M8 a Arandela M8»; HU-04-01)
- [x] 1.2 Agregar test `relacion_constructs_similar_with_bidirectional_group` — `cargo test -p domain -- relacion_constructs_similar_with_bidirectional_group` no compila o falla (escenario «Construcción válida similar con grupo bidireccional»; HU-04-02)
- [x] 1.3 Agregar tests `default_bidireccional_similar_a_is_true` y `default_bidireccional_substitute_and_complement_are_false` — `cargo test -p domain -- default_bidireccional_similar_a_is_true default_bidireccional_substitute_and_complement_are_false` falla (escenarios happy de default por tipo; HU-04-02)
- [x] 1.4 Agregar tests `resolve_bidireccional_explicit_flag_overrides_default` y `tipo_relacion_rejects_english_or_unknown` — `cargo test -p domain -- resolve_bidireccional_explicit_flag_overrides_default tipo_relacion_rejects_english_or_unknown` falla (escenarios «flag explícito prevalece sobre el default» y «TipoRelacion en inglés del contrato»)

## 2. Tests (red) — reglas y puerto

- [x] 2.1 Agregar test unhappy `relacion_rejects_self_relation` — `cargo test -p domain -- relacion_rejects_self_relation` falla (escenario «Relación consigo mismo»; **Listo cuando** / HU-04-01; `code` `self_relation`)
- [x] 2.2 Agregar tests `relacion_accepts_distinct_origen_destino` y `relacion_accepts_prioridad_one_and_nota_500` — `cargo test -p domain -- relacion_accepts_distinct_origen_destino relacion_accepts_prioridad_one_and_nota_500` falla (escenarios «origen y destino distintos» y «prioridad 1 y nota de 500 caracteres»)
- [x] 2.3 Agregar tests unhappy `relacion_rejects_prioridad_below_one` y `relacion_rejects_nota_over_500` — `cargo test -p domain -- relacion_rejects_prioridad_below_one relacion_rejects_nota_over_500` falla (escenarios Listo cuando de `prioridad` y `nota`)
- [x] 2.4 Agregar tests `relation_repo_trait_exposes_crud` (dummy que implementa el trait), `new_relacion_omitted_bidireccional_uses_type_default` y `relation_repo_not_declared_in_catalog` — `cargo test -p domain -- relation_repo_trait_exposes_crud new_relacion_omitted_bidireccional_uses_type_default relation_repo_not_declared_in_catalog` falla (escenarios «El puerto declara create, update, delete y list_for_product», «NewRelacion omite bidireccional», «RelationRepo no vive en catalog»; DC-05)

## 3. Implementación (green) — módulo `relacion`

- [x] 3.1 Crear `vortex-core/crates/domain/src/relacion.rs` con `TipoRelacion` (wire español 4.6.1; `"similar"`/`"substitute"`/`"complement"` rechazados), `default_bidireccional`, `resolve_bidireccional` y marcador `GrupoBidireccional`; reexportar desde `lib.rs` — `cargo test -p domain -- tipo_relacion_rejects_english_or_unknown default_bidireccional_similar_a_is_true default_bidireccional_substitute_and_complement_are_false resolve_bidireccional_explicit_flag_overrides_default` sale 0
- [x] 3.2 Implementar `RelacionDraft` y `Relacion::try_new` (campos 4.6.1, `Id<Relacion>`, `origen`/`destino`, `prioridad ≥ 1`, `nota` ≤ 500) — `cargo test -p domain -- relacion_constructs_complement_perno_to_arandela relacion_constructs_similar_with_bidirectional_group relacion_accepts_distinct_origen_destino relacion_accepts_prioridad_one_and_nota_500 relacion_rejects_prioridad_below_one relacion_rejects_nota_over_500` sale 0
- [x] 3.3 Completar `origen == destino` → `InvalidInputCode::SelfRelation` con `FieldError.field` `destino` — `cargo test -p domain -- relacion_rejects_self_relation` sale 0 (escenario Listo cuando / HU-04-01)
- [x] 3.4 Declarar `NewRelacion`, `UpdateRelacion` y el trait `RelationRepo` (`create`, `update`, `delete`, `list_for_product`; `impl Future` sin `async-trait` ni `tokio`) más dummy de test — `cargo test -p domain -- relation_repo_trait_exposes_crud new_relacion_omitted_bidireccional_uses_type_default relation_repo_not_declared_in_catalog` sale 0
- [x] 3.5 Confirmar que no hay persistencia ni `RELATE` en el módulo, y que `domain` no declara crates nuevas — `rg -n "surrealdb|RELATE" vortex-core/crates/domain/src/relacion.rs` sin matches; `./scripts/check-workspace.sh` sale 0

## 4. Cierre (regresión y non-goals)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `./scripts/check-domain-deps.sh` sale 0 (escenario «domain no persiste relaciones»: sin `surrealdb` ni `storage` en el árbol)
- [x] 4.3 `git diff` vacío en `vortex-core/crates/storage`, `vortex-core/crates/api`, `vortex-core/crates/catalog`, `vortex-core/crates/recommend`, `docs/openapi.yaml` (non-goals: T-04-03, T-04-05, HTTP)
- [x] 4.4 `openspec validate T-04-02` sale 0
