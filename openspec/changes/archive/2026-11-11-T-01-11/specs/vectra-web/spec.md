# Spec Delta

Ticket padre: T-01-11. Change: T-01-11. HU: HU-01-04 (navegación Operador / Administrador; el contenido de cada sección queda en tickets posteriores).

## ADDED Requirements

### Requirement: Layout de la vista administración

Cuando el *router* pinta `data-view="admin"` (`#/admin/<sección>` con slug `[a-z0-9-]+`), el *outlet* MUST incluir, en este orden visual:

1. Un encabezado con un enlace `<a href="#/">` cuyo texto visible está en español (p. ej. "Volver al operador"). Ese enlace MUST ser suficiente para volver a la vista operador **en un paso** (Listo cuando del ticket): un único cambio de *hash* a `#/` pinta `data-view="operator"` sin recargar el documento.
2. Una navegación (`<nav>` o equivalente con nombre accesible en español, p. ej. `aria-label="Administración"`) con **exactamente** estos ítems, en este orden, cada uno un `<a>` cuyo `href` es `#/admin/<slug>` y cuyo texto visible es la etiqueta en español:

| Slug | Etiqueta |
|---|---|
| `productos` | Productos |
| `categorias` | Categorías |
| `marcas` | Marcas |
| `atributos` | Atributos |
| `stock` | Stock |
| `relaciones` | Relaciones |
| `sinonimos` | Sinónimos |
| `notas-tecnicas` | Notas técnicas |

3. Una región `data-region="admin-content"` reservada al contenido de la sección activa. En este change MUST estar vacía de UI de negocio: MUST NOT contener `vx-product-card`, MUST NOT montar formularios de alta/edición de catálogo, MUST NOT llamar a `/api/v1/*` ni a `/health`. MAY mostrar un título o texto placeholder de la sección (español) sin datos.
4. El ajuste "Nombre de esta terminal" (requirement siguiente), visible en la vista administración.

El ítem del menú cuyo slug coincide con `data-section` MUST tener `aria-current="page"`. Los demás MUST NOT tener `aria-current="page"`. Si el slug no está en la tabla (sigue siendo administración válida por T-01-01), MUST pintar el mismo layout (menú + región vacía + ajuste de terminal) y MUST NOT marcar ningún ítem con `aria-current="page"`.

MUST conservar el enlace del operador `<a href="#/admin/productos">`. MUST NOT pedir credenciales ni mostrar formulario de login (`input[type="password"]` o usuario/contraseña). MUST NOT incluir `vx-omnibox`. `base.css` MAY definir clases de layout (p. ej. `.vx-admin`) usando solo *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos.

#### Scenario: el menú lista las ocho secciones y el enlace de vuelta (happy, Listo cuando)

- **WHEN** la app inicia con hash `#/admin/productos` (o se navega desde `#/`)
- **THEN** el *outlet* muestra `data-view="admin"` y `data-section="productos"`
- **AND** hay un `<nav>` con enlaces `href` a `#/admin/productos`, `#/admin/categorias`, `#/admin/marcas`, `#/admin/atributos`, `#/admin/stock`, `#/admin/relaciones`, `#/admin/sinonimos` y `#/admin/notas-tecnicas`
- **AND** hay un enlace `href="#/"`
- **AND** el ítem Productos tiene `aria-current="page"`

#### Scenario: cambiar de sección no recarga (happy)

- **WHEN** la vista administración está en `#/admin/productos`
- **AND** el *hash* pasa a `#/admin/categorias`
- **THEN** el *outlet* muestra `data-view="admin"` y `data-section="categorias"`
- **AND** el ítem Categorías tiene `aria-current="page"`
- **AND** Productos ya no tiene `aria-current="page"`
- **AND** no se invoca recarga del documento

#### Scenario: volver al operador en un paso (happy, Listo cuando)

- **WHEN** la vista administración está visible (`#/admin/productos` u otra sección del menú)
- **AND** se sigue el enlace `href="#/"` (o el *hash* pasa a `#/`)
- **THEN** el *outlet* muestra `data-view="operator"`
- **AND** ya no está visible `data-view="admin"`
- **AND** no se invoca recarga del documento

#### Scenario: la región de contenido no monta ABM (unhappy)

- **WHEN** el hash es `#/admin/productos` (u otra sección del menú)
- **THEN** `data-region="admin-content"` no contiene `vx-product-card`
- **AND** no hay `<form>` de alta o edición de catálogo
- **AND** no se envía ninguna petición HTTP

#### Scenario: slug de admin no listado no marca el menú (unhappy)

- **WHEN** el hash es `#/admin/foo`
- **THEN** el *outlet* muestra `data-view="admin"` y `data-section="foo"`
- **AND** el menú de ocho secciones sigue visible
- **AND** ningún ítem del menú tiene `aria-current="page"`
- **AND** no se lanza excepción

#### Scenario: administración no pide credenciales (unhappy / frontera)

- **WHEN** se navega a `#/admin/productos`
- **THEN** no hay `input[type="password"]` ni campo de usuario
- **AND** no hay `vx-omnibox`

### Requirement: Nombre de esta terminal

La vista administración MUST incluir un `vx-field` con etiqueta visible **"Nombre de esta terminal"** (español) y atributo `name="terminal"`. El valor MUST persistirse en `localStorage` bajo la clave `vectra.terminal` (la misma que `api.js` lee para `X-Vectra-Terminal`).

Al pintar la vista, si la clave tiene un valor no vacío (tras recortar espacios), el campo MUST mostrar ese valor. Si falta o está vacía, el campo MUST quedar vacío y MUST NOT fallar.

Al confirmar el valor del campo (evento `change`):

- Un valor recortado no vacío de **como máximo 64 caracteres** MUST guardarse en `vectra.terminal` (el valor guardado MUST ser el recortado).
- Un valor recortado vacío MUST borrar la clave (`removeItem`) y MUST NOT dejar `" "`.
- Un valor recortado de **más de 64 caracteres** MUST NOT guardarse. El `vx-field` MUST exponer `error` con un mensaje en español (no el código técnico) y MUST NOT lanzar.
- Si `localStorage` lanza al leer o escribir, MUST ignorarse el fallo y MUST NOT lanzar.

MUST NOT enviar peticiones HTTP al guardar. MUST NOT importar `api.js` desde el *router* para este ajuste. El *router* (o el binder de la vista) MUST aceptar un `storage` inyectable en tests (mismo patrón que `createApi`).

#### Scenario: un nombre válido se persiste (happy)

- **WHEN** el campo "Nombre de esta terminal" confirma el valor `"Caja-1"`
- **THEN** `storage["vectra.terminal"]` vale `"Caja-1"`
- **AND** el campo no tiene `error`
- **AND** no se envía ninguna petición HTTP

#### Scenario: al entrar se muestra el nombre guardado (happy)

- **WHEN** `storage["vectra.terminal"]` vale `"Mostrador"`
- **AND** se pinta `#/admin/productos`
- **THEN** el `vx-field` `name="terminal"` tiene `value` `"Mostrador"`

#### Scenario: más de 64 caracteres no se guarda (unhappy)

- **WHEN** el campo confirma un valor recortado de 65 caracteres o más
- **THEN** `vectra.terminal` no cambia (sigue el valor previo o ausente)
- **AND** el `vx-field` tiene atributo `error` con texto en español
- **AND** no se lanza excepción

#### Scenario: valor vacío borra la clave (unhappy)

- **WHEN** `vectra.terminal` vale `"Caja-1"`
- **AND** el campo confirma `""` o solo espacios
- **THEN** la clave `vectra.terminal` ya no está en el storage
- **AND** no se lanza excepción

#### Scenario: storage que falla no rompe la vista (unhappy)

- **WHEN** `storage.getItem` o `storage.setItem` lanza
- **AND** se pinta la vista administración o se confirma el campo
- **THEN** no se lanza excepción
- **AND** la vista `data-view="admin"` sigue visible

## MODIFIED Requirements

### Requirement: Tests del esqueleto y del router

Antes de archive, la suite MUST incluir estos tests (nombres estables; ubicación / tipo). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `index_html_declares_lang_es_and_versioned_module` | `scripts/check-vectra-web.sh` o fixture `scripts/fixtures/vectra-web/` | harness de árbol |
| `vectra_web_tree_includes_css_app_router_components` | `scripts/check-vectra-web.sh` | harness de árbol |
| `index_html_links_versioned_tokens_and_base` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_button_field_chip` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_modal_panel_toast` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_product_card` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_omnibox` | `scripts/check-vectra-web.sh` | harness de árbol |
| `keyboard_list_tree_includes_module` | `scripts/check-vectra-web.sh` | harness de árbol |
| `api_tree_includes_api_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `errors_tree_includes_errors_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_html_catalog_loads_three_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_overlay_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_product_card_variants` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo (Listo cuando T-01-05) |
| `components_html_catalog_loads_omnibox` | `scripts/fixtures/vectra-web/omnibox.test.mjs` o `components.test.mjs` | catálogo |
| `components_html_catalog_loads_keyboard_list` | `scripts/fixtures/vectra-web/keyboard-list.test.mjs` | catálogo (Listo cuando T-01-10) |
| `router_operator_to_admin_productos_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_admin_productos_to_operator_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_empty_hash_is_operator` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_unknown_hash_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_admin_without_section_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_invalid_admin_slug_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_operator_layout_has_omnibox_results_panel` | `scripts/fixtures/vectra-web/router.test.mjs` | layout operador |
| `router_admin_view_has_no_omnibox` | `scripts/fixtures/vectra-web/router.test.mjs` | unhappy / frontera |
| `router_admin_layout_has_menu_and_back_link` | `scripts/fixtures/vectra-web/router.test.mjs` | layout admin (Listo cuando T-01-11) |
| `router_admin_menu_lists_eight_sections` | `router.test.mjs` | layout admin |
| `router_admin_nav_to_categorias_does_not_reload` | `router.test.mjs` | layout admin |
| `router_admin_back_to_operator_in_one_step` | `router.test.mjs` | layout admin (Listo cuando T-01-11) |
| `router_admin_current_section_has_aria_current` | `router.test.mjs` | layout admin |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (No hace) |
| `router_admin_unknown_slug_has_menu_without_current` | `router.test.mjs` | unhappy |
| `router_admin_does_not_call_api` | `router.test.mjs` | unhappy |
| `admin_terminal_name_persists_to_storage` | `scripts/fixtures/vectra-web/admin.test.mjs` o `router.test.mjs` | happy |
| `admin_terminal_name_loads_from_storage` | `admin.test.mjs` o `router.test.mjs` | happy |
| `admin_terminal_name_over_64_does_not_save` | `admin.test.mjs` o `router.test.mjs` | unhappy |
| `admin_terminal_name_empty_clears_storage` | `admin.test.mjs` o `router.test.mjs` | unhappy |
| `admin_terminal_storage_failure_does_not_throw` | `admin.test.mjs` o `router.test.mjs` | unhappy |
| `omnibox_focuses_on_connect` | `scripts/fixtures/vectra-web/omnibox.test.mjs` | happy (Listo cuando T-01-08) |
| `omnibox_focuses_on_vx_closed` | `omnibox.test.mjs` | happy (Listo cuando T-01-08) |
| `omnibox_side_panel_close_focuses_omnibox` | `omnibox.test.mjs` | happy (Listo cuando T-01-08) |
| `omnibox_escape_clears_query` | `omnibox.test.mjs` | happy |
| `omnibox_escape_on_empty_query_does_not_throw` | `omnibox.test.mjs` | unhappy |
| `omnibox_vx_closed_when_disconnected_does_not_throw` | `omnibox.test.mjs` | unhappy |
| `omnibox_escape_with_open_panel_does_not_clear_query` | `omnibox.test.mjs` | unhappy |
| `omnibox_question_prefix_sets_ai_mode_and_emits_vx_query` | `omnibox.test.mjs` | happy (Listo cuando T-01-09) |
| `omnibox_text_mode_emits_vx_query_with_mode_text` | `omnibox.test.mjs` | happy |
| `omnibox_click_ai_sets_mode_and_indicator` | `omnibox.test.mjs` | happy |
| `omnibox_selector_shows_text_voice_ai` | `omnibox.test.mjs` | happy |
| `omnibox_image_mode_visible_with_vision` | `omnibox.test.mjs` | happy |
| `omnibox_question_not_at_start_stays_text` | `omnibox.test.mjs` | unhappy |
| `omnibox_image_mode_hidden_without_vision` | `omnibox.test.mjs` | unhappy |
| `omnibox_mode_does_not_call_search` | `omnibox.test.mjs` | unhappy |
| `omnibox_escape_on_ai_prefix_resets_to_text` | `omnibox.test.mjs` | unhappy |
| `omnibox_unknown_mode_falls_back_to_text` | `omnibox.test.mjs` | unhappy |
| `keyboard_list_arrow_down_selects_first_and_updates_activedescendant` | `scripts/fixtures/vectra-web/keyboard-list.test.mjs` | happy |
| `keyboard_list_enter_opens_active_item` | `keyboard-list.test.mjs` | happy (Listo cuando T-01-10) |
| `keyboard_list_escape_restores_omnibox_without_clearing` | `keyboard-list.test.mjs` | happy (Listo cuando T-01-10) |
| `keyboard_list_arrow_from_omnibox_selects_first` | `keyboard-list.test.mjs` | happy |
| `keyboard_list_plain_options_are_reusable` | `keyboard-list.test.mjs` | happy |
| `keyboard_list_empty_does_not_throw` | `keyboard-list.test.mjs` | unhappy |
| `keyboard_list_enter_without_selection_does_not_open` | `keyboard-list.test.mjs` | unhappy |
| `keyboard_list_escape_without_omnibox_does_not_throw` | `keyboard-list.test.mjs` | unhappy |
| `keyboard_list_does_not_call_search` | `keyboard-list.test.mjs` | unhappy |
| `tokens_root_declares_vx_contract` | `scripts/fixtures/vectra-web/` (`node --test`) | *tokens* |
| `tokens_text_surface_pairs_meet_wcag_aa` | `scripts/fixtures/vectra-web/` (`node --test`) | contraste WCAG |
| `tokens_low_contrast_pair_fails` | `scripts/fixtures/vectra-web/` (`node --test`) | contraste (unhappy, fixture) |
| `tokens_reduced_transparency_disables_blur` | `scripts/fixtures/vectra-web/` (`node --test`) | *tokens* / `base.css` |
| `base_css_applies_bg_text_focus_and_touch_min` | `scripts/fixtures/vectra-web/` (`node --test`) | `base.css` |
| `vx_button_shadow_and_adopted_stylesheets` | `scripts/fixtures/vectra-web/components.test.mjs` | componente |
| `vx_button_loading_sets_aria_busy_and_disabled` | `components.test.mjs` | componente |
| `vx_field_error_without_aria_invalid_fails` | `components.test.mjs` | unhappy |
| `vx_chip_disabled_does_not_toggle` | `components.test.mjs` | unhappy |
| `vx_modal_escape_emits_vx_closed` | `components.test.mjs` | overlay |
| `vx_modal_tab_stays_inside` | `components.test.mjs` | overlay |
| `vx_side_panel_close_restores_focus_to_opener` | `components.test.mjs` | overlay (Listo cuando T-01-04) |
| `vx_toast_info_aria_live_polite` | `components.test.mjs` | toast |
| `overlay_already_closed_escape_does_not_emit` | `components.test.mjs` | unhappy |
| `vx_toast_without_aria_live_fails` | `components.test.mjs` | unhappy |
| `vx_toast_unknown_variant_falls_back_to_info` | `components.test.mjs` | unhappy |
| `vx_product_card_full_renders_search_hit_fields` | `components.test.mjs` | componente |
| `vx_product_card_compact_hides_location_and_image` | `components.test.mjs` | componente |
| `vx_product_card_zero_stock_sets_out_of_stock` | `components.test.mjs` | componente |
| `vx_product_card_name_slot_projects_highlight` | `components.test.mjs` | componente |
| `vx_product_card_html_in_name_attr_is_text` | `components.test.mjs` | unhappy |
| `vx_product_card_unknown_variant_falls_back_to_full` | `components.test.mjs` | unhappy |
| `vx_product_card_missing_image_shows_placeholder` | `components.test.mjs` | unhappy |
| `vx_product_card_zero_stock_without_marker_fails` | `components.test.mjs` | unhappy |
| `api_request_parses_json_ok` | `scripts/fixtures/vectra-web/api.test.mjs` | happy |
| `api_request_posts_json_body` | `api.test.mjs` | happy |
| `api_request_sends_terminal_header` | `api.test.mjs` | happy |
| `api_request_forwards_abort_signal` | `api.test.mjs` | happy |
| `api_request_problem_json_not_found` | `api.test.mjs` | unhappy (Listo cuando T-01-06) |
| `api_request_invalid_input_exposes_errors` | `api.test.mjs` | unhappy |
| `api_request_network_failure_is_network_code` | `api.test.mjs` | unhappy |
| `api_request_abort_is_not_network` | `api.test.mjs` | unhappy |
| `api_request_non_json_error_is_internal` | `api.test.mjs` | unhappy |
| `api_request_omits_terminal_header_when_empty` | `api.test.mjs` | unhappy |
| `errors_map_covers_openapi_error_codes` | `scripts/fixtures/vectra-web/errors.test.mjs` | happy (Listo cuando T-01-07) |
| `errors_map_includes_network` | `errors.test.mjs` | happy |
| `errors_message_for_not_found_is_spanish` | `errors.test.mjs` | happy |
| `errors_show_error_sets_toast_error_variant` | `errors.test.mjs` | happy |
| `errors_show_error_assigns_field_errors_to_vx_field` | `errors.test.mjs` | happy |
| `errors_map_missing_openapi_code_fails` | `errors.test.mjs` | unhappy (Listo cuando) |
| `errors_unknown_code_uses_fallback_not_raw_code` | `errors.test.mjs` | unhappy |
| `errors_abort_does_not_show_toast` | `errors.test.mjs` | unhappy |
| `errors_unmatched_field_does_not_throw` | `errors.test.mjs` | unhappy |
| `errors_toast_text_is_not_technical_code` | `errors.test.mjs` | unhappy |

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, admin, *tokens*, componentes, `api`, `errors`, `omnibox` y `keyboard-list`) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica), que no hubo recarga, que la vista operador declara las tres regiones con `vx-omnibox` / `vx-side-panel` mientras admin no monta `vx-omnibox`, y que la vista admin declara el menú de ocho secciones, el enlace `#/`, `aria-current` en la sección activa, `data-region="admin-content"` sin UI de negocio y el `vx-field` de terminal. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, la presencia de los módulos de componentes (incluidos modal, panel, toast, product-card y omnibox), `js/api.js`, `js/errors.js`, `js/keyboard-list.js` y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA, unhappy path de loading, error de campo y chip deshabilitado, los de overlay/toast de la tabla, y los de `vx-product-card` de la tabla. Los de `api` MUST asertar JSON 2xx, cabecera de terminal, `signal`, `ApiError` `not_found` / `invalid_input` / `network` / `internal`, abort distinto de `network`, y omisión de cabecera sin nombre. Los de `errors` MUST asertar cobertura del enum `ErrorCode` + `network`, mensajes en español, `showError` → toast `error`, asignación a `vx-field` por `name`, fallback de `code` desconocido, abort sin toast, campo sin match sin throw, y que el toast no muestra el `code` técnico. Los de `omnibox` MUST asertar foco al conectar, foco tras `vx-closed` / cierre de panel, Escape que limpia, modos (`?hola` → `ai` + `vx-query`, selector Texto/Voz/IA, imagen con/sin `vision`) y los unhappy de consulta vacía, host desconectado, Escape con panel abierto, `?` no inicial, `mode` desconocido y ausencia de fetch. Los de `keyboard-list` MUST asertar recorrido con flechas y `aria-activedescendant`, Enter que abre, Escape que restaura el Omnibox sin borrar la consulta, reuso con opciones que no son tarjetas, y los unhappy de lista vacía, Enter sin selección, sin Omnibox y ausencia de fetch. Los de administración / terminal MUST asertar persistencia de `vectra.terminal` (≤ 64), carga inicial, rechazo de > 64 con `error` en el campo, borrado de clave vacía, y que un storage que lanza no rompe la vista.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan
- **AND** los de *tokens*, contraste, transparencia reducida y `base.css` pasan
- **AND** los de componentes (botón, campo, chip, modal, panel, toast, product-card) pasan
- **AND** los de `api.js` (JSON 2xx, `not_found`, `network`, abort) pasan
- **AND** los de `errors.js` (cobertura OpenAPI, toast, `vx-field`, abort, `code` desconocido) pasan
- **AND** los de `vx-omnibox` (layout, foco, Escape, modos, `vx-query`, unhappy) pasan
- **AND** los de `keyboard-list` (flechas, Enter, Escape al Omnibox, unhappy) pasan
- **AND** los de layout admin (menú de ocho secciones, vuelta al operador, `aria-current`, contenido vacío) pasan
- **AND** los de nombre de terminal (persistencia, > 64, vacío, storage que falla) pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`
