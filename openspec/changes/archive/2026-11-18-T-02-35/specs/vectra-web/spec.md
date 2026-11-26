# Spec Delta

Ticket padre: T-02-35. Change: T-02-35. HU: HU-02-04 (alta de producto). El escenario «aparece en la búsqueda» queda en T-03-07 / T-03-15. Atributos y códigos quedan en T-02-36. Edición con `version` queda en T-02-37.

## ADDED Requirements

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

Opciones de categoría y marca MUST ser **solo activas** (`active !== false`), hidratadas desde los lookups ya pedidos por el listado o desde `GET /api/v1/categories` y `GET /api/v1/brands` **sin** `includeInactive`. MUST NOT ofrecer una categoría o marca dada de baja (el backend responde `invalid_input`). Unidad MUST listar el enum `UnitOfMeasure` (`unit`, `pair`, `box`, `meter`, `kilogram`, `liter`) con etiquetas en español (Unidad, Par, Caja, Metro, Kilogramo, Litro); valor inicial `unit`.

Al guardar, si `sku`, `name`, `productType`, `categoryId` o `price` quedan vacíos tras recortar espacios, MUST NOT enviar `POST`. Si están completos, MUST enviar `POST /api/v1/products` con cuerpo `ProductCreate`:

- `sku`, `name`, `productType`, `categoryId`, `price` (decimal en **string**, p. ej. `"120.50"`)
- `description` omitido o `null` si está vacío
- `brandId` omitido si no hay marca
- `unitOfMeasure` (default `unit`)
- `location` omitido si pasillo, estante y posición están vacíos; si hay alguno, objeto `{ aisle, shelf, position }` con nulos/omitidos en los vacíos
- `initialStock` omitido o `"0"` si está vacío; si hay valor, string decimal no negativo
- MUST NOT incluir `attributes` ni `codes` (o arrays vacíos). MUST NOT pintar controles de atributos ni de códigos (**No hace** / T-02-36)
- MUST NOT enviar `stock` ni `version` ni `active`

`api.request` ya adjunta `X-Vectra-Terminal` si hay nombre de terminal (T-01-06 / T-01-11); esta vista MUST NOT duplicar esa cabecera.

Tras `201` con un `Product`, MUST volver al listado y MUST recargar la **primera** página con los filtros que había al abrir la ficha. El producto creado (activo, `version` 1) MUST aparecer cuando esos filtros son los de activos (Listo cuando). MUST NOT abrir ficha de edición (T-02-37). MUST NOT llamar a `/api/v1/search` ni a `/health`.

Los errores MUST mostrarse con `showError` / `messageFor(code)`: el toast usa el `code` (`invalid_input`, `duplicate_sku`, `unavailable`, `network`, …), no el `title` crudo. Si `err.errors` trae `{ field, message }`, MUST asignarse al `vx-field` cuyo `name` es exactamente `field` (requirement T-01-07). La ficha MUST permanecer abierta si el POST falla. `base.css` MAY añadir clases de ficha/pestañas solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

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
- **THEN** se envía `POST /api/v1/products` cuyo cuerpo incluye esos campos (`price` e `initialStock` como string) y no incluye `attributes` ni `codes` (o los envía `[]`)
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

- **WHEN** se inspecciona la ficha de alta
- **THEN** no hay controles de atributos (`attributes[n]`) ni de códigos (`codes`)
- **AND** el cuerpo del POST, si se envía, no incluye `attributes` ni `codes` distintos de `[]`

### Requirement: Tests de la UI alta de producto

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (crear un producto desde la UI y verlo en el listado) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `products_empty_list_offers_create` | `scripts/fixtures/vectra-web/products.test.mjs` | happy |
| `sheet_new_product_opens_datos_tab` | `scripts/fixtures/vectra-web/product-sheet.test.mjs` | happy |
| `sheet_create_perno_posts_product_create` | `product-sheet.test.mjs` | happy (HU-02-04, Listo cuando) |
| `sheet_create_returns_to_list_with_new_sku` | `product-sheet.test.mjs` | happy (Listo cuando) |
| `sheet_cancel_does_not_post` | `product-sheet.test.mjs` | happy |
| `sheet_stock_and_relations_tabs_are_disabled` | `product-sheet.test.mjs` | happy / frontera |
| `sheet_duplicate_sku_shows_duplicate_sku` | `product-sheet.test.mjs` | unhappy (HU-02-04) |
| `sheet_invalid_input_sets_sku_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_invalid_input_sets_category_id_field_error` | `product-sheet.test.mjs` | unhappy |
| `sheet_network_shows_network` | `product-sheet.test.mjs` | unhappy |
| `sheet_empty_required_does_not_post` | `product-sheet.test.mjs` | unhappy |
| `sheet_no_attributes_or_codes` | `product-sheet.test.mjs` | unhappy (No hace) |
| `sheet_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |

`check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/product-sheet.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de alta cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `product-sheet.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de ficha (unhappy)

- **WHEN** falta `vectra-web/js/admin/product-sheet.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `product-sheet.js`

## MODIFIED Requirements

### Requirement: UI listado de productos en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="productos"` (`#/admin/productos`), `data-region="admin-content"` MUST montar el listado administrativo de productos. La vista MUST cargar `GET /api/v1/products` con `active=true` en la primera página (solo activos; [docs/openapi.yaml](../../../../../docs/openapi.yaml) `listProducts`). MUST pintar una **tabla** cuyas filas son `items` de `ProductSummaryPage`, en el orden de la API (id ULID ascendente). Cada fila MUST mostrar `sku`, `name`, `productType`, `brand` (o vacío si es `null`), `categoryPath` unido (p. ej. `Fijaciones > Pernos`), `price`, `stock` y el estado (`active`). MUST NOT pintar `vx-product-card`. MUST NOT hidratar `imageUrl` ni mostrar `<img>` de producto (`imageUrl` es `null` hasta T-02-39).

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`):

- **Nuevo producto**: abre la ficha de alta del requirement «UI alta de producto (ficha Datos)». MUST estar disponible también cuando `items` está vacío.
- **Filtro categoría**: opciones hidratadas desde `GET /api/v1/categories?includeInactive=true`. Al elegir una categoría MUST enviar `categoryId` de esa categoría; MUST NOT expandir descendientes en el cliente (la API incluye la rama; HU-02-10 «Filtro por rama»).
- **Filtro marca**: opciones hidratadas desde `GET /api/v1/brands?includeInactive=true`. Al elegir una marca MUST enviar `brandId`.
- **Filtro estado**: `Activos` (`active=true`, defecto) e `Inactivos` (`active=false`). MUST NOT haber un valor «todos» (el contrato no lo ofrece).
- **Cargar más**: si `nextCursor` no es `null`, MUST mostrar la acción; al activarla MUST hacer `GET` con `cursor` igual a ese valor **y los mismos filtros**, y MUST **añadir** las nuevas filas a la tabla. Si `nextCursor` es `null`, MUST NOT mostrar «Cargar más».
- **Dar de baja**: `DELETE /api/v1/products/{productId}` sobre un producto activo (HU-02-06).
- **Reactivar**: sobre un inactivo, MUST `GET /api/v1/products/{productId}` y luego `PATCH` con `{ "version": <versión de la ficha>, "active": true }` (HU-02-06; `ProductSummary` no trae `version`).

Un producto con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactivo») y MUST ofrecer reactivar, no dar de baja. Un activo MUST ofrecer dar de baja, no reactivar.

Al cambiar un filtro MUST pedir de nuevo la **primera** página (sin `cursor`) con los filtros actuales y MUST reemplazar las filas (no append). Tras un `DELETE`/`PATCH` exitoso MUST recargar la primera página con los mismos filtros.

MUST NOT ofrecer edición en línea en la **tabla**: MUST NOT haber `contenteditable`, MUST NOT haber `<input>` / `vx-field` en celdas de `sku`, `name`, `price` o `stock` (**No hace** de T-02-34). Mientras el listado está visible (ficha cerrada) MUST NOT haber `<form>` de alta ni enviar `POST /api/v1/products`. MUST NOT abrir ficha de **edición** (T-02-37). MUST NOT llamar a `/api/v1/search` ni a `/health` desde esta vista. MUST NOT enviar `q`. Si el `GET` de listado devuelve `items` vacío, MUST mostrar el listado vacío **y** MUST ofrecer «Nuevo producto».

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
- **AND** no se pinta un diff de campos (eso es T-02-37)

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
