# Spec Delta

Ticket padre: T-02-36. Change: T-02-36. HU: HU-02-04 (alta de producto). Edición con `version` queda en T-02-37. Código de barras (`barcode`) queda post-MVP.

## ADDED Requirements

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

## MODIFIED Requirements

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
