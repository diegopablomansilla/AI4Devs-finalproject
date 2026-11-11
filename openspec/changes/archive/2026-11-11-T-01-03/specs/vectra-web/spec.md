# Spec Delta

Ticket padre: T-01-03. Change: T-01-03. HU: HU-01-02.

## ADDED Requirements

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

`vectra-web/dev/components.html` MUST existir, `lang="es"`, MUST enlazar `../css/tokens.css` y `../css/base.css`, MUST cargar `../js/components/index.js` como módulo y MUST incluir al menos un `vx-button`, un `vx-field` y un `vx-chip` en el documento.

#### Scenario: catálogo declara los tres tags (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** contiene las etiquetas `vx-button`, `vx-field` y `vx-chip`
- **AND** carga el módulo de componentes

#### Scenario: falta un tag en la muestra (unhappy)

- **WHEN** falta cualquiera de los tres tags en `components.html`
- **THEN** el harness termina con código distinto de 0

## MODIFIED Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/components/index.js`, `shared.js`, `vx-button.js`, `vx-field.js`, `vx-chip.js`
- `vectra-web/dev/components.html`

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module y MUST importar `js/components/index.js` para registrar `vx-button`, `vx-field` y `vx-chip`. Este change MUST NOT agregar modal, toast ni product-card.

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
- **AND** existen `js/components/vx-button.js`, `vx-field.js`, `vx-chip.js`, `shared.js`, `index.js` y `dev/components.html`

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

### Requirement: Tests del esqueleto y del router

Antes de archive, la suite MUST incluir estos tests (nombres estables; ubicación / tipo). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `index_html_declares_lang_es_and_versioned_module` | `scripts/check-vectra-web.sh` o fixture `scripts/fixtures/vectra-web/` | harness de árbol |
| `vectra_web_tree_includes_css_app_router_components` | `scripts/check-vectra-web.sh` | harness de árbol |
| `index_html_links_versioned_tokens_and_base` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_button_field_chip` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_html_catalog_loads_three_tags` | `scripts/fixtures/vectra-web/components.test.mjs` | catálogo |
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, *tokens* y componentes) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica) y que no hubo recarga. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, y la presencia de los módulos de componentes y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA y unhappy path de loading, error de campo y chip deshabilitado.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan
- **AND** los de *tokens*, contraste, transparencia reducida y `base.css` pasan
- **AND** los de componentes (botón, campo, chip) pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`
