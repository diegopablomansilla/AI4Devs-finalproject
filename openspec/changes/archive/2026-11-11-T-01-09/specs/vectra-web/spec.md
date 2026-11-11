# Spec Delta

Ticket padre: T-01-09. Change: T-01-09. HU: HU-01-03 (escenario Modos; ejecución de cada modo queda fuera — T-03-15 / T-05-20 / T-06-09 / T-07-11).

## ADDED Requirements

### Requirement: Modos del Omnibox

`vx-omnibox` MUST mostrar un selector de modos con controles etiquetados en español **Texto**, **Voz** e **IA**. Cada control MUST cumplir `min-height` y `min-width` de `var(--vx-touch-min)`. El modo activo MUST indicarse de forma visible (p. ej. estilo distinto con *tokens* `--vx-*`) y en accesibilidad (`aria-pressed="true"` en el control activo, `"false"` en los demás). El atributo `mode` del host MUST reflejar el modo efectivo: `"text"`, `"voice"`, `"ai"` o `"image"`. El valor por defecto MUST ser `"text"`.

MUST existir un control **Imagen**. MUST permanecer oculto (no visible y no enfocable) mientras el host **no** tenga el atributo booleano `vision`. Con `vision` presente, MUST mostrarse y MUST ser seleccionable.

Si el `value` de la consulta **empieza** por el carácter `?`, el modo efectivo MUST ser `"ai"` (prefijo; Listo cuando). Quitar el prefijo MUST volver el modo a `"text"` salvo que el operador haya fijado **IA** con el selector. Un `?` que no esté al inicio MUST NOT cambiar el modo.

Al cambiar el texto del input (`input`) o al elegir un modo en el selector, el host MUST despachar un evento `vx-query` con `bubbles: true` y `composed: true`. `event.detail` MUST ser `{ mode, text }` donde `mode` es el modo efectivo y `text` es el valor actual del input (incluye `?` si está).

Elegir **IA** en el selector MUST fijar modo `"ai"` aunque la consulta no tenga prefijo. Elegir **Texto** MUST fijar `"text"` si no hay prefijo `?`. Elegir **Voz** o **Imagen** (esta última solo con `vision`) MUST fijar ese modo. Escribir en el input con modo `"voice"` o `"image"` MUST pasar a `"text"` o `"ai"` según el prefijo (el teclado es consulta escrita).

MUST NOT ejecutar el modo: MUST NOT llamar a `/search`, `/ai`, `/speech`, `getUserMedia`, cámara ni importar `api.js`. El evento `vx-query` es el único contrato hacia T-03-15 / T-05-20 / T-06-09.

Escape MUST seguir vaciando la consulta (T-01-08). Tras vaciar, si el modo venía solo del prefijo, MUST quedar `"text"`. MUST NOT lanzar si la consulta ya estaba vacía.

#### Scenario: escribir ?hola activa IA y emite vx-query (happy, Listo cuando)

- **WHEN** un `vx-omnibox` conectado tiene modo `"text"`
- **AND** el operador deja el input en `?hola` (evento `input`)
- **THEN** el indicador y el atributo `mode` del host quedan en `"ai"`
- **AND** el control **IA** tiene `aria-pressed="true"`
- **AND** se dispara `vx-query` con `detail.mode === "ai"` y `detail.text === "?hola"`

#### Scenario: consulta de texto emite vx-query en modo text (happy)

- **WHEN** el operador escribe `tornillo` sin prefijo `?`
- **THEN** `mode` del host es `"text"`
- **AND** se dispara `vx-query` con `detail.mode === "text"` y `detail.text === "tornillo"`

#### Scenario: el selector fija IA sin prefijo (happy)

- **WHEN** el operador activa el control **IA** con consulta `broca`
- **THEN** `mode` es `"ai"`
- **AND** se dispara `vx-query` con `detail.mode === "ai"` y `detail.text === "broca"`

#### Scenario: el selector muestra Texto, Voz e IA (happy)

- **WHEN** se conecta un `vx-omnibox` sin atributo `vision`
- **THEN** hay controles accesibles etiquetados Texto, Voz e IA
- **AND** el control **Imagen** no está visible ni en el orden de foco
- **AND** el modo inicial es `"text"`

#### Scenario: vision muestra el modo Imagen (happy)

- **WHEN** el host tiene el atributo `vision`
- **THEN** el control **Imagen** está visible y es seleccionable
- **AND** al activarlo, `mode` es `"image"` y se dispara `vx-query` con `detail.mode === "image"`

#### Scenario: ? que no está al inicio no activa IA (unhappy)

- **WHEN** el operador escribe `hola?` (el `?` no es el primer carácter)
- **THEN** el modo sigue `"text"`
- **AND** `vx-query` tiene `detail.mode === "text"`

#### Scenario: imagen oculta sin vision no se puede elegir (unhappy)

- **WHEN** el host no tiene `vision`
- **THEN** el control **Imagen** no está visible
- **AND** no hay forma de dejar `mode` en `"image"` desde la UI
- **AND** no se lanza excepción

#### Scenario: modo voz o IA no llama a la API (unhappy)

- **WHEN** el operador elige **Voz** o **IA**, o escribe `?hola`
- **THEN** no se envía ninguna petición HTTP
- **AND** no se importa `api.js`
- **AND** no se abre micrófono ni cámara

#### Scenario: Escape con prefijo vuelve a texto y no lanza (unhappy)

- **WHEN** el input vale `?hola` (modo `"ai"` por prefijo) y tiene el foco
- **AND** se pulsa Escape
- **THEN** el `value` queda `""`
- **AND** el modo queda `"text"`
- **AND** no se lanza excepción

#### Scenario: mode desconocido no rompe el selector (unhappy)

- **WHEN** se asigna al host `mode="nope"` (o un valor fuera de `text|voice|ai|image`)
- **THEN** el modo efectivo queda `"text"`
- **AND** no se lanza excepción
- **AND** el indicador muestra Texto

## MODIFIED Requirements

### Requirement: vx-omnibox

`vectra-web/js/components/vx-omnibox.js` MUST registrar el custom element `vx-omnibox`. MUST usar `attachShadow({ mode: "open" })` y `adoptedStyleSheets` (hojas compartidas de *tokens*/*base* más hoja local). MUST usar `var(--vx-*)`. El control de texto MUST usar `min-height` y `min-width` de `var(--vx-touch-min)`.

El host o su contenido MUST exponer un landmark de búsqueda (`role="search"`) y un nombre accesible en español (p. ej. `aria-label="Buscar"` o etiqueta visible "Buscar"). MUST renderizar un `<input>` de consulta (tipo `search` o `text`) en el shadow.

La consulta MUST ser el `value` del input y MUST reflejarse en el atributo `value` del host cuando cambia. Un `value` inicial en el host MUST copiarse al input.

El selector de modos, el prefijo `?` y el evento `vx-query` MUST cumplir el requirement «Modos del Omnibox». MUST NOT llamar a `/search` ni a otros endpoints. MUST NOT importar `api.js`.

#### Scenario: el Omnibox es un campo de búsqueda accesible (happy)

- **WHEN** se conecta un `vx-omnibox` al documento
- **THEN** tiene Shadow DOM abierto y `adoptedStyleSheets`
- **AND** hay un `<input>` de consulta con `min-height`/`min-width` de `var(--vx-touch-min)`
- **AND** hay `role="search"` y nombre accesible en español

#### Scenario: Escape limpia la consulta (happy)

- **WHEN** un `vx-omnibox` tiene consulta no vacía (p. ej. `tornillo`) y su input tiene el foco
- **AND** se pulsa Escape
- **THEN** el `value` del input y del host queda `""`

#### Scenario: Escape con consulta vacía no lanza (unhappy)

- **WHEN** un `vx-omnibox` tiene consulta `""` y su input tiene el foco
- **AND** se pulsa Escape
- **THEN** no se lanza excepción
- **AND** el `value` sigue `""`

#### Scenario: Escape con el panel abierto no vacía la consulta (unhappy)

- **WHEN** un `vx-omnibox` tiene consulta no vacía
- **AND** un `vx-side-panel` del mismo documento está `open` (foco atrapado en el panel)
- **AND** se pulsa Escape
- **THEN** el panel se cierra (comportamiento T-01-04)
- **AND** la consulta del Omnibox **no** queda vacía

#### Scenario: no emite vx-query ni llama a search (unhappy)

- **WHEN** el operador escribe en el Omnibox o pulsa Escape
- **THEN** no se envía una petición HTTP a `/search` ni a otro path
- **AND** el evento `vx-query` (si se dispara) MUST cumplir el requirement «Modos del Omnibox» (este escenario ya no prohíbe el evento; T-01-08 lo vetaba)

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
| `api_tree_includes_api_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `errors_tree_includes_errors_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_html_catalog_loads_three_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_overlay_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_product_card_variants` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo (Listo cuando T-01-05) |
| `components_html_catalog_loads_omnibox` | `scripts/fixtures/vectra-web/omnibox.test.mjs` o `components.test.mjs` | catálogo |
| `router_operator_to_admin_productos_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_admin_productos_to_operator_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_empty_hash_is_operator` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_unknown_hash_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_admin_without_section_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_invalid_admin_slug_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_operator_layout_has_omnibox_results_panel` | `scripts/fixtures/vectra-web/router.test.mjs` | layout operador |
| `router_admin_view_has_no_omnibox` | `scripts/fixtures/vectra-web/router.test.mjs` | unhappy / frontera |
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, *tokens*, componentes, `api`, `errors` y `omnibox`) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica), que no hubo recarga, y que la vista operador declara las tres regiones con `vx-omnibox` / `vx-side-panel` mientras admin no monta `vx-omnibox`. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, la presencia de los módulos de componentes (incluidos modal, panel, toast, product-card y omnibox), `js/api.js`, `js/errors.js` y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA, unhappy path de loading, error de campo y chip deshabilitado, los de overlay/toast de la tabla, y los de `vx-product-card` de la tabla. Los de `api` MUST asertar JSON 2xx, cabecera de terminal, `signal`, `ApiError` `not_found` / `invalid_input` / `network` / `internal`, abort distinto de `network`, y omisión de cabecera sin nombre. Los de `errors` MUST asertar cobertura del enum `ErrorCode` + `network`, mensajes en español, `showError` → toast `error`, asignación a `vx-field` por `name`, fallback de `code` desconocido, abort sin toast, campo sin match sin throw, y que el toast no muestra el `code` técnico. Los de `omnibox` MUST asertar foco al conectar, foco tras `vx-closed` / cierre de panel, Escape que limpia, modos (`?hola` → `ai` + `vx-query`, selector Texto/Voz/IA, imagen con/sin `vision`) y los unhappy de consulta vacía, host desconectado, Escape con panel abierto, `?` no inicial, `mode` desconocido y ausencia de fetch.

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

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`
