# Spec Delta

Ticket padre: T-02-37. Change: T-02-37. HU: HU-02-05.

## ADDED Requirements

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

## MODIFIED Requirements

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