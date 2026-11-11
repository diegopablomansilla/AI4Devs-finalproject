# Spec Delta

## Purpose

Orquesta en el crate `catalog` los casos de uso de categorías (crear, editar, mover, dar de baja y reactivar) sobre el puerto `CategoryRepo`, con rechazo H-10 y tests contra un repositorio en memoria, sin HTTP.

Ticket padre: T-02-05. HU: HU-02-01.

## ADDED Requirements

### Requirement: Crear, editar y mover categorías

El crate `catalog` MUST exponer casos de uso de categoría que delegan en `CategoryRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. Crear MUST aceptar `nombre`, `slug` opcional y `padre` opcional y MUST persistir vía `CategoryRepo::create`. Tras crear "Pernos" con padre "Fijaciones", la categoría resultante MUST tener `ruta = ["Fijaciones", "Pernos"]`. Editar MUST cambiar `nombre` conservando `padre` vía `CategoryRepo::update`. Mover MUST cambiar `padre` conservando `nombre` vía `CategoryRepo::update`. Los errores del puerto (`duplicate_slug`, `category_cycle`, `not_found`, `invalid_input`) MUST propagarse sin remapeo. MUST NOT reindexar productos (T-03-08).

#### Scenario: Crear subcategoría (happy)

- **WHEN** existe la categoría "Fijaciones" y el caso de uso crea "Pernos" con ese padre
- **THEN** la operación termina sin error
- **AND** `ruta` es `["Fijaciones", "Pernos"]`

#### Scenario: Editar nombre (happy)

- **WHEN** existe "Fijaciones" y el caso de uso la renombra a "Tornillería"
- **THEN** la categoría persistida tiene `nombre` `"Tornillería"`
- **AND** `padre` no cambia

#### Scenario: Mover de padre (happy)

- **WHEN** "Pernos" es hija de "Fijaciones" y existe "Herramientas"
- **AND** el caso de uso mueve "Pernos" debajo de "Herramientas"
- **THEN** el `padre` persistido es el `id` de "Herramientas"

#### Scenario: Slug duplicado al crear (unhappy)

- **WHEN** ya existe una categoría con el mismo `slug` bajo el mismo padre
- **AND** el caso de uso intenta crear otra con ese `slug`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `duplicate_slug`

#### Scenario: Mover debajo de un descendiente (unhappy)

- **WHEN** "Pernos" es hija de "Fijaciones"
- **AND** el caso de uso intenta mover "Fijaciones" debajo de "Pernos"
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `category_cycle`

#### Scenario: Editar id desconocido (unhappy)

- **WHEN** el caso de uso edita un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Baja y reactivación con rechazo H-10

Dar de baja MUST ser baja lógica (`activo = false`) vía `CategoryRepo::set_activo`. MUST rechazarse con `DomainError::Conflict` y `code` `conflict` (hallazgo H-10; el Gateway mapeará `409` en T-02-06) si la categoría tiene al menos una subcategoría con `activo = true` o si el puerto `CategoryOccupancy::has_active_products` indica productos activos en esa categoría. Subcategorías inactivas y productos inactivos MUST NOT bloquear. Si la categoría ya está inactiva, dar de baja MUST ser éxito idempotente. Reactivar MUST poner `activo = true` vía `set_activo` y MUST NOT consultar occupancy. `id` desconocido MUST ser `not_found`. MUST NOT hacer `DELETE` físico. MUST NOT consultar la tabla `producto` (T-02-14): occupancy es un puerto inyectado.

#### Scenario: Baja de hoja sin productos (happy)

- **WHEN** "Pernos" no tiene hijas activas ni productos activos
- **AND** el caso de uso la da de baja
- **THEN** la categoría persistida tiene `activo` falso

#### Scenario: Reactivar (happy)

- **WHEN** "Pernos" está inactiva
- **AND** el caso de uso la reactiva
- **THEN** la categoría persistida tiene `activo` verdadero

#### Scenario: Baja con subcategoría activa (unhappy)

- **WHEN** "Pernos" es hija activa de "Fijaciones"
- **AND** el caso de uso da de baja "Fijaciones"
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `conflict`
- **AND** "Fijaciones" sigue activa

#### Scenario: Baja con producto activo (unhappy)

- **WHEN** "Pernos" no tiene hijas activas
- **AND** occupancy indica que tiene productos activos
- **AND** el caso de uso da de baja "Pernos"
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `conflict`
- **AND** "Pernos" sigue activa

#### Scenario: Baja de id desconocido (unhappy)

- **WHEN** el caso de uso da de baja un `id` inexistente
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Tests en memoria de cada caso

Los tests del crate `catalog` MUST usar un `CategoryRepo` en memoria (sin SurrealDB) y un occupancy configurable. MUST cubrir cada caso de uso del ticket y la baja rechazada (Listo cuando). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_create_child_sets_ruta` | `catalog` | unitario (memoria), happy, Listo cuando / HU-02-01 |
| `catalog_rename_keeps_padre` | `catalog` | unitario (memoria), happy |
| `catalog_move_updates_padre` | `catalog` | unitario (memoria), happy |
| `catalog_deactivate_leaf_sets_inactive` | `catalog` | unitario (memoria), happy |
| `catalog_reactivate_sets_active` | `catalog` | unitario (memoria), happy |
| `catalog_create_duplicate_slug_is_duplicate_slug` | `catalog` | unitario (memoria), unhappy |
| `catalog_move_under_descendant_is_category_cycle` | `catalog` | unitario (memoria), unhappy |
| `catalog_rename_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_deactivate_with_active_child_is_conflict` | `catalog` | unitario (memoria), unhappy, H-10 / Listo cuando |
| `catalog_deactivate_with_active_product_is_conflict` | `catalog` | unitario (memoria), unhappy, H-10 / Listo cuando |
| `catalog_deactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB
