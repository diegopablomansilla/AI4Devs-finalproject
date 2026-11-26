# catalog-atributo-def Specification

## Purpose
Orquesta en el crate `catalog` los casos de uso de definiciones de atributo técnico (crear, editar, listar, dar de baja y reactivar) sobre el puerto `AttributeDefRepo`, sin la regla `attribute_in_use`, con tests contra un repositorio en memoria y fachada object-safe para el Gateway, sin HTTP.

Ticket padre: T-02-12. HU: HU-02-03.

## Requirements

### Requirement: Crear, editar y listar definiciones de atributo

El crate `catalog` MUST exponer casos de uso de `AtributoDef` que delegan en `AttributeDefRepo` y MUST NOT abrir SurrealDB ni mapear a HTTP. Crear MUST aceptar `clave`, `etiqueta`, `tipo`, `unidad` opcional, `opciones` opcional, `facetable`, `buscable` y `orden` y MUST persistir vía `AttributeDefRepo::create`. Tras crear "grado" de tipo `enum` con opciones `["2", "5", "8"]` y `facetable` verdadero, la definición resultante MUST tener `clave` `"grado"`, `tipo` `enum`, esas opciones y `facetable` verdadero. Editar MUST fusionar los campos enviados y, para `clave`/`tipo`/`opciones`, MUST aplicar la regla `attribute_in_use` (requirement «Regla attribute_in_use al editar») antes de `AttributeDefRepo::update`. Listar MUST delegar en `AttributeDefRepo::list_all` y MUST NOT filtrar ni ordenar (el Gateway aplica `includeInactive` y el orden por `orden`/`clave`). Los errores del puerto (`conflict`, `not_found`, `invalid_input`, `attribute_in_use`) MUST propagarse sin remapeo. MUST NOT reindexar productos (T-03-08).

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
- **AND** ningún producto la usa
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

Dar de baja MUST ser baja lógica (`activo = false`) vía `AttributeDefRepo::set_activo`. MUST NOT rechazarse por productos asociados: MUST NOT consultar `AttributeUsage` ni devolver `attribute_in_use` (T-02-23 cubre solo edición de `key`/`type`/`options` en uso). Si la definición ya está inactiva, dar de baja MUST ser éxito idempotente. Reactivar MUST poner `activo = true` vía `set_activo`. `id` desconocido MUST ser `not_found`. MUST NOT hacer `DELETE` físico.

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

Los tests del crate `catalog` MUST usar un `AttributeDefRepo` en memoria (sin SurrealDB) y un `AttributeUsage` configurable. MUST cubrir cada caso de uso del ticket, incluido el unhappy path (HU-02-03 atributo en uso; enum sin opciones; clave duplicada). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_create_grado_enum_sets_options` | `catalog` | unitario (memoria), happy, HU-02-03 |
| `catalog_create_medida_texto` | `catalog` | unitario (memoria), happy |
| `catalog_update_etiqueta` | `catalog` | unitario (memoria), happy |
| `catalog_update_clave_unused_succeeds` | `catalog` | unitario (memoria), happy |
| `catalog_update_clave_in_use_is_attribute_in_use` | `catalog` | unitario (memoria), unhappy, HU-02-03, Listo cuando |
| `catalog_update_tipo_incompatible_in_use_is_attribute_in_use` | `catalog` | unitario (memoria), unhappy, HU-02-03 |
| `catalog_update_opciones_incompatible_in_use_is_attribute_in_use` | `catalog` | unitario (memoria), unhappy |
| `catalog_update_etiqueta_in_use_succeeds` | `catalog` | unitario (memoria), happy |
| `catalog_update_enum_to_texto_in_use_succeeds` | `catalog` | unitario (memoria), happy |
| `catalog_deactivate_in_use_succeeds` | `catalog` | unitario (memoria), happy |
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

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de definición de atributo (crear, get, listar, editar, dar de baja, reactivar) para que el Gateway la inyecte como `Arc<dyn …>`. El servicio MUST implementar esa fachada y MUST recibir `AttributeUsage` además de `AttributeDefRepo`. MUST NOT mapear a HTTP ni abrir SurrealDB. La fachada MUST NOT exponer métodos de occupancy: el Gateway solo ve los casos de uso; `update` MAY devolver `attribute_in_use`.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de definiciones de atributo con un `AttributeDefRepo` y un `AttributeUsage`
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones

### Requirement: Regla attribute_in_use al editar

`AttributeDefService::update` MUST consultar `AttributeUsage::valores` **antes** de `AttributeDefRepo::update`. MUST NOT migrar ni convertir valores persistidos (No hace del ticket).

Si `valores` no está vacío y `clave` del input difiere de la persistida, MUST fallar con `DomainError::Conflict` y `code` `attribute_in_use` y MUST NOT llamar a `AttributeDefRepo::update`.

Si `valores` no está vacío y el nuevo `tipo` u `opciones` deja algún valor incompatible, MUST fallar con `attribute_in_use` y MUST NOT persistir. Compatibilidad MUST ser la misma que `Producto::try_new` / validación de atributos: `texto` exige `ValorAtributo::Texto`; `numero` exige `Numero`; `booleano` exige `Booleano`; `enum` exige `Texto` cuyo valor recortado está en `opciones`. MUST NOT interpretar un texto como número (sin migración). Ampliar `opciones` de un `enum` (valores existentes siguen en el conjunto) MUST suceder. `enum` → `texto` MUST suceder (los valores de enum ya son texto).

Si `valores` está vacío, cambiar `clave` o `tipo` MUST suceder (regresión T-02-12). Cambio de `etiqueta`, `unidad`, `facetable`, `buscable` u `orden` MUST suceder aunque haya productos. Dar de baja o reactivar MUST NOT consultar occupancy ni devolver `attribute_in_use`.

#### Scenario: Cambiar clave de medida en uso (unhappy, HU-02-03, Listo cuando)

- **WHEN** existe la definición `"medida"` y `AttributeUsage` devuelve un valor `"M8"`
- **AND** el caso de uso cambia `clave` a `"medida_mm"`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `attribute_in_use`
- **AND** la definición persistida conserva `clave` `"medida"`

#### Scenario: Cambiar tipo incompatible de medida en uso (unhappy, HU-02-03)

- **WHEN** `"medida"` está en uso con valor texto `"M8"`
- **AND** el caso de uso cambia `tipo` a `numero`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `attribute_in_use`
- **AND** `tipo` persistido sigue siendo `texto`

#### Scenario: Quitar opción de enum en uso (unhappy)

- **WHEN** `"grado"` (`enum` `["2","5","8"]`) está en uso con valor `"5"`
- **AND** el caso de uso deja `opciones` `["2","8"]`
- **THEN** la operación falla
- **AND** el error es `attribute_in_use`

#### Scenario: Editar etiqueta de medida en uso (happy)

- **WHEN** `"medida"` está en uso
- **AND** el caso de uso cambia solo `etiqueta` a `"Medida métrica"`
- **THEN** la operación termina sin error
- **AND** `etiqueta` es `"Medida métrica"`
- **AND** `clave` sigue siendo `"medida"`

#### Scenario: enum a texto con valores existentes (happy)

- **WHEN** `"grado"` está en uso con valor texto `"5"`
- **AND** el caso de uso cambia `tipo` a `texto` (sin `opciones`)
- **THEN** la operación termina sin error
- **AND** `tipo` es `texto`

#### Scenario: clave de definición no usada (happy, regresión T-02-12)

- **WHEN** existe `"medida"` y `AttributeUsage` devuelve lista vacía
- **AND** el caso de uso cambia `clave` a `"medida_mm"`
- **THEN** la operación termina sin error
- **AND** `clave` es `"medida_mm"`

#### Scenario: Baja de definición en uso (happy)

- **WHEN** `"medida"` está en uso
- **AND** el caso de uso la da de baja
- **THEN** la operación termina sin error
- **AND** `activo` es falso
- **AND** el error no es `attribute_in_use`
