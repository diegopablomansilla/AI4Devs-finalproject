# domain-categoria Specification

## Purpose
Define la entidad `Categoria`, la generación de *slug* desde el nombre, la profundidad máxima del árbol, la detección de ciclo como función pura y el puerto `CategoryRepo` en el crate `domain`, para que el repositorio (T-02-03) implemente un contrato hexagonal sin lógica de árbol en SurrealDB.

## Requirements

### Requirement: Entidad Categoria con identidad tipada

El crate `domain` MUST exponer una entidad `Categoria` cuyos campos cubren el modelo [4.4.1](../../../../../docs/4-modelo.md#441-categoria--árbol) con nombres en español ([5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<Categoria>`), `nombre` (`string`), `slug`, `padre` (`Option<Id<Categoria>>`), `ancestros` (`Vec<Id<Categoria>>`), `ruta` (`Vec<String>`), `activo` (`bool`), `creado_en` y `actualizado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`categoria:`). Construir una `Categoria` MUST exigir `nombre` no vacío tras recortar espacios y de longitud ≤ 100 (OpenAPI `CategoryCreate.name`). MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP ni a `problem+json`.

#### Scenario: Construcción de raíz válida (happy)

- **WHEN** se construye una `Categoria` raíz (`padre` ausente, `ancestros` vacío) con `nombre` `"Fijaciones"`, *slug* válido, `ruta` `["Fijaciones"]` y `activo` verdadero
- **THEN** la construcción termina sin error
- **AND** el `id` no contiene `:`
- **AND** `nombre` es `"Fijaciones"`
- **AND** `padre` está ausente
- **AND** `ancestros` está vacío

#### Scenario: Construcción de hija válida (happy)

- **WHEN** se construye una `Categoria` con `padre` igual al `id` de `"Fijaciones"`, `ancestros` con ese `id`, `nombre` `"Pernos"` y `ruta` `["Fijaciones", "Pernos"]`
- **THEN** la construcción termina sin error
- **AND** la profundidad resultante es 2

#### Scenario: nombre vacío (unhappy)

- **WHEN** se intenta construir una `Categoria` con `nombre` vacío o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `Categoria`

### Requirement: Generación de slug desde el nombre

El crate `domain` MUST generar el *slug* de una categoría a partir de `nombre` cuando no se aporta uno explícito. El valor MUST coincidir con el patrón OpenAPI de `CategoryCreate.slug`: `^[a-z0-9]+(-[a-z0-9]+)*$`. La generación MUST: recortar espacios; pasar a minúsculas ASCII; transliterar vocales acentuadas y `ñ`/`ü` del español (`áéíóúüñ` → `aeiouun` y sus mayúsculas); sustituir cualquier otro carácter no `[a-z0-9]` por un guion; colapsar guiones repetidos; eliminar guiones al inicio y al final. MUST NOT añadir una crate de Unicode al crate `domain`. Un *slug* explícito MUST aceptarse solo si ya cumple el patrón; en caso contrario MUST fallar con `invalid_input`.

#### Scenario: Slug desde nombre ASCII (happy)

- **WHEN** se genera el *slug* desde el nombre `"Fijaciones"`
- **THEN** el resultado es `"fijaciones"`

#### Scenario: Slug desde nombre con tilde y eñe (happy)

- **WHEN** se genera el *slug* desde `"Tornillería"` y desde `"Caños"`
- **THEN** los resultados son `"tornilleria"` y `"canos"` respectivamente

#### Scenario: Slug desde nombre con espacios y código (happy)

- **WHEN** se genera el *slug* desde `"  Pernos M8 x 40  "`
- **THEN** el resultado es `"pernos-m8-x-40"`

#### Scenario: nombre que no produce slug (unhappy)

- **WHEN** se genera el *slug* desde `""`, `"   "` o `"???"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un *slug*

#### Scenario: slug explícito fuera de patrón (unhappy)

- **WHEN** se valida un *slug* explícito `"Tornillería"`, `"Pernos_M8"` o `"-abc-"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un *slug*

### Requirement: Profundidad máxima 4

La profundidad de una categoría MUST ser `ancestros.len() + 1` (raíz = 1). El árbol MUST admitir como máximo 4 niveles, alineado a las facetas `categoria.lvl0`…`categoria.lvl3` de [4.11.2](../../../../../docs/4-modelo.md#4112-configuración-del-índice). Construir o validar una categoría de profundidad 1 a 4 MUST terminar sin error. Profundidad 5 o mayor MUST fallar con `DomainError` `code` `invalid_input`. MUST NOT persistir.

#### Scenario: Raíz tiene profundidad 1 (happy)

- **WHEN** se calcula la profundidad de una categoría con `ancestros` vacío
- **THEN** el resultado es 1

#### Scenario: Cuatro niveles permitidos (happy)

- **WHEN** se valida una categoría con tres ancestros (profundidad 4)
- **THEN** la validación termina sin error

#### Scenario: Quinto nivel rechazado (unhappy)

- **WHEN** se valida una categoría con cuatro ancestros (profundidad 5)
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

### Requirement: Detección de ciclo como función pura

El crate `domain` MUST exponer una función pura que, dados el `id` del nodo, el `id` del nuevo padre (opcional) y la lista de ancestros del nuevo padre, decide si esa asignación formaría un ciclo. MUST ser ciclo: el nuevo padre es el propio nodo; o el `id` del nodo aparece en los ancestros del nuevo padre (el destino está en el subárbol del nodo, p. ej. mover `"Fijaciones"` debajo de `"Pernos"`). Un padre ausente (mover a raíz) MUST NOT ser ciclo. La función MUST NOT I/O ni consultar un repositorio. Cuando el dominio rechaza la asignación por ciclo, el error MUST ser `DomainError::InvalidInput` con `code` `category_cycle` ([5.4](../../../../../docs/5-contract-api.md#54-errores)). MUST NOT usar `self_relation` (ese `code` es de relaciones de producto, T-04-02).

#### Scenario: Padre válido no cicla (happy)

- **WHEN** se evalúa mover el nodo `Pernos` debajo de un padre `Herramientas` cuyos ancestros no incluyen a `Pernos`
- **THEN** la función indica que no hay ciclo

#### Scenario: Mover a raíz no cicla (happy)

- **WHEN** se evalúa asignar padre ausente a un nodo que ya tiene padre
- **THEN** la función indica que no hay ciclo

#### Scenario: Padre igual al nodo (unhappy)

- **WHEN** se evalúa asignar como padre el propio `id` del nodo
- **THEN** la función indica ciclo
- **AND** el rechazo de dominio expone `code` `category_cycle`

#### Scenario: Mover debajo de un descendiente (unhappy)

- **WHEN** `"Pernos"` es hija de `"Fijaciones"` (los ancestros de `"Pernos"` incluyen el `id` de `"Fijaciones"`)
- **AND** se evalúa mover `"Fijaciones"` debajo de `"Pernos"`
- **THEN** la función indica ciclo
- **AND** el rechazo de dominio expone `code` `category_cycle`

### Requirement: Puerto CategoryRepo sin persistencia

El crate `domain` MUST exponer el puerto `CategoryRepo` como *trait* con operaciones `create`, `get` y `list_all`. Los `id` MUST ser `Id<Categoria>` sin prefijo de tabla. Los errores MUST ser `DomainError`. El crate `domain` MUST NOT implementar el adaptador SurrealDB ni calcular `ancestros`/`ruta` al persistir (T-02-03). El puerto MUST NOT declarar `update` (T-02-04). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create, get y list_all (happy)

- **WHEN** se inspecciona el puerto `CategoryRepo` en `domain`
- **THEN** expone `create`, `get` y `list_all`
- **AND** los resultados de éxito usan `Categoria` y `Id<Categoria>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: domain no persiste categorías (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `CREATE` sobre la tabla `categoria`
