# Tasks

## 1. Tests (red) — listado

- [x] 1.1 Crear `scripts/fixtures/vectra-web/brands.test.mjs` con `brands_list_requests_include_inactive` y `brands_empty_list_offers_create` — `node --test scripts/fixtures/vectra-web/brands.test.mjs` falla en rojo (aún no existe `vectra-web/js/admin/brands.js`)
- [x] 1.2 Agregar en `scripts/check-vectra-web.sh` la comprobación `brands_missing_module_fails` (el archivo `vectra-web/js/admin/brands.js` MUST existir) y registrar `brands.test.mjs` en `node --test` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo

## 2. Implementación (green) — módulo, listado y GET

- [x] 2.1 Crear `vectra-web/js/admin/brands.js` exportando `mountBrands(el, { request, showError })`: `GET /api/v1/brands?includeInactive=true`, HTML de listado con `name`/`slug`/`data-active`, acción de crear si `items` está vacío — pasan los tests de 1.1; `./scripts/check-vectra-web.sh` deja de fallar por el archivo ausente
- [x] 2.2 En `vectra-web/css/base.css` añadir `.vx-brand-*` con *tokens* `--vx-*` existentes (sin *tokens* nuevos, sin estilos de logo) — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde

## 3. Tests (red) — mutaciones happy (HU-02-02)

- [x] 3.1 Agregar `brands_create_fixser_posts_name` y `brands_rename_patches_name` — los dos fallan en rojo (`POST`/`PATCH` aún no existen)
- [x] 3.2 Agregar `brands_deactivate_deletes` y `brands_reactivate_patches_active_true` — los dos fallan en rojo

## 4. Implementación (green) — crear, editar, baja y reactivar

- [x] 4.1 Implementar crear (`POST { name }`, sin `slug`; recargar con GET; `name` vacío no envía) — pasa `brands_create_fixser_posts_name`
- [x] 4.2 Implementar editar (`PATCH { name }`), baja (`DELETE`) y reactivar (`PATCH { active: true }`) — pasan 3.1–3.2; la inactiva se distingue y ofrece reactivar

## 5. Tests (red) — unhappy path

- [x] 5.1 Agregar `brands_duplicate_slug_shows_duplicate_slug`, `brands_not_found_shows_not_found` y `brands_network_shows_network` — fallan en rojo (`showError` aún no se llama)
- [x] 5.2 Agregar `brands_empty_name_does_not_post`, `brands_no_logo_ui` y `brands_does_not_send_slug` — fallan en rojo

## 6. Implementación (green) — errores por `code` y No hace

- [x] 6.1 En `mountBrands`, `catch` de `request` llama `showError(err)` (mensaje por `code`, listado previo intacto); el HTML no tiene `<input type="file">`, campo de logo ni `<img>` de marca; `POST`/`PATCH` de nombre no incluyen `slug` — pasan 5.1–5.2

## 7. Cableado del router (green) y regresión admin

- [x] 7.1 `startRouter` acepta `mountBrands` opcional y, si `section === "marcas"` y hay `request`, llama `mountBrands` sobre `[data-region="admin-content"]`; `app.js` inyecta `api.request`, `errors.showError` y `mountBrands` — un test de `brands.test.mjs` o `router.test.mjs` con fake `request` observa el GET; `router_admin_content_region_has_no_business_ui` y `router_admin_does_not_call_api` siguen verdes en `#/admin/productos` y `#/admin/stock`

## 8. Verificación de cierre

- [x] 8.1 `./scripts/check-vectra-web.sh` verde (suite previa + `brands.test.mjs` happy/unhappy); comentario del script actualizado
- [x] 8.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 8.3 `openspec validate T-02-10` verde