# Spec Delta

Ticket padre: T-01-04. Change: T-01-04. HU: HU-01-02.

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/components/index.js`, `shared.js`, `vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`
- `vectra-web/dev/components.html`

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module y MUST importar `js/components/index.js` para registrar `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel` y `vx-toast`. Este change MUST NOT agregar `vx-product-card`.

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
- **AND** existen `js/components/vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `shared.js`, `index.js` y `dev/components.html`

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

### Requirement: Tests del esqueleto y del router

Antes de archive, la suite MUST incluir estos tests (nombres estables; ubicación / tipo). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `index_html_declares_lang_es_and_versioned_module` | `scripts/check-vectra-web.sh` o fixture `scripts/fixtures/vectra-web/` | harness de árbol |
| `vectra_web_tree_includes_css_app_router_components` | `scripts/check-vectra-web.sh` | harness de árbol |
| `index_html_links_versioned_tokens_and_base` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_button_field_chip` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_modal_panel_toast` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_html_catalog_loads_three_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
| `components_html_catalog_loads_overlay_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, *tokens* y componentes) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica) y que no hubo recarga. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, y la presencia de los módulos de componentes (incluidos modal, panel y toast) y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA, unhappy path de loading, error de campo y chip deshabilitado, y los de overlay/toast de la tabla.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan
- **AND** los de *tokens*, contraste, transparencia reducida y `base.css` pasan
- **AND** los de componentes (botón, campo, chip, modal, panel, toast) pasan

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

`vectra-web/dev/components.html` MUST existir, `lang="es"`, MUST enlazar `../css/tokens.css` y `../css/base.css`, MUST cargar `../js/components/index.js` como módulo y MUST incluir al menos un `vx-button`, un `vx-field`, un `vx-chip`, un `vx-modal`, un `vx-side-panel` y un `vx-toast`. MUST incluir un control que abra el panel de muestra de modo que, al cerrarlo, el foco vuelva a ese control.

#### Scenario: catálogo declara los tres tags (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene las etiquetas `vx-button`, `vx-field` y `vx-chip`
- **AND** carga el módulo de componentes

#### Scenario: catálogo declara overlays (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene las etiquetas `vx-modal`, `vx-side-panel` y `vx-toast`
- **AND** hay un control asociado a abrir el panel

#### Scenario: falta un tag en la muestra (unhappy)

- **WHEN** falta cualquiera de los tags `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel` o `vx-toast` en `components.html`
- **THEN** el harness termina con código distinto de 0
