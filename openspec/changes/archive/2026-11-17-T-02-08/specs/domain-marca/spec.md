# Spec Delta

Ticket padre: T-02-08. Change: T-02-08. HU: HU-02-02.

## Purpose

Define la entidad `Marca`, el reuso de `Slug` desde el nombre y el puerto `BrandRepo` en el crate `domain`, para que el adaptador SurrealDB (mismo ticket) persista un maestro plano sin HTTP.

## ADDED Requirements

### Requirement: Entidad Marca con identidad tipada

El crate `domain` MUST exponer una entidad `Marca` cuyos campos cubren el modelo [4.4.2](../../../../../docs/4-modelo.md#442-marca) con nombres en español ([5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<Marca>`), `nombre` (`string`), `slug` (`Slug` existente de `domain-categoria`), `activo` (`bool`), `creado_en` y `actualizado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`marca:`). Construir una `Marca` MUST exigir `nombre` no vacío tras recortar espacios y de longitud ≤ 100 (OpenAPI `BrandCreate.name`). MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT duplicar el tipo `Slug` ni añadir crates Unicode.

#### Scenario: Construcción válida Fixser (happy, HU-02-02)

- **WHEN** se construye una `Marca` con `nombre` `"Fixser"`, *slug* `"fixser"` y `activo` verdadero
- **THEN** la construcción termina sin error
- **AND** el `id` no contiene `:`
- **AND** `nombre` es `"Fixser"`
- **AND** `activo` es verdadero

#### Scenario: nombre vacío (unhappy)

- **WHEN** se intenta construir una `Marca` con `nombre` vacío o solo espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `Marca`

#### Scenario: nombre demasiado largo (unhappy)

- **WHEN** se intenta construir una `Marca` con `nombre` de más de 100 caracteres
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce una `Marca`

### Requirement: Puerto BrandRepo sin persistencia

El crate `domain` MUST exponer el puerto `BrandRepo` como *trait* con operaciones `create`, `get`, `list_all`, `update` y `set_activo`. Los `id` MUST ser `Id<Marca>` sin prefijo de tabla. Los errores MUST ser `DomainError`. `create` MUST aceptar `NewMarca { nombre, slug: Option<Slug> }`. `update` MUST aceptar `UpdateMarca { id, nombre, slug: Option<Slug> }`. `set_activo` MUST aceptar `id` y `bool` y devolver `Marca`. El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT consultar `producto` ni rechazar baja por ocupación (T-02-09 / T-02-14). MUST NOT depender de `surrealdb` ni de otros crates del workspace.

#### Scenario: El puerto declara create, get, list_all, update y set_activo (happy)

- **WHEN** se inspecciona el puerto `BrandRepo` en `domain`
- **THEN** expone `create`, `get`, `list_all`, `update` y `set_activo`
- **AND** los resultados de éxito usan `Marca` y `Id<Marca>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: domain no persiste marcas (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `CREATE` sobre la tabla `marca`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `marca_constructs_fixser` | `domain` (`src/marca.rs`) | unitario happy (HU-02-02) |
| `marca_rejects_blank_nombre` | `domain` | unitario unhappy |
| `marca_rejects_nombre_too_long` | `domain` | unitario unhappy |
| `brand_repo_trait_exposes_create_get_list_all_update_set_activo` | `domain` | unitario happy (dummy) |
