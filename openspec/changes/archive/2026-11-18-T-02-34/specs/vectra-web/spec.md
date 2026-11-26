# Spec Delta

Ticket padre: T-02-34. Change: T-02-34. HU: HU-02-10 (listado admin) y HU-02-06 (baja/reactivación en la tabla). El escenario HU-02-06 «deja de aparecer en la búsqueda» queda en T-03-07; el de recomendaciones en T-04-06. Alta y ficha quedan en T-02-35.

## ADDED Requirements

### Requirement: UI listado de productos en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="productos"` (`#/admin/productos`), `data-region="admin-content"` MUST montar el listado administrativo de productos. La vista MUST cargar `GET /api/v1/products` con `active=true` en la primera página (solo activos; [docs/openapi.yaml](../../../../../docs/openapi.yaml) `listProducts`). MUST pintar una **tabla** cuyas filas son `items` de `ProductSummaryPage`, en el orden de la API (id ULID ascendente). Cada fila MUST mostrar `sku`, `name`, `productType`, `brand` (o vacío si es `null`), `categoryPath` unido (p. ej. `Fijaciones > Pernos`), `price`, `stock` y el estado (`active`). MUST NOT pintar `vx-product-card`. MUST NOT hidratar `imageUrl` ni mostrar `<img>` de producto (`imageUrl` es `null` hasta T-02-39).

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`):

- **Filtro categoría**: opciones hidratadas desde `GET /api/v1/categories?includeInactive=true`. Al elegir una categoría MUST enviar `categoryId` de esa categoría; MUST NOT expandir descendientes en el cliente (la API incluye la rama; HU-02-10 «Filtro por rama»).
- **Filtro marca**: opciones hidratadas desde `GET /api/v1/brands?includeInactive=true`. Al elegir una marca MUST enviar `brandId`.
- **Filtro estado**: `Activos` (`active=true`, defecto) e `Inactivos` (`active=false`). MUST NOT haber un valor «todos» (el contrato no lo ofrece).
- **Cargar más**: si `nextCursor` no es `null`, MUST mostrar la acción; al activarla MUST hacer `GET` con `cursor` igual a ese valor **y los mismos filtros**, y MUST **añadir** las nuevas filas a la tabla. Si `nextCursor` es `null`, MUST NOT mostrar «Cargar más».
- **Dar de baja**: `DELETE /api/v1/products/{productId}` sobre un producto activo (HU-02-06).
- **Reactivar**: sobre un inactivo, MUST `GET /api/v1/products/{productId}` y luego `PATCH` con `{ "version": <versión de la ficha>, "active": true }` (HU-02-06; `ProductSummary` no trae `version`).

Un producto con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactivo») y MUST ofrecer reactivar, no dar de baja. Un activo MUST ofrecer dar de baja, no reactivar.

Al cambiar un filtro MUST pedir de nuevo la **primera** página (sin `cursor`) con los filtros actuales y MUST reemplazar las filas (no append). Tras un `DELETE`/`PATCH` exitoso MUST recargar la primera página con los mismos filtros.

MUST NOT ofrecer edición en línea: MUST NOT haber `contenteditable`, MUST NOT haber `<input>` / `vx-field` en celdas de `sku`, `name`, `price` o `stock` (**No hace**). MUST NOT enviar `POST /api/v1/products` (alta = T-02-35). MUST NOT navegar a una ficha de edición (T-02-35 / T-02-37). MUST NOT llamar a `/api/v1/search` ni a `/health` desde esta vista. MUST NOT enviar `q`. Si el `GET` de listado devuelve `items` vacío, MUST mostrar el listado vacío **sin** acción de crear producto.

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

### Requirement: Tests de la UI listado de productos

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (escenarios de HU-02-10 a mano: filtro por rama «Fijaciones» y filtro por inactivos con reactivar) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `products_list_requests_active_true` | `scripts/fixtures/vectra-web/products.test.mjs` | happy |
| `products_lookups_request_categories_and_brands` | `products.test.mjs` | happy |
| `products_empty_list_does_not_post` | `products.test.mjs` | happy |
| `products_filter_category_sends_category_id` | `products.test.mjs` | happy (HU-02-10) |
| `products_filter_inactive_shows_reactivate` | `products.test.mjs` | happy (HU-02-10) |
| `products_load_more_appends_with_cursor` | `products.test.mjs` | happy |
| `products_deactivate_deletes` | `products.test.mjs` | happy (HU-02-06) |
| `products_reactivate_gets_then_patches_active_true` | `products.test.mjs` | happy (HU-02-06) |
| `products_network_shows_network` | `products.test.mjs` | unhappy |
| `products_not_found_shows_not_found` | `products.test.mjs` | unhappy |
| `products_invalid_input_shows_invalid_input` | `products.test.mjs` | unhappy |
| `products_version_conflict_shows_version_conflict` | `products.test.mjs` | unhappy |
| `products_no_inline_edit` | `products.test.mjs` | unhappy (No hace) |
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

## MODIFIED Requirements

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
