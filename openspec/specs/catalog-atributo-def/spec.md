# catalog-atributo-def Specification

## Purpose
Orquesta en el crate `catalog` los casos de uso de definiciones de atributo técnico (crear, editar, listar, dar de baja y reactivar) sobre el puerto `AttributeDefRepo`, sin la regla `attribute_in_use`, con tests contra un repositorio en memoria y fachada object-safe para el Gateway, sin HTTP.

Ticket padre: T-02-12. HU: HU-02-03.

## Requirements

### Requirement: Crear, editar y listar definiciones de atributo

El crate `catalog` MUST exponer casos de uso de `AtributoDef` que delegan en `AttributeDefRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. Crear MUST aceptar `clave`, `etiqueta`, `tipo`, `unidad` opcional, `opciones` opcional, `facetable`, `buscable` y `orden` y MUST persistir vía `AttributeDefRepo::create`. Tras crear "grado" de tipo `enum` con opciones `["2", "5", "8"]` y `facetable` verdadero, la definición resultante MUST tener `clave` `"grado"`, `tipo` `enum`, esas opciones y `facetable` verdadero. Editar MUST cambiar los campos enviados vía `AttributeDefRepo::update` (el puerto exige el snapshot editable completo: el caso de uso MUST cargar la fila actual y fusionar). Listar MUST delegar en `AttributeDefRepo::list_all` y MUST NOT filtrar ni ordenar (el Gateway aplica `includeInactive` y el orden por `orden`/`clave`). Los errores del puerto (`conflict`, `not_found`, `invalid_input`) MUST propagarse sin remapeo. MUST NOT reindexar productos (T-03-08). MUST NOT consultar `producto`. MUST NOT aplicar `attribute_in_use` (T-02-23).

#### Scenario: Crear grado enum facetable (happy, HU-02-03)

- **WHEN** el caso de uso crea la definición con `clave` `"grado"`, `etiqueta` `"Grado"`, `tipo` `enum`, `opciones` `["2", "5", "8"]` y `facetable` verdadero
- **THEN** la operación termina sin error
- **AND** `clave` es `"grado"`
- **AND** `tipo` es `enum`
- **AND** `opciones` son `["2", "5", "8"]`
- **AND** `facetable` es verdadero
- **AND** `activo` es verdadero

#### Scenario: Crear medida texto (happy)

- **WHEN** el caso de uso crea la definición con `clave` `"medida"`, `etiqueta` `"Medida"`, `tipo` `texto`, `unidad` `"mm"` y sin `opciones`
- **THEN** la operación termina sin error
- **AND** `tipo` es `texto`
- **AND** `unidad` es `"mm"`

#### Scenario: Editar etiqueta (happy)

- **WHEN** existe "grado" y el caso de uso actualiza `etiqueta` a `"Grado SAE"`
- **THEN** la definición persistida tiene `etiqueta` `"Grado SAE"`
- **AND** `clave` y `tipo` no cambian

#### Scenario: Editar clave de definición no usada (happy)

- **WHEN** existe "medida" y el caso de uso cambia `clave` a `"medida_mm"`
- **THEN** la operación termina sin error
- **AND** `clave` es `"medida_mm"`
- **AND** el error no es `attribute_in_use`

#### Scenario: Clave duplicada al crear (unhappy)

- **WHEN** ya existe una definición con `clave` `"grado"`
- **AND** el caso de uso intenta crear otra con esa `clave`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `conflict`

#### Scenario: enum sin opciones (unhappy, HU-02-03)

- **WHEN** el caso de uso intenta crear una definición con `tipo` `enum` y `opciones` ausentes o vacías
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`

#### Scenario: Editar id desconocido (unhappy)

- **WHEN** el caso de uso edita un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Baja y reactivación sin attribute_in_use

Dar de baja MUST ser baja lógica (`activo = false`) vía `AttributeDefRepo::set_activo`. MUST NOT rechazarse por productos asociados: no hay puerto de occupancy y MUST NOT consultarse `producto` (T-02-23 cubre solo edición de `key`/`type` en uso). Si la definición ya está inactiva, dar de baja MUST ser éxito idempotente. Reactivar MUST poner `activo = true` vía `set_activo`. `id` desconocido MUST ser `not_found`. MUST NOT hacer `DELETE` físico. MUST NOT devolver `attribute_in_use`.

#### Scenario: Baja lógica (happy)

- **WHEN** existe la definición "grado" activa
- **AND** el caso de uso la da de baja
- **THEN** la definición persistida tiene `activo` falso

#### Scenario: Reactivar (happy)

- **WHEN** "grado" está inactiva
- **AND** el caso de uso la reactiva
- **THEN** la definición persistida tiene `activo` verdadero

#### Scenario: Baja de id desconocido (unhappy)

- **WHEN** el caso de uso da de baja un `id` que el repositorio no tiene
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`

### Requirement: Tests en memoria de cada caso

Los tests del crate `catalog` MUST usar un `AttributeDefRepo` en memoria (sin SurrealDB). MUST cubrir cada caso de uso del ticket, incluido el unhappy path (HU-02-03 enum sin opciones; clave duplicada). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_create_grado_enum_sets_options` | `catalog` | unitario (memoria), happy, HU-02-03 |
| `catalog_create_medida_texto` | `catalog` | unitario (memoria), happy |
| `catalog_update_etiqueta` | `catalog` | unitario (memoria), happy |
| `catalog_update_clave_unused_succeeds` | `catalog` | unitario (memoria), happy (sin T-02-23) |
| `catalog_deactivate_sets_inactive` | `catalog` | unitario (memoria), happy |
| `catalog_reactivate_sets_active` | `catalog` | unitario (memoria), happy |
| `catalog_create_duplicate_clave_is_conflict` | `catalog` | unitario (memoria), unhappy |
| `catalog_create_enum_without_options_is_invalid_input` | `catalog` | unitario (memoria), unhappy, HU-02-03 |
| `catalog_update_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_deactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB

### Requirement: Fachada object-safe

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de definición de atributo (crear, get, listar, editar, dar de baja, reactivar) para que el Gateway la inyecte como `Arc<dyn …>`. El servicio MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB. MUST NOT exponer occupancy ni `attribute_in_use`.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de definiciones de atributo con un `AttributeDefRepo`
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
