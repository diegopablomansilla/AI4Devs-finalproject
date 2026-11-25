# Spec Delta

Ticket padre: T-03-16. Change: T-03-16. HU: HU-03-05 (Resaltado seguro).

## ADDED Requirements

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
| `search_highlight_description_paints_mark` | `search.test.mjs` | happy |
| `search_highlight_null_falls_back_to_name` | `search.test.mjs` | happy |
| `search_highlight_img_onerror_no_img_element` | `search.test.mjs` | unhappy |
| `highlight_missing_module_fails` | `check-vectra-web.sh` | unhappy |

`check-vectra-web.sh` MUST exigir `highlight.js` y ejecutar `highlight.test.mjs`.

#### Scenario: la suite de highlight pasa en check-vectra-web (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `highlight.test.mjs` y termina con código 0 respecto a highlight

#### Scenario: falta highlight.js (unhappy)

- **WHEN** falta `vectra-web/js/highlight.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `highlight.js`

## MODIFIED Requirements

### Requirement: Búsqueda mientras se escribe en el Omnibox

Cuando el *router* pinta `data-view="operator"`, MUST montar la búsqueda sobre `data-region="omnibox"` y `data-region="results"`. `vx-omnibox` MUST seguir emitiendo `vx-query` y MUST NOT llamar a `/search` ni importar el cliente HTTP (requirement «Modos del Omnibox»).

En cada `vx-query` con `detail.mode === "text"` y texto recortado no vacío, MUST:

1. Cancelar la petición anterior con `AbortController` (el `signal` MUST reenviarse al cliente HTTP, T-01-06 / [5.7](../../../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación)).
2. Invocar `GET /api/v1/search` en el mismo origen con query `q` igual al texto (sin prefijo de API extra; MUST usar `encodeURIComponent` o equivalente). MUST NOT enviar `facets`, `categoryId`, `brandId`, `inStock` ni `attr`. `sort`/`offset`/`limit` MAY omitirse (defaults del contrato: relevancia, 0, 20). Si el texto supera 200 caracteres, MUST recortarlo a 200 antes de enviarlo (`maxLength` de `q` en [docs/openapi.yaml](../../../../../docs/openapi.yaml)).
3. MUST NOT aplicar *debounce* ni umbral de teclas: una petición por evento de consulta.

MUST pintar en `data-region="results"` **solo** la respuesta cuya petición no fue abortada (Listo cuando). Hits de una petición abortada MUST NOT reemplazar la lista. Si llega un `AbortError`, MUST NOT pintar y MUST NOT invocar `showError`.

Cada hit MUST mostrarse con **nombre** (desde `highlight.name` sanitizado vía `appendSanitizedHtml` si existe, si no `name` en texto plano) y **SKU** como texto plano. MAY mostrar `highlight.description` sanitizado. La lista MUST ser un `role="listbox"` operable por teclado con el helper de T-01-10. MUST NOT usar `innerHTML` del payload sin `highlight.js`. MUST NOT exigir `vx-product-card` ni pintar marca, precio, stock, ubicación o imagen (**No hace** / T-03-17). MUST NOT abrir ficha ni panel de recomendaciones al activar un hit (T-04-08).

Consulta vacía (incluido Escape que deja el Omnibox vacío, T-01-08) o solo espacios: MUST abortar la petición en curso, MUST vaciar `data-region="results"` de hits y MUST NOT llamar a `/search`.

`vx-query` con `mode` distinto de `"text"` (`ai`, `voice`, `image`): MUST abortar la petición en curso si existe y MUST NOT llamar a `/search`. MUST NOT ejecutar `/ai`, `/speech` ni cámara.

`hits: []` MUST dejar la lista sin opciones (MAY un mensaje en español «Sin resultados»). MUST NOT dejar hits de una consulta anterior.

Errores HTTP o de red que no sean abort (`invalid_input`, `unavailable`, `timeout`, `network`, …): MUST invocar `showError` con el error del cliente y MUST NOT pintar esa respuesta. La lista previa MAY permanecer.

Vistas `data-view="admin"` y `data-view="not-found"` MUST NOT llamar a `/search`. Al dejar la vista operador MUST abortar la petición en curso.

Textos visibles MUST estar en español. `base.css` MAY estilar la lista solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

#### Scenario: consulta de texto llama a GET /search con q (happy)

- **WHEN** la vista operador está montada
- **AND** el Omnibox emite `vx-query` con `mode: "text"` y `text: "perno m8"`
- **THEN** se envía `GET /api/v1/search` cuyo query incluye `q` con ese texto
- **AND** no se envían `facets`, `categoryId`, `brandId` ni `inStock`

#### Scenario: escribir rápido cancela la petición anterior y pinta solo la última (happy, Listo cuando)

- **WHEN** la vista operador está montada
- **AND** llegan `vx-query` de texto `"p"`, luego `"pe"`, luego `"perno m8"` antes de que respondan las anteriores
- **THEN** cada petición salvo la última queda abortada (`AbortSignal.aborted === true`)
- **AND** `data-region="results"` muestra los hits de `"perno m8"`
- **AND** no muestra hits que solo correspondían a `"p"` o `"pe"`

#### Scenario: la lista muestra nombre y SKU como texto (happy)

- **WHEN** `GET /api/v1/search` responde 200 con un hit `name: "Perno de Acero"` y `sku: "PER-STEEL"`, o con `highlight.name: "<mark>Perno</mark> de Acero"`
- **THEN** `data-region="results"` contiene ese nombre y ese SKU visibles
- **AND** si hay `highlight.name` con `<mark>`, existe un elemento `mark` en la fila
- **AND** no hay `vx-product-card` en la región
- **AND** el contenedor tiene `role="listbox"`

#### Scenario: flechas desde el Omnibox recorren los resultados (happy)

- **WHEN** hay al menos un hit pintado
- **AND** el input del Omnibox tiene el foco
- **AND** se pulsa ArrowDown
- **THEN** el primer hit queda activo (`aria-selected="true"` / `aria-activedescendant`)
- **AND** no se lanza excepción

#### Scenario: la respuesta abortada no se pinta (unhappy)

- **WHEN** una petición de `"p"` se aborta al escribir `"pe"`
- **AND** después llega el cuerpo de `"p"`
- **THEN** `data-region="results"` no muestra esos hits

#### Scenario: AbortError no llama a showError (unhappy)

- **WHEN** el cliente HTTP rechaza con `AbortError` al cancelar
- **THEN** no se invoca `showError`
- **AND** no se lanza excepción

#### Scenario: modo IA, voz o imagen no llama a search (unhappy)

- **WHEN** llega `vx-query` con `mode: "ai"` y `text: "?hola"`, o `mode: "voice"`, o `mode: "image"`
- **THEN** no se envía `GET /api/v1/search`
- **AND** no se llama a `/ai`, `/speech` ni a la cámara

#### Scenario: consulta vacía limpia sin fetch (unhappy)

- **WHEN** hay hits visibles de una consulta previa
- **AND** llega `vx-query` con `mode: "text"` y `text: ""` (p. ej. Escape en el Omnibox)
- **THEN** no se envía `GET /api/v1/search`
- **AND** `data-region="results"` queda sin hits

#### Scenario: error de API muestra toast y no pinta (unhappy)

- **WHEN** `GET /api/v1/search` responde 400, 503 o 504 con `problem+json`, o el cliente lanza error `network`
- **THEN** se invoca `showError` con ese error
- **AND** no se pintan hits de esa respuesta

#### Scenario: hits vacíos dejan la lista sin opciones (unhappy)

- **WHEN** la respuesta 200 trae `hits: []` para la consulta vigente
- **THEN** `data-region="results"` no muestra opciones de producto
- **AND** no se invoca `showError`

#### Scenario: HTML de highlight se muestra como texto (unhappy)

- **WHEN** un hit trae `highlight.name` con `<mark>` u otro HTML (p. ej. `<img onerror>`)
- **THEN** solo `<mark>` se interpreta como elemento (vía `highlight.js`)
- **AND** no hay descendiente `img` ni otro tag fuera de `mark`
- **AND** el payload peligroso permanece como texto escapado

#### Scenario: administración no llama a search (unhappy)

- **WHEN** el hash es `#/admin/productos`
- **THEN** no se envía `GET /api/v1/search`

### Requirement: Tests de búsqueda mientras se escribe

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm).

| Test | Dónde | Tipo |
|---|---|---|
| `search_text_query_calls_get_search_with_q` | `search.test.mjs` | happy |
| `search_cancels_previous_and_paints_last_only` | `search.test.mjs` | happy, Listo cuando |
| `search_renders_name_and_sku_as_text` | `search.test.mjs` | happy |
| `search_arrow_down_from_omnibox_selects_first_hit` | `search.test.mjs` | happy |
| `search_aborted_response_does_not_paint` | `search.test.mjs` | unhappy |
| `search_abort_error_does_not_call_showError` | `search.test.mjs` | unhappy |
| `search_non_text_mode_does_not_call_search` | `search.test.mjs` | unhappy |
| `search_empty_query_clears_without_fetch` | `search.test.mjs` | unhappy |
| `search_problem_json_calls_showError` | `search.test.mjs` | unhappy |
| `search_empty_hits_shows_empty_list` | `search.test.mjs` | unhappy |
| `search_highlight_name_paints_mark` | `search.test.mjs` | happy |
| `search_highlight_description_paints_mark` | `search.test.mjs` | happy |
| `search_highlight_null_falls_back_to_name` | `search.test.mjs` | happy |
| `search_highlight_img_onerror_no_img_element` | `search.test.mjs` | unhappy |
| `search_admin_view_does_not_call_search` | `search.test.mjs` | unhappy |
| `search_missing_module_fails` | `check-vectra-web.sh` | unhappy |

#### Scenario: la suite de búsqueda pasa en check-vectra-web (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** termina con código 0
- **AND** corre `search.test.mjs`

#### Scenario: falta search.js (unhappy)

- **WHEN** falta `vectra-web/js/search.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `search.js`

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