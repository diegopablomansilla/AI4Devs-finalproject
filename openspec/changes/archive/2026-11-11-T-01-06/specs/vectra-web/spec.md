# Spec Delta

Ticket padre: T-01-06. Change: T-01-06. HU: HU-01-05. Contrato de error: esquema `Problem` / `ErrorCode` de `docs/openapi.yaml` y [5.4](../../../../../docs/5-contract-api.md#54-errores).

## ADDED Requirements

### Requirement: Cliente HTTP JSON y ApiError

`vectra-web/js/api.js` MUST exportar `request` y `ApiError`. `request(method, path, options)` MUST enviar una petición HTTP JSON al mismo origen. `path` MUST ser una ruta absoluta desde el origen (p. ej. `/api/v1/search`, `/health`, `/no-such-route`) y MUST NOT prefijar `/api/v1` por su cuenta (T-01-12 llama `/health`). MUST usar ES Module, MUST NOT exigir `package.json` ni un *bundler*. MUST NOT implementar SSE ni *multipart*. MUST NOT mapear `code` a mensajes de UI (T-01-07).

`options` MAY incluir `body` (valor serializable a JSON) y `signal` (`AbortSignal`). Si `body` está presente, MUST enviar `JSON.stringify(body)` y la cabecera `Content-Type: application/json`. MUST reenviar `signal` a la capa HTTP para que el caller cancele con `AbortController` ([5.7](../../../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación)).

Si `localStorage` tiene un valor no vacío (tras recortar espacios) en la clave `vectra.terminal`, MUST enviar la cabecera `X-Vectra-Terminal` con ese valor en la petición. Si la clave falta, está vacía o no hay almacenamiento, MUST omitir la cabecera y MUST NOT fallar por eso.

Una respuesta 2xx MUST resolverse con el JSON parseado. Si el cuerpo está vacío (p. ej. 204), MUST resolverse con `null`. MUST NOT interpretar HTML ni SSE como JSON de negocio.

Una respuesta no OK cuyo JSON incluye `code` (string) MUST rechazarse con `ApiError` cuyas propiedades `status`, `code` y `title` copian el cuerpo (y `status` HTTP si el cuerpo no trae `status`). `errors` MUST ser el array `errors` del cuerpo o `[]` si falta. Eso cubre el Listo cuando: un 404 `application/problem+json` con `code: "not_found"` produce `ApiError` con `code: "not_found"`.

Si la capa HTTP rechaza por fallo de red (p. ej. `TypeError` de `fetch`), MUST rechazar con `ApiError` cuyo `code` es `"network"`. `"network"` no está en `ErrorCode` de OpenAPI; T-01-07 lo agrega al mapa de mensajes.

Si la petición se aborta (`AbortError` / `name === "AbortError"`), MUST rechazar con ese error y MUST NOT convertirlo en `code: "network"`.

Una respuesta no OK sin JSON parseable o sin `code` string MUST rechazarse con `ApiError` de `code: "internal"` y el `status` HTTP, sin lanzar otra clase.

`ApiError` MUST ser instancia de `Error` y MUST exponer `{ status, code, title, errors }`.

#### Scenario: JSON 2xx se parsea (happy)

- **WHEN** `request("GET", "/api/v1/search")` recibe 200 con cuerpo `{"hits":[]}` y `Content-Type: application/json`
- **THEN** la promesa se resuelve con un objeto cuya propiedad `hits` es un array vacío

#### Scenario: POST JSON envía Content-Type y cuerpo (happy)

- **WHEN** `request("POST", "/api/v1/products", { body: { sku: "P-1" } })` se invoca
- **THEN** la petición HTTP usa método `POST` y ruta `/api/v1/products`
- **AND** lleva `Content-Type: application/json`
- **AND** el cuerpo es el JSON de `{ sku: "P-1" }`

#### Scenario: cabecera de terminal desde localStorage (happy)

- **WHEN** `localStorage["vectra.terminal"]` vale `"Caja-1"`
- **AND** se llama `request("GET", "/health")`
- **THEN** la petición incluye `X-Vectra-Terminal: Caja-1`

#### Scenario: signal de AbortController se reenvía (happy)

- **WHEN** se llama `request("GET", "/api/v1/search", { signal })` con un `AbortSignal`
- **THEN** la capa HTTP recibe ese mismo `signal`

#### Scenario: 404 problem+json es ApiError not_found (unhappy)

- **WHEN** `request("GET", "/no-such-route")` recibe 404 `application/problem+json` con `code: "not_found"` y `title` no vacío
- **THEN** la promesa se rechaza con `ApiError`
- **AND** `error.code` es `"not_found"`
- **AND** `error.status` es `404`
- **AND** `error.title` es el `title` del cuerpo

#### Scenario: invalid_input expone errors por campo (unhappy)

- **WHEN** `request` recibe 400 `problem+json` con `code: "invalid_input"` y `errors: [{ "field": "sku", "message": "requerido" }]`
- **THEN** se rechaza con `ApiError` cuyo `code` es `"invalid_input"`
- **AND** `error.errors` tiene un elemento con `field: "sku"`

#### Scenario: fallo de red es code network (unhappy)

- **WHEN** la capa HTTP rechaza con un error que no es abort (p. ej. `TypeError: Failed to fetch`)
- **THEN** se rechaza con `ApiError` cuyo `code` es `"network"`

#### Scenario: abort no se disfraza de network (unhappy)

- **WHEN** el `signal` aborta la petición
- **THEN** se rechaza con un error cuyo `name` es `"AbortError"`
- **AND** no es un `ApiError` con `code: "network"`

#### Scenario: error no JSON es internal (unhappy)

- **WHEN** `request` recibe 500 con cuerpo HTML o texto que no es JSON con `code`
- **THEN** se rechaza con `ApiError` cuyo `code` es `"internal"`
- **AND** `error.status` es `500`

#### Scenario: sin nombre de terminal no envía la cabecera (unhappy)

- **WHEN** `vectra.terminal` está ausente o es `""`
- **AND** se llama `request("GET", "/health")`
- **THEN** la petición no incluye la cabecera `X-Vectra-Terminal`

## MODIFIED Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/api.js`
- `vectra-web/js/components/index.js`, `shared.js`, `vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`
- `vectra-web/dev/components.html`

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module y MUST importar `js/components/index.js` para registrar `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel`, `vx-toast` y `vx-product-card`.

#### Scenario: el documento está en español y carga el módulo versionado (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** el elemento `html` tiene `lang="es"`
- **AND** existe un script `type="module"` cuyo `src` es `/static/{{VERSION}}/js/app.js`
- **AND** existe el *outlet* de la aplicación
- **AND** el archivo no referencia un *bundler* ni un `package.json`

#### Scenario: el árbol incluye css, app, router y components (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existen `css/` y `js/components/` como directorios
- **AND** existen los archivos `js/app.js`, `js/router.js` y `js/api.js`
- **AND** existen `css/tokens.css` y `css/base.css`
- **AND** existen `js/components/vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`, `shared.js`, `index.js` y `dev/components.html`

#### Scenario: index.html enlaza tokens y base versionados (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** hay un `link` cuyo `href` es `/static/{{VERSION}}/css/tokens.css`
- **AND** hay un `link` cuyo `href` es `/static/{{VERSION}}/css/base.css`
- **AND** el `link` de `tokens.css` aparece antes que el de `base.css`

#### Scenario: falta index.html o lang=es (unhappy)

- **WHEN** no existe `vectra-web/index.html`, o el `html` no tiene `lang="es"`
- **THEN** el harness de verificación de `vectra-web` termina con código distinto de 0
- **AND** el mensaje nombra el archivo o el atributo que falta

#### Scenario: el script no es módulo versionado (unhappy)

- **WHEN** `index.html` no carga un ES Module en `/static/{{VERSION}}/js/app.js` (falta `type="module"`, falta `{{VERSION}}`, o el path no es ese)
- **THEN** el harness de verificación termina con código distinto de 0
- **AND** el mensaje indica el `src` o el `type` esperado

#### Scenario: faltan hojas o no están versionadas (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css` o `vectra-web/css/base.css`, o `index.html` no enlaza ambos `href` `/static/{{VERSION}}/css/…`
- **THEN** el harness de verificación termina con código distinto de 0
- **AND** el mensaje nombra el archivo o el `href` que falta

#### Scenario: falta vx-button.js (unhappy)

- **WHEN** falta `vectra-web/js/components/vx-button.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `vx-button.js`

#### Scenario: falta vx-modal.js (unhappy)

- **WHEN** falta `vectra-web/js/components/vx-modal.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `vx-modal.js`

#### Scenario: falta vx-product-card.js (unhappy)

- **WHEN** falta `vectra-web/js/components/vx-product-card.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `vx-product-card.js`

#### Scenario: falta api.js (unhappy)

- **WHEN** falta `vectra-web/js/api.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `api.js`

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
| `api_tree_includes_api_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_html_catalog_loads_three_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_overlay_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_product_card_variants` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo (Listo cuando) |
| `router_operator_to_admin_productos_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_admin_productos_to_operator_does_not_reload` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* (DOM de prueba) |
| `router_empty_hash_is_operator` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_unknown_hash_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_admin_without_section_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
| `router_invalid_admin_slug_is_not_found` | `scripts/fixtures/vectra-web/` (`node --test`) | *router* |
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
| `vx_side_panel_close_restores_focus_to_opener` | `components.test.mjs` | overlay (Listo cuando) |
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
| `api_request_problem_json_not_found` | `api.test.mjs` | unhappy (Listo cuando) |
| `api_request_invalid_input_exposes_errors` | `api.test.mjs` | unhappy |
| `api_request_network_failure_is_network_code` | `api.test.mjs` | unhappy |
| `api_request_abort_is_not_network` | `api.test.mjs` | unhappy |
| `api_request_non_json_error_is_internal` | `api.test.mjs` | unhappy |
| `api_request_omits_terminal_header_when_empty` | `api.test.mjs` | unhappy |

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, *tokens*, componentes y `api`) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica) y que no hubo recarga. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, la presencia de los módulos de componentes (incluidos modal, panel, toast y product-card), `js/api.js` y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA, unhappy path de loading, error de campo y chip deshabilitado, los de overlay/toast de la tabla, y los de `vx-product-card` de la tabla. Los de `api` MUST asertar JSON 2xx, cabecera de terminal, `signal`, `ApiError` `not_found` / `invalid_input` / `network` / `internal`, abort distinto de `network`, y omisión de cabecera sin nombre.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan
- **AND** los de *tokens*, contraste, transparencia reducida y `base.css` pasan
- **AND** los de componentes (botón, campo, chip, modal, panel, toast, product-card) pasan
- **AND** los de `api.js` (JSON 2xx, `not_found`, `network`, abort) pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`
