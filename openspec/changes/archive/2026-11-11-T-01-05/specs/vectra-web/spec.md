# Spec Delta

Ticket padre: T-01-05. Change: T-01-05. HU: HU-01-02 (catálogo); campos alineados a HU-03-05 / `SearchHit`.

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
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
- **AND** existen los archivos `js/app.js` y `js/router.js`
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, *tokens* y componentes) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica) y que no hubo recarga. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, y la presencia de los módulos de componentes (incluidos modal, panel, toast y product-card) y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA, unhappy path de loading, error de campo y chip deshabilitado, los de overlay/toast de la tabla, y los de `vx-product-card` de la tabla.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan
- **AND** los de *tokens*, contraste, transparencia reducida y `base.css` pasan
- **AND** los de componentes (botón, campo, chip, modal, panel, toast, product-card) pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`

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
