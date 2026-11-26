# Spec Delta

Ticket padre: T-04-08. Change: `T-04-08`. HU: HU-04-05 (Tres bloques, Productos inactivos; no Latencia), HU-04-06 (Sustitutos destacados sin stock). Endpoint `GET /api/v1/products/{productId}/recommendations` = T-04-07 (fuera de este change).

## ADDED Requirements

### Requirement: Panel de recomendaciones

MUST existir `vectra-web/js/recommendations.js` como ES Module. En la vista operador, MUST usar el `vx-side-panel` de `data-region="side-panel"` (T-01-04 / T-01-08). `vx-side-panel` MUST NOT ganar lógica de recomendaciones: el módulo (o `search.js` que lo invoca) MUST llenar el contenido y llamar `show` / `hide`.

Al **abrir un resultado** — Enter sobre el hit activo del listbox (T-01-10) o clic en su `vx-product-card` — MUST:

1. Leer `data-product-id` de esa tarjeta (el `id` ULID de `SearchHit` o `ProductSummary`). Si falta o está vacío, MUST NOT llamar al endpoint y MUST NOT abrir el panel.
2. Cancelar una petición de recomendaciones anterior con `AbortController` (`signal` reenviado al cliente HTTP, T-01-06).
3. Invocar `GET /api/v1/products/{productId}/recommendations` en el mismo origen (`encodeURIComponent` en el segmento). MUST NOT enviar query extra ni body. MUST NOT llamar a `/relations`.
4. Si la ronda no fue abortada y la respuesta es 200 (`Recommendations` de [docs/openapi.yaml](../../../../../docs/openapi.yaml)): abrir el panel (`open` / `show`) y pintar **tres bloques** en este orden, con títulos en español:
   - **Similares** ← `similar`
   - **Sustitutos** ← `substitutes`
   - **Se usa junto con** ← `complements`
5. Cada ítem MUST pintarse como `vx-product-card` `variant="compact"` en el **orden del array** (prioridad ya resuelta por T-04-06). MUST NOT reordenar. MUST NOT filtrar de nuevo por `activo` ni `stock` (el backend ya excluye inactivos y sustitutos con `stock = 0`).

Mapeo `RecommendedProduct` → tarjeta:

| RecommendedProduct | Atributo |
|---|---|
| `id` | `data-product-id` |
| `sku` | `sku` |
| `name` | `name` |
| `stock` | `stock` |
| `price` | `price` |
| `location` | `location` (omitir si `null`) |
| `imageUrl` | `image-url` (omitir si `null` o vacío) |

MUST NOT pintar `note` ni `relationId` como texto obligatorio. MUST NOT usar `variant="full"` en el panel. MUST NOT asignar JSON crudo a `innerHTML`.

Los tres títulos MUST mostrarse aunque un array llegue vacío (sin tarjetas en ese bloque). Un destino inactivo no aparece si la API no lo envía (HU-04-05 «Productos inactivos»).

Si `highlightSubstitutes === true`, el bloque **Sustitutos** MUST quedar destacado de forma no solo-color (p. ej. `data-highlight-substitutes` más texto «Destacados» o borde + etiqueta; tokens `--vx-*` existentes). Si es `false`, MUST NOT destacar ese bloque.

Las tarjetas compactas del panel MUST ser un `role="listbox"` operable por teclado (T-01-10); los títulos de bloque MUST NOT ser opciones. Enter o clic en una tarjeta del panel MUST abrir **ese** producto (misma petición, mismo panel; aborta la ronda anterior). MUST NOT agregar al carrito (T-08-06).

Al cerrar el panel (Escape o control de cierre de T-01-04), MUST emitirse `vx-closed`; el foco MUST volver al Omnibox (T-01-08). MUST NOT reimplementar la trampa de foco.

`AbortError` / ronda abortada: MUST NOT pintar esa respuesta y MUST NOT invocar `showError`. Errores 400/404/503/`network` que no sean abort: MUST invocar `showError` y MUST NOT pintar esa respuesta. Si el panel aún no estaba `open`, MUST NOT abrirlo. Vistas admin y not-found MUST NOT llamar a `/recommendations`.

Textos visibles MUST estar en español. `base.css` MAY estilar bloques y destaque solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo. MUST NOT exigir Playwright.

#### Scenario: tres bloques al abrir un resultado (happy, Listo cuando HU-04-05)

- **WHEN** hay un hit pintado con `data-product-id` de "Perno M8"
- **AND** el operador lo abre (Enter en el hit activo, o clic)
- **AND** `GET /api/v1/products/{productId}/recommendations` responde 200 con ítems en `similar`, `substitutes` y `complements`
- **THEN** el `vx-side-panel` de `data-region="side-panel"` queda `open`
- **AND** se ven los títulos «Similares», «Sustitutos» y «Se usa junto con»
- **AND** cada bloque muestra `vx-product-card` `compact` en el orden del array correspondiente
- **AND** se envió exactamente esa GET (sin `/relations`)

#### Scenario: el orden del panel sigue la prioridad de la API (happy)

- **WHEN** `similar` llega `[{ name: "B", priority: 2 }, { name: "A", priority: 1 }]` (el servidor ya ordenó)
- **THEN** las tarjetas de Similares aparecen en ese mismo orden
- **AND** la UI no reordena por `priority` numérico

#### Scenario: producto inactivo no aparece (happy, HU-04-05)

- **WHEN** la respuesta 200 no incluye un destino dado de baja (el backend lo filtró)
- **THEN** el panel no muestra esa SKU ni ese `id`

#### Scenario: sustitutos destacados sin stock del origen (happy, Listo cuando HU-04-06)

- **WHEN** la respuesta trae `highlightSubstitutes: true` y `substitutes` con un ítem de stock `"12"` (y ninguno de stock `"0"`)
- **THEN** el bloque «Sustitutos» está destacado
- **AND** muestra el sustituto de stock 12
- **AND** no muestra un sustituto de stock 0 (no viene en el payload)

#### Scenario: producto con stock no destaca sustitutos (happy, HU-04-06)

- **WHEN** la respuesta trae `highlightSubstitutes: false` y `substitutes` no vacío
- **THEN** el bloque «Sustitutos» se muestra
- **AND** no está destacado

#### Scenario: elegir un recomendado abre ese producto (happy)

- **WHEN** el panel está abierto con una tarjeta compacta `data-product-id` de otro producto
- **AND** el operador la elige (Enter o clic)
- **THEN** se envía `GET /api/v1/products/{eseId}/recommendations`
- **AND** el panel permanece o se reabre `open` con los tres bloques de esa respuesta

#### Scenario: al cerrar el foco vuelve al Omnibox (happy)

- **WHEN** el panel de recomendaciones está `open` y hay un `vx-omnibox` en la vista
- **AND** se cierra (Escape o control de cierre)
- **THEN** se dispara `vx-closed`
- **AND** el foco queda en el input del Omnibox

#### Scenario: arrays vacíos igual muestran los tres títulos (happy)

- **WHEN** la respuesta 200 trae `similar: []`, `substitutes: []`, `complements: []`
- **THEN** el panel se abre
- **AND** se ven los tres títulos
- **AND** no hay `vx-product-card` compactas en el panel

#### Scenario: hit sin id no llama al endpoint (unhappy)

- **WHEN** una tarjeta de resultados no tiene `data-product-id` (o está vacío)
- **AND** el operador la abre
- **THEN** no se envía `GET /api/v1/products/.../recommendations`
- **AND** el panel no queda `open`

#### Scenario: 404 o 503 muestra toast y no pinta (unhappy)

- **WHEN** `GET .../recommendations` responde 404 `not_found` o 503 `unavailable` (`problem+json`), o el cliente lanza `network`
- **THEN** se invoca `showError` con ese error
- **AND** no se pintan bloques de esa respuesta
- **AND** si el panel no estaba abierto, no queda `open`

#### Scenario: AbortError no llama a showError ni pinta (unhappy)

- **WHEN** el operador abre un producto y luego otro antes de que responda el primero
- **THEN** la petición anterior queda abortada
- **AND** no se invoca `showError` por el abort
- **AND** el panel muestra solo la respuesta del segundo producto

#### Scenario: admin no llama a recommendations (unhappy)

- **WHEN** el hash es `#/admin/productos`
- **THEN** no se envía `GET /api/v1/products/{id}/recommendations`

#### Scenario: no hay botón de carrito (unhappy / No hace)

- **WHEN** el panel está abierto con recomendaciones
- **THEN** no hay control «Agregar» ni se persiste un carrito

### Requirement: Tests del panel de recomendaciones

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm).

| Test | Dónde | Tipo |
|---|---|---|
| `search_enter_opens_recommendations_panel` | `search.test.mjs` o `recommendations.test.mjs` | happy, Listo cuando HU-04-05 |
| `search_click_opens_recommendations_panel` | `search.test.mjs` o `recommendations.test.mjs` | happy |
| `recs_three_blocks_in_api_order` | `recommendations.test.mjs` | happy |
| `recs_highlights_substitutes_when_flag_true` | `recommendations.test.mjs` | happy, Listo cuando HU-04-06 |
| `recs_does_not_highlight_substitutes_when_flag_false` | `recommendations.test.mjs` | happy |
| `recs_compact_cards_not_full` | `recommendations.test.mjs` | happy |
| `recs_empty_arrays_still_show_headings` | `recommendations.test.mjs` | happy |
| `recs_opening_recommended_product_fetches_that_id` | `recommendations.test.mjs` | happy |
| `recs_close_returns_focus_to_omnibox` | `recommendations.test.mjs` | happy |
| `recs_inactive_sku_absent_from_payload_is_absent` | `recommendations.test.mjs` | happy |
| `recs_missing_product_id_does_not_fetch` | `recommendations.test.mjs` | unhappy |
| `recs_404_calls_showError_and_does_not_paint` | `recommendations.test.mjs` | unhappy |
| `recs_503_calls_showError` | `recommendations.test.mjs` | unhappy |
| `recs_network_calls_showError` | `recommendations.test.mjs` | unhappy |
| `recs_abort_does_not_paint_stale` | `recommendations.test.mjs` | unhappy |
| `recs_abort_does_not_call_showError` | `recommendations.test.mjs` | unhappy |
| `recs_admin_view_does_not_fetch` | `recommendations.test.mjs` | unhappy |
| `recs_no_add_to_cart_control` | `recommendations.test.mjs` | unhappy |
| `recs_missing_module_fails` | `check-vectra-web.sh` | unhappy |

`check-vectra-web.sh` MUST exigir `vectra-web/js/recommendations.js` y ejecutar `recommendations.test.mjs`. MUST NOT exigir Playwright ni `cargo test` de este ticket más allá de la regresión del workspace.

#### Scenario: la suite de recomendaciones pasa en check-vectra-web (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** termina con código 0
- **AND** corre `recommendations.test.mjs`

#### Scenario: falta recommendations.js (unhappy)

- **WHEN** falta `vectra-web/js/recommendations.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `recommendations.js`

## MODIFIED Requirements

### Requirement: Layout de la vista operador

Cuando el *router* pinta `data-view="operator"` (`#/`, `#` o hash vacío), el *outlet* MUST incluir, en este orden visual de arriba hacia abajo:

1. Una región `data-region="omnibox"` que contiene exactamente un `vx-omnibox`.
2. Una región `data-region="results"` para la lista de búsqueda (requirement «Búsqueda mientras se escribe en el Omnibox»). Al pintar la vista, MUST estar vacía de hits y de `vx-product-card` (T-03-17). MUST NOT llamar a `/search` hasta un `vx-query` de modo texto con consulta no vacía.
3. Una región `data-region="side-panel"` que contiene un `vx-side-panel` **sin** atributo `open` al pintar la vista. El requirement «Panel de recomendaciones» lo abre al activar un resultado.

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

Cada tarjeta de resultado (hits y coincidencias exactas) MUST exponer `data-product-id` con el `id` ULID del ítem cuando existe. MUST NOT inventar un id.

La lista MUST ser un `role="listbox"` operable por teclado con el helper de T-01-10; cada tarjeta MUST ser una opción de esa lista (la etiqueta «Código exacto» MUST NOT ser una opción). El primer `ArrowDown` desde el Omnibox MUST activar la primera coincidencia exacta si existe, si no el primer hit de `/search`. Enter sobre el hit activo, o un clic en la tarjeta, MUST abrir el panel de recomendaciones (requirement «Panel de recomendaciones»). MUST NOT abrir la ficha de administración. MUST NOT usar la variante `compact` en esta lista. MUST NOT invocar un lector de código de barras.

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

#### Scenario: Enter abre el panel de recomendaciones (happy, T-04-08)

- **WHEN** hay al menos un hit pintado con `id` ULID
- **AND** ese hit está activo
- **AND** se pulsa Enter
- **THEN** se envía `GET /api/v1/products/{id}/recommendations`
- **AND** el `vx-side-panel` queda `open` cuando la respuesta es 200

#### Scenario: clic en la tarjeta abre el panel (happy, T-04-08)

- **WHEN** hay un hit pintado con `id` ULID
- **AND** el operador hace clic en esa `vx-product-card`
- **THEN** se envía `GET /api/v1/products/{id}/recommendations`

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
| `search_enter_opens_recommendations_panel` | `search.test.mjs` o `recommendations.test.mjs` | happy, T-04-08 |
| `search_click_opens_recommendations_panel` | `search.test.mjs` o `recommendations.test.mjs` | happy, T-04-08 |
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
