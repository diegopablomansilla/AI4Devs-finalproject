# vectra-web Specification

## Purpose
Define el esqueleto de la SPA Vectra en `vectra-web/`: documento HTML en español, módulos ES sin paso de *build*, *router* por *hash* entre la vista del operador y la de administración, una vista de ruta no encontrada, y el tema oscuro Glassmorphism (*design tokens* y CSS base) de modo que cambiar `#/` por `#/admin/productos` actualice el contenido sin recargar la página y los pares texto/superficie cumplan contraste WCAG AA sobre `#0A0A0A`.

## Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/api.js`
- `vectra-web/js/errors.js`
- `vectra-web/js/components/index.js`, `shared.js`, `vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`, `vx-omnibox.js`
- `vectra-web/dev/components.html`

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module y MUST importar `js/components/index.js` para registrar `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel`, `vx-toast`, `vx-product-card` y `vx-omnibox`. Este change MUST NOT exigir que `app.js` importe `errors.js` (los callers de negocio llegan en tickets posteriores).

#### Scenario: el documento está en español y carga el módulo versionado (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** el elemento `html` tiene `lang="es"`
- **AND** existe un script `type="module"` cuyo `src` es `/static/{{VERSION}}/js/app.js`
- **AND** existe el *outlet* de la aplicación
- **AND** el archivo no referencia un *bundler* ni un `package.json`

#### Scenario: el árbol incluye css, app, router y components (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existen `css/` y `js/components/` como directorios
- **AND** existen los archivos `js/app.js`, `js/router.js`, `js/api.js` y `js/errors.js`
- **AND** existen `css/tokens.css` y `css/base.css`
- **AND** existen `js/components/vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`, `vx-omnibox.js`, `shared.js`, `index.js` y `dev/components.html`

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

#### Scenario: falta errors.js (unhappy)

- **WHEN** falta `vectra-web/js/errors.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `errors.js`

#### Scenario: falta vx-omnibox.js (unhappy)

- **WHEN** falta `vectra-web/js/components/vx-omnibox.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `vx-omnibox.js`

### Requirement: Router hash Operador y Administración sin recargar

El *router* MUST interpretar el *hash* de la URL (fragmento tras `#`) y MUST pintar una sola vista en el *outlet*:

| Hash | Vista | Atributo observable |
|---|---|---|
| vacío, `#` o `#/` | Operador | `data-view="operator"` |
| `#/admin/<sección>` con `<sección>` = un slug `[a-z0-9-]+` | Administración de esa sección | `data-view="admin"` y `data-section="<sección>"` |

La vista operador MUST incluir un enlace `<a href="#/admin/productos">` (texto en español) y MUST cumplir el requirement «Layout de la vista operador». La vista administración MUST incluir un enlace `<a href="#/">` para volver al operador (texto en español). MUST NOT pedir credenciales ni mostrar un formulario de login al entrar a `#/admin/…`.

Al arrancar, el *router* MUST pintar la vista que corresponde al *hash* actual. Al cambiar el *hash* (navegación del usuario o asignación a `location.hash`), MUST actualizar el *outlet* y MUST NOT recargar el documento (`location.reload` / carga completa de `index.html`). `#/admin/productos` MUST ser una sección válida.

#### Scenario: navegar de operador a admin/productos cambia la vista sin recargar (happy)

- **WHEN** la app está iniciada con *hash* `#/` (vista operador visible)
- **AND** el *hash* pasa a `#/admin/productos` (clic en el enlace o cambio de `location.hash`)
- **THEN** el *outlet* muestra un elemento con `data-view="admin"` y `data-section="productos"`
- **AND** ya no está visible la vista `data-view="operator"`
- **AND** no se invoca recarga del documento

#### Scenario: volver de admin/productos al operador no recarga (happy)

- **WHEN** la app está iniciada con *hash* `#/admin/productos`
- **AND** el *hash* pasa a `#/`
- **THEN** el *outlet* muestra `data-view="operator"`
- **AND** ya no está visible `data-view="admin"`
- **AND** no se invoca recarga del documento

#### Scenario: hash vacío se trata como operador (happy)

- **WHEN** la app inicia con *hash* vacío o `#`
- **THEN** el *outlet* muestra `data-view="operator"`

#### Scenario: administración no pide credenciales (happy)

- **WHEN** se navega a `#/admin/productos`
- **THEN** el *outlet* muestra la vista administración
- **AND** no hay formulario de usuario/contraseña ni desafío de autenticación

### Requirement: Hash desconocido o mal formado muestra vista no encontrada

Un *hash* que no coincide con las filas de la tabla del requirement «Router hash Operador y Administración sin recargar» MUST pintar una vista de no encontrada en el *outlet* (`data-view="not-found"`) y MUST NOT recargar el documento. Eso incluye, al menos: `#/no-existe`, `#/admin` (sin sección), `#/admin/` (sección vacía), `#/admin/productos/extra` (segmento de más), un slug con mayúsculas (`#/admin/Productos`) y un slug con caracteres fuera de `[a-z0-9-]+`. La vista no encontrada MUST incluir un enlace `<a href="#/">` al operador. MUST NOT dejar el *outlet* vacío. MUST NOT pintar a la vez `data-view="operator"` o `data-view="admin"` con `data-view="not-found"`.

#### Scenario: ruta desconocida es not-found sin recargar (unhappy)

- **WHEN** la app está iniciada
- **AND** el *hash* pasa a `#/no-existe`
- **THEN** el *outlet* muestra `data-view="not-found"`
- **AND** no muestra `data-view="operator"` ni `data-view="admin"`
- **AND** no se invoca recarga del documento

#### Scenario: admin sin sección es not-found (unhappy)

- **WHEN** el *hash* es `#/admin` o `#/admin/`
- **THEN** el *outlet* muestra `data-view="not-found"`

#### Scenario: admin con segmento extra o slug inválido es not-found (unhappy)

- **WHEN** el *hash* es `#/admin/productos/extra` o `#/admin/Productos` o `#/admin/prod uctos`
- **THEN** el *outlet* muestra `data-view="not-found"`
- **AND** no muestra `data-section="productos"`

### Requirement: Design tokens oscuros Glassmorphism

`vectra-web/css/tokens.css` MUST declarar en `:root` las custom properties de la tabla (prefijo `--vx-`). `--vx-color-bg` MUST ser exactamente `#0A0A0A`. `--vx-color-surface` MUST ser un color con canal alfa estrictamente menor que 1 (superficie translúcida). `--vx-color-surface-solid` MUST ser un color opaco (alfa 1) distinto de transparente. `--vx-touch-min` MUST equivaler a `44px`. `--vx-surface-blur` MUST ser una longitud distinta de `0` usada por el *glass*. Las escalas de espacio y radio MUST incluir al menos los nombres de la tabla. Este change MUST NOT agregar Web Components en `js/components/`.

| Token | Rol |
|---|---|
| `--vx-color-bg` | Fondo de página (`#0A0A0A`) |
| `--vx-color-surface` | Superficie *glass* translúcida |
| `--vx-color-surface-solid` | Superficie opaca (transparencia reducida) |
| `--vx-color-text` | Texto principal |
| `--vx-color-text-muted` | Texto secundario |
| `--vx-color-accent` | Acento |
| `--vx-color-error` | Error |
| `--vx-color-success` | Éxito |
| `--vx-color-focus-ring` | Anillo de foco |
| `--vx-color-on-accent` | Texto sobre relleno acento |
| `--vx-color-on-error` | Texto sobre relleno error |
| `--vx-color-on-success` | Texto sobre relleno éxito |
| `--vx-space-1` … `--vx-space-4` | Espaciado (cuatro peldaños) |
| `--vx-radius-sm`, `--vx-radius-md` | Radios |
| `--vx-touch-min` | Tamaño táctil mínimo (`44px`) |
| `--vx-surface-blur` | Desenfoque de `backdrop-filter` |

#### Scenario: tokens.css declara el contrato --vx-* (happy)

- **WHEN** se lee `vectra-web/css/tokens.css`
- **THEN** `:root` define todos los *tokens* de la tabla
- **AND** `--vx-color-bg` es `#0A0A0A`
- **AND** `--vx-color-surface` tiene alfa menor que 1
- **AND** `--vx-touch-min` es `44px`
- **AND** `--vx-surface-blur` no es `0`

#### Scenario: falta un token o el fondo no es #0A0A0A (unhappy)

- **WHEN** `tokens.css` omite un *token* de la tabla, o `--vx-color-bg` no es `#0A0A0A`, o `--vx-touch-min` no es `44px`
- **THEN** el test de *tokens* termina con código distinto de 0
- **AND** el mensaje nombra la custom property incumplida

#### Scenario: la superficie por defecto no es translúcida (unhappy)

- **WHEN** `--vx-color-surface` es opaco (alfa 1) o no se puede interpretar como color con alfa
- **THEN** el test de *tokens* termina con código distinto de 0
- **AND** el mensaje indica que la superficie debe ser translúcida

### Requirement: Contraste WCAG AA de pares texto/superficie

Los pares texto/superficie de la tabla MUST tener ratio de contraste WCAG 2.1 (luminancia relativa) ≥ 4,5:1. Para `--vx-color-surface` (translúcido) el oráculo MUST componer el color sobre `--vx-color-bg` antes de medir. El mismo umbral aplica a la superficie sólida. El Listo cuando del ticket (axe DevTools sobre la SPA) MUST poder medirse porque `base.css` aplica fondo y color de texto al documento; la suite automatizada MUST usar el cálculo WCAG 2.1 sin axe-core ni npm.

| Texto | Superficie |
|---|---|
| `--vx-color-text` | `--vx-color-bg` |
| `--vx-color-text` | `--vx-color-surface` compositeda sobre `--vx-color-bg` |
| `--vx-color-text` | `--vx-color-surface-solid` |
| `--vx-color-text-muted` | `--vx-color-bg` |
| `--vx-color-text-muted` | `--vx-color-surface` compositeda sobre `--vx-color-bg` |
| `--vx-color-text-muted` | `--vx-color-surface-solid` |
| `--vx-color-on-accent` | `--vx-color-accent` |
| `--vx-color-on-error` | `--vx-color-error` |
| `--vx-color-on-success` | `--vx-color-success` |

#### Scenario: los pares de tokens.css cumplen 4,5:1 (happy)

- **WHEN** se calculan los pares de la tabla a partir de `vectra-web/css/tokens.css`
- **THEN** cada par tiene contraste ≥ 4,5:1
- **AND** el test nombra `tokens_text_surface_pairs_meet_wcag_aa`

#### Scenario: un par por debajo de 4,5:1 falla (unhappy)

- **WHEN** el oráculo de contraste recibe un juego de *tokens* cuyo `--vx-color-text` sobre `--vx-color-bg` (u otro par de la tabla) queda bajo 4,5:1
- **THEN** el test termina con código distinto de 0
- **AND** el mensaje nombra el par y el ratio obtenido

### Requirement: Alternativa sólida con prefers-reduced-transparency

`tokens.css` o `base.css` MUST incluir `@media (prefers-reduced-transparency: reduce)`. Dentro de ese *media*, `.vx-surface` MUST usar `--vx-color-surface-solid` como fondo y MUST NOT aplicar `backdrop-filter` distinto de `none`. Fuera de ese *media*, `.vx-surface` MUST usar `--vx-color-surface` y `backdrop-filter` con `--vx-surface-blur`.

#### Scenario: el glass tiene fallback sólido (happy)

- **WHEN** se leen las hojas de `vectra-web/css/`
- **THEN** existe `@media (prefers-reduced-transparency: reduce)`
- **AND** dentro del *media*, `.vx-surface` no deja `backdrop-filter` activo
- **AND** el fondo de `.vx-surface` en ese *media* usa `--vx-color-surface-solid`

#### Scenario: falta el media query o el glass sigue con blur (unhappy)

- **WHEN** no existe `@media (prefers-reduced-transparency: reduce)`, o dentro de él `.vx-surface` conserva `backdrop-filter` con blur
- **THEN** el test de transparencia reducida termina con código distinto de 0
- **AND** el mensaje nombra `prefers-reduced-transparency` o `backdrop-filter`

### Requirement: CSS base aplica tokens al documento

`vectra-web/css/base.css` MUST aplicar `--vx-color-bg` como fondo de `html` o `body` y `--vx-color-text` como color de texto de `body`. MUST definir `.vx-surface`. MUST mostrar un anillo de foco visible (`:focus-visible`) usando `--vx-color-focus-ring`. MUST aplicar `min-height` y `min-width` de `--vx-touch-min` a controles nativos interactivos (`a`, `button`, `input`, `select`, `textarea`). MUST NOT introducir Web Components ni un *bundler*.

#### Scenario: base.css pinta fondo, texto, foco y 44 px (happy)

- **WHEN** se lee `vectra-web/css/base.css`
- **THEN** `body` (o `html`) usa `var(--vx-color-bg)` y `var(--vx-color-text)`
- **AND** existe `.vx-surface`
- **AND** `:focus-visible` usa `var(--vx-color-focus-ring)`
- **AND** los controles nativos listados usan `var(--vx-touch-min)`

#### Scenario: base.css no aplica el fondo o el mínimo táctil (unhappy)

- **WHEN** `base.css` no referencia `--vx-color-bg`, o no aplica `--vx-touch-min` a `button` (u otro control de la lista)
- **THEN** el test de base termina con código distinto de 0
- **AND** el mensaje nombra la custom property o el selector que falta

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

### Requirement: Hojas compartidas adoptedStyleSheets

`vectra-web/js/components/shared.js` MUST export una función que cargue `css/tokens.css` y `css/base.css` (URLs relativas a `import.meta.url` del módulo) en instancias `CSSStyleSheet` y MUST permitir asignarlas a `shadowRoot.adoptedStyleSheets` junto con una hoja local del componente. Cada Web Component de este change MUST usar `attachShadow({ mode: "open" })` y MUST usar `adoptedStyleSheets` (no solo `<style>` inline sin adopted). MUST NOT agregar `package.json`.

#### Scenario: shared carga tokens y base (happy)

- **WHEN** se invoca el cargador compartido con un proveedor de texto CSS (tests o `fetch` en navegador)
- **THEN** devuelve al menos dos `CSSStyleSheet` con reglas no vacías derivadas de tokens y base

#### Scenario: componente sin adoptedStyleSheets (unhappy)

- **WHEN** un custom element de este change se define sin asignar `adoptedStyleSheets` en su shadow
- **THEN** el test de componentes termina con código distinto de 0

### Requirement: vx-button

`vectra-web/js/components/vx-button.js` MUST registrar `vx-button` como custom element. MUST renderizar un `<button type="button">` en el shadow (o respetar atributo `type` si es `submit`). Atributos: `variant` (`primary` | `ghost` | `danger`, default `primary`), `disabled`, `loading`. Con `loading` el botón MUST estar deshabilitado y MUST exponer `aria-busy="true"`. Con `disabled` MUST estar deshabilitado. El control interactivo MUST usar `min-height` y `min-width` de `var(--vx-touch-min)`.

#### Scenario: loading deshabilita y marca ocupado (happy)

- **WHEN** un `vx-button` tiene atributo `loading`
- **THEN** el `<button>` interno está `disabled`
- **AND** tiene `aria-busy="true"`

#### Scenario: loading sin aria-busy (unhappy)

- **WHEN** el botón interno no tiene `aria-busy="true"` mientras `loading` está presente
- **THEN** el test termina con código distinto de 0

### Requirement: vx-field

`vectra-web/js/components/vx-field.js` MUST registrar `vx-field`. MUST mostrar etiqueta (`label`), texto de ayuda opcional (`help`), mensaje de error opcional (`error`) y un `<input>` (tipo por atributo `type`, default `text`). MUST enlazar ayuda y error al input con `aria-describedby`. Si `error` no está vacío, el input MUST tener `aria-invalid="true"` y el bloque de error MUST tener `role="alert"`. Atributo `disabled` MUST deshabilitar el input. MUST usar `var(--vx-touch-min)` en el input.

#### Scenario: error activa aria-invalid y alert (happy)

- **WHEN** `vx-field` tiene atributo `error` con texto
- **THEN** el input tiene `aria-invalid="true"`
- **AND** existe un nodo de error con `role="alert"`

#### Scenario: error sin aria-invalid (unhappy)

- **WHEN** `error` tiene texto pero el input no tiene `aria-invalid="true"`
- **THEN** el test termina con código distinto de 0

### Requirement: vx-chip

`vectra-web/js/components/vx-chip.js` MUST registrar `vx-chip`. MUST renderizar un `<button type="button">` con `aria-pressed` reflejando `selected` (booleano). Atributo `disabled` MUST deshabilitar el botón. Al activar (click) cuando no está `disabled`, MUST emitir evento `vx-chip-toggle` con `detail.selected` booleano. MAY mostrar atributo `count`. MUST usar `var(--vx-touch-min)`.

#### Scenario: toggle emite vx-chip-toggle (happy)

- **WHEN** un chip habilitado recibe click
- **THEN** se dispara un evento `vx-chip-toggle`
- **AND** `detail.selected` es booleano

#### Scenario: disabled no emite toggle (unhappy)

- **WHEN** un chip con `disabled` recibe click
- **THEN** no se dispara `vx-chip-toggle`

### Requirement: Página de muestra components.html

`vectra-web/dev/components.html` MUST existir, `lang="es"`, MUST enlazar `../css/tokens.css` y `../css/base.css`, MUST cargar `../js/components/index.js` como módulo y MUST incluir al menos un `vx-button`, un `vx-field`, un `vx-chip`, un `vx-modal`, un `vx-side-panel`, un `vx-toast`, un `vx-product-card` con `variant="full"` y un `vx-product-card` con `variant="compact"`, ambos con datos de ejemplo (SKU, nombre, precio, stock), y un `vx-omnibox`. MUST incluir un control que abra el panel de muestra de modo que, al cerrarlo, el foco vuelva a ese control **salvo** que haya un `vx-omnibox` conectado, en cuyo caso el foco MUST quedar en el Omnibox (Listo cuando T-01-08). Al menos una tarjeta de muestra MUST tener `stock="0"`. Al menos la variante `full` MUST incluir marca, ubicación e imagen o *placeholder*.

#### Scenario: catálogo declara los tres tags (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene las etiquetas `vx-button`, `vx-field` y `vx-chip`
- **AND** carga el módulo de componentes

#### Scenario: catálogo declara overlays (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene las etiquetas `vx-modal`, `vx-side-panel` y `vx-toast`
- **AND** hay un control asociado a abrir el panel

#### Scenario: catálogo declara ambas variantes de product-card (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene un `vx-product-card` con `variant="full"` y otro con `variant="compact"`
- **AND** hay datos de ejemplo (sku y name) en esas etiquetas

#### Scenario: catálogo declara vx-omnibox (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene la etiqueta `vx-omnibox`

#### Scenario: falta un tag en la muestra (unhappy)

- **WHEN** falta cualquiera de los tags `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel`, `vx-toast`, `vx-product-card` o `vx-omnibox` en `components.html`
- **THEN** el harness termina con código distinto de 0

### Requirement: vx-modal

`vectra-web/js/components/vx-modal.js` MUST registrar el custom element `vx-modal`. MUST usar `attachShadow({ mode: "open" })` y `adoptedStyleSheets` (hojas compartidas de *tokens*/*base* más hoja local). El atributo booleano `open` MUST controlar si el diálogo está visible. Mientras está abierto, el host o el diálogo en el shadow MUST tener `role="dialog"` y `aria-modal="true"`. MUST incluir un control de cierre operable por teclado con `min-height` y `min-width` de `var(--vx-touch-min)`. MUST atrapar el foco: Tab desde el último control enfocable y Shift+Tab desde el primero MUST permanecer dentro del diálogo. Al pasar a abierto, MUST recordar el elemento que tenía el foco. Escape o el control de cierre MUST quitar `open`, MUST emitir un evento `vx-closed` con `bubbles` y `composed` en `true`, y MUST devolver el foco a ese elemento si sigue conectado al documento. MUST NOT implementar lógica de negocio ni mensajes de API.

#### Scenario: Escape cierra y emite vx-closed (happy)

- **WHEN** un `vx-modal` está `open` y recibe la tecla Escape
- **THEN** deja de estar `open`
- **AND** se dispara un evento `vx-closed` con `bubbles` y `composed`

#### Scenario: Tab no sale del diálogo (happy)

- **WHEN** un `vx-modal` está `open` y el foco está en su último control enfocable
- **AND** se pulsa Tab
- **THEN** el foco permanece dentro del diálogo

#### Scenario: overlay ya cerrado no emite vx-closed (unhappy)

- **WHEN** un `vx-modal` no está `open`
- **AND** recibe Escape
- **THEN** no se dispara `vx-closed`

#### Scenario: open sin role dialog falla (unhappy)

- **WHEN** un `vx-modal` está `open` y no expone `role="dialog"`
- **THEN** el test de componentes termina con código distinto de 0

### Requirement: vx-side-panel

`vectra-web/js/components/vx-side-panel.js` MUST registrar `vx-side-panel`. MUST usar Shadow DOM abierto y `adoptedStyleSheets` como `vx-modal`. El atributo `open` MUST controlar visibilidad. Abierto MUST tener `role="dialog"` y `aria-modal="true"`. MUST atrapar el foco, MUST cerrar con Escape y con un control de cierre de `var(--vx-touch-min)`, MUST emitir `vx-closed` (`bubbles`, `composed`) y MUST devolver el foco al elemento que lo abrió si sigue en el documento (criterio **Listo cuando** del ticket). MUST NOT implementar lógica de recomendaciones ni de negocio.

#### Scenario: al cerrar el panel el foco vuelve al abridor (happy)

- **WHEN** un control de la página de muestra (o del test) abre un `vx-side-panel`
- **AND** el panel se cierra (Escape o control de cierre)
- **THEN** el foco vuelve a ese control
- **AND** se dispara `vx-closed`

#### Scenario: cierre sin abridor no lanza (unhappy)

- **WHEN** un `vx-side-panel` se abre sin un elemento abridor recordado
- **AND** se cierra
- **THEN** no lanza excepción
- **AND** emite `vx-closed`

#### Scenario: panel cerrado no mueve el foco (unhappy)

- **WHEN** un `vx-side-panel` no está `open`
- **AND** recibe Escape
- **THEN** no se dispara `vx-closed`
- **AND** `document.activeElement` no cambia por causa del panel

### Requirement: vx-toast

`vectra-web/js/components/vx-toast.js` MUST registrar `vx-toast`. MUST usar Shadow DOM abierto y `adoptedStyleSheets`. El atributo `variant` MUST aceptar `info` (default) y `error`. Con `variant="info"` MUST exponer `aria-live="polite"`. Con `variant="error"` MUST exponer `aria-live="assertive"`. MUST NOT atrapar el foco. MUST NOT emitir `vx-closed` (ese evento es de diálogos; T-01-08 lo usará para devolver foco al Omnibox). Una `variant` desconocida MUST tratarse como `info` y MUST NOT lanzar. MUST NOT mapear códigos de API ni textos de negocio (T-01-07).

#### Scenario: info es polite y error es assertive (happy)

- **WHEN** un `vx-toast` tiene `variant="info"` (o sin `variant`)
- **THEN** el live region tiene `aria-live="polite"`
- **WHEN** el mismo toast pasa a `variant="error"`
- **THEN** el live region tiene `aria-live="assertive"`

#### Scenario: toast sin aria-live falla (unhappy)

- **WHEN** un `vx-toast` visible no expone `aria-live` `polite` ni `assertive`
- **THEN** el test de componentes termina con código distinto de 0

#### Scenario: variant desconocida no lanza (unhappy)

- **WHEN** un `vx-toast` recibe `variant="warning"` u otro valor fuera de `info`/`error`
- **THEN** no lanza excepción
- **AND** el live region queda con `aria-live="polite"`

### Requirement: vx-product-card

`vectra-web/js/components/vx-product-card.js` MUST registrar el custom element `vx-product-card`. MUST usar `attachShadow({ mode: "open" })` y `adoptedStyleSheets` (hojas compartidas de *tokens*/*base* más hoja local). MUST usar `var(--vx-*)` y clase `vx-surface` (o equivalente sobre el host) para la superficie *glass*. MUST NOT implementar sanitización de HTML (T-03-16), MUST NOT llamar a `/search` ni a otros endpoints, MUST NOT emitir eventos de negocio.

Atributos (nombres kebab-case; valores de texto, precios y stock como string decimal según `SearchHit`):

| Atributo | Rol |
|---|---|
| `variant` | `full` (default, resultados) o `compact` (recomendaciones) |
| `sku` | SKU visible |
| `name` | Nombre de respaldo (texto plano) |
| `brand` | Marca (variante `full`) |
| `price` | Precio |
| `stock` | Cantidad de stock |
| `location` | Ubicación física (variante `full`) |
| `image-url` | URL de imagen (variante `full`) |

Variante `full` MUST mostrar nombre, SKU, marca, precio, stock, ubicación y el área de imagen. Variante `compact` MUST mostrar nombre, SKU, precio y stock, y MUST NOT mostrar ubicación ni imagen aunque esos atributos estén presentes. Una `variant` desconocida MUST tratarse como `full` y MUST NOT lanzar.

El nombre MUST proyectarse desde un *slot* con `name="name"` cuando hay contenido asignado (para que T-03-16 inyecte HTML ya sanitizado con `<mark>`). Si el *slot* está vacío, MUST pintar el atributo `name` con texto plano (`textContent`); MUST NOT interpretar el atributo `name` como HTML.

Si `image-url` no está vacío en variante `full`, MUST renderizar un `<img>` con ese `src` y `alt` derivado del nombre. Si falta o está vacío, MUST mostrar un *placeholder* y MUST NOT lanzar ni dejar un `<img>` sin `src`.

Si `stock` representa cero (`0`, `0.0`, `0.00` u otro decimal equivalente a cero), el host MUST exponer estado visual "sin stock" observable (`data-out-of-stock` presente y texto visible "Sin stock" en español). MUST NOT indicar "sin stock" solo con color. Un `stock` ausente o no numérico MUST NOT lanzar y MUST NOT marcar "sin stock".

#### Scenario: variante completa muestra los campos de SearchHit (happy)

- **WHEN** un `vx-product-card` tiene `variant="full"` (o sin `variant`) y atributos de ejemplo `sku`, `name`, `brand`, `price`, `stock`, `location`, `image-url`
- **THEN** el shadow muestra nombre, SKU, marca, precio, stock, ubicación e imagen
- **AND** usa Shadow DOM abierto y `adoptedStyleSheets`

#### Scenario: variante compacta oculta ubicación e imagen (happy)

- **WHEN** un `vx-product-card` tiene `variant="compact"` y también `location` e `image-url`
- **THEN** muestra nombre, SKU, precio y stock
- **AND** no muestra ubicación ni `<img>`

#### Scenario: stock cero marca sin stock (happy)

- **WHEN** un `vx-product-card` tiene `stock="0"`
- **THEN** el host tiene `data-out-of-stock`
- **AND** hay texto visible "Sin stock"

#### Scenario: slot name proyecta el resaltado (happy)

- **WHEN** un `vx-product-card` tiene un hijo con `slot="name"`
- **THEN** ese contenido es el título visible
- **AND** no se usa el atributo `name` como título

#### Scenario: HTML en el atributo name es texto (unhappy)

- **WHEN** un `vx-product-card` tiene `name="<img src=x onerror=alert(1)>"` y el *slot* `name` está vacío
- **THEN** el título visible contiene el literal `<img`
- **AND** no hay un `<img>` descendiente creado a partir de ese atributo

#### Scenario: variant desconocida no lanza (unhappy)

- **WHEN** un `vx-product-card` recibe `variant="hero"`
- **THEN** no lanza excepción
- **AND** se comporta como `full` (muestra ubicación si el atributo está)

#### Scenario: sin image-url muestra placeholder (unhappy)

- **WHEN** un `vx-product-card` `full` no tiene `image-url` o está vacío
- **THEN** no lanza excepción
- **AND** no hay `<img>` con `src` vacío
- **AND** hay un *placeholder* de imagen

#### Scenario: stock cero sin marcador visual falla (unhappy)

- **WHEN** un `vx-product-card` tiene `stock="0"` y no expone `data-out-of-stock` ni el texto "Sin stock"
- **THEN** el test de componentes termina con código distinto de 0

### Requirement: Cliente HTTP JSON y ApiError

`vectra-web/js/api.js` MUST exportar `request` y `ApiError`. `request(method, path, options)` MUST enviar una petición HTTP JSON al mismo origen. `path` MUST ser una ruta absoluta desde el origen (p. ej. `/api/v1/search`, `/health`, `/no-such-route`) y MUST NOT prefijar `/api/v1` por su cuenta (T-01-12 llama `/health`). MUST usar ES Module, MUST NOT exigir `package.json` ni un *bundler*. MUST NOT implementar SSE ni *multipart*. MUST NOT mapear `code` a mensajes de UI (T-01-07).

`options` MAY incluir `body` (valor serializable a JSON) y `signal` (`AbortSignal`). Si `body` está presente, MUST enviar `JSON.stringify(body)` y la cabecera `Content-Type: application/json`. MUST reenviar `signal` a la capa HTTP para que el caller cancele con `AbortController` ([5.7](../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación)).

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

### Requirement: Mensajes en español por ErrorCode y showError

`vectra-web/js/errors.js` MUST exportar un mapa de mensajes, `messageFor(code)` y `showError(err)`. MUST usar ES Module, MUST NOT exigir `package.json` ni un *bundler*. MUST NOT contener textos de pantallas de negocio (etiquetas de formularios, vacíos de listados, copy de búsqueda). MUST NOT importar `api.js` (evita acoplar el mapa al cliente HTTP).

El mapa MUST incluir una clave por cada valor del enum `ErrorCode` en `docs/openapi.yaml` **y** la clave `"network"` (código de cliente de T-01-06, ausente del enum). Cada valor MUST ser un mensaje en español, no vacío, distinto del propio `code`. `messageFor(code)` MUST devolver ese mensaje. Un `code` ausente del mapa MUST devolver el mismo mensaje que `"internal"` y MUST NOT lanzar. MUST NOT devolver el `code` crudo como texto visible.

Los mensajes de los `code` que el Gateway ya titula MUST coincidir con esos títulos (misma redacción que HU-01-05 / T-00-11). Tabla normativa:

| `code` | Mensaje |
|---|---|
| `invalid_input` | La solicitud no es válida |
| `not_found` | Recurso no encontrado |
| `conflict` | Conflicto con el estado actual |
| `version_conflict` | El producto fue modificado por otra terminal |
| `duplicate_sku` | El SKU ya existe |
| `duplicate_slug` | El slug ya existe |
| `duplicate_relation` | La relación ya existe |
| `self_relation` | Un producto no puede relacionarse consigo mismo |
| `category_cycle` | La categoría formaría un ciclo |
| `attribute_in_use` | El atributo está en uso |
| `negative_stock` | El stock no puede ser negativo |
| `payload_too_large` | El cuerpo es demasiado grande |
| `unsupported_media_type` | El tipo de contenido no está soportado |
| `unavailable` | El servicio no está disponible |
| `ai_unavailable` | El modo IA no está disponible |
| `speech_unavailable` | El reconocimiento de voz no está disponible |
| `timeout` | La operación superó el tiempo de espera |
| `internal` | Error interno |
| `network` | No hay conexión con el servidor |

`showError(err)` MUST:

- Si `err.name` es `"AbortError"`, MUST NOT pintar toast ni asignar campos (T-01-06: el abort de búsqueda no es un error de operador).
- En cualquier otro caso, MUST mostrar en un `vx-toast` el `messageFor(err.code)` (si `err.code` falta, el fallback de `internal`) con `variant="error"`. El texto visible del toast MUST NOT ser el `code` técnico (`not_found`, `network`, etc.) ni un stack.
- Si `err.errors` es un array con elementos `{ field, message }`, MUST asignar cada `message` (texto plano) al `vx-field` cuyo atributo `name` es exactamente `field`. Un `field` sin `vx-field` coincidente MUST ignorarse y MUST NOT lanzar.
- MUST NOT interpretar `message` de campo como HTML.

El toast y el contenedor de campos MUST ser inyectables en tests (mismo patrón que `createApi`). En el documento de la app, si no hay `vx-toast`, `showError` MUST crear uno y MUST NOT lanzar.

#### Scenario: messageFor cubre un code de OpenAPI (happy)

- **WHEN** se llama `messageFor("not_found")`
- **THEN** el resultado es `"Recurso no encontrado"`
- **AND** no es la cadena `"not_found"`

#### Scenario: messageFor cubre network (happy)

- **WHEN** se llama `messageFor("network")`
- **THEN** el resultado es `"No hay conexión con el servidor"`

#### Scenario: showError pinta toast error con el mensaje del code (happy)

- **WHEN** `showError` recibe `{ code: "version_conflict" }` y un `vx-toast`
- **THEN** el toast queda con `variant="error"`
- **AND** el texto visible es `"El producto fue modificado por otra terminal"`

#### Scenario: showError asigna errors a vx-field por name (happy)

- **WHEN** hay un `vx-field` con `name="sku"`
- **AND** `showError` recibe `{ code: "invalid_input", errors: [{ field: "sku", message: "requerido" }] }`
- **THEN** ese `vx-field` tiene atributo `error` igual a `"requerido"`
- **AND** el toast muestra `"La solicitud no es válida"`

#### Scenario: code desconocido usa fallback y no el code crudo (unhappy)

- **WHEN** se llama `messageFor("no_such_code")` o `showError({ code: "no_such_code" })`
- **THEN** el texto es el de `"internal"` (`"Error interno"`)
- **AND** no se muestra `"no_such_code"`
- **AND** no se lanza excepción

#### Scenario: AbortError no muestra toast (unhappy)

- **WHEN** `showError` recibe un error con `name: "AbortError"`
- **THEN** el `vx-toast` no cambia de texto ni de `variant`
- **AND** no se asignan atributos `error` a `vx-field`

#### Scenario: field sin vx-field coincidente no lanza (unhappy)

- **WHEN** `showError` recibe `{ code: "invalid_input", errors: [{ field: "attributes[1].value", message: "inválido" }] }`
- **AND** no hay `vx-field` con `name="attributes[1].value"`
- **THEN** no se lanza excepción
- **AND** el toast sigue mostrando el mensaje de `invalid_input`

#### Scenario: toast no muestra el code técnico (unhappy)

- **WHEN** `showError` recibe `{ code: "not_found", title: "not_found" }`
- **THEN** el texto visible del toast no es `"not_found"`
- **AND** es `"Recurso no encontrado"`

### Requirement: Cobertura ErrorCode de OpenAPI contra el mapa

Un chequeo automatizado (suite `node --test` ejecutada por `./scripts/check-vectra-web.sh`) MUST leer `docs/openapi.yaml`, extraer los valores del enum `ErrorCode` y compararlos con las claves del mapa de `errors.js`. MUST terminar con código 0 solo si **todos** los valores del enum tienen mensaje no vacío. MUST exigir también la clave `"network"`. MUST NOT exigir un parser YAML de npm: el extraído se hace sobre el texto del archivo. Este chequeo es el **Listo cuando** del ticket.

Si se pasa un mapa al que le falta un valor del enum (p. ej. se omite `timeout`), el chequeo MUST fallar con código distinto de 0 y MUST nombrar el `code` ausente.

#### Scenario: el mapa cubre el enum ErrorCode y network (happy)

- **WHEN** se ejecuta el chequeo de cobertura contra `docs/openapi.yaml` y el mapa de `vectra-web/js/errors.js`
- **THEN** termina con código 0
- **AND** cada valor del enum `ErrorCode` tiene mensaje
- **AND** existe la clave `network`

#### Scenario: falta un code del esquema (unhappy)

- **WHEN** el chequeo compara el enum `ErrorCode` con un mapa que omite al menos un valor del enum (p. ej. `timeout`)
- **THEN** termina con código distinto de 0
- **AND** el mensaje nombra el `code` que falta

### Requirement: Layout de la vista operador

Cuando el *router* pinta `data-view="operator"` (`#/`, `#` o hash vacío), el *outlet* MUST incluir, en este orden visual de arriba hacia abajo:

1. Una región `data-region="omnibox"` que contiene exactamente un `vx-omnibox`.
2. Una región `data-region="results"` reservada para resultados (T-03-15). En este change MUST estar vacía de `vx-product-card` y MUST NOT llamar a `/search`.
3. Una región `data-region="side-panel"` que contiene un `vx-side-panel` **sin** atributo `open` (lugar para T-04-08).

MUST conservar el enlace `<a href="#/admin/productos">` (texto en español) de T-01-01. MUST NOT pedir credenciales. Las vistas `data-view="admin"` y `data-view="not-found"` MUST NOT incluir `vx-omnibox`.

`base.css` MAY definir clases de layout (p. ej. `.vx-operator`) usando solo *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos.

#### Scenario: la vista operador tiene Omnibox, resultados y panel (happy)

- **WHEN** la app inicia con hash `#/` (o vacío / `#`)
- **THEN** el *outlet* muestra `data-view="operator"`
- **AND** existe `data-region="omnibox"` con un `vx-omnibox`
- **AND** existe `data-region="results"`
- **AND** existe `data-region="side-panel"` con un `vx-side-panel` que no está `open`
- **AND** hay un enlace `href="#/admin/productos"`

#### Scenario: administración no monta el Omnibox (unhappy)

- **WHEN** el hash es `#/admin/productos`
- **THEN** el *outlet* muestra `data-view="admin"`
- **AND** no hay `vx-omnibox` en el *outlet*

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

### Requirement: Foco del Omnibox al cargar y al cerrar un panel

Al conectar un `vx-omnibox` al documento (carga de la vista operador o vuelta desde administración), MUST poner el foco en su input de consulta. Ese es el **Listo cuando** «al cargar».

El componente MUST escuchar el evento `vx-closed` (`bubbles`, `composed`, emitido por `vx-modal` / `vx-side-panel`). Al recibirlo, si el host sigue conectado, MUST poner el foco en su input. MUST hacerlo en un *microtask* (o turno posterior al listener síncrono del overlay) para que el foco del Omnibox gane sobre la restauración al abridor de T-01-04. Si el host ya no está conectado, MUST ignorar el evento y MUST NOT lanzar.

#### Scenario: al cargar el foco está en el Omnibox (happy)

- **WHEN** se pinta la vista operador (o se conecta un `vx-omnibox` de prueba)
- **THEN** `document.activeElement` es el input de consulta del Omnibox

#### Scenario: al cerrar un panel el foco vuelve al Omnibox (happy)

- **WHEN** la vista operador tiene un `vx-omnibox` y un `vx-side-panel` abierto
- **AND** el panel se cierra (Escape o control de cierre) y emite `vx-closed`
- **THEN** tras el *microtask*, `document.activeElement` es el input de consulta del Omnibox

#### Scenario: vx-closed con Omnibox desconectado no lanza (unhappy)

- **WHEN** un `vx-omnibox` se desconecta del documento
- **AND** se dispara `vx-closed` en el documento
- **THEN** no se lanza excepción
- **AND** no se mueve el foco hacia un nodo desconectado
