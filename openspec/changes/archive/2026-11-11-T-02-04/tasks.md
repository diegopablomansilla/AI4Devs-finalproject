# Tasks

## 1. Tests (red) — domain

- [x] 1.1 Agregar test `category_repo_trait_exposes_update` en `domain/src/categoria.rs` — `cargo test -p domain -- category_repo_trait_exposes` no compila

## 2. Tests (red) — happy

- [x] 2.1 `category_repo_renames_three_level_branch_ruta` — Listo cuando
- [x] 2.2 `category_repo_update_emits_outbox_per_affected`

## 3. Tests (red) — unhappy

- [x] 3.1 `category_repo_move_under_descendant_is_category_cycle` — Listo cuando / HU-02-01
- [x] 3.2 `category_repo_move_under_self_is_category_cycle`, `category_repo_update_unknown_id_is_not_found`, `category_repo_update_missing_padre_is_not_found`, `category_repo_move_rejects_depth_five`, `category_repo_rename_rejects_duplicate_slug`

## 4. Implementación (green)

- [x] 4.1 `UpdateCategoria` + `CategoryRepo::update` en `domain`
- [x] 4.2 `update` en `storage/src/categoria.rs` (subárbol, `Tx`, outbox)

## 5. Cierre

- [x] 5.1 `cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`, `openspec validate T-02-04`
