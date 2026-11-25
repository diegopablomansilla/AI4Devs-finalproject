# vectra-web Specification

## Purpose
Define el esqueleto de la SPA Vectra en `vectra-web/`: documento HTML en español, módulos ES sin paso de *build*, *router* por *hash* entre la vista del operador y la de administración, una vista de ruta no encontrada, y el tema oscuro Glassmorphism (*design tokens* y CSS base) de modo que cambiar `#/` por `#/admin/productos` actualice el contenido sin recargar la página y los pares texto/superficie cumplan contraste WCAG AA sobre `#0A0A0A`.

## Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/manifest.webmanifest`
- `vectra-web/sw.js`
- `vectra-web/icons/icon-192.png`, `icon-512.png`, `icon-192-maskable.png`, `icon-512-maskable.png`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/api.js`
- `vectra-web/js/errors.js`
- `vectra-web/js/keyboard-list.js`
- `vectra-web/js/search.js`
- `vectra-web/js/highlight.js`
- `vectra-web/js/health.js`
- `vectra-web/js/sw-update.js`
- `vectra-web/js/components/index.js`, `shared.js`, `vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`, `vx-omnibox.js`
- `vectra-web/dev/components.html`

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST enlazar el *manifest* con `href="/manifest.webmanifest"` y MUST declarar `theme-color` `#0A0A0A` (requirement «Manifest PWA de Vectra»). MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module, MUST importar `js/components/index.js` para registrar `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel`, `vx-toast`, `vx-product-card` y `vx-omnibox`, MUST importar el montaje de búsqueda de `js/search.js` y pasarlo al *router* (requirement «Búsqueda mientras se escribe en el Omnibox»), MUST arrancar el *poller* de `js/health.js`, MUST registrar el Service Worker (requirement «Service Worker del app shell»), y MUST arrancar el *watcher* de actualización (requirement «Actualización del Service Worker»). MUST NOT exigir que `app.js` importe `errors.js` ni `keyboard-list.js` (`showError` ya se inyecta; el listbox lo usa el módulo de búsqueda). `js/search.js` MUST importar `js/highlight.js`.

#### Scenario: el documento está en español y carga el módulo versionado (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** el elemento `html` tiene `lang="es"`
- **AND** existe un script `type="module"` cuyo `src` es `/static/{{VERSION}}/js/app.js`
- **AND** existe el *outlet* de la aplicación
- **AND** el archivo no referencia un *bundler* ni un `package.json`

#### Scenario: el árbol incluye css, app, router y components (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existen `css/` y `js/components/` como directorios
- **AND** existen los archivos `js/app.js`, `js/router.js`, `js/api.js`, `js/errors.js`, `js/keyboard-list.js`, `js/search.js`, `js/highlight.js`, `js/health.js` y `js/sw-update.js`
- **AND** existen `css/tokens.css` y `css/base.css`
- **AND** existen `js/components/vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`, `vx-omnibox.js`, `shared.js`, `index.js` y `dev/components.html`

#### Scenario: index.html enlaza tokens y base versionados (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** hay un `link` cuyo `href` es `/static/{{VERSION}}/css/tokens.css`
- **AND** hay un `link` cuyo `href` es `/static/{{VERSION}}/css/base.css`
- **AND** el `link` de `tokens.css` aparece antes que el de `base.css`

#### Scenario: el árbol incluye manifest e íconos (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existe `manifest.webmanifest`
- **AND** existen `icons/icon-192.png`, `icons/icon-512.png`, `icons/icon-192-maskable.png` e `icons/icon-512-maskable.png`

#### Scenario: el árbol incluye sw.js (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existe `sw.js`

#### Scenario: el árbol incluye sw-update.js (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existe `js/sw-update.js`

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

#### Scenario: falta keyboard-list.js (unhappy)

- **WHEN** falta `vectra-web/js/keyboard-list.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `keyboard-list.js`

#### Scenario: falta health.js (unhappy)

- **WHEN** falta `vectra-web/js/health.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `health.js`

#### Scenario: falta manifest.webmanifest (unhappy)

- **WHEN** falta `vectra-web/manifest.webmanifest`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `manifest.webmanifest`

#### Scenario: falta sw.js (unhappy)

- **WHEN** falta `vectra-web/sw.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `sw.js`

#### Scenario: falta sw-update.js (unhappy)

- **WHEN** falta `vectra-web/js/sw-update.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `sw-update.js`

#### Scenario: falta search.js (unhappy)

- **WHEN** falta `vectra-web/js/search.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `search.js`

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
| `index_html_links_unversioned_manifest` | `scripts/check-vectra-web.sh` | harness de árbol |
| `index_html_declares_theme_color` | `scripts/check-vectra-web.sh` | harness de árbol |
| `manifest_tree_includes_webmanifest` | `scripts/check-vectra-web.sh` | harness de árbol |
| `manifest_tree_includes_png_icons` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_button_field_chip` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_modal_panel_toast` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_product_card` | `scripts/check-vectra-web.sh` | harness de árbol |
| `components_tree_includes_omnibox` | `scripts/check-vectra-web.sh` | harness de árbol |
| `keyboard_list_tree_includes_module` | `scripts/check-vectra-web.sh` | harness de árbol |
| `api_tree_includes_api_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `errors_tree_includes_errors_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `health_tree_includes_health_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `sw_tree_includes_sw_js` | `scripts/check-vectra-web.sh` | harness de árbol |
| `sw_update_tree_includes_module` | `scripts/check-vectra-web.sh` | harness de árbol |
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
| `router_operator_header_has_data_health` | `router.test.mjs` | layout operador |
| `router_admin_view_has_no_omnibox` | `scripts/fixtures/vectra-web/router.test.mjs` | unhappy / frontera |
| `router_admin_layout_has_menu_and_back_link` | `scripts/fixtures/vectra-web/router.test.mjs` | layout admin (Listo cuando T-01-11) |
| `router_admin_menu_lists_eight_sections` | `router.test.mjs` | layout admin |
| `router_admin_nav_to_categorias_does_not_reload` | `router.test.mjs` | layout admin |
| `router_admin_back_to_operator_in_one_step` | `router.test.mjs` | layout admin (Listo cuando T-01-11) |
| `router_admin_current_section_has_aria_current` | `router.test.mjs` | layout admin |
| `router_admin_header_has_data_health` | `router.test.mjs` | layout admin |
| `router_operator_header_has_offline_banner` | `router.test.mjs` | layout operador |
| `router_admin_header_has_offline_banner` | `router.test.mjs` | layout admin |
| `router_operator_header_has_update_banner` | `router.test.mjs` | layout operador |
| `router_admin_header_has_update_banner` | `router.test.mjs` | layout admin |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (No hace T-01-11) |
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
| `omnibox_ai_unavailable_disables_ai_and_ignores_prefix` | `omnibox.test.mjs` | unhappy (Listo cuando T-01-12) |
| `omnibox_voice_unavailable_disables_voice` | `omnibox.test.mjs` | unhappy |
| `omnibox_ai_unavailable_click_stays_text` | `omnibox.test.mjs` | unhappy |
| `health_maps_llm_down_to_ai_false` | `scripts/fixtures/vectra-web/health.test.mjs` | happy (Listo cuando T-01-12) |
| `health_llm_down_disables_ai_mode_text_stays` | `health.test.mjs` | happy (Listo cuando T-01-12) |
| `health_polls_every_10s` | `health.test.mjs` | happy |
| `health_refetches_on_focus` | `health.test.mjs` | happy |
| `health_indicator_shows_degraded` | `health.test.mjs` | happy |
| `health_speech_not_up_disables_voice` | `health.test.mjs` | happy |
| `health_vision_absent_hides_image` | `health.test.mjs` | happy |
| `health_network_failure_disables_optional_keeps_text` | `health.test.mjs` | unhappy |
| `health_503_starting_parses_snapshot` | `health.test.mjs` | unhappy |
| `health_invalid_body_does_not_throw` | `health.test.mjs` | unhappy |
| `health_does_not_show_reduced_search_banner` | `health.test.mjs` | unhappy (No hace) |
| `health_network_failure_shows_offline_banner` | `health.test.mjs` | happy (Listo cuando T-01-14) |
| `health_reachable_hides_offline_banner` | `health.test.mjs` | happy |
| `health_degraded_does_not_show_offline_banner` | `health.test.mjs` | unhappy |
| `manifest_json_is_valid` | `scripts/fixtures/vectra-web/manifest.test.mjs` | happy |
| `manifest_declares_vectra_standalone_and_colors` | `manifest.test.mjs` | happy (Listo cuando T-01-13) |
| `manifest_declares_192_512_and_maskable_icons` | `manifest.test.mjs` | happy (Listo cuando T-01-13) |
| `manifest_icon_files_match_declared_sizes` | `manifest.test.mjs` | happy |
| `manifest_invalid_json_fails` | `manifest.test.mjs` | unhappy (fixture) |
| `manifest_without_standalone_fails` | `manifest.test.mjs` | unhappy (fixture) |
| `manifest_without_maskable_icon_fails` | `manifest.test.mjs` | unhappy (fixture) |
| `manifest_icon_src_file_missing_fails` | `manifest.test.mjs` | unhappy (fixture) |
| `manifest_wrong_icon_dimensions_fail` | `manifest.test.mjs` | unhappy (fixture) |
| `app_registers_service_worker` | `scripts/fixtures/vectra-web/sw.test.mjs` o `manifest.test.mjs` | happy |
| `sw_precache_is_versioned_app_shell` | `scripts/fixtures/vectra-web/sw.test.mjs` | happy |
| `sw_precache_includes_sw_update` | `sw.test.mjs` | happy |
| `sw_navigation_online_uses_network` | `sw.test.mjs` | happy |
| `sw_navigation_offline_serves_cached_shell` | `sw.test.mjs` | happy (Listo cuando T-01-14) |
| `sw_activate_deletes_old_shell_caches` | `sw.test.mjs` | happy (Listo cuando T-01-15) |
| `sw_waiting_shows_update_banner` | `sw.test.mjs` | happy |
| `sw_reload_control_reloads_without_skip_waiting` | `sw.test.mjs` | happy (Listo cuando T-01-15) |
| `sw_health_and_api_are_network_only` | `sw.test.mjs` | unhappy (No hace) |
| `sw_precache_with_api_or_media_fails` | `sw.test.mjs` | unhappy (fixture) |
| `sw_register_without_service_worker_does_not_throw` | `sw.test.mjs` | unhappy |
| `sw_does_not_skip_waiting_or_claim` | `sw.test.mjs` | unhappy (No hace T-01-15) |
| `sw_without_waiting_hides_update_banner` | `sw.test.mjs` | unhappy |
| `sw_first_install_does_not_show_update_banner` | `sw.test.mjs` | unhappy |
| `sw_activate_keeps_current_cache` | `sw.test.mjs` | unhappy |
| `sw_update_check_failure_does_not_throw` | `sw.test.mjs` | unhappy |
| `sw_update_tree_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy |
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, admin, *tokens*, componentes, `api`, `errors`, `omnibox`, `keyboard-list`, `health`, `manifest` y `sw`) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión más `manifest_replaces_version` y `sw_js_replaces_version` en el crate `api`). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica), que no hubo recarga, que la vista operador declara las tres regiones con `vx-omnibox` / `vx-side-panel` mientras admin no monta `vx-omnibox`, que operador y admin declaran `[data-health]`, `[data-offline-banner]` y `[data-update-banner]` en el encabezado, y que la vista admin declara el menú de ocho secciones, el enlace `#/`, `aria-current` en la sección activa, `data-region="admin-content"` sin UI de negocio y el `vx-field` de terminal. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, el *link* `/manifest.webmanifest`, `theme-color` `#0A0A0A`, la presencia de `manifest.webmanifest` e íconos PNG, los módulos de componentes (incluidos modal, panel, toast, product-card y omnibox), `js/api.js`, `js/errors.js`, `js/keyboard-list.js`, `js/health.js`, `js/sw-update.js`, `sw.js` y `dev/components.html`. Los de `sw` MUST asertar precache versionado del *app shell* (incluido `sw-update.js`), navegación red-primero con respaldo en caché, `/health` y `/api/*` solo red, registro en `app.js`, `activate` que borra `vectra-shell-*` viejas, aviso «Nueva versión disponible» con Recargar, y los unhappy de precache con catálogo, `serviceWorker` ausente, falta de `sw.js` / `sw-update.js`, `skipWaiting` / `clients.claim`, primera instalación y `update()` que falla.

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
- **AND** los de `vx-omnibox` (layout, foco, Escape, modos, `vx-query`, `ai-unavailable`, unhappy) pasan
- **AND** los de `keyboard-list` (flechas, Enter, Escape al Omnibox, unhappy) pasan
- **AND** los de layout admin (menú de ocho secciones, vuelta al operador, `aria-current`, contenido vacío) pasan
- **AND** los de nombre de terminal (persistencia, > 64, vacío, storage que falla) pasan
- **AND** los de `health.js` (`llm: down`, poll 10 s, foco, red, 503, banner PA-04, sin banner de búsqueda reducida) pasan
- **AND** los de *manifest* (JSON, standalone, íconos 192/512 *maskable*) pasan
- **AND** los de `sw.js` (precache versionado, navegación offline, `/api` y `/health` solo red, registro, `activate`, aviso de versión, Recargar, sin `skipWaiting`) pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`

#### Scenario: falta sw.js (unhappy)

- **WHEN** falta `vectra-web/sw.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `sw.js`

#### Scenario: falta sw-update.js (unhappy)

- **WHEN** falta `vectra-web/js/sw-update.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `sw-update.js`

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

`vectra-web/dev/components.html` MUST existir, `lang="es"`, MUST enlazar `../css/tokens.css` y `../css/base.css`, MUST cargar `../js/components/index.js` como módulo y MUST incluir al menos un `vx-button`, un `vx-field`, un `vx-chip`, un `vx-modal`, un `vx-side-panel`, un `vx-toast`, un `vx-product-card` con `variant="full"` y un `vx-product-card` con `variant="compact"`, ambos con datos de ejemplo (SKU, nombre, precio, stock), y un `vx-omnibox`. MUST incluir un control que abra el panel de muestra de modo que, al cerrarlo, el foco vuelva a ese control **salvo** que haya un `vx-omnibox` conectado, en cuyo caso el foco MUST quedar en el Omnibox (Listo cuando T-01-08). Al menos una tarjeta de muestra MUST tener `stock="0"`. Al menos la variante `full` MUST incluir marca, ubicación e imagen o *placeholder*. MUST incluir una lista de ejemplo navegable por teclado (al menos dos ítems, p. ej. `vx-product-card`) asociada a `js/keyboard-list.js` (Listo cuando T-01-10: recorrer, abrir y cerrar sin mouse).

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

#### Scenario: catálogo declara lista navegable por teclado (happy)

- **WHEN** se lee `vectra-web/dev/components.html`
- **THEN** carga `../js/keyboard-list.js` (o un módulo que lo importe)
- **AND** hay un contenedor de lista de ejemplo con al menos dos ítems

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
2. Una región `data-region="results"` para la lista de búsqueda (requirement «Búsqueda mientras se escribe en el Omnibox»). Al pintar la vista, MUST estar vacía de hits y de `vx-product-card` (T-03-17). MUST NOT llamar a `/search` hasta un `vx-query` de modo texto con consulta no vacía.
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
- **AND** no hay `vx-omnibox` en el *outlet`

### Requirement: Layout de la vista administración

Cuando el *router* pinta `data-view="admin"` (`#/admin/<sección>` con slug `[a-z0-9-]+`), el *outlet* MUST incluir, en este orden visual:

1. Un encabezado con un enlace `<a href="#/">` cuyo texto visible está en español (p. ej. "Volver al operador"). Ese enlace MUST ser suficiente para volver a la vista operador **en un paso** (Listo cuando del ticket T-01-11): un único cambio de *hash* a `#/` pinta `data-view="operator"` sin recargar el documento.
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

3. Una región `data-region="admin-content"` reservada al contenido de la sección activa. Si `data-section="productos"`, MUST montar el listado del requirement «UI listado de productos en administración». Si `data-section="categorias"`, MUST montar el ABM del requirement «UI de categorías en administración». Si `data-section="marcas"`, MUST montar el ABM del requirement «UI de marcas en administración». Si `data-section="atributos"`, MUST montar el ABM del requirement «UI de atributos técnicos en administración». En cualquier otra sección del menú (y en un slug no listado) MUST seguir vacía de UI de negocio: MUST NOT contener `vx-product-card`, MUST NOT montar formularios de alta/edición de catálogo, MUST NOT llamar a `/api/v1/*` ni a `/health`. MAY mostrar un título o texto placeholder de la sección (español) sin datos.
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

- **WHEN** el hash es `#/admin/stock` (o `#/admin/relaciones`, u otra sección del menú distinta de `productos`, `categorias`, `marcas` y `atributos`)
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

### Requirement: Navegación por teclado de listas

MUST existir `vectra-web/js/keyboard-list.js` como ES Module. MUST exportar una función que asocia navegación por teclado a un contenedor de lista y un conjunto de ítems (no asume `vx-product-card`; T-03-15 y T-03-21 reutilizan el mismo contrato).

El contenedor MUST tener `role="listbox"` y MUST ser enfocable (`tabindex="0"` o equivalente). Cada ítem activo MUST tener `role="option"` y un `id` estable (el módulo MAY asignarlo si falta). El contenedor MUST exponer `aria-activedescendant` con el `id` de la opción activa, o MUST quitar el atributo cuando no hay selección. La opción activa MUST tener `aria-selected="true"`; las demás `"false"` o sin el atributo.

Con al menos un ítem:

- **ArrowDown** selecciona el siguiente (desde ninguna selección, el primero). Tras el último, MUST volver al primero.
- **ArrowUp** selecciona el anterior (desde ninguna selección, el último). Tras el primero, MUST volver al último.
- **Enter** MUST invocar el callback de abrir con el ítem activo y MUST NOT lanzar.
- **Escape** con el foco en la lista MUST devolver el foco al control de consulta del Omnibox (input en el shadow de `vx-omnibox`, o el elemento que el caller pase como restauración) y MUST NOT vaciar la consulta del Omnibox (T-01-08 limpia solo con el input enfocado). MUST NOT lanzar.

El módulo MUST escuchar `keydown` en la lista. MUST aceptar controladores extra (p. ej. el input del Omnibox) para **ArrowDown**, **ArrowUp** y **Enter** con el mismo efecto, de modo que se recorra la lista sin mouse mientras el Omnibox tiene el foco. MUST NOT manejar **Escape** en esos controladores extra (sigue valiendo T-01-08). MUST llamar `preventDefault` en las flechas y Enter que consume.

MUST NOT llamar a `/search` ni a otros endpoints. MUST NOT importar `api.js`. MUST NOT pintar la región `data-region="results"` del *router* (sigue vacía para T-03-15).

La función MUST devolver una función de limpieza que quita los listeners. MUST aceptar un `getItems` (o lista de nodos) evaluado en cada tecla, para que T-03-15 pueda reemplazar el DOM.

`dev/components.html` MUST incluir una lista de ejemplo (al menos dos ítems, p. ej. `vx-product-card`) cableada a este módulo y un `vx-omnibox`. Enter en un ítem de esa lista MUST abrir el `vx-side-panel` de muestra (o equivalente visible) con datos del ítem. Cerrar el panel (Escape con el panel abierto, T-01-04) MUST seguir devolviendo el foco al Omnibox (T-01-08). Ese recorrido + abrir + cerrar sin mouse es el **Listo cuando**.

#### Scenario: flechas recorren y aria-activedescendant sigue al activo (happy)

- **WHEN** una lista de ejemplo con al menos dos ítems está asociada al módulo
- **AND** se pulsa ArrowDown
- **THEN** el primer ítem queda activo (`aria-selected="true"`)
- **AND** `aria-activedescendant` del listbox es el `id` de ese ítem
- **AND** un segundo ArrowDown activa el siguiente ítem y actualiza `aria-activedescendant`

#### Scenario: Enter abre el ítem activo (happy, Listo cuando)

- **WHEN** hay un ítem activo
- **AND** se pulsa Enter (en la lista o en el input del Omnibox usado como controlador)
- **THEN** se invoca el callback de abrir con ese ítem
- **AND** en el catálogo, el panel de muestra queda abierto

#### Scenario: Escape en la lista vuelve al Omnibox sin borrar la consulta (happy, Listo cuando)

- **WHEN** la lista tiene el foco y el Omnibox tiene consulta no vacía
- **AND** se pulsa Escape
- **THEN** el foco queda en el input de consulta del Omnibox
- **AND** la consulta no queda vacía

#### Scenario: ArrowDown desde el Omnibox recorre la lista (happy)

- **WHEN** el input del Omnibox tiene el foco y la lista tiene ítems
- **AND** se pulsa ArrowDown
- **THEN** el primer ítem queda activo
- **AND** no se envía ninguna petición HTTP

#### Scenario: ítems que no son product-card también se recorren (happy)

- **WHEN** `getItems` devuelve elementos que no son `vx-product-card` (p. ej. opciones de texto, como las sugerencias de T-03-21)
- **AND** se pulsa ArrowDown y Enter
- **THEN** se activa el primero y se invoca el callback de abrir con ese nodo
- **AND** no se lanza excepción

#### Scenario: lista vacía no lanza (unhappy)

- **WHEN** `getItems` devuelve cero ítems
- **AND** se pulsa ArrowDown, ArrowUp, Enter o Escape
- **THEN** no se lanza excepción
- **AND** no se invoca el callback de abrir
- **AND** `aria-activedescendant` no apunta a un `id` inexistente

#### Scenario: Enter sin selección no abre (unhappy)

- **WHEN** la lista tiene ítems pero ninguno está activo
- **AND** se pulsa Enter
- **THEN** no se invoca el callback de abrir
- **AND** no se lanza excepción

#### Scenario: sin Omnibox Escape no lanza (unhappy)

- **WHEN** no hay función ni elemento de restauración de foco
- **AND** la lista tiene el foco y se pulsa Escape
- **THEN** no se lanza excepción

#### Scenario: no llama a search (unhappy)

- **WHEN** se recorre, se abre o se pulsa Escape en la lista de ejemplo
- **THEN** no se envía ninguna petición HTTP
- **AND** no se importa `api.js`

#### Scenario: falta keyboard-list.js (unhappy)

- **WHEN** falta `vectra-web/js/keyboard-list.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `keyboard-list.js`

### Requirement: Poller de /health y capacidades

MUST existir `vectra-web/js/health.js` como ES Module. MUST consultar `GET /health` (mismo origen, path absoluto `/health`) al arrancar, **cada 10 segundos** y al recuperar el foco de la ventana (`focus` en `window` y/o `visibilitychange` a `visible`). MUST NOT exigir `package.json` ni un *bundler*.

`app.js` MUST arrancar el *poller* una vez (junto al *router*). El *router* MUST NOT iniciar el *poller* al pintar una vista: pintar `#/admin/…` sin haber arrancado `health.js` MUST NOT enviar HTTP (sigue valiendo `router_admin_does_not_call_api`).

Cada snapshot MUST publicarse como evento `vx-health` (`bubbles`, `composed`) en `document` con `detail` `{ status, capabilities }` donde `capabilities` es `{ textSearch, ai, voice, vision }` (booleanos) y `status` es el agregado del cuerpo (`ok` | `degraded` | `starting` | `down`) o `"unknown"` si no hay snapshot usable.

Mapeo desde el esquema `Health` de `docs/openapi.yaml` (el cliente MUST ignorar campos desconocidos):

| Capacidad | Condición |
|---|---|
| `ai` | `components.llm.status === "up"` |
| `voice` | `components.speech.status === "up"` |
| `vision` | `components.vision.status === "up"` si el campo existe; si falta, `false` |
| `textSearch` | `true` en este change (MUST NOT deshabilitar el modo texto; T-03-31 cubre Meilisearch degradado) |

HTTP **200** (`ok` / `degraded`) y **503** (`starting` / `down`) MUST parsearse como JSON `Health` (no son `problem+json`). Un fallo de red, abort, cuerpo no JSON o JSON sin `components` MUST producir `status: "unknown"`, `ai`/`voice`/`vision` en `false`, `textSearch` en `true`, MUST NOT lanzar y MUST NOT mostrar el aviso "Búsqueda en modo reducido". Un fallo de red MUST mostrar el aviso del requirement «Aviso sin conexión con el Nodo Maestro»; un cuerpo inválido o 503 parseable MUST NOT mostrar ese aviso.

Tras cada snapshot (y tras un cambio de *hash*, porque el *outlet* se reemplaza), MUST aplicar el snapshot al indicador `[data-health]`, al banner `[data-offline-banner]` y al `vx-omnibox` visible si existe.

MUST NOT llamar a `/search`, `/api/v1/*`, ni mutar OpenAPI. MUST NOT pintar toasts de `problem+json` (T-01-07).

#### Scenario: llm down deshabilita IA y deja el texto (happy, Listo cuando)

- **WHEN** `GET /health` responde 200 con `status: "degraded"` y `components.llm.status` `"down"` (resto `up` o `disabled`)
- **THEN** `capabilities.ai` es `false` y `capabilities.textSearch` es `true`
- **AND** el control **IA** del Omnibox está deshabilitado
- **AND** el control **Texto** sigue operable (no `disabled`)
- **AND** el indicador `[data-health]` muestra estado degradado en español

#### Scenario: consulta cada 10 s (happy)

- **WHEN** el *poller* arranca
- **THEN** hay una petición `GET /health` inmediata
- **AND** el intervalo de repetición es 10 000 ms

#### Scenario: al recuperar el foco vuelve a consultar (happy)

- **WHEN** el *poller* está arrancado
- **AND** la ventana recibe `focus` o el documento pasa a `visibilityState === "visible"`
- **THEN** se envía `GET /health` (además del intervalo)

#### Scenario: speech no up deshabilita voz (happy)

- **WHEN** el snapshot tiene `components.speech.status` distinto de `"up"` (`down`, `disabled`, `starting` o `degraded`)
- **THEN** `capabilities.voice` es `false`
- **AND** el control **Voz** del Omnibox está deshabilitado

#### Scenario: vision ausente no muestra Imagen (happy)

- **WHEN** el JSON de `/health` no incluye `components.vision` (contrato actual T-00-13)
- **THEN** `capabilities.vision` es `false`
- **AND** el `vx-omnibox` no tiene atributo `vision` (modo Imagen oculto, T-01-09)

#### Scenario: fallo de red no tira y deja el texto (unhappy)

- **WHEN** `GET /health` falla por red (`TypeError` / `code: "network"`)
- **THEN** no se lanza excepción
- **AND** `status` queda `"unknown"`
- **AND** `ai` y `voice` son `false` y `textSearch` es `true`
- **AND** el modo **Texto** sigue operable
- **AND** `[data-offline-banner]` está visible con el texto `Sin conexión con el Nodo Maestro`

#### Scenario: 503 starting se parsea como Health (unhappy)

- **WHEN** `GET /health` responde 503 con JSON `Health` (`status: "starting"`, `components.llm.status` `"starting"`)
- **THEN** no se trata como `problem+json`
- **AND** `capabilities.ai` es `false`
- **AND** el indicador muestra arranque en español
- **AND** no se lanza excepción
- **AND** `[data-offline-banner]` no está visible

#### Scenario: cuerpo inválido no tira (unhappy)

- **WHEN** `GET /health` responde 200 con HTML o JSON sin `components`
- **THEN** no se lanza excepción
- **AND** `status` queda `"unknown"`
- **AND** el modo **Texto** sigue operable

#### Scenario: no muestra aviso de búsqueda reducida (unhappy, No hace)

- **WHEN** el snapshot tiene `components.meilisearch.status` `"down"` (agregado `degraded`)
- **THEN** no hay texto visible "Búsqueda en modo reducido"
- **AND** el modo **Texto** sigue operable
- **AND** `[data-offline-banner]` no está visible

### Requirement: Indicador de estado en el encabezado

Cuando el *router* pinta `data-view="operator"` o `data-view="admin"`, el encabezado MUST incluir un elemento con atributo `data-health`. Ese elemento MUST tener `aria-live="polite"` (o un descendiente live region). El texto visible MUST estar en español y MUST reflejar el último `status` publicado (`ok`, `degraded`, `starting`, `down`, `unknown`). MUST NOT indicar el estado solo con color. MUST NOT usar *tokens* nuevos (clases con `--vx-*` existentes).

La vista `not-found` MAY omitir el indicador.

#### Scenario: operador y admin tienen data-health (happy)

- **WHEN** se pinta `#/` o `#/admin/productos`
- **THEN** existe `[data-health]` dentro del encabezado de esa vista
- **AND** tiene (él o un descendiente) `aria-live="polite"`

#### Scenario: el indicador no es solo color (unhappy)

- **WHEN** el último `status` es `"degraded"`
- **AND** `[data-health]` no contiene texto en español que distinga el estado
- **THEN** el test del indicador termina con código distinto de 0

### Requirement: Modos IA y voz deshabilitados por capacidades

`vx-omnibox` MUST observar atributos booleanos `ai-unavailable` y `voice-unavailable` (además de `vision` de T-01-09). Si el atributo está **ausente**, el comportamiento de «Modos del Omnibox» no cambia (los tests T-01-09 siguen verdes).

Con `ai-unavailable` presente:

- El control **IA** MUST estar `disabled` y MUST exponer `aria-disabled="true"` (o el `disabled` nativo).
- Un clic en **IA** MUST NOT cambiar el modo ni emitir `vx-query` con `mode: "ai"`.
- Si el `value` empieza por `?`, el modo efectivo MUST ser `"text"` (el prefijo ya no gana).
- Si el modo efectivo era `"ai"`, MUST pasar a `"text"`.

Con `voice-unavailable` presente:

- El control **Voz** MUST estar `disabled`.
- Un clic en **Voz** MUST NOT dejar `mode` en `"voice"`.
- Si el modo era `"voice"`, MUST pasar a `"text"`.

El control **Texto** MUST permanecer habilitado. MUST NOT llamar a `/search` ni importar `api.js`.

El *poller* MUST setear o quitar esos atributos (y `vision`) en el `vx-omnibox` del *outlet* según `capabilities`.

#### Scenario: ai-unavailable ignora el prefijo y deshabilita IA (unhappy / Listo cuando)

- **WHEN** un `vx-omnibox` tiene `ai-unavailable`
- **AND** el operador deja el input en `?hola`
- **THEN** el control **IA** está deshabilitado
- **AND** `mode` del host es `"text"`
- **AND** si se dispara `vx-query`, `detail.mode` no es `"ai"`

#### Scenario: voice-unavailable no se puede elegir (unhappy)

- **WHEN** un `vx-omnibox` tiene `voice-unavailable`
- **AND** el operador activa el control **Voz**
- **THEN** `mode` no queda `"voice"`
- **AND** el control **Voz** está deshabilitado
- **AND** no se lanza excepción

#### Scenario: clic en IA deshabilitada no cambia el modo (unhappy)

- **WHEN** un `vx-omnibox` tiene `ai-unavailable` y modo `"text"`
- **AND** se hace clic en **IA**
- **THEN** `mode` sigue `"text"`
- **AND** no se envía ninguna petición HTTP

### Requirement: Manifest PWA de Vectra

MUST existir `vectra-web/manifest.webmanifest` como JSON UTF-8 válido (Web App Manifest). MUST NOT exigir `package.json` ni un *bundler*.

El documento MUST declarar:

| Campo | Valor |
|---|---|
| `name` | `Vectra` |
| `short_name` | `Vectra` |
| `display` | `standalone` |
| `background_color` | `#0A0A0A` |
| `theme_color` | `#0A0A0A` |
| `start_url` | `/` |
| `scope` | `/` |
| `lang` | `es` |

`index.html` MUST incluir `<link rel="manifest" href="/manifest.webmanifest">`. Ese `href` MUST ser exactamente `/manifest.webmanifest` (ruta de *app shell* de T-00-19, `Cache-Control: no-cache`). MUST NOT versionarlo bajo `/static/{{VERSION}}/`. MUST incluir `<meta name="theme-color" content="#0A0A0A">`.

Los `src` de `icons` MUST usar el placeholder literal `{{VERSION}}` (el Gateway lo sustituye al servir el *manifest*). El registro del Service Worker lo define el requirement «Service Worker del app shell» (T-01-14). MUST NOT invocar Lighthouse CLI ni agregar npm.

Los criterios Lighthouse de *manifest* (nombre, `display` standalone, colores, íconos 192/512 *maskable*, *link*) más el Service Worker de este change cierran el sello «instalable». La suite automatiza esos criterios sin Lighthouse CLI.

#### Scenario: el manifest declara Vectra standalone y colores (happy, Listo cuando)

- **WHEN** se parsea `vectra-web/manifest.webmanifest` como JSON
- **THEN** `name` y `short_name` son `"Vectra"`
- **AND** `display` es `"standalone"`
- **AND** `background_color` y `theme_color` son `"#0A0A0A"`
- **AND** `start_url` es `"/"`
- **AND** `lang` es `"es"`

#### Scenario: index.html enlaza el manifest y el theme-color (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** hay un `link` con `rel="manifest"` y `href="/manifest.webmanifest"`
- **AND** hay un `meta` con `name="theme-color"` y `content="#0A0A0A"`
- **AND** el `href` del *manifest* no contiene `/static/` ni `{{VERSION}}`

#### Scenario: JSON inválido falla (unhappy)

- **WHEN** `manifest.webmanifest` no es JSON parseable
- **THEN** el test del *manifest* termina con código distinto de 0
- **AND** el mensaje indica que el JSON es inválido

#### Scenario: sin display standalone falla (unhappy)

- **WHEN** el JSON no tiene `display` igual a `"standalone"`
- **THEN** el test termina con código distinto de 0
- **AND** el mensaje nombra `display`

#### Scenario: index.html sin link al manifest falla (unhappy)

- **WHEN** `index.html` no tiene `link rel="manifest"` hacia `/manifest.webmanifest`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra el *manifest* o el `href` esperado

#### Scenario: no registra Service Worker (unhappy, No hace)

T-01-13 prohibía el SW; T-01-14 lo exige (mismo escenario, THEN invertido). El detalle está en el requirement «Service Worker del app shell».

- **WHEN** se leen `vectra-web/index.html` y `vectra-web/js/app.js`
- **THEN** `js/app.js` contiene `serviceWorker.register`
- **AND** existe `vectra-web/sw.js`

### Requirement: Íconos PNG 192/512 y maskable

`vectra-web/manifest.webmanifest` MUST incluir un array `icons` con al menos:

| Recurso en disco | `sizes` | `type` | `purpose` |
|---|---|---|---|
| `vectra-web/icons/icon-192.png` | `192x192` | `image/png` | `any` |
| `vectra-web/icons/icon-512.png` | `512x512` | `image/png` | `any` |
| `vectra-web/icons/icon-192-maskable.png` | `192x192` | `image/png` | `maskable` |
| `vectra-web/icons/icon-512-maskable.png` | `512x512` | `image/png` | `maskable` |

Cada entrada MUST tener `src` `/static/{{VERSION}}/icons/<archivo>` coincidente con el archivo en disco. Cada PNG MUST ser un PNG válido cuya cabecera IHDR declare exactamente el ancho y alto de `sizes`. MUST NOT usar data URI ni SVG como único ícono de instalación. MUST NOT exigir un archivo con `purpose` que mezcle `any maskable` en un solo PNG (Lighthouse pide *maskable* con zona segura aparte).

#### Scenario: hay íconos 192, 512 y maskable (happy, Listo cuando)

- **WHEN** se parsea `icons` del *manifest*
- **THEN** existe una entrada `192x192` con `purpose` que incluye `any`
- **AND** existe una entrada `512x512` con `purpose` que incluye `any`
- **AND** existe una entrada `192x192` con `purpose` que incluye `maskable`
- **AND** existe una entrada `512x512` con `purpose` que incluye `maskable`
- **AND** cada `src` empieza por `/static/{{VERSION}}/icons/` y termina en `.png`

#### Scenario: los PNG coinciden con sizes (happy)

- **WHEN** se lee la cabecera IHDR de cada archivo declarado en `icons`
- **THEN** ancho y alto coinciden con el `sizes` de esa entrada
- **AND** el archivo existe bajo `vectra-web/icons/`

#### Scenario: falta un ícono maskable (unhappy)

- **WHEN** el array `icons` no tiene ninguna entrada cuyo `purpose` incluya `maskable`
- **THEN** el test termina con código distinto de 0
- **AND** el mensaje nombra `maskable`

#### Scenario: el src no apunta a un archivo (unhappy)

- **WHEN** un `src` de `icons` no corresponde a un archivo regular bajo `vectra-web/icons/`
- **THEN** el test termina con código distinto de 0
- **AND** el mensaje nombra el `src` o el archivo que falta

#### Scenario: dimensiones distintas de sizes fallan (unhappy)

- **WHEN** un PNG declarado como `192x192` o `512x512` tiene otro ancho o alto en IHDR
- **THEN** el test termina con código distinto de 0
- **AND** el mensaje nombra el archivo y las dimensiones obtenidas

### Requirement: Service Worker del app shell

MUST existir `vectra-web/sw.js` como script clásico (MUST NOT ser ES Module). MUST NOT exigir `package.json` ni un *bundler*. El documento MUST contener el placeholder literal `{{VERSION}}` en el nombre de caché y en las URLs versionadas del precache (el Gateway lo sustituye al servir `GET /sw.js`).

El nombre de caché MUST ser `vectra-shell-{{VERSION}}`. En `install`, el worker MUST precachear al menos:

- `/`
- `/index.html`
- `/manifest.webmanifest`
- `/static/{{VERSION}}/css/tokens.css`
- `/static/{{VERSION}}/css/base.css`
- `/static/{{VERSION}}/js/app.js`
- `/static/{{VERSION}}/js/router.js`
- `/static/{{VERSION}}/js/api.js`
- `/static/{{VERSION}}/js/errors.js`
- `/static/{{VERSION}}/js/keyboard-list.js`
- `/static/{{VERSION}}/js/health.js`
- `/static/{{VERSION}}/js/sw-update.js`
- `/static/{{VERSION}}/js/components/index.js`
- `/static/{{VERSION}}/js/components/shared.js`
- `/static/{{VERSION}}/js/components/overlay.js`
- `/static/{{VERSION}}/js/components/vx-button.js`
- `/static/{{VERSION}}/js/components/vx-field.js`
- `/static/{{VERSION}}/js/components/vx-chip.js`
- `/static/{{VERSION}}/js/components/vx-modal.js`
- `/static/{{VERSION}}/js/components/vx-side-panel.js`
- `/static/{{VERSION}}/js/components/vx-toast.js`
- `/static/{{VERSION}}/js/components/vx-product-card.js`
- `/static/{{VERSION}}/js/components/vx-omnibox.js`
- `/static/{{VERSION}}/icons/icon-192.png`
- `/static/{{VERSION}}/icons/icon-512.png`
- `/static/{{VERSION}}/icons/icon-192-maskable.png`
- `/static/{{VERSION}}/icons/icon-512-maskable.png`

La lista de precache MUST NOT incluir paths bajo `/api/`, `/health`, `/media/` ni `dev/components.html`.

En `fetch`, el worker MUST aplicar:

| Request | Estrategia |
|---|---|
| `mode` `navigate` | red primero; si `fetch` falla, responder con la entrada cacheada de `/` o `/index.html` |
| pathname `/health` o que empieza por `/api/` | solo red; MUST NOT escribir la respuesta en caché |
| pathname que empieza por `/media/` | solo red; MUST NOT escribir la respuesta en caché |

MUST NOT llamar `skipWaiting` ni `clients.claim` (No hace de T-01-15: actualizaciones forzadas sin recargar). El ciclo de actualización (`activate`, aviso, Recargar) lo define el requirement «Actualización del Service Worker».

`js/app.js` MUST registrar el worker con `navigator.serviceWorker.register("/sw.js")` cuando `navigator.serviceWorker` existe. Si no existe, MUST NOT lanzar. MUST NOT registrar desde un script inline en `index.html`. MUST arrancar el *watcher* del requirement «Actualización del Service Worker».

#### Scenario: sw.js declara precache versionado del app shell (happy)

- **WHEN** se lee `vectra-web/sw.js`
- **THEN** el nombre de caché contiene `vectra-shell-{{VERSION}}`
- **AND** el precache incluye `/`, `/index.html`, `/manifest.webmanifest`, `css/tokens.css`, `js/app.js`, `js/sw-update.js` y al menos un ícono PNG bajo `/static/{{VERSION}}/`
- **AND** ninguna URL de precache contiene `/api/`, `/health` ni `/media/`

#### Scenario: navegación online usa la red (happy)

- **WHEN** el worker está instalado con el *app shell* en caché
- **AND** llega un `fetch` con `mode` `navigate` y `fetch` resuelve un HTML de red
- **THEN** la respuesta es ese HTML de red

#### Scenario: navegación sin red sirve el shell cacheado (happy, Listo cuando)

- **WHEN** el worker está instalado con `/` o `/index.html` en caché
- **AND** llega un `fetch` con `mode` `navigate` y `fetch` falla
- **THEN** la respuesta es el HTML cacheado del *app shell*

#### Scenario: app.js registra el Service Worker (happy)

- **WHEN** se lee `vectra-web/js/app.js`
- **THEN** aparece `serviceWorker.register` con argumento `"/sw.js"`
- **AND** existe `vectra-web/sw.js`

#### Scenario: /health y /api no tienen fallback de caché (unhappy, No hace)

- **WHEN** llega un `fetch` a `/health` o a `/api/v1/search` y `fetch` falla
- **THEN** la respuesta no proviene de Cache Storage
- **AND** no se escribe esa URL en la caché del *app shell*

#### Scenario: precache con /api o /media falla (unhappy)

- **WHEN** la lista de precache incluye un path bajo `/api/` o `/media/`
- **THEN** el test del Service Worker termina con código distinto de 0
- **AND** el mensaje nombra `/api/` o `/media/`

#### Scenario: sin serviceWorker no tira (unhappy)

- **WHEN** `navigator.serviceWorker` no está definido
- **AND** arranca `app.js` (o la función de registro)
- **THEN** no se lanza excepción

#### Scenario: falta sw.js (unhappy)

- **WHEN** falta `vectra-web/sw.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `sw.js`

### Requirement: Aviso sin conexión con el Nodo Maestro

Cuando el *router* pinta `data-view="operator"` o `data-view="admin"`, el encabezado MUST incluir un elemento con atributo `data-offline-banner`. Ese elemento MUST tener `aria-live="assertive"` (o un descendiente live region). El texto visible MUST ser exactamente `Sin conexión con el Nodo Maestro`. MUST NOT indicar el aviso solo con color. MUST NOT usar *tokens* nuevos (clases con `--vx-*` existentes). La vista `not-found` MAY omitir el banner.

Tras cada snapshot de `/health` (y tras un cambio de *hash*), si el `GET /health` falló por red (`TypeError` / fallo de `fetch`), el banner MUST estar visible. Si `/health` responde 200 o 503 con cuerpo parseable, el banner MUST estar oculto (`hidden` o equivalente no visible). Un snapshot `degraded` (p. ej. Meilisearch down) MUST NOT mostrar este aviso ni "Búsqueda en modo reducido".

`status` del *poller* MUST seguir siendo `"unknown"` ante fallo de red. El indicador `[data-health]` y el mapa `errors.js` (`network`) MUST NOT cambiar de copy en este change.

#### Scenario: operador y admin tienen data-offline-banner (happy)

- **WHEN** se pinta `#/` o `#/admin/productos`
- **THEN** existe `[data-offline-banner]` dentro del encabezado de esa vista
- **AND** tiene (él o un descendiente) `aria-live="assertive"`
- **AND** el texto visible es `Sin conexión con el Nodo Maestro`

#### Scenario: fallo de red muestra el aviso PA-04 (happy, Listo cuando)

- **WHEN** `GET /health` falla por red (`TypeError`)
- **THEN** `[data-offline-banner]` está visible
- **AND** su texto es `Sin conexión con el Nodo Maestro`
- **AND** `status` del snapshot sigue `"unknown"`
- **AND** el modo **Texto** sigue operable

#### Scenario: nodo alcanzable oculta el aviso (happy)

- **WHEN** `GET /health` responde 200 `ok` o 503 `starting`
- **THEN** `[data-offline-banner]` no está visible

#### Scenario: degradado no muestra el aviso PA-04 (unhappy)

- **WHEN** el snapshot tiene `status: "degraded"` y `components.meilisearch.status` `"down"`
- **THEN** `[data-offline-banner]` no está visible
- **AND** no hay texto visible "Búsqueda en modo reducido"

### Requirement: Actualización del Service Worker

MUST existir `vectra-web/js/sw-update.js` como ES Module. MUST NOT exigir `package.json` ni un *bundler*. `js/app.js` MUST arrancar el *watcher* de este módulo una vez (junto al registro del Service Worker). El *router* MUST NOT iniciar el *watcher*.

En `activate`, `vectra-web/sw.js` MUST borrar de Cache Storage las cachés cuyo nombre empieza por `vectra-shell-` y no es igual a `CACHE` (`vectra-shell-{{VERSION}}`). MUST NOT borrar la caché actual. MUST NOT borrar cachés cuyo nombre no empieza por `vectra-shell-`. MUST NOT llamar `self.skipWaiting()` ni `clients.claim()` (No hace del ticket).

Tras `navigator.serviceWorker.register("/sw.js")`, el *watcher* MUST:

- Si `registration.waiting` existe **y** hay un `navigator.serviceWorker.controller` (hay un worker controlando: es una actualización, no la primera instalación), mostrar el aviso.
- Escuchar `updatefound` en el registration: cuando el worker que instala pasa a `installed` y existe `controller`, mostrar el aviso.
- Llamar `registration.update()` al recuperar el foco de la ventana (`focus` en `window` y/o `visibilitychange` a `visible`).

MUST NOT invocar `location.reload()` ni `skipWaiting` solo por detectar *waiting*. Si `serviceWorker` no existe, `register` rechaza o `update()` falla, MUST NOT lanzar.

Cuando el *router* pinta `data-view="operator"` o `data-view="admin"`, el encabezado MUST incluir un elemento con atributo `data-update-banner`. Ese elemento MUST tener `aria-live="polite"` (o un descendiente live region). El texto visible MUST incluir exactamente `Nueva versión disponible`. MUST incluir un control operable por teclado con atributo `data-update-reload` y nombre accesible **Recargar**, con `min-height` y `min-width` de `var(--vx-touch-min)`. MUST NOT indicar el aviso solo con color. MUST NOT usar *tokens* nuevos. La vista `not-found` MAY omitir el banner.

El banner MUST estar oculto (`hidden` o equivalente) por defecto y cuando no hay worker en espera. MUST estar visible cuando el *watcher* detecta *waiting* con *controller*. Tras un cambio de *hash* (el *outlet* se reemplaza), el *watcher* MUST volver a aplicar visibilidad. Al activar **Recargar**, MUST llamar `location.reload()` y MUST NOT enviar `skipWaiting` ni `clients.claim`.

MUST NOT reutilizar `[data-offline-banner]` ni `vx-toast` para este aviso. MUST NOT cambiar el copy PA-04 ni el indicador `[data-health]`.

La lista de precache de `sw.js` MUST incluir `/static/{{VERSION}}/js/sw-update.js`.

#### Scenario: activate borra cachés viejas del shell (happy, Listo cuando)

- **WHEN** Cache Storage tiene `vectra-shell-old` y `vectra-shell-{{VERSION}}`
- **AND** el worker dispara `activate` con `CACHE` igual a `vectra-shell-{{VERSION}}`
- **THEN** `vectra-shell-old` ya no está en Cache Storage
- **AND** `vectra-shell-{{VERSION}}` sigue existiendo

#### Scenario: waiting con controller muestra el aviso (happy)

- **WHEN** hay un `controller` activo
- **AND** `registration.waiting` existe (o el worker de `updatefound` pasa a `installed`)
- **THEN** `[data-update-banner]` está visible
- **AND** su texto visible incluye `Nueva versión disponible`
- **AND** existe `[data-update-reload]` con nombre accesible Recargar

#### Scenario: Recargar recarga el documento (happy, Listo cuando)

- **WHEN** el aviso está visible
- **AND** se activa `[data-update-reload]`
- **THEN** se invoca `location.reload()`
- **AND** no se llama `skipWaiting` ni `clients.claim`

#### Scenario: operador y admin tienen data-update-banner (happy)

- **WHEN** se pinta `#/` o `#/admin/productos`
- **THEN** existe `[data-update-banner]` dentro del encabezado de esa vista
- **AND** tiene (él o un descendiente) `aria-live="polite"`
- **AND** el texto visible incluye `Nueva versión disponible`
- **AND** existe `[data-update-reload]`

#### Scenario: skipWaiting o clients.claim están prohibidos (unhappy, No hace)

- **WHEN** se lee `vectra-web/sw.js` y `vectra-web/js/sw-update.js`
- **THEN** no aparece `skipWaiting` ni `clients.claim`

#### Scenario: sin waiting el aviso queda oculto (unhappy)

- **WHEN** el registration no tiene `waiting` y no hay `updatefound` a `installed`
- **THEN** `[data-update-banner]` no está visible

#### Scenario: primera instalación no muestra el aviso (unhappy)

- **WHEN** `register("/sw.js")` resuelve un registration con `waiting` o `installing`
- **AND** no hay `navigator.serviceWorker.controller`
- **THEN** `[data-update-banner]` no está visible

#### Scenario: activate no borra la caché actual (unhappy)

- **WHEN** Cache Storage solo tiene `vectra-shell-{{VERSION}}`
- **AND** el worker dispara `activate`
- **THEN** esa caché sigue existiendo

#### Scenario: fallo de update no tira (unhappy)

- **WHEN** `registration.update()` rechaza o `navigator.serviceWorker` no existe
- **THEN** no se lanza excepción
- **AND** no se invoca `location.reload()`

### Requirement: Tests de actualización del Service Worker

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `sw_update_tree_includes_module` | `scripts/check-vectra-web.sh` | harness de árbol |
| `sw_precache_includes_sw_update` | `scripts/fixtures/vectra-web/sw.test.mjs` | happy |
| `sw_activate_deletes_old_shell_caches` | `sw.test.mjs` | happy (Listo cuando) |
| `sw_waiting_shows_update_banner` | `sw.test.mjs` | happy |
| `sw_reload_control_reloads_without_skip_waiting` | `sw.test.mjs` | happy (Listo cuando) |
| `router_operator_header_has_update_banner` | `scripts/fixtures/vectra-web/router.test.mjs` | layout operador |
| `router_admin_header_has_update_banner` | `router.test.mjs` | layout admin |
| `sw_does_not_skip_waiting_or_claim` | `sw.test.mjs` | unhappy (No hace) |
| `sw_without_waiting_hides_update_banner` | `sw.test.mjs` | unhappy |
| `sw_first_install_does_not_show_update_banner` | `sw.test.mjs` | unhappy |
| `sw_activate_keeps_current_cache` | `sw.test.mjs` | unhappy |
| `sw_update_check_failure_does_not_throw` | `sw.test.mjs` | unhappy |
| `sw_update_tree_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy |

`./scripts/check-vectra-web.sh` MUST ejecutar estos tests junto con la suite previa de T-01-14. MUST NOT agregar `package.json` ni Playwright. `cargo test --workspace` MUST seguir en verde (regresión; este change no agrega tests Rust).

#### Scenario: la suite de actualización cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `activate` (borra viejas, conserva actual) pasan
- **AND** los del aviso (waiting, Recargar, primera instalación, sin waiting) pasan
- **AND** `sw_does_not_skip_waiting_or_claim` pasa

#### Scenario: falta sw-update.js (unhappy)

- **WHEN** falta `vectra-web/js/sw-update.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `sw-update.js`

### Requirement: UI de categorías en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="categorias"` (`#/admin/categorias`), `data-region="admin-content"` MUST montar el ABM de categorías. La vista MUST cargar `GET /api/v1/categories?includeInactive=true` (lista plana `{ items }` de [docs/openapi.yaml](../../../../../docs/openapi.yaml)) y MUST armar un árbol anidado con `parentId` (`null` = raíz). Cada nodo MUST mostrar `name` y MAY mostrar `path` unido (p. ej. `Fijaciones > Pernos`). Una categoría con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactiva») y MUST ofrecer reactivar, no dar de baja.

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`):

- **Nueva categoría** (raíz): `POST /api/v1/categories` con `{ "name" }` y sin `parentId` (o `parentId: null`).
- **Crear subcategoría** en un nodo: `POST` con `{ "name", "parentId" }` igual al `id` del nodo (HU-02-01 «Crear subcategoría»).
- **Renombrar**: `PATCH /api/v1/categories/{categoryId}` con `{ "name" }` (HU-02-01 «Renombrar una rama»: el `path` de la rama lo recalcula el servidor; la UI MUST volver a listar y mostrar el `path` actualizado).
- **Mover**: selector de padre (`<select>` o equivalente), no arrastrar y soltar. `PATCH` con `{ "parentId" }` (`null` = raíz). El selector MUST incluir una opción «Raíz» y MUST omitir la categoría que se mueve y sus descendientes.
- **Dar de baja**: `DELETE /api/v1/categories/{categoryId}` sobre una categoría activa.
- **Reactivar**: `PATCH` con `{ "active": true }` sobre una inactiva.

MUST NOT enviar `slug` en create/update (el servidor lo genera desde `name`). MUST NOT pintar `draggable`, ni escuchar `dragstart` / `drop` en el árbol (**No hace**). MUST NOT llamar otros paths de `/api/v1` ni `/health` desde esta vista. Tras un `POST`/`PATCH`/`DELETE` exitoso, MUST volver a `GET` e hidratar el árbol. Un `name` vacío (solo espacios) MUST NOT enviarse.

Los errores MUST mostrarse con `showError` / `messageFor(code)` (requirement «Mensajes en español por ErrorCode»): el toast usa el `code` de `problem+json` (`category_cycle`, `duplicate_slug`, `conflict`, `invalid_input`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `base.css` MAY añadir clases del árbol solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Si no hay categorías, MUST mostrar el árbol vacío y la acción de crear raíz. Textos visibles MUST estar en español.

#### Scenario: crear subcategoría actualiza el árbol y el path (happy, HU-02-01)

- **WHEN** existe la categoría activa "Fijaciones"
- **AND** el administrador crea "Pernos" con padre "Fijaciones"
- **THEN** se envía `POST /api/v1/categories` con `name` `"Pernos"` y `parentId` el id de "Fijaciones"
- **AND** el árbol muestra "Pernos" anidada bajo "Fijaciones"
- **AND** el `path` visible de "Pernos" es `Fijaciones > Pernos` (o equivalente con los elementos `["Fijaciones", "Pernos"]`)

#### Scenario: crear categoría raíz en árbol vacío (happy)

- **WHEN** `GET` devuelve `{ "items": [] }`
- **THEN** la región muestra la acción de crear categoría (raíz)
- **AND** al confirmar el nombre `"Fijaciones"` se envía `POST` con `name` `"Fijaciones"` y sin `parentId` de otra categoría

#### Scenario: renombrar actualiza el path de la rama (happy, HU-02-01)

- **WHEN** existen "Fijaciones" y su hija "Pernos" (`path` `["Fijaciones", "Pernos"]`)
- **AND** el administrador renombra "Fijaciones" a "Tornillería"
- **THEN** se envía `PATCH` de "Fijaciones" con `{ "name": "Tornillería" }`
- **AND** tras recargar, el árbol muestra "Tornillería" y el `path` de "Pernos" incluye `"Tornillería"`

#### Scenario: mover con selector de padre (happy)

- **WHEN** "Pernos" es hija de "Fijaciones" y existe "Herramientas"
- **AND** el administrador elige "Herramientas" como nuevo padre de "Pernos"
- **THEN** se envía `PATCH` de "Pernos" con `{ "parentId" }` igual al id de "Herramientas"
- **AND** no se dispara ningún evento `dragstart` ni `drop`

#### Scenario: dar de baja y reactivar (happy)

- **WHEN** "Pernos" es una hoja activa
- **AND** el administrador la da de baja
- **THEN** se envía `DELETE /api/v1/categories/{id}`
- **AND** tras recargar, "Pernos" se muestra inactiva y ofrece reactivar
- **AND** al reactivar se envía `PATCH` con `{ "active": true }`

#### Scenario: el listado pide inactivas (happy)

- **WHEN** se pinta `#/admin/categorias` con un cliente HTTP inyectado
- **THEN** la primera petición es `GET /api/v1/categories?includeInactive=true`

#### Scenario: evitar ciclos muestra category_cycle (unhappy, HU-02-01)

- **WHEN** "Pernos" es hija de "Fijaciones"
- **AND** el servidor responde `400` `application/problem+json` con `code` `category_cycle` al mover
- **THEN** se muestra el mensaje en español de `category_cycle` (no el string `category_cycle` como texto del toast)
- **AND** el árbol no se reemplaza por un estado vacío

#### Scenario: slug duplicado muestra duplicate_slug (unhappy, HU-02-01)

- **WHEN** el servidor responde `409` con `code` `duplicate_slug` al crear
- **THEN** se muestra el mensaje en español de `duplicate_slug`

#### Scenario: baja con hija activa muestra conflict (unhappy, H-10)

- **WHEN** el servidor responde `409` con `code` `conflict` al dar de baja
- **THEN** se muestra el mensaje en español de `conflict`

#### Scenario: name vacío no llama a POST (unhappy)

- **WHEN** el administrador confirma crear con el nombre vacío o solo espacios
- **THEN** no se envía `POST /api/v1/categories`

#### Scenario: fallo de red muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `GET` inicial con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** no se lanza una excepción no capturada

#### Scenario: no hay arrastrar y soltar (unhappy, No hace)

- **WHEN** se inspecciona el árbol de categorías
- **THEN** ningún nodo tiene atributo `draggable="true"`
- **AND** no hay listeners de `dragstart` ni `drop` registrados por la vista

### Requirement: Tests de la UI de categorías

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (escenarios de HU-02-01 a mano, salvo búsqueda) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `categories_tree_builds_from_parent_id` | `scripts/fixtures/vectra-web/categories.test.mjs` | happy |
| `categories_create_subcategory_posts_name_and_parent` | `categories.test.mjs` | happy (HU-02-01) |
| `categories_create_root_posts_without_parent` | `categories.test.mjs` | happy |
| `categories_rename_patches_name` | `categories.test.mjs` | happy (HU-02-01) |
| `categories_move_patches_parent_id` | `categories.test.mjs` | happy |
| `categories_deactivate_deletes` | `categories.test.mjs` | happy |
| `categories_reactivate_patches_active_true` | `categories.test.mjs` | happy |
| `categories_list_requests_include_inactive` | `categories.test.mjs` | happy |
| `categories_empty_tree_offers_create_root` | `categories.test.mjs` | happy |
| `categories_cycle_shows_category_cycle` | `categories.test.mjs` | unhappy (HU-02-01) |
| `categories_duplicate_slug_shows_duplicate_slug` | `categories.test.mjs` | unhappy (HU-02-01) |
| `categories_deactivate_conflict_shows_conflict` | `categories.test.mjs` | unhappy (H-10) |
| `categories_network_shows_network` | `categories.test.mjs` | unhappy |
| `categories_empty_name_does_not_post` | `categories.test.mjs` | unhappy |
| `categories_no_drag_and_drop` | `categories.test.mjs` | unhappy (No hace) |
| `categories_parent_options_exclude_self_and_descendants` | `categories.test.mjs` | unhappy |
| `categories_tree_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (secciones que no son `categorias`) |
| `router_admin_does_not_call_api` | `router.test.mjs` | unhappy (`#/admin/productos` y `#/admin/stock`) |

`check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/categories.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de categorías cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `categories.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de categorías (unhappy)

- **WHEN** falta `vectra-web/js/admin/categories.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `categories.js`

### Requirement: UI de marcas en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="marcas"` (`#/admin/marcas`), `data-region="admin-content"` MUST montar el ABM de marcas. La vista MUST cargar `GET /api/v1/brands?includeInactive=true` (lista plana `{ items }` de [docs/openapi.yaml](../../../../../docs/openapi.yaml)). Cada ítem MUST mostrar `name` y MAY mostrar `slug` como texto secundario. Una marca con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactiva») y MUST ofrecer reactivar, no dar de baja.

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`):

- **Nueva marca**: `POST /api/v1/brands` con `{ "name" }` (HU-02-02 «Alta de marca»).
- **Editar**: `PATCH /api/v1/brands/{brandId}` con `{ "name" }`.
- **Dar de baja**: `DELETE /api/v1/brands/{brandId}` sobre una marca activa (HU-02-02 «Baja lógica»).
- **Reactivar**: `PATCH` con `{ "active": true }` sobre una inactiva.

MUST NOT enviar `slug` en create/update (el servidor lo genera desde `name`). MUST NOT pintar logos: MUST NOT haber `<input type="file">`, MUST NOT haber campo o control de logo, MUST NOT haber `<img>` de marca (**No hace**). MUST NOT llamar otros paths de `/api/v1` ni `/health` desde esta vista. MUST NOT usar el query `q`. Tras un `POST`/`PATCH`/`DELETE` exitoso, MUST volver a `GET` e hidratar el listado. Un `name` vacío (solo espacios) MUST NOT enviarse.

La baja MUST enviarse aunque el administrador no tenga productos (T-02-09 permite occupancy; la UI MUST NOT consultar `producto` ni bloquear la baja). Los errores MUST mostrarse con `showError` / `messageFor(code)` (requirement «Mensajes en español por ErrorCode»): el toast usa el `code` de `problem+json` (`duplicate_slug`, `invalid_input`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `base.css` MAY añadir clases del listado solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Si no hay marcas, MUST mostrar el listado vacío y la acción de crear. Textos visibles MUST estar en español.

#### Scenario: alta de Fixser actualiza el listado (happy, HU-02-02)

- **WHEN** el administrador crea la marca `"Fixser"`
- **THEN** se envía `POST /api/v1/brands` con `name` `"Fixser"` y sin `slug`
- **AND** tras recargar, el listado muestra `"Fixser"` activa

#### Scenario: crear marca en listado vacío (happy)

- **WHEN** `GET` devuelve `{ "items": [] }`
- **THEN** la región muestra la acción de crear marca
- **AND** al confirmar el nombre `"Fixser"` se envía `POST` con `name` `"Fixser"`

#### Scenario: editar nombre (happy)

- **WHEN** existe `"Fixser"` activa
- **AND** el administrador la renombra a `"Fixser Pro"`
- **THEN** se envía `PATCH /api/v1/brands/{id}` con `{ "name": "Fixser Pro" }`
- **AND** tras recargar, el listado muestra `"Fixser Pro"`

#### Scenario: dar de baja y reactivar (happy, HU-02-02)

- **WHEN** `"Fixser"` es una marca activa
- **AND** el administrador la da de baja
- **THEN** se envía `DELETE /api/v1/brands/{id}`
- **AND** tras recargar, `"Fixser"` se muestra inactiva y ofrece reactivar
- **AND** al reactivar se envía `PATCH` con `{ "active": true }`

#### Scenario: el listado pide inactivas (happy)

- **WHEN** se pinta `#/admin/marcas` con un cliente HTTP inyectado
- **THEN** la primera petición es `GET /api/v1/brands?includeInactive=true`

#### Scenario: slug duplicado muestra duplicate_slug (unhappy)

- **WHEN** el servidor responde `409` con `code` `duplicate_slug` al crear
- **THEN** se muestra el mensaje en español de `duplicate_slug`

#### Scenario: id desconocido muestra not_found (unhappy)

- **WHEN** el servidor responde `404` con `code` `not_found` al editar o dar de baja
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** el listado no se reemplaza por un estado vacío

#### Scenario: name vacío no llama a POST (unhappy)

- **WHEN** el administrador confirma crear con el nombre vacío o solo espacios
- **THEN** no se envía `POST /api/v1/brands`

#### Scenario: fallo de red muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `GET` inicial con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** no se lanza una excepción no capturada

#### Scenario: no hay UI de logos (unhappy, No hace)

- **WHEN** se inspecciona el ABM de marcas
- **THEN** no hay `<input type="file">`
- **AND** no hay control o campo de logo
- **AND** no hay `<img>` de marca

#### Scenario: no envía slug (unhappy)

- **WHEN** el administrador crea o edita una marca con nombre no vacío
- **THEN** el cuerpo de `POST` o `PATCH` no incluye `slug`

### Requirement: Tests de la UI de marcas

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (escenarios de HU-02-02 a mano: alta "Fixser" y baja lógica; las referencias en producto se validan con T-02-14) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `brands_list_requests_include_inactive` | `scripts/fixtures/vectra-web/brands.test.mjs` | happy |
| `brands_create_fixser_posts_name` | `brands.test.mjs` | happy (HU-02-02) |
| `brands_empty_list_offers_create` | `brands.test.mjs` | happy |
| `brands_rename_patches_name` | `brands.test.mjs` | happy |
| `brands_deactivate_deletes` | `brands.test.mjs` | happy (HU-02-02) |
| `brands_reactivate_patches_active_true` | `brands.test.mjs` | happy |
| `brands_duplicate_slug_shows_duplicate_slug` | `brands.test.mjs` | unhappy |
| `brands_not_found_shows_not_found` | `brands.test.mjs` | unhappy |
| `brands_network_shows_network` | `brands.test.mjs` | unhappy |
| `brands_empty_name_does_not_post` | `brands.test.mjs` | unhappy |
| `brands_no_logo_ui` | `brands.test.mjs` | unhappy (No hace) |
| `brands_does_not_send_slug` | `brands.test.mjs` | unhappy |
| `brands_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (secciones que no son `categorias` ni `marcas`) |
| `router_admin_does_not_call_api` | `router.test.mjs` | unhappy (`#/admin/productos` y `#/admin/stock`) |

`check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/brands.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de marcas cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `brands.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de marcas (unhappy)

- **WHEN** falta `vectra-web/js/admin/brands.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `brands.js`

### Requirement: UI de atributos técnicos en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="atributos"` (`#/admin/atributos`), `data-region="admin-content"` MUST montar el ABM de definiciones de atributo técnico. La vista MUST cargar `GET /api/v1/attribute-definitions?includeInactive=true` (lista plana `{ items }` de [docs/openapi.yaml](../../../../../docs/openapi.yaml)). MUST pintar las filas **en el orden de `items`** (la API ya ordena por `order` ascendente y, en empate, por `key`). MUST NOT reordenar por `label` ni por `key` en el cliente. Cada ítem MUST mostrar `label` y MAY mostrar `key`, `type` y `unit` como texto secundario. Una definición con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactiva») y MUST ofrecer reactivar, no dar de baja.

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`), un formulario con:

- `key` (snake_case, obligatorio en alta)
- `label` (obligatorio)
- `type`: uno de `text`, `number`, `boolean`, `enum` (valores de API; etiquetas visibles en español)
- `unit` (opcional)
- `options`: visible **solo** cuando `type` es `enum`; ausente o no visible para los demás tipos
- `facetable` (booleano)
- `searchable` (booleano)
- `order` (entero; si el administrador no lo cambia, el alta MAY omitirlo y el servidor usa `0`)

Acciones:

- **Nueva definición**: `POST /api/v1/attribute-definitions` con el cuerpo `AttributeDefinitionCreate` (HU-02-03 «Atributo enumerado»; Listo cuando: "grado").
- **Editar**: `PATCH /api/v1/attribute-definitions/{attributeId}` con los campos cambiados.
- **Dar de baja**: `DELETE /api/v1/attribute-definitions/{attributeId}` sobre una definición activa.
- **Reactivar**: `PATCH` con `{ "active": true }` sobre una inactiva.

Al crear "grado" el `POST` MUST enviar `key` `"grado"`, `label` no vacío, `type` `"enum"`, `options` `["2", "5", "8"]` y `facetable` verdadero. Si `type` no es `enum`, el cuerpo MUST NOT incluir `options` (o MUST enviar `null`). Un `key` o `label` vacío (solo espacios) MUST NOT enviarse.

MUST NOT pintar vista previa de facetas: MUST NOT haber chips de filtro de muestra, MUST NOT haber un panel o control cuyo propósito sea previsualizar facetas de búsqueda (**No hace**). MUST NOT llamar otros paths de `/api/v1` ni `/health` desde esta vista. Tras un `POST`/`PATCH`/`DELETE` exitoso, MUST volver a `GET` e hidratar el listado.

La UI MUST NOT consultar `producto` ni bloquear edición/baja por occupancy (`attribute_in_use` es T-02-23). Los errores MUST mostrarse con `showError` / `messageFor(code)` (requirement «Mensajes en español por ErrorCode»): el toast usa el `code` de `problem+json` (`invalid_input`, `conflict`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `base.css` MAY añadir clases del listado solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Si no hay definiciones, MUST mostrar el listado vacío y la acción de crear. Textos visibles MUST estar en español.

#### Scenario: alta de grado enum facetable actualiza el listado (happy, HU-02-03, Listo cuando)

- **WHEN** el administrador crea la definición con clave `"grado"`, tipo `enum`, opciones `"2"`, `"5"` y `"8"`, y `facetable` verdadero
- **THEN** se envía `POST /api/v1/attribute-definitions` con `key` `"grado"`, `type` `"enum"`, `options` `["2", "5", "8"]` y `facetable` verdadero
- **AND** tras recargar, el listado muestra `"grado"` (o su `label`) activa

#### Scenario: el listado respeta el order de la API (happy)

- **WHEN** `GET` devuelve `{ "items": [ { "key": "grado", "order": 0 }, { "key": "medida", "order": 10 } ] }`
- **THEN** la primera fila visible corresponde a `"grado"`
- **AND** la segunda corresponde a `"medida"`

#### Scenario: opciones visibles solo para enum (happy)

- **WHEN** el administrador abre el formulario de alta y elige `type` `enum`
- **THEN** el control de opciones está visible
- **AND** al cambiar `type` a `text`, `number` o `boolean` el control de opciones deja de estar visible

#### Scenario: crear definición en listado vacío (happy)

- **WHEN** `GET` devuelve `{ "items": [] }`
- **THEN** la región muestra la acción de crear definición
- **AND** al confirmar "grado" enum 2/5/8 facetable se envía el `POST` del escenario de alta

#### Scenario: editar label (happy)

- **WHEN** existe `"grado"` activa
- **AND** el administrador cambia `label` a `"Grado SAE"`
- **THEN** se envía `PATCH /api/v1/attribute-definitions/{id}` con `{ "label": "Grado SAE" }`
- **AND** tras recargar, el listado muestra `"Grado SAE"`

#### Scenario: dar de baja y reactivar (happy)

- **WHEN** `"grado"` es una definición activa
- **AND** el administrador la da de baja
- **THEN** se envía `DELETE /api/v1/attribute-definitions/{id}`
- **AND** tras recargar, `"grado"` se muestra inactiva y ofrece reactivar
- **AND** al reactivar se envía `PATCH` con `{ "active": true }`

#### Scenario: el listado pide inactivas (happy)

- **WHEN** se pinta `#/admin/atributos` con un cliente HTTP inyectado
- **THEN** la primera petición es `GET /api/v1/attribute-definitions?includeInactive=true`

#### Scenario: enum sin opciones muestra invalid_input (unhappy, HU-02-03)

- **WHEN** el administrador confirma crear con `type` `enum` y opciones vacías o ausentes
- **THEN** se envía `POST` con `type` `"enum"` y `options` ausentes, vacías o solo strings vacíos
- **AND** si el servidor responde `400` con `code` `invalid_input`, se muestra el mensaje en español de `invalid_input`

#### Scenario: clave duplicada muestra conflict (unhappy)

- **WHEN** el servidor responde `409` con `code` `conflict` al crear
- **THEN** se muestra el mensaje en español de `conflict`

#### Scenario: id desconocido muestra not_found (unhappy)

- **WHEN** el servidor responde `404` con `code` `not_found` al editar o dar de baja
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** el listado no se reemplaza por un estado vacío

#### Scenario: key o label vacío no llama a POST (unhappy)

- **WHEN** el administrador confirma crear con `key` o `label` vacío o solo espacios
- **THEN** no se envía `POST /api/v1/attribute-definitions`

#### Scenario: fallo de red muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `GET` inicial con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** no se lanza una excepción no capturada

#### Scenario: no hay vista previa de facetas (unhappy, No hace)

- **WHEN** se inspecciona el ABM de atributos
- **THEN** no hay un control, panel o lista de chips cuyo propósito sea previsualizar facetas de búsqueda
- **AND** no se llama a `/api/v1/search`

#### Scenario: type no enum no envía options (unhappy)

- **WHEN** el administrador crea una definición con `type` `text` (o `number` / `boolean`) y `key`/`label` no vacíos
- **THEN** el cuerpo de `POST` no incluye `options` o envía `options` `null`

### Requirement: Tests de la UI de atributos técnicos

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (crear "grado" enum 2/5/8 facetable a mano) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `attrs_list_requests_include_inactive` | `scripts/fixtures/vectra-web/attribute-definitions.test.mjs` | happy |
| `attrs_list_preserves_api_order` | `attribute-definitions.test.mjs` | happy |
| `attrs_create_grado_enum_facetable` | `attribute-definitions.test.mjs` | happy (HU-02-03, Listo cuando) |
| `attrs_empty_list_offers_create` | `attribute-definitions.test.mjs` | happy |
| `attrs_options_visible_only_for_enum` | `attribute-definitions.test.mjs` | happy |
| `attrs_edit_patches_label` | `attribute-definitions.test.mjs` | happy |
| `attrs_deactivate_deletes` | `attribute-definitions.test.mjs` | happy |
| `attrs_reactivate_patches_active_true` | `attribute-definitions.test.mjs` | happy |
| `attrs_enum_without_options_shows_invalid_input` | `attribute-definitions.test.mjs` | unhappy (HU-02-03) |
| `attrs_duplicate_key_shows_conflict` | `attribute-definitions.test.mjs` | unhappy |
| `attrs_not_found_shows_not_found` | `attribute-definitions.test.mjs` | unhappy |
| `attrs_network_shows_network` | `attribute-definitions.test.mjs` | unhappy |
| `attrs_empty_key_or_label_does_not_post` | `attribute-definitions.test.mjs` | unhappy |
| `attrs_non_enum_does_not_send_options` | `attribute-definitions.test.mjs` | unhappy |
| `attrs_no_facet_preview` | `attribute-definitions.test.mjs` | unhappy (No hace) |
| `attrs_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (secciones que no son `categorias`, `marcas` ni `atributos`) |
| `router_admin_does_not_call_api` | `router.test.mjs` | unhappy (`#/admin/productos` y `#/admin/stock`) |

`check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/attribute-definitions.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de atributos cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `attribute-definitions.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de atributos (unhappy)

- **WHEN** falta `vectra-web/js/admin/attribute-definitions.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `attribute-definitions.js`

### Requirement: UI listado de productos en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="productos"` (`#/admin/productos`), `data-region="admin-content"` MUST montar el listado administrativo de productos. La vista MUST cargar `GET /api/v1/products` con `active=true` en la primera página (solo activos; [docs/openapi.yaml](../../../../../docs/openapi.yaml) `listProducts`). MUST pintar una **tabla** cuyas filas son `items` de `ProductSummaryPage`, en el orden de la API (id ULID ascendente). Cada fila MUST mostrar `sku`, `name`, `productType`, `brand` (o vacío si es `null`), `categoryPath` unido (p. ej. `Fijaciones > Pernos`), `price`, `stock` y el estado (`active`). MUST NOT pintar `vx-product-card`. MUST NOT hidratar `imageUrl` ni mostrar `<img>` de producto (`imageUrl` es `null` hasta T-02-39).

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`):

- **Nuevo producto**: abre la ficha de alta del requirement «UI alta de producto (ficha Datos)». MUST estar disponible también cuando `items` está vacío.
- **Editar**: en cada fila (activa o inactiva), abre la ficha del requirement «UI edición de producto (ficha Datos)» (T-02-37).
- **Filtro categoría**: opciones hidratadas desde `GET /api/v1/categories?includeInactive=true`. Al elegir una categoría MUST enviar `categoryId` de esa categoría; MUST NOT expandir descendientes en el cliente (la API incluye la rama; HU-02-10 «Filtro por rama»).
- **Filtro marca**: opciones hidratadas desde `GET /api/v1/brands?includeInactive=true`. Al elegir una marca MUST enviar `brandId`.
- **Filtro estado**: `Activos` (`active=true`, defecto) e `Inactivos` (`active=false`). MUST NOT haber un valor «todos» (el contrato no lo ofrece).
- **Cargar más**: si `nextCursor` no es `null`, MUST mostrar la acción; al activarla MUST hacer `GET` con `cursor` igual a ese valor **y los mismos filtros**, y MUST **añadir** las nuevas filas a la tabla. Si `nextCursor` es `null`, MUST NOT mostrar «Cargar más».
- **Dar de baja**: `DELETE /api/v1/products/{productId}` sobre un producto activo (HU-02-06).
- **Reactivar**: sobre un inactivo, MUST `GET /api/v1/products/{productId}` y luego `PATCH` con `{ "version": <versión de la ficha>, "active": true }` (HU-02-06; `ProductSummary` no trae `version`).

Un producto con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactivo») y MUST ofrecer reactivar, no dar de baja. Un activo MUST ofrecer dar de baja, no reactivar.

Al cambiar un filtro MUST pedir de nuevo la **primera** página (sin `cursor`) con los filtros actuales y MUST reemplazar las filas (no append). Tras un `DELETE`/`PATCH` exitoso MUST recargar la primera página con los mismos filtros.

MUST NOT ofrecer edición en línea en la **tabla**: MUST NOT haber `contenteditable`, MUST NOT haber `<input>` / `vx-field` en celdas de `sku`, `name`, `price` o `stock` (**No hace** de T-02-34). Mientras el listado está visible (ficha cerrada) MUST NOT haber `<form>` de alta ni enviar `POST /api/v1/products`. MUST NOT llamar a `/api/v1/search` ni a `/health` desde esta vista. MUST NOT enviar `q`. Si el `GET` de listado devuelve `items` vacío, MUST mostrar el listado vacío **y** MUST ofrecer «Nuevo producto».

Los errores MUST mostrarse con `showError` / `messageFor(code)` (requirement «Mensajes en español por ErrorCode»): el toast usa el `code` de `problem+json` (`invalid_input`, `not_found`, `version_conflict`, `unavailable`, `network`, …), no el `title` crudo. Si falla un lookup de categoría/marca, MUST mostrar el error y MAY dejar el `<select>` sin opciones; MUST NOT impedir mostrar filas si el `GET` de productos ya respondió. `base.css` MAY añadir clases de tabla/filtros solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: el listado pide activos en la primera página (happy)

- **WHEN** se pinta `#/admin/productos` con un cliente HTTP inyectado
- **THEN** una de las peticiones es `GET /api/v1/products` con query `active=true` y sin `cursor`
- **AND** también se pide `GET /api/v1/categories?includeInactive=true` y `GET /api/v1/brands?includeInactive=true`

#### Scenario: filtro por rama envía categoryId (happy, HU-02-10)

- **WHEN** las categorías incluyen `"Fijaciones"` con un `id` conocido
- **AND** el administrador elige esa categoría
- **THEN** se envía `GET /api/v1/products` con `categoryId` igual a ese `id` y `active=true`
- **AND** el cliente no calcula descendientes

#### Scenario: filtro por inactivos ofrece reactivar (happy, HU-02-10)

- **WHEN** el administrador filtra por inactivos
- **THEN** se envía `GET /api/v1/products` con `active=false`
- **AND** cada fila inactiva ofrece reactivar y no ofrece dar de baja

#### Scenario: cargar más concatena con cursor (happy)

- **WHEN** la primera página devuelve `nextCursor` no nulo y al menos una fila
- **AND** el administrador activa «Cargar más»
- **THEN** se envía `GET /api/v1/products` con `cursor` igual a ese `nextCursor` y los mismos filtros
- **AND** la tabla muestra las filas de ambas páginas

#### Scenario: listado vacío no ofrece alta (happy)

- **WHEN** `GET /api/v1/products` devuelve `{ "items": [], "nextCursor": null }`
- **THEN** la región muestra el listado vacío
- **AND** no se envía `POST /api/v1/products`

#### Scenario: listado vacío ofrece alta y no hace POST (happy)

- **WHEN** `GET /api/v1/products` devuelve `{ "items": [], "nextCursor": null }`
- **THEN** la región muestra el listado vacío
- **AND** ofrece la acción «Nuevo producto»
- **AND** no se envía `POST /api/v1/products` hasta que se abra y envíe la ficha

#### Scenario: dar de baja envía DELETE (happy, HU-02-06)

- **WHEN** hay un producto activo en la tabla
- **AND** el administrador lo da de baja
- **THEN** se envía `DELETE /api/v1/products/{id}`
- **AND** tras recargar, se pide de nuevo la primera página con los filtros actuales

#### Scenario: reactivar lee version y envía PATCH (happy, HU-02-06)

- **WHEN** hay un producto inactivo en la tabla
- **AND** el administrador lo reactiva
- **THEN** se envía `GET /api/v1/products/{id}`
- **AND** se envía `PATCH /api/v1/products/{id}` con `{ "version": <versión de la ficha>, "active": true }`
- **AND** el cuerpo del `PATCH` no incluye `sku`, `name`, `price` ni `stock`

#### Scenario: el listado ofrece Editar (happy, T-02-37)

- **WHEN** se pinta el listado con al menos una fila
- **THEN** hay `data-action="edit"` con el `data-product-id` de esa fila

#### Scenario: fallo de red muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `GET` inicial de productos con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** no se lanza una excepción no capturada

#### Scenario: id desconocido muestra not_found (unhappy)

- **WHEN** el servidor responde `404` con `code` `not_found` al dar de baja o al `GET` de ficha para reactivar
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** la tabla no se reemplaza por un estado vacío

#### Scenario: cursor o filtro inválido muestra invalid_input (unhappy)

- **WHEN** el servidor responde `400` con `code` `invalid_input` al listar
- **THEN** se muestra el mensaje en español de `invalid_input`

#### Scenario: conflicto de versión al reactivar muestra version_conflict (unhappy)

- **WHEN** el `PATCH` de reactivación responde `409` con `code` `version_conflict`
- **THEN** se muestra el mensaje en español de `version_conflict`
- **AND** no se pinta un diff de campos (eso es solo en ficha de edición T-02-37)

#### Scenario: no hay edición en línea (unhappy, No hace)

- **WHEN** se inspecciona el listado de productos
- **THEN** no hay `contenteditable`
- **AND** no hay `<input>` ni `vx-field` en celdas de sku, nombre, precio o stock
- **AND** no hay `<form>` de alta o edición de producto

#### Scenario: no hay edición en línea en la tabla (unhappy, No hace T-02-34)

- **WHEN** se inspecciona el listado de productos (ficha de alta cerrada)
- **THEN** no hay `contenteditable`
- **AND** no hay `<input>` ni `vx-field` en celdas de sku, nombre, precio o stock
- **AND** no hay `<form>` de alta o edición de producto

### Requirement: Tests de la UI listado de productos

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (escenarios de HU-02-10 a mano: filtro por rama «Fijaciones» y filtro por inactivos con reactivar) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `products_list_requests_active_true` | `scripts/fixtures/vectra-web/products.test.mjs` | happy |
| `products_lookups_request_categories_and_brands` | `products.test.mjs` | happy |
| `products_empty_list_does_not_post` | `products.test.mjs` | happy |
| `products_empty_list_offers_create` | `products.test.mjs` | happy |
| `products_filter_category_sends_category_id` | `products.test.mjs` | happy (HU-02-10) |
| `products_filter_inactive_shows_reactivate` | `products.test.mjs` | happy (HU-02-10) |
| `products_load_more_appends_with_cursor` | `products.test.mjs` | happy |
| `products_deactivate_deletes` | `products.test.mjs` | happy (HU-02-06) |
| `products_reactivate_gets_then_patches_active_true` | `products.test.mjs` | happy (HU-02-06) |
| `products_network_shows_network` | `products.test.mjs` | unhappy |
| `products_not_found_shows_not_found` | `products.test.mjs` | unhappy |
| `products_invalid_input_shows_invalid_input` | `products.test.mjs` | unhappy |
| `products_version_conflict_shows_version_conflict` | `products.test.mjs` | unhappy |
| `products_no_inline_edit` | `products.test.mjs` | unhappy (No hace T-02-34) |
| `products_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (secciones que no son `productos`, `categorias`, `marcas` ni `atributos`) |
| `router_admin_does_not_call_api` | `router.test.mjs` | unhappy (`#/admin/stock`) |

`check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/products.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de productos cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `products.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de productos (unhappy)

- **WHEN** falta `vectra-web/js/admin/products.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `products.js`

### Requirement: UI alta de producto (ficha Datos)

Cuando el administrador activa **Nuevo producto** en `#/admin/productos`, `data-region="admin-content"` MUST reemplazar el listado por la **ficha de alta**. El hash MUST seguir siendo `#/admin/productos` (MUST NOT exigir un tercer segmento). Cancelar MUST volver al listado **sin** enviar `POST`.

La ficha MUST mostrar pestañas en español, operables por teclado (tamaño táctil ≥ `--vx-touch-min`):

| Pestaña | Estado en este ticket |
|---|---|
| Datos | seleccionada y operable |
| Stock | visible, no operable (`aria-disabled` o equivalente) |
| Relaciones | visible, no operable |

MUST NOT pintar pestaña Imágenes (v2, T-02-39). Activar Stock o Relaciones MUST NOT enviar HTTP ni cambiar de pestaña. `role="tablist"` / `role="tab"` MAY usarse; la pestaña Datos MUST tener `aria-selected="true"`.

La pestaña **Datos** MUST incluir, en español, los campos de `ProductCreate` ([docs/openapi.yaml](../../../../../docs/openapi.yaml)):

| Campo visible | Control | `name` (JSON / `vx-field`) | Obligatorio |
|---|---|---|---|
| SKU | `vx-field` | `sku` | sí |
| Nombre | `vx-field` | `name` | sí |
| Tipo | `vx-field` | `productType` | sí |
| Descripción | `vx-field` | `description` | no |
| Categoría | `<select>` + `vx-field` receptor de error | `categoryId` | sí |
| Marca | `<select>` + `vx-field` receptor de error | `brandId` | no |
| Unidad | `<select>` + `vx-field` receptor de error | `unitOfMeasure` | no (default `unit`) |
| Precio | `vx-field` | `price` | sí |
| Pasillo / Estante / Posición | `vx-field` | `location.aisle`, `location.shelf`, `location.position` | no |
| Stock inicial | `vx-field` | `initialStock` | no |
| Atributos y códigos | editor tipado | `attributes[n].value`, `codes.*` | no (requirement «UI atributos y códigos en la ficha de alta») |

Opciones de categoría y marca MUST ser **solo activas** (`active !== false`), hidratadas desde los lookups ya pedidos por el listado o desde `GET /api/v1/categories` y `GET /api/v1/brands` **sin** `includeInactive`. MUST NOT ofrecer una categoría o marca dada de baja (el backend responde `invalid_input`). Unidad MUST listar el enum `UnitOfMeasure` (`unit`, `pair`, `box`, `meter`, `kilogram`, `liter`) con etiquetas en español (Unidad, Par, Caja, Metro, Kilogramo, Litro); valor inicial `unit`.

Al guardar, si `sku`, `name`, `productType`, `categoryId` o `price` quedan vacíos tras recortar espacios, MUST NOT enviar `POST`. Si están completos, MUST enviar `POST /api/v1/products` con cuerpo `ProductCreate`:

- `sku`, `name`, `productType`, `categoryId`, `price` (decimal en **string**, p. ej. `"120.50"`)
- `description` omitido o `null` si está vacío
- `brandId` omitido si no hay marca
- `unitOfMeasure` (default `unit`)
- `location` omitido si pasillo, estante y posición están vacíos; si hay alguno, objeto `{ aisle, shelf, position }` con nulos/omitidos en los vacíos
- `initialStock` omitido o `"0"` si está vacío; si hay valor, string decimal no negativo
- `attributes` y `codes` según el requirement «UI atributos y códigos en la ficha de alta» (omitidos o `[]` si no hay valores)
- MUST NOT enviar `stock` ni `version` ni `active`

`api.request` ya adjunta `X-Vectra-Terminal` si hay nombre de terminal (T-01-06 / T-01-11); esta vista MUST NOT duplicar esa cabecera.

Tras `201` con un `Product`, MUST volver al listado y MUST recargar la **primera** página con los filtros que había al abrir la ficha. El producto creado (activo, `version` 1) MUST aparecer cuando esos filtros son los de activos. MUST NOT abrir ficha de edición (T-02-37). MUST NOT llamar a `/api/v1/search` ni a `/health`.

Los errores MUST mostrarse con `showError` / `messageFor(code)`: el toast usa el `code` (`invalid_input`, `duplicate_sku`, `unavailable`, `network`, …), no el `title` crudo. Si `err.errors` trae `{ field, message }`, MUST asignarse al `vx-field` cuyo `name` es exactamente `field` (requirement T-01-07), incluido `attributes[n].value`. La ficha MUST permanecer abierta si el POST falla. `base.css` MAY añadir clases de ficha/pestañas solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: Nuevo producto abre la ficha Datos (happy)

- **WHEN** el listado de `#/admin/productos` está visible
- **AND** el administrador activa «Nuevo producto»
- **THEN** la región muestra la ficha con la pestaña Datos seleccionada
- **AND** hay controles con `name` `sku`, `name`, `productType`, `categoryId`, `price` e `initialStock`
- **AND** no se envía `POST /api/v1/products` todavía
- **AND** el hash sigue siendo `#/admin/productos`

#### Scenario: alta completa envía ProductCreate y vuelve al listado (happy, HU-02-04, Listo cuando)

- **WHEN** la ficha está abierta con categoría y marca activas hidratadas
- **AND** el administrador completa SKU `PER-M8X40-ZN`, nombre `Perno hexagonal M8 x 40 zincado`, tipo `Perno`, precio `120.50`, stock inicial `50` y una categoría
- **AND** guarda
- **THEN** se envía `POST /api/v1/products` cuyo cuerpo incluye esos campos (`price` e `initialStock` como string) y `attributes` / `codes` según el requirement «UI atributos y códigos en la ficha de alta»
- **AND** tras `201`, se pinta de nuevo el listado
- **AND** se pide `GET /api/v1/products` de la primera página con los filtros previos
- **AND** si la respuesta incluye el SKU creado, la tabla lo muestra

#### Scenario: cancelar no envía POST (happy)

- **WHEN** la ficha de alta está abierta
- **AND** el administrador cancela
- **THEN** vuelve el listado
- **AND** no se envió `POST /api/v1/products`

#### Scenario: pestañas Stock y Relaciones no operan (happy / frontera)

- **WHEN** la ficha de alta está abierta
- **THEN** hay pestañas Datos, Stock y Relaciones
- **AND** Datos está seleccionada
- **AND** no hay pestaña Imágenes
- **AND** activar Stock o Relaciones no envía HTTP

#### Scenario: SKU duplicado muestra duplicate_sku (unhappy, HU-02-04)

- **WHEN** el `POST` responde `409` con `code` `duplicate_sku`
- **THEN** se muestra el mensaje en español de `duplicate_sku`
- **AND** la ficha permanece abierta
- **AND** no se reemplaza por el listado

#### Scenario: invalid_input pinta el campo sku (unhappy)

- **WHEN** el `POST` responde `400` con `code` `invalid_input` y `errors: [{ "field": "sku", "message": "requerido" }]`
- **THEN** el `vx-field` con `name="sku"` recibe el mensaje de campo
- **AND** el toast muestra el mensaje en español de `invalid_input`
- **AND** la ficha permanece abierta

#### Scenario: invalid_input pinta categoryId (unhappy)

- **WHEN** el `POST` responde `400` con `code` `invalid_input` y `errors: [{ "field": "categoryId", "message": "inválido" }]`
- **THEN** el `vx-field` con `name="categoryId"` recibe el mensaje de campo
- **AND** no se lanza una excepción no capturada

#### Scenario: fallo de red en el alta muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `POST` con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** la ficha permanece abierta
- **AND** no se lanza una excepción no capturada

#### Scenario: campos obligatorios vacíos no envían POST (unhappy)

- **WHEN** la ficha está abierta y SKU, nombre, tipo, categoría o precio están vacíos
- **AND** el administrador guarda
- **THEN** no se envía `POST /api/v1/products`

#### Scenario: la ficha no ofrece atributos ni códigos (unhappy, No hace)

El No hace de T-02-35 (sin atributos ni códigos) queda superseded. Este escenario se conserva para archive y ahora exige lo contrario más el No hace vigente (`barcode`):

- **WHEN** se inspecciona la ficha de alta
- **THEN** hay controles de atributos (`attributes[n].value`) y de códigos (`codes.manufacturer` / `codes.oem`) si el lookup trajo definiciones, o el editor está presente aunque `items` sea `[]`
- **AND** no hay control ni opción `barcode`
- **AND** el cuerpo del POST, si se envía, no incluye `codes[].type` `"barcode"`

### Requirement: Tests de la UI alta de producto

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). Crear un producto desde la UI y verlo en el listado MUST seguir cubierto. Los tests de atributos/códigos están en el requirement «Tests de atributos y códigos en la ficha».

| Test | Ubicación | Tipo |
|---|---|---|
| `products_empty_list_offers_create` | `scripts/fixtures/vectra-web/products.test.mjs` | happy |
| `sheet_new_product_opens_datos_tab` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_create_perno_posts_product_create` | `product-sheet.test.mjs` | happy (HU-02-04) |
| `sheet_create_returns_to_list_with_new_sku` | `product-sheet.test.mjs` | happy |
| `sheet_cancel_does_not_post` | `product-sheet.test.mjs` | happy |
| `sheet_stock_and_relations_tabs_are_disabled` | `product-sheet.test.mjs` | happy / frontera |
| `sheet_duplicate_sku_shows_duplicate_sku` | `product-sheet.test.mjs` | unhappy (HU-02-04) |
| `sheet_invalid_input_sets_sku_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_invalid_input_sets_category_id_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_network_shows_network` | `product-sheet.test.mjs` | unhappy |
| `sheet_empty_required_does_not_post` | `product-sheet.test.mjs` | unhappy |
| `sheet_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |

`sheet_no_attributes_or_codes` MUST NOT permanecer como aserción de ausencia: ese No hace de T-02-35 quedó superseded. `check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/product-sheet.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de alta cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `product-sheet.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de ficha (unhappy)

- **WHEN** falta `vectra-web/js/admin/product-sheet.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `product-sheet.js`

### Requirement: UI atributos y códigos en la ficha de alta

Cuando la ficha de alta está abierta (requirement «UI alta de producto (ficha Datos)»), la pestaña **Datos** MUST incluir un **editor de atributos** y un **editor de códigos**. MUST NOT exigir un tercer segmento de hash.

Al abrir la ficha, MUST pedir `GET /api/v1/attribute-definitions` **sin** `includeInactive` (solo activas; [docs/openapi.yaml](../../../../../docs/openapi.yaml) `listAttributeDefinitions`). MUST pintar las definiciones con `active !== false` **en el orden de `items`** (la API ordena por `order`, empate por `key`). MUST NOT reordenar por `label` en el cliente. MUST NOT pintar una definición inactiva si el lookup las devolviera. Si el `GET` falla, MUST mostrar el error con `showError` / `messageFor(code)` y MAY dejar el editor de atributos vacío; MUST NOT impedir completar SKU/nombre/categoría/precio ni enviar el POST de alta.

Cada definición activa MUST mostrar, en español, `label` y, si `unit` no es `null` ni vacío, la unidad junto a la etiqueta (p. ej. `Largo (mm)`). El control MUST ser según `type`:

| `type` | Control | Valor enviado |
|---|---|---|
| `text` | `vx-field` | string |
| `number` | `vx-field` de **texto** (MUST NOT `type="number"`) | JSON **number** si el texto es un decimal parseable; si no (p. ej. `"cuarenta"`), string |
| `boolean` | `<select>` (vacío / Sí / No) + `vx-field` receptor de error | JSON `true` / `false`; omitido si vacío |
| `enum` | `<select>` con `options` de la definición + `vx-field` receptor de error | string igual a la opción; omitido si vacío |

El `vx-field` (o el receptor junto al `<select>`) del atributo en índice `i` entre las definiciones **renderizadas** MUST tener `name="attributes[${i}].value"` para que `showError` (T-01-07) coincida con `errors[].field`. MUST NOT exigir que todos los atributos tengan valor: un valor vacío tras recortar espacios MUST omitirse del array `attributes` del POST (no se envía `definitionId` sin valor).

El editor de códigos MUST ofrecer, en español, dos campos opcionales:

| Campo visible | `ProductCode.type` | Control |
|---|---|---|
| Código de fabricante | `manufacturer` | `vx-field` `name="codes.manufacturer"` (o `codes[n].value` del ítem enviado con ese type) |
| Código OEM | `oem` | `vx-field` `name="codes.oem"` |

MUST NOT pintar un control de código de barras ni una opción `barcode` (**No hace**, post-MVP). Un código vacío MUST omitirse del array `codes`. Si ambos están vacíos, MUST omitir `codes` o enviarlo `[]`.

Al guardar (mismos obligatorios de la ficha Datos), el `POST /api/v1/products` MUST incluir, además de los campos de T-02-35:

- `attributes`: array de `{ definitionId, value }` en el orden de las definiciones renderizadas que tienen valor. `value` de `number` parseable MUST ser JSON number (p. ej. `40`, no `"40"`), para que el Gateway lo mapee a `ValorAtributo::Numero`. `"cuarenta"` MUST ir como string.
- `codes`: array de `{ type, value }` con `type` `manufacturer` y/o `oem` si hay valor.

Si no hay atributos ni códigos con valor, MAY omitir las claves o enviar `[]`. MUST NOT enviar `type: "barcode"`.

Si el POST responde `400` con `code` `invalid_input` y `errors[].field` `attributes[n].value`, MUST pintar el `message` **sobre el control del atributo cuyo `definitionId` es el n-ésimo elemento de `attributes` enviado en ese POST** (Listo cuando: definición `"largo"` `type=number`, valor `"cuarenta"`). El toast MUST usar `messageFor("invalid_input")`. La ficha MUST permanecer abierta. `base.css` MAY añadir clases del editor solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: editor muestra control según tipo y unidad (happy)

- **WHEN** el lookup de definiciones incluye `largo` (`type` `number`, `unit` `"mm"`), `medida` (`type` `text`), `zincado` (`type` `boolean`) y `grado` (`type` `enum`, `options` `["2","5","8"]`)
- **AND** el administrador abre la ficha de alta
- **THEN** se pide `GET /api/v1/attribute-definitions` sin `includeInactive`
- **AND** hay un `vx-field` para largo cuya etiqueta incluye `mm`
- **AND** hay un `vx-field` de texto para medida
- **AND** hay un `<select>` para zincado y un `<select>` con opciones 2/5/8 para grado
- **AND** no hay `input type="number"` en el editor de atributos

#### Scenario: alta completa envía attributes y codes (happy, HU-02-04)

- **WHEN** las definiciones activas incluyen medida (text) y largo (number)
- **AND** el administrador completa el perno M8×40 (SKU `PER-M8X40-ZN`, categoría, precio, stock inicial 50), medida `M8`, largo `40` y código de fabricante `HX-0840Z`
- **AND** guarda
- **THEN** el `POST /api/v1/products` incluye `attributes` con esos `definitionId` y valores (`40` como JSON number) y `codes` con `{ type: "manufacturer", value: "HX-0840Z" }`
- **AND** el cuerpo no incluye un code `barcode`

#### Scenario: lookup de atributos no bloquea el alta (happy)

- **WHEN** el `GET` de definiciones devuelve `{ "items": [] }`
- **AND** el administrador completa SKU, nombre, tipo, categoría y precio
- **AND** guarda
- **THEN** se envía `POST /api/v1/products`
- **AND** `attributes` y `codes` están ausentes o son `[]`

#### Scenario: largo = cuarenta pinta el error sobre ese campo (unhappy, HU-02-04, Listo cuando)

- **WHEN** existe la definición activa `largo` de tipo `number`
- **AND** el administrador escribe `"cuarenta"` en el control de largo y guarda un alta con obligatorios completos
- **AND** el servidor responde `400` con `code` `invalid_input` y `errors: [{ "field": "attributes[n].value", "message": "expected number value" }]`
- **THEN** el `POST` envió `attributes[n].value` igual a `"cuarenta"` (string) para esa definición
- **AND** el `vx-field` de largo recibe ese mensaje de campo
- **AND** el toast muestra el mensaje en español de `invalid_input`
- **AND** la ficha permanece abierta

#### Scenario: definiciones inactivas no se pintan (unhappy)

- **WHEN** el lookup devolviera una definición con `active: false` (p. ej. si el cliente pidiera `includeInactive=true`)
- **THEN** esa definición no tiene control en la ficha
- **AND** no se envía su `definitionId` en `attributes`

#### Scenario: no hay código de barras (unhappy, No hace)

- **WHEN** se inspecciona la ficha de alta
- **THEN** no hay control ni opción cuyo `type` o etiqueta sea código de barras / `barcode`
- **AND** el cuerpo del POST, si se envía, no incluye `codes[].type` `"barcode"`

#### Scenario: fallo de red en definiciones muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `GET /api/v1/attribute-definitions` con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** la ficha permanece abierta
- **AND** no se lanza una excepción no capturada

### Requirement: Tests de atributos y códigos en la ficha

Antes de archive, la suite MUST incluir estos tests (nombres estables) **además** de los de «Tests de la UI alta de producto». Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (error de largo = cuarenta sobre ese campo) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `sheet_attribute_controls_match_type_and_unit` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_create_perno_posts_attributes_and_codes` | `product-sheet.test.mjs` | happy (HU-02-04) |
| `sheet_empty_attributes_omitted_from_post` | `product-sheet.test.mjs` | happy |
| `sheet_cuarenta_on_largo_sets_attribute_field_error` | `product-sheet.test.mjs` | unhappy (HU-02-04, Listo cuando) |
| `sheet_inactive_attribute_defs_are_hidden` | `product-sheet.test.mjs` | unhappy |
| `sheet_no_barcode_code` | `product-sheet.test.mjs` | unhappy (No hace) |
| `sheet_attribute_defs_network_shows_network` | `product-sheet.test.mjs` | unhappy |

MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de atributos y códigos cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `product-sheet.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: el Listo cuando está cubierto por un test nombrado (unhappy)

- **WHEN** se inspecciona `product-sheet.test.mjs`
- **THEN** existe el test `sheet_cuarenta_on_largo_sets_attribute_field_error`

### Requirement: UI edición de producto (ficha Datos)

Cuando el administrador activa **Editar** en una fila del listado de `#/admin/productos`, `data-region="admin-content"` MUST mostrar la **ficha de edición** (MUST NOT cambiar el hash). MUST pedir `GET /api/v1/products/{productId}` y `GET /api/v1/attribute-definitions` (sin `includeInactive`). El título MUST ser «Editar producto» (MUST NOT «Nuevo producto»). El `<form>` MUST tener `data-form="edit-product"` y un atributo oculto con la `version` actual del producto.

La ficha MUST reutilizar la pestaña Datos de la ficha de alta: mismos campos editables (SKU, nombre, tipo, descripción, categoría, marca, unidad, precio, ubicación, atributos, códigos). MUST hidratar valores desde el `Product` del GET. MUST NOT pintar ni permitir editar `initialStock`. El **stock** actual MUST mostrarse en **solo lectura** (MUST NOT `vx-field` ni `<input>` editable para `stock`). MUST NOT enviar `stock`, `initialStock` ni `active` en el PATCH de ficha (baja/reactivar permanece en el listado).

Al **Guardar**, MUST enviar `PATCH /api/v1/products/{productId}` con cuerpo `ProductUpdate`: `version` obligatoria más los campos de ficha presentes en el formulario, alineados con [docs/openapi.yaml](../../../../../docs/openapi.yaml). Atributos MUST enviarse como en el alta; MUST preservar en el array atributos de definiciones **inactivas** presentes en el GET aunque no tengan control pintado. Códigos `barcode` u otros tipos del GET MUST preservarse aunque no haya control.

Tras **200**, MUST volver al listado y recargar la primera página con los filtros actuales. **Cancelar** MUST volver al listado sin PATCH.

Errores MUST usar `showError` / `messageFor(code)`. `invalid_input` MUST pintar `errors[].field` en `vx-field` (incluido `attributes[n].value` con remap por `definitionId` como en T-02-36). La ficha MUST permanecer abierta si el PATCH falla (excepto tras `version_conflict`, ver requirement siguiente). MUST NOT agregar *tokens* CSS nuevos ni Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: Editar abre ficha hidratada (happy)

- **WHEN** el listado muestra un producto activo
- **AND** el administrador activa Editar
- **THEN** se pide `GET /api/v1/products/{id}` y `GET /api/v1/attribute-definitions`
- **AND** la región muestra «Editar producto» y `data-form="edit-product"`
- **AND** el SKU del GET aparece en el control de SKU

#### Scenario: Edición exitosa incrementa version (happy, HU-02-05)

- **WHEN** el producto tiene `version` 3 y precio `"120.50"`
- **AND** el administrador cambia el precio a `130.00` y guarda
- **THEN** se envía `PATCH` con `{ "version": 3, "price": "130.00", … }` y sin `stock`
- **AND** tras 200 vuelve al listado

#### Scenario: stock es solo lectura (happy, HU-02-05)

- **WHEN** la ficha de edición está abierta y el producto tiene `stock` `"50"`
- **THEN** hay un bloque visible con el stock
- **AND** no hay `vx-field` ni `<input>` con `name` `stock` ni `initialStock`

#### Scenario: Conflicto recarga y muestra campos cambiados (unhappy, HU-02-05, Listo cuando)

- **WHEN** la ficha tenía snapshot `version` 3 y precio `"120.50"`
- **AND** el administrador cambió el nombre localmente
- **AND** el `PATCH` responde `409` con `code` `version_conflict`
- **THEN** se pide de nuevo `GET /api/v1/products/{id}` con `version` 4 y precio `"130.00"`
- **AND** el toast muestra el mensaje en español de `version_conflict`
- **AND** un elemento `role="status"` lista el cambio en **Precio** (u otro campo distinto)
- **AND** el precio visible es `130.00` (valor del servidor, no el nombre local sin guardar)

#### Scenario: no combina cambios locales tras conflicto (unhappy, No hace)

- **WHEN** tras `version_conflict` el GET trae nombre `"Perno A"` y el usuario había escrito `"Perno B"` localmente
- **THEN** el nombre visible es `"Perno A"`
- **AND** no se envía un segundo PATCH automático

#### Scenario: invalid_input en PATCH (unhappy)

- **WHEN** el PATCH responde `400` con `code` `invalid_input` y `errors[].field` `sku`
- **THEN** el `vx-field` de SKU recibe el mensaje
- **AND** la ficha permanece abierta

#### Scenario: not_found al abrir edición (unhappy)

- **WHEN** `GET /api/v1/products/{id}` responde `404` `not_found` al abrir Editar
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** MUST volver al listado o no dejar un formulario vacío indefinido

#### Scenario: red en PATCH (unhappy)

- **WHEN** el `PATCH` falla con error `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** la ficha permanece abierta

### Requirement: Tests de edición de producto

Antes de archive, la suite MUST incluir:

| Test | Ubicación | Tipo |
|---|---|---|
| `sheet_edit_opens_hydrated_datos_tab` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_edit_price_patches_with_version` | `product-sheet.test.mjs` | happy (HU-02-05) |
| `sheet_edit_stock_is_read_only` | `product-sheet.test.mjs` | happy / frontera |
| `sheet_edit_version_conflict_reloads_and_lists_changed_fields` | `product-sheet.test.mjs` | unhappy (Listo cuando) |
| `sheet_edit_does_not_merge_local_unsaved_fields` | `product-sheet.test.mjs` | unhappy (No hace) |
| `sheet_edit_invalid_input_sets_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_edit_not_found_shows_not_found` | `product-sheet.test.mjs` | unhappy |
| `sheet_edit_network_shows_network` | `product-sheet.test.mjs` | unhappy |
| `sheet_edit_empty_required_does_not_patch` | `product-sheet.test.mjs` | unhappy |
| `products_list_offers_edit` | `scripts/fixtures/vectra-web/products.test.mjs` | happy |
| `diff_changed_product_fields_lists_price` | `product-sheet.test.mjs` | unhappy (unitario) |

`./scripts/check-vectra-web.sh` MUST ejecutarlos. `cargo test --workspace` MUST seguir en verde.

#### Scenario: la suite de edición corre en check (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** incluye `product-sheet.test.mjs` con los tests de la tabla de este requirement
- **AND** incluye `products_list_offers_edit` en `products.test.mjs`

### Requirement: UI pestaña Stock (ingresos y ajustes)

Cuando la **ficha de edición** está abierta (requirement «UI edición de producto (ficha Datos)»), la pestaña **Stock** MUST ser operable: MUST NOT tener `aria-disabled`. Activarla MUST seleccionarla (`aria-selected="true"` en Stock, `"false"` en Datos) y MUST mostrar el panel de stock **sin** cambiar el hash (sigue `#/admin/productos`). MUST NOT enviar HTTP solo por cambiar de pestaña. La pestaña **Relaciones** MUST seguir no operable. MUST NOT pintar pestaña Imágenes (v2, T-02-39).

En la ficha de **alta**, Stock MUST permanecer no operable (requirement «UI alta de producto (ficha Datos)»); MUST NOT habilitarse ni enviar `POST .../stock-movements` desde el alta.

El panel Stock MUST estar en español y operable por teclado (controles con tamaño táctil ≥ `--vx-touch-min`). MUST incluir:

| Campo visible | Control | `name` (JSON / `vx-field`) | Obligatorio |
|---|---|---|---|
| Tipo | `<select>` | `type` | sí (`inbound` o `adjustment`; etiquetas Ingreso / Ajuste) |
| Cantidad | `vx-field` | `quantity` | sí (string decimal con signo, ≠ 0; `inbound` > 0) |
| Motivo | `vx-field` | `reason` | sí si `type` es `adjustment`; no si `inbound` |
| Stock actual | solo lectura | — | — (valor de la ficha; MUST NOT ser `<input>` editable) |

MUST haber una acción de registrar (p. ej. «Registrar movimiento») en un `<form data-form="stock-movement">`. MUST NOT ofrecer editar ni borrar un movimiento (ni `data-action` de update/delete sobre un id de movimiento). MUST NOT pintar un listado paginado de historial ni una acción de exportar. MUST NOT llamar `GET /api/v1/products/{productId}/stock-movements`.

Al registrar, si faltan tipo o cantidad (vacíos / solo espacios), o si el tipo es `adjustment` y el motivo está vacío, MUST NOT enviar `POST`. Si el tipo es `adjustment` y el motivo falta, MUST tratarlo como `invalid_input` sobre `reason` (HU-02-07 «Ajuste sin motivo»).

Si el formulario es válido, MUST enviar `POST /api/v1/products/{productId}/stock-movements` con cuerpo `StockMovementCreate` alineado con [docs/openapi.yaml](../../../../../docs/openapi.yaml): `type` (`inbound` \| `adjustment`), `quantity` como **string** decimal, `reason` omitido o `null` en ingreso vacío, `reason` string en ajuste. MUST NOT enviar `initial` ni `sale`. MUST NOT enviar `version` ni `stock` ni PATCH de ficha. `api.request` ya adjunta `X-Vectra-Terminal` si hay nombre de terminal (T-01-06); esta vista MUST NOT duplicar esa cabecera.

Tras **201** `StockMovement`, MUST permanecer en la pestaña Stock y MUST mostrar el `resultingStock` (p. ej. `role="status"`). MUST actualizar el stock de solo lectura de la ficha a ese valor. MUST NOT volver al listado. MUST NOT cambiar la `version` visible de la ficha.

Errores MUST usar `showError` / `messageFor(code)`: el toast usa el `code` (`invalid_input`, `negative_stock`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `invalid_input` MUST pintar `errors[].field` en el `vx-field` cuyo `name` es `quantity` o `reason`. `negative_stock` MUST mostrar el mensaje en español de `errors.js` y MUST dejar el stock de solo lectura **igual que antes del POST**. La ficha MUST permanecer abierta si el POST falla. `base.css` MAY añadir clases de pestaña/panel Stock solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Textos visibles MUST estar en español.

#### Scenario: pestaña Stock operable en edición (happy)

- **WHEN** la ficha de edición está abierta
- **THEN** la pestaña Stock no tiene `aria-disabled`
- **AND** Datos está seleccionada
- **AND** Relaciones sigue no operable
- **AND** no hay pestaña Imágenes

#### Scenario: activar Stock muestra el formulario (happy)

- **WHEN** la ficha de edición está abierta
- **AND** el administrador activa la pestaña Stock
- **THEN** Stock queda `aria-selected="true"`
- **AND** hay `data-form="stock-movement"` con controles `type`, `quantity` y `reason`
- **AND** el stock actual de la ficha es visible en solo lectura
- **AND** no se envía HTTP por el cambio de pestaña
- **AND** el hash sigue `#/admin/productos`

#### Scenario: Ingreso actualiza stock resultante (happy, HU-02-07, Listo cuando)

- **WHEN** el producto tiene stock `"10"`
- **AND** el administrador registra un ingreso de cantidad `"20"`
- **THEN** se envía `POST /api/v1/products/{id}/stock-movements` con `{ "type": "inbound", "quantity": "20" }` y sin `reason` vacío como obligatorio
- **AND** `quantity` viaja como string (MUST NOT número JSON)
- **AND** el cuerpo no incluye `version` ni `stock`
- **AND** tras 201 con `resultingStock` `"30"` se muestra ese valor
- **AND** el stock de solo lectura pasa a `"30"`
- **AND** la pestaña Stock permanece abierta

#### Scenario: Ajuste a cero con motivo (happy, HU-02-07)

- **WHEN** el producto tiene stock `"5"`
- **AND** el administrador registra un ajuste de `"-5"` con motivo `"Conteo físico"`
- **THEN** se envía `POST` con `{ "type": "adjustment", "quantity": "-5", "reason": "Conteo físico" }`
- **AND** tras 201 con `resultingStock` `"0"` se muestra ese valor

#### Scenario: Ajuste sin motivo no envía POST (unhappy, HU-02-07, Listo cuando)

- **WHEN** el administrador elige tipo Ajuste y cantidad `"-1"` sin motivo
- **THEN** no se envía `POST /api/v1/products/{id}/stock-movements`
- **AND** el control `reason` recibe un error de campo
- **AND** se muestra el mensaje en español de `invalid_input`

#### Scenario: Stock negativo no cambia el stock (unhappy, HU-02-07, Listo cuando)

- **WHEN** el producto tiene stock `"5"`
- **AND** el `POST` de ajuste `"-8"` con motivo responde `400` con `code` `negative_stock`
- **THEN** se muestra el mensaje en español de `negative_stock`
- **AND** el stock de solo lectura sigue `"5"`
- **AND** la ficha permanece abierta en Stock

#### Scenario: invalid_input pinta quantity (unhappy)

- **WHEN** el POST responde `400` con `code` `invalid_input` y `errors[].field` `quantity`
- **THEN** el `vx-field` de cantidad recibe el mensaje
- **AND** el toast muestra el mensaje en español de `invalid_input`
- **AND** la ficha permanece abierta

#### Scenario: not_found en POST de movimiento (unhappy)

- **WHEN** el POST responde `404` con `code` `not_found`
- **THEN** se muestra el mensaje en español de `not_found`
- **AND** la ficha permanece abierta

#### Scenario: red en POST de movimiento (unhappy)

- **WHEN** el POST falla con error `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** la ficha permanece abierta

#### Scenario: no hay edición ni borrado de movimientos (unhappy, HU-02-07 Inmutabilidad)

- **WHEN** se inspecciona el panel Stock
- **THEN** no hay acción de editar ni borrar un movimiento
- **AND** no hay `PATCH` ni `DELETE` hacia `stock-movements`

#### Scenario: no hay historial ni exportar (unhappy, No hace)

- **WHEN** se inspecciona el panel Stock o se registra un movimiento
- **THEN** no se envía `GET /api/v1/products/{id}/stock-movements`
- **AND** no hay acción de exportar el historial
- **AND** no hay tabla paginada de movimientos

#### Scenario: cantidad vacía no envía POST (unhappy)

- **WHEN** el tipo es Ingreso y la cantidad está vacía
- **AND** el administrador intenta registrar
- **THEN** no se envía `POST`

#### Scenario: alta sigue con Stock no operable (happy / frontera)

- **WHEN** la ficha de alta está abierta
- **THEN** la pestaña Stock tiene `aria-disabled="true"` o equivalente
- **AND** activarla no envía `POST .../stock-movements`

### Requirement: Tests de pestaña Stock

Antes de archive, la suite MUST incluir:

| Test | Ubicación | Tipo |
|---|---|---|
| `sheet_edit_stock_tab_is_operable` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_stock_shows_form_without_http` | `product-sheet.test.mjs` | happy |
| `sheet_stock_inbound_posts_and_shows_resulting_stock` | `product-sheet.test.mjs` | happy (HU-02-07, Listo cuando) |
| `sheet_stock_adjustment_to_zero_with_reason` | `product-sheet.test.mjs` | happy (HU-02-07) |
| `sheet_stock_adjustment_without_reason_shows_invalid_input` | `product-sheet.test.mjs` | unhappy (HU-02-07, Listo cuando) |
| `sheet_stock_negative_keeps_stock_and_shows_negative_stock` | `product-sheet.test.mjs` | unhappy (HU-02-07, Listo cuando) |
| `sheet_stock_invalid_input_sets_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_not_found_shows_not_found` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_network_shows_network` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_empty_quantity_does_not_post` | `product-sheet.test.mjs` | unhappy |
| `sheet_stock_has_no_edit_or_delete_movement` | `product-sheet.test.mjs` | unhappy (Inmutabilidad) |
| `sheet_stock_does_not_get_or_export_history` | `product-sheet.test.mjs` | unhappy (No hace) |
| `sheet_create_stock_tab_stays_disabled` | `product-sheet.test.mjs` | happy / frontera |

`./scripts/check-vectra-web.sh` MUST ejecutarlos. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5). El Listo cuando del ticket (escenarios de HU-02-07 a mano) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable (la cabecera `X-Vectra-Terminal` queda cubierta por `api.js` / T-01-06).

#### Scenario: la suite de Stock corre en check (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** incluye `product-sheet.test.mjs` con los tests de la tabla de este requirement

#### Scenario: el Listo cuando está cubierto por tests nombrados (unhappy)

- **WHEN** se inspecciona `product-sheet.test.mjs`
- **THEN** existen `sheet_stock_inbound_posts_and_shows_resulting_stock`, `sheet_stock_adjustment_without_reason_shows_invalid_input` y `sheet_stock_negative_keeps_stock_and_shows_negative_stock`

### Requirement: Búsqueda mientras se escribe en el Omnibox

Cuando el *router* pinta `data-view="operator"`, MUST montar la búsqueda sobre `data-region="omnibox"` y `data-region="results"`. `vx-omnibox` MUST seguir emitiendo `vx-query` y MUST NOT llamar a `/search` ni a `/search/by-code` ni importar el cliente HTTP (requirement «Modos del Omnibox»).

En cada `vx-query` con `detail.mode === "text"` y texto recortado no vacío, MUST:

1. Cancelar las peticiones anteriores (`/search` y, si existían, `by-code`) con `AbortController` (el `signal` MUST reenviarse al cliente HTTP, T-01-06 / [5.7](../../../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación)). Ambas peticiones de la misma consulta MUST compartir el mismo `AbortController` (o abortarse juntas).
2. Invocar `GET /api/v1/search` en el mismo origen con query `q` igual al texto (sin prefijo de API extra; MUST usar `encodeURIComponent` o equivalente). MUST NOT enviar `facets`, `categoryId`, `brandId`, `inStock` ni `attr`. `sort`/`offset`/`limit` MAY omitirse (defaults del contrato: relevancia, 0, 20). Si el texto supera 200 caracteres, MUST recortarlo a 200 antes de enviarlo (`maxLength` de `q` en [docs/openapi.yaml](../../../../../docs/openapi.yaml)).
3. MUST NOT aplicar *debounce* ni umbral de teclas: una ronda de peticiones por evento de consulta.
4. Si el texto recortado **parece un código** — sin espacios en blanco, longitud 1..=64, y compuesto solo de letras ASCII, dígitos o guiones (`[A-Za-z0-9-]+`) — MUST invocar **en paralelo** `GET /api/v1/search/by-code/{code}` con `{code}` igual a ese texto (`encodeURIComponent` en el segmento de path). MUST NOT recortar a 200 para `by-code`: si la longitud supera 64, MUST NOT llamar a `by-code`. Si el texto **no** parece un código, MUST NOT llamar a `by-code`.

MUST pintar en `data-region="results"` **solo** las respuestas cuya ronda no fue abortada. Hits de una petición abortada MUST NOT reemplazar la lista. Si llega un `AbortError` (en `/search`, en `by-code` o en ambos), MUST NOT pintar esa ronda y MUST NOT invocar `showError` por el abort.

Si `by-code` responde 200 con `items` no vacío, MUST fijar esas coincidencias **arriba** de la lista, con la etiqueta visible **"Código exacto"** (español, no interactiva: MUST NOT usar `vx-chip` ni un control que reciba foco de teclado). Cada ítem de `by-code` MUST pintarse como `vx-product-card` `full` mapeando `ProductSummary`:

| ProductSummary | Atributo `vx-product-card` |
|---|---|
| `sku` | `sku` |
| `name` | `name` |
| `brand` | `brand` (omitir o vacío si `null`) |
| `price` | `price` |
| `stock` | `stock` |
| `imageUrl` | `image-url` (omitir si `null` o vacío) |

`ProductSummary` no trae `location` ni `highlight`: las tarjetas exactas MUST omitir ubicación y MUST NOT inventar `<mark>`. Si `stock` es cero, MUST aplicar el estado «Sin stock» de T-03-17.

Los `hits` de `/search` MUST pintarse **después** del grupo exacto. Un hit cuyo `id` coincide con un ítem ya pinneado MUST NOT repetirse. `items: []` (o `by-code` ausente porque no parece código) MUST NOT mostrar la etiqueta «Código exacto».

Cada hit de `/search` MUST renderizarse como un `vx-product-card` con variante `full` (atributo `variant="full"` o omitido; default del componente). MUST mapear el `SearchHit` de [docs/openapi.yaml](../../../../../docs/openapi.yaml) a atributos de la tarjeta:

| SearchHit | Atributo `vx-product-card` |
|---|---|
| `sku` | `sku` |
| `name` | `name` (texto plano de respaldo) |
| `brand` | `brand` (omitir o vacío si `null`) |
| `price` | `price` |
| `stock` | `stock` |
| `location` | `location` (omitir o vacío si `null`) |
| `imageUrl` | `image-url` (omitir si `null` o vacío) |

El **nombre** de un hit de `/search` MUST preferir `highlight.name` sanitizado vía `appendSanitizedHtml` en un hijo `slot="name"` cuando ese campo existe y no está vacío; si `highlight` es `null` o `highlight.name` falta, MUST usar `name` como texto plano (atributo `name` y/o texto del slot). SKU, marca, precio, stock y ubicación MUST ir como atributos (la tarjeta los pinta con `textContent`). MUST NOT asignar el payload crudo a `innerHTML`. MUST NOT pintar `highlight.description` (la tarjeta no tiene ese campo).

Si `imageUrl` está ausente o vacío, la tarjeta MUST mostrar el *placeholder* de imagen del componente (HU-03-05 MVP parcial: sin foto principal). Si `imageUrl` tiene valor, MUST pasarlo a `image-url`.

Si `stock` representa cero, la tarjeta MUST exponer el estado visual «sin stock» del componente (`data-out-of-stock` y texto «Sin stock»). MUST NOT indicar falta de stock solo con color.

La lista MUST ser un `role="listbox"` operable por teclado con el helper de T-01-10; cada tarjeta MUST ser una opción de esa lista (la etiqueta «Código exacto» MUST NOT ser una opción). El primer `ArrowDown` desde el Omnibox MUST activar la primera coincidencia exacta si existe, si no el primer hit de `/search`. MUST NOT abrir ficha ni panel de recomendaciones al activar un hit (T-04-08). MUST NOT usar la variante `compact` en esta lista. MUST NOT invocar un lector de código de barras.

Consulta vacía (incluido Escape que deja el Omnibox vacío, T-01-08) o solo espacios: MUST abortar las peticiones en curso, MUST vaciar `data-region="results"` de hits, de `vx-product-card` y de la etiqueta «Código exacto», y MUST NOT llamar a `/search` ni a `by-code`.

`vx-query` con `mode` distinto de `"text"` (`ai`, `voice`, `image`): MUST abortar las peticiones en curso si existen y MUST NOT llamar a `/search` ni a `by-code`. MUST NOT ejecutar `/ai`, `/speech` ni cámara.

`hits: []` y `items` de `by-code` vacío MUST dejar la lista sin opciones ni `vx-product-card` ni etiqueta «Código exacto» (MAY un mensaje en español «Sin resultados»). MUST NOT dejar hits de una consulta anterior.

Un hit o ítem que no es un objeto, o al que le faltan campos, MUST NOT lanzar: MUST omitir ese elemento o pintarlo con los campos presentes vacíos.

Las dos peticiones fallan de forma **independiente** (no abort):

- `/search` falla y `by-code` 200: MUST pintar el grupo exacto (si hay `items`) y MUST invocar `showError` con el error de `/search`.
- `by-code` falla y `/search` 200: MUST pintar los hits de `/search` **sin** etiqueta «Código exacto» y MUST invocar `showError` con el error de `by-code`.
- ambas fallan: MUST invocar `showError` al menos una vez y MUST NOT pintar esa ronda.
- lista previa MAY permanecer si no hay nada que pintar.

Vistas `data-view="admin"` y `data-view="not-found"` MUST NOT llamar a `/search` ni a `by-code`. Al dejar la vista operador MUST abortar las peticiones en curso.

Textos visibles MUST estar en español. `base.css` MAY estilar el grupo «Código exacto» solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

#### Scenario: consulta de texto llama a GET /search con q (happy)

- **WHEN** la vista operador está montada
- **AND** el Omnibox emite `vx-query` con `mode: "text"` y `text: "perno m8"`
- **THEN** se envía `GET /api/v1/search` cuyo query incluye `q` con ese texto
- **AND** no se envían `facets`, `categoryId`, `brandId` ni `inStock`
- **AND** no se envía `GET /api/v1/search/by-code/{code}`

#### Scenario: consulta que parece código llama a by-code en paralelo (happy)

- **WHEN** la vista operador está montada
- **AND** el Omnibox emite `vx-query` con `mode: "text"` y `text: "OEM-PER-8840"`
- **THEN** se envía `GET /api/v1/search?q=OEM-PER-8840`
- **AND** se envía `GET /api/v1/search/by-code/OEM-PER-8840` (path con el código; MAY ir percent-encoded)
- **AND** ambas peticiones se disparan antes de que responda cualquiera de las dos

#### Scenario: OEM compartido pinta ambas piezas primero (happy, Listo cuando)

- **WHEN** `GET /api/v1/search/by-code/OEM-PER-8840` responde 200 con `items` de dos productos (`PER-ACERO-01` y `PER-ACERO-02`)
- **AND** `GET /api/v1/search?q=OEM-PER-8840` responde 200 con hits que MAY incluir esos SKUs u otros
- **THEN** `data-region="results"` muestra el texto «Código exacto»
- **AND** las dos primeras `vx-product-card` de la lista corresponden a esos dos productos (cualquier orden relativo entre ellas)
- **AND** esas dos tarjetas no se repiten más abajo

#### Scenario: escribir rápido cancela la petición anterior y pinta solo la última (happy, Listo cuando)

- **WHEN** la vista operador está montada
- **AND** llegan `vx-query` de texto `"p"`, luego `"pe"`, luego `"perno m8"` antes de que respondan las anteriores
- **THEN** cada petición de las rondas `"p"` y `"pe"` (incluidas las de `by-code`, porque esas consultas parecen código) queda abortada (`AbortSignal.aborted === true`)
- **AND** `data-region="results"` muestra los hits de `"perno m8"`
- **AND** no muestra hits que solo correspondían a `"p"` o `"pe"`

#### Scenario: la lista muestra nombre y SKU como texto (happy)

- **WHEN** `GET /api/v1/search` responde 200 con un hit `name: "Perno de Acero"` y `sku: "PER-STEEL"`, o con `highlight.name: "<mark>Perno</mark> de Acero"`
- **THEN** `data-region="results"` contiene ese nombre y ese SKU visibles
- **AND** si hay `highlight.name` con `<mark>`, existe un elemento `mark` en la fila
- **AND** hay un `vx-product-card` en la región
- **AND** el contenedor tiene `role="listbox"`

#### Scenario: cada hit se pinta como vx-product-card completa (happy, Listo cuando HU-03-05)

- **WHEN** `GET /api/v1/search` responde 200 con un hit `name: "Perno de Acero"`, `sku: "PER-STEEL"`, `brand: "Fixser"`, `price: "120.00"`, `stock: "40"`, `location: "P3 · E-B · 12"` y `highlight.name: "<mark>Perno</mark> de Acero"`
- **THEN** `data-region="results"` contiene un `vx-product-card` por hit
- **AND** esa tarjeta muestra nombre, SKU, marca, precio, stock y ubicación
- **AND** existe un elemento `mark` en el nombre (slot `name`)
- **AND** el contenedor tiene `role="listbox"`
- **AND** no se usa `variant="compact"`

#### Scenario: sin stock se indica en la tarjeta (happy)

- **WHEN** un hit trae `stock: "0"` (o decimal equivalente a cero)
- **THEN** su `vx-product-card` expone `data-out-of-stock`
- **AND** el texto visible incluye «Sin stock»

#### Scenario: sin imageUrl se muestra placeholder (happy, MVP)

- **WHEN** un hit trae `imageUrl` `null` o ausente
- **THEN** la tarjeta no renderiza un `<img>` de producto
- **AND** muestra el *placeholder* de imagen del componente

#### Scenario: imageUrl presente se pasa a la tarjeta (happy)

- **WHEN** un hit trae `imageUrl: "https://example.test/perno.jpg"`
- **THEN** la tarjeta recibe `image-url` con esa URL
- **AND** el `<img>` de la variante `full` usa ese `src`

#### Scenario: flechas desde el Omnibox recorren los resultados (happy)

- **WHEN** hay al menos un hit pintado como `vx-product-card`
- **AND** el input del Omnibox tiene el foco
- **AND** se pulsa ArrowDown
- **THEN** el primer hit queda activo (`aria-selected="true"` / `aria-activedescendant`)
- **AND** no se lanza excepción

#### Scenario: ArrowDown con coincidencias exactas activa la primera exacta (happy)

- **WHEN** hay al menos una tarjeta en el grupo «Código exacto» y hits de `/search` debajo
- **AND** el input del Omnibox tiene el foco
- **AND** se pulsa ArrowDown
- **THEN** la primera tarjeta del grupo exacto queda activa
- **AND** la etiqueta «Código exacto» no queda como opción activa

#### Scenario: la respuesta abortada no se pinta (unhappy)

- **WHEN** una petición de `"p"` se aborta al escribir `"pe"`
- **AND** después llega el cuerpo de `"p"` (de `/search` o de `by-code`)
- **THEN** `data-region="results"` no muestra esos hits ni esa etiqueta «Código exacto»

#### Scenario: AbortError no llama a showError (unhappy)

- **WHEN** el cliente HTTP rechaza con `AbortError` al cancelar `/search` o `by-code`
- **THEN** no se invoca `showError`
- **AND** no se lanza excepción

#### Scenario: modo IA, voz o imagen no llama a search (unhappy)

- **WHEN** llega `vx-query` con `mode: "ai"` y `text: "?hola"`, o `mode: "voice"`, o `mode: "image"`
- **THEN** no se envía `GET /api/v1/search`
- **AND** no se envía `GET /api/v1/search/by-code/{code}`
- **AND** no se llama a `/ai`, `/speech` ni a la cámara

#### Scenario: consulta vacía limpia sin fetch (unhappy)

- **WHEN** hay hits visibles de una consulta previa
- **AND** llega `vx-query` con `mode: "text"` y `text: ""` (p. ej. Escape en el Omnibox)
- **THEN** no se envía `GET /api/v1/search`
- **AND** no se envía `GET /api/v1/search/by-code/{code}`
- **AND** `data-region="results"` queda sin hits ni `vx-product-card` ni etiqueta «Código exacto»

#### Scenario: error de API muestra toast y no pinta (unhappy)

- **WHEN** `GET /api/v1/search` responde 400, 503 o 504 con `problem+json`, o el cliente lanza error `network`
- **AND** no hubo `by-code` (la consulta no parece código) o `by-code` también falló
- **THEN** se invoca `showError` con ese error
- **AND** no se pintan hits de esa respuesta de `/search`

#### Scenario: by-code falla y /search llega 200 (unhappy)

- **WHEN** la consulta parece un código
- **AND** `GET /api/v1/search/by-code/{code}` responde 400 o 503 con `problem+json`, o el cliente lanza `network`
- **AND** `GET /api/v1/search` responde 200 con hits
- **THEN** se invoca `showError` con el error de `by-code`
- **AND** `data-region="results"` pinta los hits de `/search`
- **AND** no muestra la etiqueta «Código exacto»

#### Scenario: /search falla y by-code llega 200 (unhappy)

- **WHEN** la consulta parece un código
- **AND** `GET /api/v1/search` responde 503 o el cliente lanza `network`
- **AND** `GET /api/v1/search/by-code/{code}` responde 200 con `items` no vacío
- **THEN** se invoca `showError` con el error de `/search`
- **AND** `data-region="results"` pinta las tarjetas exactas con la etiqueta «Código exacto»

#### Scenario: hits vacíos dejan la lista sin opciones (unhappy)

- **WHEN** la respuesta 200 trae `hits: []` para la consulta vigente
- **AND** no hay `items` de `by-code` (no parece código, o `items: []`)
- **THEN** `data-region="results"` no muestra opciones de producto ni `vx-product-card` ni etiqueta «Código exacto»
- **AND** no se invoca `showError`

#### Scenario: by-code vacío no pinnea (unhappy)

- **WHEN** la consulta parece un código
- **AND** `GET /api/v1/search/by-code/{code}` responde 200 con `items: []`
- **AND** `GET /api/v1/search` responde 200 con hits
- **THEN** `data-region="results"` no muestra la etiqueta «Código exacto»
- **AND** pinta solo los hits de `/search`

#### Scenario: consulta con espacios no llama a by-code (unhappy)

- **WHEN** el Omnibox emite `vx-query` con `mode: "text"` y `text: "perno m8"`
- **THEN** no se envía `GET /api/v1/search/by-code/{code}`

#### Scenario: código más largo que 64 no llama a by-code (unhappy)

- **WHEN** el texto recortado tiene 65 caracteres de `[A-Za-z0-9-]` y ningún espacio
- **THEN** se envía `GET /api/v1/search` (con `q` recortado a 200)
- **AND** no se envía `GET /api/v1/search/by-code/{code}`

#### Scenario: HTML de highlight se muestra como texto (unhappy)

- **WHEN** un hit trae `highlight.name` con `<mark>` u otro HTML (p. ej. `<img onerror>`)
- **THEN** solo `<mark>` se interpreta como elemento (vía `highlight.js`)
- **AND** no hay descendiente `img` ni otro tag fuera de `mark`
- **AND** el payload peligroso permanece como texto escapado

#### Scenario: HTML en campos de texto no se interpreta (unhappy)

- **WHEN** un hit trae `brand`, `sku`, `price` o `location` con markup (p. ej. `<img onerror>`)
- **THEN** ese markup se muestra como texto en la tarjeta
- **AND** no se crea un `<img>` ni se ejecuta script a partir de esos campos

#### Scenario: hit malformado no lanza (unhappy)

- **WHEN** `hits` incluye `null`, un string, o un objeto sin `sku`/`name`
- **THEN** no se lanza excepción
- **AND** los hits válidos de la misma respuesta MAY pintarse

#### Scenario: highlight.description no se pinta en la tarjeta (unhappy / no-regresión)

- **WHEN** un hit trae `highlight.description` con `<mark>zincado</mark>` y `highlight.name` vacío
- **THEN** la tarjeta usa el `name` plano
- **AND** no se exige un `mark` proveniente de la descripción

#### Scenario: administración no llama a search (unhappy)

- **WHEN** el hash es `#/admin/productos`
- **THEN** no se envía `GET /api/v1/search`
- **AND** no se envía `GET /api/v1/search/by-code/{code}`

### Requirement: Tests de búsqueda mientras se escribe

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm).

| Test | Dónde | Tipo |
|---|---|---|
| `search_text_query_calls_get_search_with_q` | `search.test.mjs` | happy |
| `search_code_like_query_calls_by_code_in_parallel` | `search.test.mjs` | happy |
| `search_shared_oem_pins_both_exact_matches_first` | `search.test.mjs` | happy, Listo cuando HU-03-08 |
| `search_cancels_previous_and_paints_last_only` | `search.test.mjs` | happy |
| `search_renders_full_product_card` | `search.test.mjs` | happy, Listo cuando HU-03-05 |
| `search_renders_name_and_sku_as_text` | `search.test.mjs` | happy (nombre+SKU visibles en la tarjeta) |
| `search_zero_stock_marks_out_of_stock` | `search.test.mjs` | happy |
| `search_missing_image_url_shows_placeholder` | `search.test.mjs` | happy |
| `search_image_url_sets_card_src` | `search.test.mjs` | happy |
| `search_arrow_down_from_omnibox_selects_first_hit` | `search.test.mjs` | happy |
| `search_arrow_down_selects_first_exact_match` | `search.test.mjs` | happy |
| `search_aborted_response_does_not_paint` | `search.test.mjs` | unhappy |
| `search_abort_error_does_not_call_showError` | `search.test.mjs` | unhappy |
| `search_non_text_mode_does_not_call_search` | `search.test.mjs` | unhappy |
| `search_empty_query_clears_without_fetch` | `search.test.mjs` | unhappy |
| `search_problem_json_calls_showError` | `search.test.mjs` | unhappy |
| `search_by_code_error_still_paints_search_hits` | `search.test.mjs` | unhappy |
| `search_search_error_still_paints_exact_matches` | `search.test.mjs` | unhappy |
| `search_empty_hits_shows_empty_list` | `search.test.mjs` | unhappy |
| `search_by_code_empty_items_does_not_pin` | `search.test.mjs` | unhappy |
| `search_phrase_with_spaces_does_not_call_by_code` | `search.test.mjs` | unhappy |
| `search_code_longer_than_64_does_not_call_by_code` | `search.test.mjs` | unhappy |
| `search_highlight_name_paints_mark` | `search.test.mjs` | happy |
| `search_highlight_null_falls_back_to_name` | `search.test.mjs` | happy |
| `search_highlight_img_onerror_no_img_element` | `search.test.mjs` | unhappy |
| `search_html_in_plain_fields_is_text` | `search.test.mjs` | unhappy |
| `search_malformed_hit_does_not_throw` | `search.test.mjs` | unhappy |
| `search_highlight_description_does_not_paint_on_card` | `search.test.mjs` | unhappy |
| `search_admin_view_does_not_call_search` | `search.test.mjs` | unhappy |
| `search_missing_module_fails` | `check-vectra-web.sh` | unhappy |

`search_highlight_description_paints_mark` MUST NOT seguir formando parte de la suite: `highlight.description` no se pinta en `vx-product-card`.

#### Scenario: la suite de búsqueda pasa en check-vectra-web (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** termina con código 0
- **AND** corre `search.test.mjs`

#### Scenario: falta search.js (unhappy)

- **WHEN** falta `vectra-web/js/search.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `search.js`

#### Scenario: los tests de by-code están nombrados (happy)

- **WHEN** se inspecciona `scripts/fixtures/vectra-web/search.test.mjs`
- **THEN** existen `search_code_like_query_calls_by_code_in_parallel` y `search_shared_oem_pins_both_exact_matches_first`
- **AND** existen `search_by_code_error_still_paints_search_hits`, `search_search_error_still_paints_exact_matches`, `search_by_code_empty_items_does_not_pin`, `search_phrase_with_spaces_does_not_call_by_code` y `search_code_longer_than_64_does_not_call_by_code`

### Requirement: Resaltado seguro (`highlight.js`)

El repositorio MUST contener `vectra-web/js/highlight.js` exportando al menos `sanitizeHighlight` y `appendSanitizedHtml`.

`sanitizeHighlight(html)` MUST escapar `&`, `<`, `>`, `"` y `'`, MUST restaurar únicamente las secuencias `<mark>` y `</mark>` (sin atributos) y MUST NOT restaurar ningún otro tag.

`appendSanitizedHtml(parent, html)` MUST invocar `sanitizeHighlight`, MUST construir hijos con `createTextNode` y `createElement('mark')` con `textContent`, y MUST NOT asignar el fragmento crudo a `innerHTML`.

#### Scenario: mark legítimo produce elemento mark (happy)

- **WHEN** `appendSanitizedHtml` recibe `<mark>Perno</mark> hexagonal`
- **THEN** el contenedor tiene un descendiente `mark` con texto `Perno`
- **AND** el texto restante incluye `hexagonal`

#### Scenario: img onerror se muestra como texto (unhappy, Listo cuando)

- **WHEN** el fragmento incluye `<img onerror="alert(1)">`
- **THEN** no se crea descendiente `img`
- **AND** el texto visible incluye la secuencia escapada del tag

#### Scenario: em y mark con atributos no se restauran (unhappy)

- **WHEN** el fragmento es `<em>x</em>` o `<mark onclick="x">y</mark>`
- **THEN** no hay elemento `em`
- **AND** no hay elemento `mark`

### Requirement: Tests de resaltado seguro

| Test | Dónde | Tipo |
|---|---|---|
| `highlight_restores_mark_around_match` | `highlight.test.mjs` | happy |
| `highlight_img_onerror_is_plain_text` | `highlight.test.mjs` | unhappy |
| `highlight_script_tag_is_plain_text` | `highlight.test.mjs` | unhappy |
| `highlight_em_not_restored` | `highlight.test.mjs` | unhappy |
| `highlight_mark_with_attributes_not_restored` | `highlight.test.mjs` | unhappy |
| `search_highlight_name_paints_mark` | `search.test.mjs` | happy |
| `search_highlight_description_does_not_paint_on_card` | `search.test.mjs` | unhappy |
| `search_highlight_null_falls_back_to_name` | `search.test.mjs` | happy |
| `search_highlight_img_onerror_no_img_element` | `search.test.mjs` | unhappy |
| `highlight_missing_module_fails` | `check-vectra-web.sh` | unhappy |

`check-vectra-web.sh` MUST exigir `highlight.js` y ejecutar `highlight.test.mjs`. MUST NOT exigir `search_highlight_description_paints_mark`.

#### Scenario: la suite de highlight pasa en check-vectra-web (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `highlight.test.mjs` y termina con código 0 respecto a highlight

#### Scenario: falta highlight.js (unhappy)

- **WHEN** falta `vectra-web/js/highlight.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `highlight.js`
