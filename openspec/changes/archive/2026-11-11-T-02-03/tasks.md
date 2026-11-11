# Tasks

## 1. Tests (red) — happy

- [x] 1.1 Agregar tests `category_repo_creates_root_fijaciones` y `category_repo_creates_child_pernos_with_ruta` en `vortex-core/crates/storage/tests/categoria.rs` — `cargo test -p storage --test categoria category_repo_creates` no compila o falla (escenarios «Alta de raíz», «Alta de hija Fijaciones > Pernos»; **Listo cuando**)
- [x] 1.2 Agregar tests `category_repo_get_returns_persisted` y `category_repo_list_all_includes_root_and_child` — `cargo test -p storage --test categoria category_repo_get category_repo_list_all` sigue en rojo (escenarios «get de categoría existente», «list_all incluye raíz e hija»)
- [x] 1.3 Agregar tests `category_repo_create_emits_outbox_upsert` y `category_repo_allows_same_slug_under_different_padres` — `cargo test -p storage --test categoria category_repo_create_emits category_repo_allows_same_slug` sigue en rojo (escenarios «create emite evento de outbox», «mismo slug bajo padres distintos»)

## 2. Tests (red) — unhappy

- [x] 2.1 Agregar tests `category_repo_rejects_duplicate_slug_under_same_padre` y `category_repo_rejects_duplicate_root_slug` — `cargo test -p storage --test categoria category_repo_rejects_duplicate` falla (escenarios «slug duplicado bajo el mismo padre», «slug duplicado entre raíces»; HU-02-01)
- [x] 2.2 Agregar tests `category_repo_get_unknown_id_is_not_found` y `category_repo_create_missing_padre_is_not_found` — `cargo test -p storage --test categoria category_repo_` sigue en rojo (escenarios «get de id inexistente», «padre inexistente»)
- [x] 2.3 Agregar test `category_repo_rejects_depth_five` — `cargo test -p storage --test categoria category_repo_rejects_depth_five` falla (escenario «quinto nivel rechazado»)

## 3. Implementación (green) — `CategoriaRepo`

- [x] 3.1 Crear `vortex-core/crates/storage/src/categoria.rs` con `CategoriaRepo<'a>` que implementa `get` y `list_all` (strip de `categoria:`, `Categoria::try_new`) y reexportar desde `lib.rs` — `cargo test -p storage --test categoria category_repo_get_returns_persisted category_repo_list_all_includes_root_and_child category_repo_get_unknown_id_is_not_found` sale 0
- [x] 3.2 Implementar `create`: *slug*, `ancestros`/`ruta` desde el padre, profundidad ≤ 4, `Tx` + `OutboxEvent` `categoria`/`upsert` — `cargo test -p storage --test categoria category_repo_creates_root_fijaciones category_repo_creates_child_pernos_with_ruta category_repo_create_emits_outbox_upsert` sale 0
- [x] 3.3 Mapear UNIQUE `(padre, slug)` (incluidas raíces) a `duplicate_slug` y padre ausente a `NotFound`; si el motor admite dos raíces con el mismo *slug*, `SELECT` previo — `cargo test -p storage --test categoria category_repo_rejects_duplicate category_repo_create_missing_padre category_repo_allows_same_slug category_repo_rejects_depth_five` sale 0
- [x] 3.4 Si apply necesita `serde` para hidratar filas, declararlo en `storage` y actualizar `storage_deps_allowed` + fixture happy; si no, no tocar allowlist — `./scripts/check-workspace.sh` sale 0
- [x] 3.5 Confirmar que el puerto no declara `update` y que no hay HTTP ni cambios en `0002_categoria.surql` — `rg -n "fn update" vortex-core/crates/storage/src/categoria.rs` sin matches; `git diff --name-only` sin `docs/openapi.yaml` ni `crates/api`

## 4. Cierre (regresión y non-goals)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `./scripts/check-domain-deps.sh` sale 0 (`domain` sin `surrealdb` ni persistencia)
- [x] 4.3 `openspec validate T-02-03` sale 0
