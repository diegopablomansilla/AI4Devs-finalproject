# Tasks

## 1. Tests (red) — slug y entidad

- [x] 1.1 Agregar tests unitarios `slug_from_nombre_ascii`, `slug_from_nombre_spanish_accents` y `slug_from_nombre_spaces_and_code` en `vortex-core/crates/domain` — `cargo test -p domain` no compila o falla (escenarios «Slug desde nombre ASCII», «Slug desde nombre con tilde y eñe», «Slug desde nombre con espacios y código»)
- [x] 1.2 Agregar tests unhappy `slug_from_nombre_rejects_empty_or_punctuation` y `slug_rejects_explicit_off_pattern` — `cargo test -p domain` sigue en rojo (escenarios «nombre que no produce slug», «slug explícito fuera de patrón»)
- [x] 1.3 Agregar tests `categoria_root_constructs` y `categoria_child_constructs_with_depth_2` — `cargo test -p domain` sigue en rojo (escenarios «Construcción de raíz válida», «Construcción de hija válida»)
- [x] 1.4 Agregar test unhappy `categoria_rejects_blank_nombre` — `cargo test -p domain -- categoria_rejects_blank_nombre` falla (escenario «nombre vacío»)

## 2. Tests (red) — profundidad, ciclo y puerto

- [x] 2.1 Agregar tests `categoria_root_depth_is_one` y `categoria_allows_depth_four` — `cargo test -p domain` sigue en rojo (escenarios «Raíz tiene profundidad 1», «Cuatro niveles permitidos»; Listo cuando)
- [x] 2.2 Agregar test unhappy `categoria_rejects_depth_five` — `cargo test -p domain -- categoria_rejects_depth_five` falla (escenario «Quinto nivel rechazado»; Listo cuando)
- [x] 2.3 Agregar tests `cycle_valid_parent_is_false` y `cycle_move_to_root_is_false` — `cargo test -p domain` sigue en rojo (escenarios «Padre válido no cicla», «Mover a raíz no cicla»)
- [x] 2.4 Agregar tests unhappy `cycle_self_parent_is_category_cycle` y `cycle_move_under_descendant_is_category_cycle` — `cargo test -p domain` sigue en rojo (escenarios «Padre igual al nodo», «Mover debajo de un descendiente»; Listo cuando / HU-02-01)
- [x] 2.5 Agregar test `category_repo_trait_exposes_create_get_list_all` (dummy que implementa el trait) — `cargo test -p domain -- category_repo_trait_exposes_create_get_list_all` falla (escenario «El puerto declara create, get y list_all»)

## 3. Implementación (green) — módulo `categoria`

- [x] 3.1 Crear `vortex-core/crates/domain/src/categoria.rs` con `Slug` (from_nombre + patrón OpenAPI, sin crate Unicode) y reexportar desde `lib.rs` — `cargo test -p domain -- slug_` sale 0 (escenarios de *slug*)
- [x] 3.2 Implementar `Categoria::try_new` (campos 4.4.1, `Id<Categoria>`, `Timestamp`, nombre 1..=100) — `cargo test -p domain -- categoria_root_constructs categoria_child_constructs_with_depth_2 categoria_rejects_blank_nombre` sale 0
- [x] 3.3 Implementar `depth` / `MAX_DEPTH = 4` y validación de profundidad 5 → `invalid_input` — `cargo test -p domain -- categoria_root_depth_is_one categoria_allows_depth_four categoria_rejects_depth_five` sale 0
- [x] 3.4 Implementar `would_form_cycle` y `reject_cycle` (`code` `category_cycle`, no `self_relation`) — `cargo test -p domain -- cycle_` sale 0
- [x] 3.5 Declarar `NewCategoria` y el trait `CategoryRepo` (`create`, `get`, `list_all`; `async fn` nativo o `impl Future`, sin `async-trait` ni `tokio`) más dummy de test — `cargo test -p domain -- category_repo_trait_exposes_create_get_list_all` sale 0
- [x] 3.6 Confirmar que no hay persistencia ni `update` en el puerto, y que `domain` no declara crates nuevas — `rg -n "surrealdb|CREATE categoria|fn update" vortex-core/crates/domain` sin matches de producto; `./scripts/check-workspace.sh` sale 0

## 4. Cierre (regresión y non-goals)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `./scripts/check-domain-deps.sh` sale 0 (escenario «domain no persiste categorías»: sin `surrealdb` ni `storage` en el árbol)
- [x] 4.3 `git diff` vacío en `vortex-core/crates/storage`, `vortex-core/crates/api`, `docs/openapi.yaml` (non-goals: T-02-03, HTTP)
- [x] 4.4 `openspec validate T-02-02` sale 0
