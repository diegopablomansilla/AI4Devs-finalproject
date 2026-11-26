# catalog-marca Specification

## Purpose

Orquesta en el crate `catalog` los casos de uso de marcas (crear, editar, listar, dar de baja y reactivar) sobre el puerto `BrandRepo`, con baja lógica permitida aunque existan productos asociados, tests contra un repositorio en memoria y fachada object-safe para el Gateway, sin HTTP.

Ticket padre: T-02-09. HU: HU-02-02.

## Requirements

### Requirement: Crear, editar y listar marcas

El crate `catalog` MUST exponer casos de uso de marca que delegan en `BrandRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. Crear MUST aceptar `nombre` y `slug` opcional y MUST persistir vía `BrandRepo::create`. Tras crear "Fixser" sin slug, la marca resultante MUST tener `nombre` `"Fixser"` y `slug` derivado del nombre (`Slug::from_nombre`). Editar MUST cambiar `nombre` y, si se envía, `slug` vía `BrandRepo::update`; si `slug` es `None`, el puerto regenera el slug desde el nombre (T-02-08). Listar MUST delegar en `BrandRepo::list_all` y MUST NOT filtrar ni ordenar (el Gateway aplica `includeInactive`, `q` y el orden por nombre). Los errores del puerto (`duplicate_slug`, `not_found`, `invalid_input`) MUST propagarse sin remapeo. MUST NOT reindexar productos (T-03-08). MUST NOT consultar `producto`.

#### Scenario: Crear Fixser (happy, HU-02-02)

- **WHEN** el caso de uso crea la marca con `nombre` `"Fixser"` y sin slug
- **THEN** la operación termina sin error
- **AND** `nombre` es `"Fixser"`
- **AND** `activo` es verdadero
- **AND** el `slug` no está vacío

#### Scenario: Editar nombre (happy)

- **WHEN** existe "Fixser" y el caso de uso la renombra a "Fixser Pro" sin slug
- **THEN** la marca persistida tiene `nombre` `"Fixser Pro"`
- **AND** `activo` no cambia

#### Scenario: Editar slug (happy)

- **WHEN** existe "Fixser" y el caso de uso actualiza el slug a uno válido distinto
- **THEN** la marca persistida tiene ese `slug`
- **AND** `nombre` no cambia si no se envió nombre nuevo

#### Scenario: Slug duplicado al crear (unhappy)

- **WHEN** ya existe una marca con el mismo `slug`
- **AND** el caso de uso intenta crear otra con ese `slug`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `duplicate_slug`

#### Scenario: Nombre vacío al crear (unhappy)

- **WHEN** el caso de uso intenta crear una marca con `nombre` vacío o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

#### Scenario: Editar id desconocido (unhappy)

- **WHEN** el caso de uso edita un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Baja y reactivación sin occupancy

Dar de baja MUST ser baja lógica (`activo = false`) vía `BrandRepo::set_activo`. MUST NOT rechazarse por productos asociados: no hay puerto de occupancy y MUST NOT consultarse `producto` (HU-02-02: los productos conservan la referencia; T-02-14). Si la marca ya está inactiva, dar de baja MUST ser éxito idempotente. Reactivar MUST poner `activo = true` vía `set_activo`. `id` desconocido MUST ser `not_found`. MUST NOT hacer `DELETE` físico. MUST NOT devolver `conflict` por ocupación.

#### Scenario: Baja lógica (happy, HU-02-02)

- **WHEN** existe la marca "Fixser" activa
- **AND** el caso de uso la da de baja
- **THEN** la marca persistida tiene `activo` falso

#### Scenario: Baja con productos asociados permitida (happy, HU-02-02)

- **WHEN** existe la marca "Fixser"
- **AND** el caso de uso la da de baja (sin consultar occupancy ni `producto`)
- **THEN** la operación termina sin error
- **AND** `activo` es falso
- **AND** el error no es `conflict`

#### Scenario: Reactivar (happy)

- **WHEN** "Fixser" está inactiva
- **AND** el caso de uso la reactiva
- **THEN** la marca persistida tiene `activo` verdadero

#### Scenario: Baja de id desconocido (unhappy)

- **WHEN** el caso de uso da de baja un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Tests en memoria de cada caso

Los tests del crate `catalog` MUST usar un `BrandRepo` en memoria (sin SurrealDB). MUST cubrir cada caso de uso del ticket, incluido el unhappy path y la baja permitida (HU-02-02). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_create_fixser_sets_slug` | `catalog` | unitario (memoria), happy, HU-02-02 |
| `catalog_rename_updates_nombre` | `catalog` | unitario (memoria), happy |
| `catalog_update_slug` | `catalog` | unitario (memoria), happy |
| `catalog_deactivate_sets_inactive` | `catalog` | unitario (memoria), happy, HU-02-02 |
| `catalog_deactivate_with_associated_products_succeeds` | `catalog` | unitario (memoria), happy, HU-02-02 / Listo cuando (baja permitida) |
| `catalog_reactivate_sets_active` | `catalog` | unitario (memoria), happy |
| `catalog_create_duplicate_slug_is_duplicate_slug` | `catalog` | unitario (memoria), unhappy |
| `catalog_create_blank_nombre_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_rename_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_deactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB

### Requirement: Fachada object-safe

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de marca (crear, get, listar, editar, dar de baja, reactivar) para que el Gateway la inyecte como `Arc<dyn …>`. El servicio de marcas MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB. MUST NOT exponer occupancy.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de marcas con un `BrandRepo`
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
