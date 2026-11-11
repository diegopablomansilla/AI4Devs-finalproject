# Spec Delta

Ticket padre: T-01-02. Change: T-01-02. HU: HU-01-02.

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: Árbol vectra-web con documento en español y ES Modules

El repositorio MUST contener el directorio `vectra-web/` en la raíz, con al menos:

- `vectra-web/index.html`
- `vectra-web/css/tokens.css`
- `vectra-web/css/base.css`
- `vectra-web/js/app.js`
- `vectra-web/js/router.js`
- `vectra-web/js/components/` (directorio; MAY estar vacío salvo un marcador de árbol)

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module. Este change MUST NOT agregar Web Components en `js/components/`.

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

### Requirement: Tests del esqueleto y del router

Antes de archive, la suite MUST incluir estos tests (nombres estables; ubicación / tipo). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo |
|---|---|---|
| `index_html_declares_lang_es_and_versioned_module` | `scripts/check-vectra-web.sh` o fixture `scripts/fixtures/vectra-web/` | harness de árbol |
| `vectra_web_tree_includes_css_app_router_components` | `scripts/check-vectra-web.sh` | harness de árbol |
| `index_html_links_versioned_tokens_and_base` | `scripts/check-vectra-web.sh` | harness de árbol |
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router **y** *tokens*) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión; este change no agrega tests Rust). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica) y que no hubo recarga. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, y la presencia de `css/tokens.css`, `css/base.css`, `js/app.js`, `js/router.js` y `js/components/`. Los de *tokens* MUST asertar el contrato `--vx-*`, el contraste ≥ 4,5:1, `prefers-reduced-transparency` y la aplicación en `base.css`.

#### Scenario: la suite acordada cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh` en la raíz del repo
- **THEN** corren los tests de la tabla
- **AND** los de `#/` ↔ `#/admin/productos` sin recarga pasan
- **AND** los de *hash* vacío, desconocido, admin sin sección y slug inválido pasan
- **AND** los de `lang="es"` y el módulo versionado pasan
- **AND** los de *tokens*, contraste, transparencia reducida y `base.css` pasan

#### Scenario: falta el router o el documento (unhappy)

- **WHEN** falta `vectra-web/js/router.js` o `vectra-web/index.html`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0

#### Scenario: falta tokens.css (unhappy)

- **WHEN** falta `vectra-web/css/tokens.css`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `tokens.css`
