# Spec Delta

Ticket padre: T-02-07. Change: T-02-07. HU: HU-02-01 (árbol de categorías; el escenario de búsqueda queda en T-03-08).

## ADDED Requirements

### Requirement: UI de categorías en administración

Cuando el *router* pinta `data-view="admin"` y `data-section="categorias"` (`#/admin/categorias`), `data-region="admin-content"` MUST montar el ABM de categorías. La vista MUST cargar `GET /api/v1/categories?includeInactive=true` (lista plana `{ items }` de [docs/openapi.yaml](../../../../../docs/openapi.yaml)) y MUST armar un árbol anidado con `parentId` (`null` = raíz). Cada nodo MUST mostrar `name` y MAY mostrar `path` unido (p. ej. `Fijaciones > Pernos`). Una categoría con `active = false` MUST distinguirse visualmente (p. ej. etiqueta «Inactiva») y MUST ofrecer reactivar, no dar de baja.

La vista MUST ofrecer, en español y operables por teclado (controles con tamaño táctil ≥ `--vx-touch-min`):

- **Nueva categoría** (raíz): `POST /api/v1/categories` con `{ "name" }` y sin `parentId` (o `parentId: null`).
- **Crear subcategoría** en un nodo: `POST` con `{ "name", "parentId" }` igual al `id` del nodo (HU-02-01 «Crear subcategoría»).
- **Renombrar**: `PATCH /api/v1/categories/{categoryId}` con `{ "name" }` (HU-02-01 «Renombrar una rama»: el `path` de la rama lo recalcula el servidor; la UI MUST volver a listar y mostrar el `path` actualizado).
- **Mover**: selector de padre (`<select>` o equivalente), no arrastrar y soltar. `PATCH` con `{ "parentId" }` (`null` = raíz). El selector MUST incluir una opción «Raíz» y MUST omitir la categoría que se mueve y sus descendientes.
- **Dar de baja**: `DELETE /api/v1/categories/{categoryId}` sobre una categoría activa.
- **Reactivar**: `PATCH` con `{ "active": true }` sobre una inactiva.

MUST NOT enviar `slug` en create/update (el servidor lo genera desde `name`). MUST NOT pintar `draggable`, ni escuchar `dragstart` / `drop` en el árbol (**No hace**). MUST NOT llamar otros paths de `/api/v1` ni `/health` desde esta vista. Tras un `POST`/`PATCH`/`DELETE` exitoso, MUST volver a `GET` e hidratar el árbol. Un `name` vacío (solo espacios) MUST NOT enviarse.

Los errores MUST mostrarse con `showError` / `messageFor(code)` (requirement «Mensajes en español por ErrorCode»): el toast usa el `code` de `problem+json` (`category_cycle`, `duplicate_slug`, `conflict`, `invalid_input`, `not_found`, `unavailable`, `network`, …), no el `title` crudo. `base.css` MAY añadir clases del árbol solo con *tokens* `--vx-*` existentes. MUST NOT agregar *tokens* nuevos ni un Web Component nuevo.

Si no hay categorías, MUST mostrar el árbol vacío y la acción de crear raíz. Textos visibles MUST estar en español.

#### Scenario: crear subcategoría actualiza el árbol y el path (happy, HU-02-01)

- **WHEN** existe la categoría activa "Fijaciones"
- **AND** el administrador crea "Pernos" con padre "Fijaciones"
- **THEN** se envía `POST /api/v1/categories` con `name` `"Pernos"` y `parentId` el id de "Fijaciones"
- **AND** el árbol muestra "Pernos" anidada bajo "Fijaciones"
- **AND** el `path` visible de "Pernos" es `Fijaciones > Pernos` (o equivalente con los elementos `["Fijaciones", "Pernos"]`)

#### Scenario: crear categoría raíz en árbol vacío (happy)

- **WHEN** `GET` devuelve `{ "items": [] }`
- **THEN** la región muestra la acción de crear categoría (raíz)
- **AND** al confirmar el nombre `"Fijaciones"` se envía `POST` con `name` `"Fijaciones"` y sin `parentId` de otra categoría

#### Scenario: renombrar actualiza el path de la rama (happy, HU-02-01)

- **WHEN** existen "Fijaciones" y su hija "Pernos" (`path` `["Fijaciones", "Pernos"]`)
- **AND** el administrador renombra "Fijaciones" a "Tornillería"
- **THEN** se envía `PATCH` de "Fijaciones" con `{ "name": "Tornillería" }`
- **AND** tras recargar, el árbol muestra "Tornillería" y el `path` de "Pernos" incluye `"Tornillería"`

#### Scenario: mover con selector de padre (happy)

- **WHEN** "Pernos" es hija de "Fijaciones" y existe "Herramientas"
- **AND** el administrador elige "Herramientas" como nuevo padre de "Pernos"
- **THEN** se envía `PATCH` de "Pernos" con `{ "parentId" }` igual al id de "Herramientas"
- **AND** no se dispara ningún evento `dragstart` ni `drop`

#### Scenario: dar de baja y reactivar (happy)

- **WHEN** "Pernos" es una hoja activa
- **AND** el administrador la da de baja
- **THEN** se envía `DELETE /api/v1/categories/{id}`
- **AND** tras recargar, "Pernos" se muestra inactiva y ofrece reactivar
- **AND** al reactivar se envía `PATCH` con `{ "active": true }`

#### Scenario: el listado pide inactivas (happy)

- **WHEN** se pinta `#/admin/categorias` con un cliente HTTP inyectado
- **THEN** la primera petición es `GET /api/v1/categories?includeInactive=true`

#### Scenario: evitar ciclos muestra category_cycle (unhappy, HU-02-01)

- **WHEN** "Pernos" es hija de "Fijaciones"
- **AND** el servidor responde `400` `application/problem+json` con `code` `category_cycle` al mover
- **THEN** se muestra el mensaje en español de `category_cycle` (no el string `category_cycle` como texto del toast)
- **AND** el árbol no se reemplaza por un estado vacío

#### Scenario: slug duplicado muestra duplicate_slug (unhappy, HU-02-01)

- **WHEN** el servidor responde `409` con `code` `duplicate_slug` al crear
- **THEN** se muestra el mensaje en español de `duplicate_slug`

#### Scenario: baja con hija activa muestra conflict (unhappy, H-10)

- **WHEN** el servidor responde `409` con `code` `conflict` al dar de baja
- **THEN** se muestra el mensaje en español de `conflict`

#### Scenario: name vacío no llama a POST (unhappy)

- **WHEN** el administrador confirma crear con el nombre vacío o solo espacios
- **THEN** no se envía `POST /api/v1/categories`

#### Scenario: fallo de red muestra network (unhappy)

- **WHEN** el cliente HTTP rechaza el `GET` inicial con `code` `network`
- **THEN** se muestra el mensaje en español de `network`
- **AND** no se lanza una excepción no capturada

#### Scenario: no hay arrastrar y soltar (unhappy, No hace)

- **WHEN** se inspecciona el árbol de categorías
- **THEN** ningún nodo tiene atributo `draggable="true"`
- **AND** no hay listeners de `dragstart` ni `drop` registrados por la vista

### Requirement: Tests de la UI de categorías

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. MUST ejecutarse con `node --test` desde `./scripts/check-vectra-web.sh` (sin jsdom, sin `package.json`, sin dependencias npm). El Listo cuando del ticket (escenarios de HU-02-01 a mano, salvo búsqueda) MUST poder ejecutarse contra `vortex serve` + esta UI; la suite automatizada MUST cubrir el mismo comportamiento observable.

| Test | Ubicación | Tipo |
|---|---|---|
| `categories_tree_builds_from_parent_id` | `scripts/fixtures/vectra-web/categories.test.mjs` | happy |
| `categories_create_subcategory_posts_name_and_parent` | `categories.test.mjs` | happy (HU-02-01) |
| `categories_create_root_posts_without_parent` | `categories.test.mjs` | happy |
| `categories_rename_patches_name` | `categories.test.mjs` | happy (HU-02-01) |
| `categories_move_patches_parent_id` | `categories.test.mjs` | happy |
| `categories_deactivate_deletes` | `categories.test.mjs` | happy |
| `categories_reactivate_patches_active_true` | `categories.test.mjs` | happy |
| `categories_list_requests_include_inactive` | `categories.test.mjs` | happy |
| `categories_empty_tree_offers_create_root` | `categories.test.mjs` | happy |
| `categories_cycle_shows_category_cycle` | `categories.test.mjs` | unhappy (HU-02-01) |
| `categories_duplicate_slug_shows_duplicate_slug` | `categories.test.mjs` | unhappy (HU-02-01) |
| `categories_deactivate_conflict_shows_conflict` | `categories.test.mjs` | unhappy (H-10) |
| `categories_network_shows_network` | `categories.test.mjs` | unhappy |
| `categories_empty_name_does_not_post` | `categories.test.mjs` | unhappy |
| `categories_no_drag_and_drop` | `categories.test.mjs` | unhappy (No hace) |
| `categories_parent_options_exclude_self_and_descendants` | `categories.test.mjs` | unhappy |
| `categories_tree_missing_module_fails` | `scripts/check-vectra-web.sh` | unhappy (harness) |
| `router_admin_content_region_has_no_business_ui` | `router.test.mjs` | unhappy (secciones que no son `categorias`) |
| `router_admin_does_not_call_api` | `router.test.mjs` | unhappy (`#/admin/productos` y `#/admin/stock`) |

`check-vectra-web.sh` MUST fallar si falta `vectra-web/js/admin/categories.js`. MUST NOT exigir Playwright. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` en `vortex-core/` MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) §5).

#### Scenario: la suite de categorías cubre happy y unhappy (happy)

- **WHEN** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** corre `categories.test.mjs` con los tests de la tabla
- **AND** el comando termina con código 0 sobre el árbol que cumple esta spec

#### Scenario: falta el módulo de categorías (unhappy)

- **WHEN** falta `vectra-web/js/admin/categories.js`
- **AND** se ejecuta `./scripts/check-vectra-web.sh`
- **THEN** el comando termina con código distinto de 0
- **AND** el mensaje nombra `categories.js`

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

3. Una región `data-region="admin-content"` reservada al contenido de la sección activa. Si `data-section="categorias"`, MUST montar el ABM del requirement «UI de categorías en administración». En cualquier otra sección del menú (y en un slug no listado) MUST seguir vacía de UI de negocio: MUST NOT contener `vx-product-card`, MUST NOT montar formularios de alta/edición de catálogo, MUST NOT llamar a `/api/v1/*` ni a `/health`. MAY mostrar un título o texto placeholder de la sección (español) sin datos.
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

- **WHEN** el hash es `#/admin/productos` (o `#/admin/marcas`, `#/admin/stock`, u otra sección del menú distinta de `categorias`)
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
