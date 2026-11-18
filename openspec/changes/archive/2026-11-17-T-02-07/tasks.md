# Tasks

## 1. Tests (red) — árbol y listado

- [x] 1.1 Crear `scripts/fixtures/vectra-web/categories.test.mjs` con `categories_tree_builds_from_parent_id`, `categories_list_requests_include_inactive` y `categories_empty_tree_offers_create_root` — `node --test scripts/fixtures/vectra-web/categories.test.mjs` falla en rojo (aún no existe `vectra-web/js/admin/categories.js`)
- [x] 1.2 Agregar en `scripts/check-vectra-web.sh` la comprobación `categories_tree_missing_module_fails` (el archivo `vectra-web/js/admin/categories.js` MUST existir) y registrar `categories.test.mjs` en `node --test` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo

## 2. Implementación (green) — módulo, árbol y GET

- [x] 2.1 Crear `vectra-web/js/admin/categories.js` exportando `buildTree` y `mountCategories(el, { request, showError })`: `GET /api/v1/categories?includeInactive=true`, HTML anidado con `name`/`path`/`data-active`, acción de crear raíz si `items` está vacío — pasan los tests de 1.1; `./scripts/check-vectra-web.sh` deja de fallar por el archivo ausente
- [x] 2.2 En `vectra-web/css/base.css` añadir `.vx-category-*` con *tokens* `--vx-*` existentes (sin *tokens* nuevos, sin `draggable`) — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde

## 3. Tests (red) — mutaciones happy (HU-02-01)

- [x] 3.1 Agregar `categories_create_subcategory_posts_name_and_parent`, `categories_create_root_posts_without_parent` y `categories_rename_patches_name` — los tres fallan en rojo (`POST`/`PATCH` aún no existen)
- [x] 3.2 Agregar `categories_move_patches_parent_id`, `categories_deactivate_deletes` y `categories_reactivate_patches_active_true` — los tres fallan en rojo

## 4. Implementación (green) — crear, renombrar, mover, baja y reactivar

- [x] 4.1 Implementar crear raíz y subcategoría (`POST` `{ name }` / `{ name, parentId }`, sin `slug`; recargar con GET; `name` vacío no envía) — pasan los tests de create de 3.1
- [x] 4.2 Implementar renombrar (`PATCH { name }`), mover (`PATCH { parentId }` con `<select>` y `parentOptions` que omite self + descendientes), baja (`DELETE`) y reactivar (`PATCH { active: true }`) — pasan 3.1–3.2; el path de la rama se actualiza tras el GET

## 5. Tests (red) — unhappy path

- [x] 5.1 Agregar `categories_cycle_shows_category_cycle`, `categories_duplicate_slug_shows_duplicate_slug` y `categories_deactivate_conflict_shows_conflict` — fallan en rojo (`showError` aún no se llama)
- [x] 5.2 Agregar `categories_network_shows_network`, `categories_empty_name_does_not_post`, `categories_no_drag_and_drop` y `categories_parent_options_exclude_self_and_descendants` — fallan en rojo

## 6. Implementación (green) — errores por `code` y No hace

- [x] 6.1 En `mountCategories`, `catch` de `request` llama `showError(err)` (mensaje por `code`, árbol previo intacto); `buildTree`/`parentOptions` cubren el filtro de descendientes; el HTML no tiene `draggable` ni listeners `dragstart`/`drop` — pasan 5.1–5.2

## 7. Cableado del router (green) y regresión admin

- [x] 7.1 `startRouter` acepta `request`/`showError` opcionales y, si `section === "categorias"` y hay `request`, llama `mountCategories` sobre `[data-region="admin-content"]`; `app.js` inyecta `api.request` y `errors.showError` — un test de `categories.test.mjs` o `router.test.mjs` con fake `request` observa el GET; `router_admin_content_region_has_no_business_ui` y `router_admin_does_not_call_api` siguen verdes en `#/admin/productos` y `#/admin/stock`

## 8. Refactor y cierre

- [x] 8.1 `./scripts/check-vectra-web.sh` verde (suite previa + `categories.test.mjs` happy/unhappy); comentario del script actualizado
- [x] 8.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 8.3 `openspec validate T-02-07` verde