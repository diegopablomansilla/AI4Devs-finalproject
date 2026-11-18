# domain-atributo-def Specification

## Purpose

Define la entidad `AtributoDef` (clave en `snake_case`, opciones obligatorias si el tipo es `enum`) y el puerto `AttributeDefRepo` en el crate `domain`, para que el adaptador SurrealDB (mismo ticket) persista el maestro de atributos técnicos sin HTTP.

## Requirements

### Requirement: Entidad AtributoDef con identidad tipada

El crate `domain` MUST exponer una entidad `AtributoDef` cuyos campos cubren el modelo [4.4.3](../../../docs/4-modelo.md#443-atributo_def--definición-de-atributos-técnicos) con nombres en español ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<AtributoDef>`), `clave` (`string`), `etiqueta` (`string`), `tipo` (uno de `texto`, `numero`, `booleano`, `enum`), `unidad` (`Option<string>`), `opciones` (`Option` de lista de `string`), `facetable` (`bool`), `buscable` (`bool`), `orden` (`int`), `activo` (`bool`), `creado_en` y `actualizado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`atributo_def:`).

Construir una `AtributoDef` MUST exigir:

- `clave` que coincida con `^[a-z][a-z0-9_]*$` (OpenAPI `AttributeDefinition.key`; `snake_case` que empieza por letra)
- `etiqueta` no vacía tras recortar espacios y de longitud ≤ 60 (OpenAPI `AttributeDefinitionCreate.label`)
- si `tipo` es `enum`, `opciones` presentes y con al menos un valor no vacío tras recortar espacios
- si `tipo` no es `enum`, `opciones` ausentes o vacías
- `unidad`, si está presente, de longitud ≤ 16 (OpenAPI `unit`)

MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT usar los nombres de API en inglés (`key`, `label`, `type`, `searchable`) como campos de la entidad. MUST NOT aplicar la regla `attribute_in_use` (T-02-23).

#### Scenario: Construcción válida grado enum (happy, HU-02-03)

- **WHEN** se construye una `AtributoDef` con `clave` `"grado"`, `etiqueta` `"Grado"`, `tipo` `enum`, `opciones` `["2", "5", "8"]`, `facetable` verdadero y `buscable` verdadero
- **THEN** la construcción termina sin error
- **AND** el `id` no contiene `:`
- **AND** `clave` es `"grado"`
- **AND** `tipo` es `enum`
- **AND** `opciones` son `["2", "5", "8"]`
- **AND** `facetable` es verdadero

#### Scenario: Construcción válida medida texto (happy)

- **WHEN** se construye una `AtributoDef` con `clave` `"medida"`, `etiqueta` `"Medida"`, `tipo` `texto`, `unidad` `"mm"` y sin `opciones`
- **THEN** la construcción termina sin error
- **AND** `tipo` es `texto`
- **AND** `unidad` es `"mm"`
- **AND** `opciones` está ausente o vacía

#### Scenario: enum sin opciones (unhappy, HU-02-03, Listo cuando)

- **WHEN** se intenta construir una `AtributoDef` con `tipo` `enum` y `opciones` ausentes o vacías
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `AtributoDef`

#### Scenario: clave inválida (unhappy)

- **WHEN** se intenta construir una `AtributoDef` con `clave` `"Grado"`, `"medida-mm"` o `"1grado"` (no `snake_case` que empiece por letra)
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `AtributoDef`

#### Scenario: etiqueta vacía (unhappy)

- **WHEN** se intenta construir una `AtributoDef` con `etiqueta` vacía o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `AtributoDef`

### Requirement: Puerto AttributeDefRepo sin persistencia

El crate `domain` MUST exponer el puerto `AttributeDefRepo` como *trait* con operaciones `create`, `get`, `list_all`, `update` y `set_activo`. Los `id` MUST ser `Id<AtributoDef>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewAtributoDef` con `clave`, `etiqueta`, `tipo`, `unidad`, `opciones`, `facetable`, `buscable` y `orden`. `update` MUST aceptar `UpdateAtributoDef` con `id` y los mismos campos editables. `set_activo` MUST aceptar `id` y `bool` y devolver `AtributoDef`. El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT consultar `producto` ni rechazar cambios por `attribute_in_use` (T-02-23). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create, get, list_all, update y set_activo (happy)

- **WHEN** se inspecciona el puerto `AttributeDefRepo` en `domain`
- **THEN** expone `create`, `get`, `list_all`, `update` y `set_activo`
- **AND** los resultados de éxito usan `AtributoDef` y `Id<AtributoDef>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: domain no persiste atributo_def (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `CREATE` sobre la tabla `atributo_def`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `atributo_def_constructs_grado_enum` | `domain` (`src/atributo_def.rs`) | unitario happy (HU-02-03) |
| `atributo_def_constructs_medida_texto` | `domain` | unitario happy |
| `atributo_def_rejects_enum_without_options` | `domain` | unitario unhappy (Listo cuando) |
| `atributo_def_rejects_invalid_clave` | `domain` | unitario unhappy |
| `atributo_def_rejects_blank_etiqueta` | `domain` | unitario unhappy |
| `attribute_def_repo_trait_exposes_create_get_list_all_update_set_activo` | `domain` | unitario happy (dummy) |
