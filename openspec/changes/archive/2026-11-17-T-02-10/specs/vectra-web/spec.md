# Spec Delta

Ticket padre: T-02-10. Change: T-02-10. HU: HU-02-02 (el escenario de referencias en `producto` queda en T-02-14).

## ADDED Requirements

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

3. Una región `data-region="admin-content"` reservada al contenido de la sección activa. Si `data-section="categorias"`, MUST montar el ABM del requirement «UI de categorías en administración». Si `data-section="marcas"`, MUST montar el ABM del requirement «UI de marcas en administración». En cualquier otra sección del menú (y en un slug no listado) MUST seguir vacía de UI de negocio: MUST NOT contener `vx-product-card`, MUST NOT montar formularios de alta/edición de catálogo, MUST NOT llamar a `/api/v1/*` ni a `/health`. MAY mostrar un título o texto placeholder de la sección (español) sin datos.
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

- **WHEN** el hash es `#/admin/productos` (o `#/admin/stock`, u otra sección del menú distinta de `categorias` y `marcas`)
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
