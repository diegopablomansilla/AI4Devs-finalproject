# Tasks

## 1. Tests (red) — layout admin

- [x] 1.1 Agregar en `scripts/fixtures/vectra-web/router.test.mjs` los tests `router_admin_layout_has_menu_and_back_link`, `router_admin_menu_lists_eight_sections`, `router_admin_nav_to_categorias_does_not_reload`, `router_admin_back_to_operator_in_one_step`, `router_admin_current_section_has_aria_current` — `node --test scripts/fixtures/vectra-web/router.test.mjs` falla en rojo (el HTML admin sigue siendo el placeholder)
- [x] 1.2 Agregar `router_admin_content_region_has_no_business_ui`, `router_admin_unknown_slug_has_menu_without_current`, `router_admin_does_not_call_api` (spy de `globalThis.fetch`, 0 llamadas; sin `<form>` ni `vx-product-card` ni password) — los tres fallan en rojo

## 2. Implementación (green) — shell admin

- [x] 2.1 En `vectra-web/js/router.js` definir `ADMIN_SECTIONS` y pintar el layout admin (encabezado con `href="#/"`, `<nav aria-label="Administración">` con los 8 enlaces, `aria-current="page"` en la sección activa, `data-region="admin-content"` con título placeholder, `vx-field` `name="terminal"`) — pasan los tests de 1.1 y 1.2; `router_admin_view_has_no_omnibox` y `router_admin_productos_to_operator_does_not_reload` siguen verdes
- [x] 2.2 En `vectra-web/css/base.css` agregar `.vx-admin`, `.vx-admin-header`, `.vx-admin-nav`, `.vx-admin-content` con *tokens* `--vx-*` existentes (sin *tokens* nuevos) — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` sigue verde

## 3. Tests (red) — nombre de terminal

- [x] 3.1 Crear `scripts/fixtures/vectra-web/admin.test.mjs` (o extender `router.test.mjs`) con `admin_terminal_name_persists_to_storage` y `admin_terminal_name_loads_from_storage`; si el archivo es nuevo, incluirlo en `node --test` de `scripts/check-vectra-web.sh` — fallan en rojo (`readTerminalName` / `writeTerminalName` aún no existen)
- [x] 3.2 Agregar `admin_terminal_name_over_64_does_not_save`, `admin_terminal_name_empty_clears_storage`, `admin_terminal_storage_failure_does_not_throw` — los tres fallan en rojo

## 4. Implementación (green) — persistencia y `change` en vx-field

- [x] 4.1 Exportar `readTerminalName` y `writeTerminalName` (clave `vectra.terminal`, trim, máximo 64, vacío → `removeItem`, catch sin throw); `startRouter` acepta `storage` opcional, rellena `value` al pintar y llama `bindTerminalName` — pasan los tests de 3.1 y 3.2
- [x] 4.2 En `vx-field`, retarget de `change` del `<input>` al host (copia `value` al atributo) para que el binder no entre al shadow — tests de terminal verdes; `vx_field_error_without_aria_invalid_fails` y el resto de `components.test.mjs` siguen verdes

## 5. Refactor y cierre

- [x] 5.1 `./scripts/check-vectra-web.sh` verde (suite previa + layout admin + terminal happy/unhappy); actualizar el comentario del script; `data-region="admin-content"` sigue sin ABM
- [x] 5.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 5.3 `openspec validate T-01-11` verde
