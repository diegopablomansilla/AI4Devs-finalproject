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

### Requirement: Router hash Operador y Administración sin recargar

El *router* MUST interpretar el *hash* de la URL (fragmento tras `#`) y MUST pintar una sola vista en el *outlet*:

| Hash | Vista | Atributo observable |
|---|---|---|
| vacío, `#` o `#/` | Operador | `data-view="operator"` |
| `#/admin/<sección>` con `<sección>` = un slug `[a-z0-9-]+` | Administración de esa sección | `data-view="admin"` y `data-section="<sección>"` |

La vista operador MUST incluir un enlace `<a href="#/admin/productos">` (texto en español). La vista administración MUST incluir un enlace `<a href="#/">` para volver al operador (texto en español). MUST NOT pedir credenciales ni mostrar un formulario de login al entrar a `#/admin/…`.

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

`vectra-web/dev/components.html` MUST existir, `lang="es"`, MUST enlazar `../css/tokens.css` y `../css/base.css`, MUST cargar `../js/components/index.js` como módulo y MUST incluir al menos un `vx-button`, un `vx-field`, un `vx-chip`, un `vx-modal`, un `vx-side-panel`, un `vx-toast`, un `vx-product-card` con `variant="full"` y un `vx-product-card` con `variant="compact"`, ambos con datos de ejemplo (SKU, nombre, precio, stock). MUST incluir un control que abra el panel de muestra de modo que, al cerrarlo, el foco vuelva a ese control. Al menos una tarjeta de muestra MUST tener `stock="0"`. Al menos la variante `full` MUST incluir marca, ubicación e imagen o *placeholder*.

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

#### Scenario: falta un tag en la muestra (unhappy)

- **WHEN** falta cualquiera de los tags `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel`, `vx-toast` o `vx-product-card` en `components.html`
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
