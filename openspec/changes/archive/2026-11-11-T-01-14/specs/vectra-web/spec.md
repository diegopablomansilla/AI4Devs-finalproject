# Spec Delta

Ticket padre: T-01-14. Change: T-01-14. HU: HU-01-01 (escenario «Pérdida de conexión con el Nodo Maestro»). No hace: caché de catálogo. La actualización del SW queda en T-01-15.

## ADDED Requirements

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

MUST NOT llamar `skipWaiting` ni `clients.claim` (T-01-15).

`js/app.js` MUST registrar el worker con `navigator.serviceWorker.register("/sw.js")` cuando `navigator.serviceWorker` existe. Si no existe, MUST NOT lanzar. MUST NOT registrar desde un script inline en `index.html`.

#### Scenario: sw.js declara precache versionado del app shell (happy)

- **WHEN** se lee `vectra-web/sw.js`
- **THEN** el nombre de caché contiene `vectra-shell-{{VERSION}}`
- **AND** el precache incluye `/`, `/index.html`, `/manifest.webmanifest`, `css/tokens.css`, `js/app.js` y al menos un ícono PNG bajo `/static/{{VERSION}}/`
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

## MODIFIED Requirements

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
- `vectra-web/js/health.js`
- `vectra-web/js/components/index.js`, `shared.js`, `vx-button.js`, `vx-field.js`, `vx-chip.js`, `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js`, `vx-product-card.js`, `vx-omnibox.js`
- `vectra-web/dev/components.html`

`index.html` MUST declarar `lang="es"` en el elemento `html`. MUST incluir un *outlet* identificable (p. ej. `id="app"`) donde el *router* pinta la vista. MUST cargar `js/app.js` como ES Module (`type="module"`) con `src` igual a `/static/{{VERSION}}/js/app.js` (el placeholder literal `{{VERSION}}`; el Gateway de T-00-19 lo sustituye al servir). MUST enlazar `css/tokens.css` y `css/base.css` (en ese orden) con `href` `/static/{{VERSION}}/css/tokens.css` y `/static/{{VERSION}}/css/base.css`. MUST enlazar el *manifest* con `href="/manifest.webmanifest"` y MUST declarar `theme-color` `#0A0A0A` (requirement «Manifest PWA de Vectra»). MUST NOT exigir un *bundler*, `package.json` ni dependencias npm para cargar la app. `js/app.js` MUST importar el *router* con `import` de ES Module, MUST importar `js/components/index.js` para registrar `vx-button`, `vx-field`, `vx-chip`, `vx-modal`, `vx-side-panel`, `vx-toast`, `vx-product-card` y `vx-omnibox`, MUST arrancar el *poller* de `js/health.js`, y MUST registrar el Service Worker (requirement «Service Worker del app shell»). Este change MUST NOT exigir que `app.js` importe `errors.js` ni `keyboard-list.js` (el cableado de resultados llega en T-03-15).

#### Scenario: el documento está en español y carga el módulo versionado (happy)

- **WHEN** se lee `vectra-web/index.html`
- **THEN** el elemento `html` tiene `lang="es"`
- **AND** existe un script `type="module"` cuyo `src` es `/static/{{VERSION}}/js/app.js`
- **AND** existe el *outlet* de la aplicación
- **AND** el archivo no referencia un *bundler* ni un `package.json`

#### Scenario: el árbol incluye css, app, router y components (happy)

- **WHEN** se inspecciona `vectra-web/`
- **THEN** existen `css/` y `js/components/` como directorios
- **AND** existen los archivos `js/app.js`, `js/router.js`, `js/api.js`, `js/errors.js`, `js/keyboard-list.js` y `js/health.js`
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
| `sw_navigation_online_uses_network` | `sw.test.mjs` | happy |
| `sw_navigation_offline_serves_cached_shell` | `sw.test.mjs` | happy (Listo cuando T-01-14) |
| `sw_health_and_api_are_network_only` | `sw.test.mjs` | unhappy (No hace) |
| `sw_precache_with_api_or_media_fails` | `sw.test.mjs` | unhappy (fixture) |
| `sw_register_without_service_worker_does_not_throw` | `sw.test.mjs` | unhappy |
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

`./scripts/check-vectra-web.sh` MUST ejecutar el harness de árbol y los tests `node --test` (router, admin, *tokens*, componentes, `api`, `errors`, `omnibox`, `keyboard-list`, `health`, `manifest` y `sw`) y MUST terminar con código 0 sobre el árbol que cumple esta spec. MUST NOT agregar `package.json` ni dependencias npm. `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde (regresión más `manifest_replaces_version` y `sw_js_replaces_version` en el crate `api`). Los tests del *router* MUST asertar el `data-view` (y `data-section` cuando aplica), que no hubo recarga, que la vista operador declara las tres regiones con `vx-omnibox` / `vx-side-panel` mientras admin no monta `vx-omnibox`, que operador y admin declaran `[data-health]` y `[data-offline-banner]` en el encabezado, y que la vista admin declara el menú de ocho secciones, el enlace `#/`, `aria-current` en la sección activa, `data-region="admin-content"` sin UI de negocio y el `vx-field` de terminal. Los del árbol MUST asertar `lang="es"`, el `src` `/static/{{VERSION}}/js/app.js`, los `href` versionados de `tokens.css` y `base.css`, el *link* `/manifest.webmanifest`, `theme-color` `#0A0A0A`, la presencia de `manifest.webmanifest` e íconos PNG, los módulos de componentes (incluidos modal, panel, toast, product-card y omnibox), `js/api.js`, `js/errors.js`, `js/keyboard-list.js`, `js/health.js`, `sw.js` y `dev/components.html`. Los de componentes MUST asertar Shadow DOM, `adoptedStyleSheets`, ARIA, unhappy path de loading, error de campo y chip deshabilitado, los de overlay/toast de la tabla, y los de `vx-product-card` de la tabla. Los de `api` MUST asertar JSON 2xx, cabecera de terminal, `signal`, `ApiError` `not_found` / `invalid_input` / `network` / `internal`, abort distinto de `network`, y omisión de cabecera sin nombre. Los de `errors` MUST asertar cobertura del enum `ErrorCode` + `network`, mensajes en español, `showError` → toast `error`, asignación a `vx-field` por `name`, fallback de `code` desconocido, abort sin toast, campo sin match sin throw, y que el toast no muestra el `code` técnico. Los de `omnibox` MUST asertar foco al conectar, foco tras `vx-closed` / cierre de panel, Escape que limpia, modos (`?hola` → `ai` + `vx-query`, selector Texto/Voz/IA, imagen con/sin `vision`), `ai-unavailable` / `voice-unavailable`, y los unhappy de consulta vacía, host desconectado, Escape con panel abierto, `?` no inicial, `mode` desconocido y ausencia de fetch. Los de `keyboard-list` MUST asertar recorrido con flechas y `aria-activedescendant`, Enter que abre, Escape que restaura el Omnibox sin borrar la consulta, reuso con opciones que no son tarjetas, y los unhappy de lista vacía, Enter sin selección, sin Omnibox y ausencia de fetch. Los de administración / terminal MUST asertar persistencia de `vectra.terminal` (≤ 64), carga inicial, rechazo de > 64 con `error` en el campo, borrado de clave vacía, y que un storage que lanza no rompe la vista. Los de `health` MUST asertar mapeo `llm: down` → IA off y texto on (Listo cuando T-01-12), intervalo 10 s, refetch al foco, indicador degradado, `speech` no `up`, `vision` ausente, banner PA-04 en fallo de red (Listo cuando T-01-14), oculto si el nodo responde, y los unhappy de red, 503 `starting`, cuerpo inválido, ausencia del aviso de búsqueda reducida y degradado sin banner PA-04. Los de `manifest` MUST asertar JSON válido, Vectra / `standalone` / `#0A0A0A`, íconos 192/512 *any* y *maskable* con IHDR coincidente, y los unhappy de JSON inválido, sin `standalone`, sin *maskable*, `src` huérfano y dimensiones incorrectas. Los de `sw` MUST asertar precache versionado del *app shell*, navegación red-primero con respaldo en caché, `/health` y `/api/*` solo red, registro en `app.js`, y los unhappy de precache con catálogo, `serviceWorker` ausente y falta de `sw.js`.

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
- **AND** los de `sw.js` (precache versionado, navegación offline, `/api` y `/health` solo red, registro) pasan

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
