# Tasks

## 1. Tests (red) — listado

- [x] 1.1 Crear `scripts/fixtures/vectra-web/attribute-definitions.test.mjs` con `attrs_list_requests_include_inactive`, `attrs_list_preserves_api_order` y `attrs_empty_list_offers_create` — `node --test scripts/fixtures/vectra-web/attribute-definitions.test.mjs` falla en rojo (aún no existe `vectra-web/js/admin/attribute-definitions.js`)
- [x] 1.2 Agregar en `scripts/check-vectra-web.sh` la comprobación `attrs_missing_module_fails` (el archivo `vectra-web/js/admin/attribute-definitions.js` MUST existir) y registrar `attribute-definitions.test.mjs` en `node --test` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo

## 2. Implementación (green) — módulo, listado y GET

- [x] 2.1 Crear `vectra-web/js/admin/attribute-definitions.js` exportando `mountAttributeDefinitions(el, { request, showError })`: `GET /api/v1/attribute-definitions?includeInactive=true`, HTML de listado en el orden de `items` con `label`/`key`/`type`/`data-active`, acción de crear si `items` está vacío — pasan los tests de 1.1; `./scripts/check-vectra-web.sh` deja de fallar por el archivo ausente
- [x] 2.2 En `vectra-web/css/base.css` añadir `.vx-attr-*` con *tokens* `--vx-*` existentes (sin *tokens* nuevos, sin estilos de preview de facetas) — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde

## 3. Tests (red) — mutaciones happy (HU-02-03)

- [x] 3.1 Agregar `attrs_create_grado_enum_facetable` y `attrs_options_visible_only_for_enum` — los dos fallan en rojo (`POST` y visibilidad de opciones aún no existen)
- [x] 3.2 Agregar `attrs_edit_patches_label`, `attrs_deactivate_deletes` y `attrs_reactivate_patches_active_true` — los tres fallan en rojo

## 4. Implementación (green) — crear, editar, baja y reactivar

- [x] 4.1 Implementar el formulario (`select` de `type`, `vx-field` de `key`/`label`/`unit`/`options`/`order`, checkboxes `facetable`/`searchable`) y el alta: `POST` de "grado" enum `["2","5","8"]` facetable; `options` visible solo si `type === "enum"`; recargar con GET — pasan 3.1
- [x] 4.2 Implementar editar (`PATCH` de campos del formulario), baja (`DELETE`) y reactivar (`PATCH { active: true }`) — pasan 3.2; la inactiva se distingue y ofrece reactivar

## 5. Tests (red) — unhappy path

- [x] 5.1 Agregar `attrs_enum_without_options_shows_invalid_input`, `attrs_duplicate_key_shows_conflict`, `attrs_not_found_shows_not_found` y `attrs_network_shows_network` — fallan en rojo (`showError` aún no se llama o el POST de enum vacío no se envía)
- [x] 5.2 Agregar `attrs_empty_key_or_label_does_not_post`, `attrs_non_enum_does_not_send_options` y `attrs_no_facet_preview` — fallan en rojo

## 6. Implementación (green) — errores por `code` y No hace

- [x] 6.1 En `mountAttributeDefinitions`, `catch` de `request` llama `showError(err)` (mensaje por `code`, listado previo intacto); enum sin opciones envía `POST` y muestra `invalid_input`; `key`/`label` vacíos no envían; `type` no enum no incluye `options`; el HTML no tiene preview de facetas ni llama a `/api/v1/search` — pasan 5.1–5.2

## 7. Cableado del router (green) y regresión admin

- [x] 7.1 `startRouter` acepta `mountAttributeDefinitions` opcional y, si `section === "atributos"` y hay `request`, llama `mountAttributeDefinitions` sobre `[data-region="admin-content"]`; `app.js` inyecta `api.request`, `errors.showError` y `mountAttributeDefinitions` — un test de `attribute-definitions.test.mjs` o `router.test.mjs` con fake `request` observa el GET; `router_admin_content_region_has_no_business_ui` y `router_admin_does_not_call_api` siguen verdes en `#/admin/productos` y `#/admin/stock`

## 8. Verificación de cierre

- [x] 8.1 `./scripts/check-vectra-web.sh` verde (suite previa + `attribute-definitions.test.mjs` happy/unhappy); comentario del script actualizado
- [x] 8.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 8.3 `openspec validate T-02-13` verde
