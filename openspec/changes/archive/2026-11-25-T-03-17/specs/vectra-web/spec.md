# Spec Delta

Ticket padre: T-03-17. Change: `T-03-17`. HU: HU-03-05 (Tarjeta de resultado, Sin stock).

## MODIFIED Requirements

### Requirement: Búsqueda mientras se escribe en el Omnibox

Cuando el *router* pinta `data-view="operator"`, MUST montar la búsqueda sobre `data-region="omnibox"` y `data-region="results"`. `vx-omnibox` MUST seguir emitiendo `vx-query` y MUST NOT llamar a `/search` ni importar el cliente HTTP (requirement «Modos del Omnibox»).

En cada `vx-query` con `detail.mode === "text"` y texto recortado no vacío, MUST:

1. Cancelar la petición anterior con `AbortController` (el `signal` MUST reenviarse al cliente HTTP, T-01-06 / [5.7](../../../../../docs/5-contract-api.md#57-búsqueda-search-as-you-type-y-cancelación)).
2. Invocar `GET /api/v1/search` en el mismo origen con query `q` igual al texto (sin prefijo de API extra; MUST usar `encodeURIComponent` o equivalente). MUST NOT enviar `facets`, `categoryId`, `brandId`, `inStock` ni `attr`. `sort`/`offset`/`limit` MAY omitirse (defaults del contrato: relevancia, 0, 20). Si el texto supera 200 caracteres, MUST recortarlo a 200 antes de enviarlo (`maxLength` de `q` en [docs/openapi.yaml](../../../../../docs/openapi.yaml)).
3. MUST NOT aplicar *debounce* ni umbral de teclas: una petición por evento de consulta.

MUST pintar en `data-region="results"` **solo** la respuesta cuya petición no fue abortada. Hits de una petición abortada MUST NOT reemplazar la lista. Si llega un `AbortError`, MUST NOT pintar y MUST NOT invocar `showError`.

Cada hit MUST renderizarse como un `vx-product-card` con variante `full` (atributo `variant="full"` o omitido; default del componente). MUST mapear el `SearchHit` de [docs/openapi.yaml](../../../../../docs/openapi.yaml) a atributos de la tarjeta:

| SearchHit | Atributo `vx-product-card` |
|---|---|
| `sku` | `sku` |
| `name` | `name` (texto plano de respaldo) |
| `brand` | `brand` (omitir o vacío si `null`) |
| `price` | `price` |
| `stock` | `stock` |
| `location` | `location` (omitir o vacío si `null`) |
| `imageUrl` | `image-url` (omitir si `null` o vacío) |

El **nombre** MUST preferir `highlight.name` sanitizado vía `appendSanitizedHtml` en un hijo `slot="name"` cuando ese campo existe y no está vacío; si `highlight` es `null` o `highlight.name` falta, MUST usar `name` como texto plano (atributo `name` y/o texto del slot). SKU, marca, precio, stock y ubicación MUST ir como atributos (la tarjeta los pinta con `textContent`). MUST NOT asignar el payload crudo a `innerHTML`. MUST NOT pintar `highlight.description` (la tarjeta no tiene ese campo).

Si `imageUrl` está ausente o vacío, la tarjeta MUST mostrar el *placeholder* de imagen del componente (HU-03-05 MVP parcial: sin foto principal). Si `imageUrl` tiene valor, MUST pasarlo a `image-url`.

Si `stock` representa cero, la tarjeta MUST exponer el estado visual «sin stock» del componente (`data-out-of-stock` y texto «Sin stock»). MUST NOT indicar falta de stock solo con color.

La lista MUST ser un `role="listbox"` operable por teclado con el helper de T-01-10; cada tarjeta MUST ser una opción de esa lista. MUST NOT abrir ficha ni panel de recomendaciones al activar un hit (T-04-08). MUST NOT usar la variante `compact` en esta lista.

Consulta vacía (incluido Escape que deja el Omnibox vacío, T-01-08) o solo espacios: MUST abortar la petición en curso, MUST vaciar `data-region="results"` de hits y de `vx-product-card` y MUST NOT llamar a `/search`.

`vx-query` con `mode` distinto de `"text"` (`ai`, `voice`, `image`): MUST abortar la petición en curso si existe y MUST NOT llamar a `/search`. MUST NOT ejecutar `/ai`, `/speech` ni cámara.

`hits: []` MUST dejar la lista sin opciones ni `vx-product-card` (MAY un mensaje en español «Sin resultados»). MUST NOT dejar hits de una consulta anterior.

Un hit que no es un objeto, o al que le faltan campos, MUST NOT lanzar: MUST omitir ese hit o pintarlo con los campos presentes vacíos.

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
- **AND** `data-region="results"` queda sin hits ni `vx-product-card`

#### Scenario: error de API muestra toast y no pinta (unhappy)

- **WHEN** `GET /api/v1/search` responde 400, 503 o 504 con `problem+json`, o el cliente lanza error `network`
- **THEN** se invoca `showError` con ese error
- **AND** no se pintan hits de esa respuesta

#### Scenario: hits vacíos dejan la lista sin opciones (unhappy)

- **WHEN** la respuesta 200 trae `hits: []` para la consulta vigente
- **THEN** `data-region="results"` no muestra opciones de producto ni `vx-product-card`
- **AND** no se invoca `showError`

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

### Requirement: Tests de búsqueda mientras se escribe

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm).

| Test | Dónde | Tipo |
|---|---|---|
| `search_text_query_calls_get_search_with_q` | `search.test.mjs` | happy |
| `search_cancels_previous_and_paints_last_only` | `search.test.mjs` | happy |
| `search_renders_full_product_card` | `search.test.mjs` | happy, Listo cuando HU-03-05 |
| `search_renders_name_and_sku_as_text` | `search.test.mjs` | happy (nombre+SKU visibles en la tarjeta) |
| `search_zero_stock_marks_out_of_stock` | `search.test.mjs` | happy |
| `search_missing_image_url_shows_placeholder` | `search.test.mjs` | happy |
| `search_image_url_sets_card_src` | `search.test.mjs` | happy |
| `search_arrow_down_from_omnibox_selects_first_hit` | `search.test.mjs` | happy |
| `search_aborted_response_does_not_paint` | `search.test.mjs` | unhappy |
| `search_abort_error_does_not_call_showError` | `search.test.mjs` | unhappy |
| `search_non_text_mode_does_not_call_search` | `search.test.mjs` | unhappy |
| `search_empty_query_clears_without_fetch` | `search.test.mjs` | unhappy |
| `search_problem_json_calls_showError` | `search.test.mjs` | unhappy |
| `search_empty_hits_shows_empty_list` | `search.test.mjs` | unhappy |
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
