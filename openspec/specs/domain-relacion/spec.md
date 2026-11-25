# domain-relacion Specification

## Purpose
Define la entidad `Relacion`, los tres tipos de arista del grafo, el default de bidireccionalidad, las reglas `self_relation` / `prioridad` / `nota` y el puerto `RelationRepo` en el crate `domain`, para que el adaptador (T-04-03 / T-04-04) y `recommend` (DC-05, T-04-05) implementen un contrato hexagonal sin persistir en este ticket.

Ticket padre: T-04-02. Change: T-04-02. HU: HU-04-01, HU-04-02.

## Requirements

### Requirement: Entidad Relacion con identidad tipada

El crate `domain` MUST exponer una entidad `Relacion` cuyos campos cubren el modelo [4.6.1](../../../../../docs/4-modelo.md#461-tablas-de-relación) con nombres en español ([5.3](../../../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `id` (`Id<Relacion>`), `origen` (`Id<Producto>`), `destino` (`Id<Producto>`), `tipo` (`TipoRelacion`), `prioridad` (`int` ≥ 1), `nota` (`Option<string>`), `grupo_bidireccional` (`Option<Id<GrupoBidireccional>>`), `creado_en` y `actualizado_en` (`Timestamp`). El `id` MUST ser un ULID de 26 caracteres **sin** prefijo de tabla (`similar_a:`, `sustituible_por:` o `complementa:`). `GrupoBidireccional` MUST ser un marcador de identidad (sin campos ni repositorio); el ULID compartido por ambas aristas de un par bidireccional se persiste en T-04-03.

Construir un `Relacion` MUST exigir las reglas de `self_relation`, `prioridad` y `nota` de los requirements siguientes. MUST NOT persistir ni abrir SurrealDB. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT usar los nombres de API en inglés (`sourceProductId`, `targetProductId`, `priority`, `note`, `bidirectionalGroup`, `similar`/`substitute`/`complement`) como campos de la entidad. MUST NOT usar `in`/`out` como nombres de campo en Rust (el adaptador de T-04-03 los mapea).

#### Scenario: Construcción válida complement Perno M8 a Arandela M8 (happy, HU-04-01)

- **WHEN** se construye un `Relacion` con `tipo` `complementa`, `origen` y `destino` distintos, `prioridad` 1 y `nota` `"Arandela plana M8"`
- **THEN** la construcción termina sin error
- **AND** el `id` no contiene `:`
- **AND** `tipo` es `complementa`
- **AND** `prioridad` es 1
- **AND** `nota` es `"Arandela plana M8"`
- **AND** `grupo_bidireccional` está ausente

#### Scenario: Construcción válida similar con grupo bidireccional (happy, HU-04-02)

- **WHEN** se construye un `Relacion` con `tipo` `similar_a`, `origen` y `destino` distintos, `prioridad` 1 y un `grupo_bidireccional` presente
- **THEN** la construcción termina sin error
- **AND** `tipo` es `similar_a`
- **AND** `grupo_bidireccional` está presente
- **AND** el `id` del grupo no contiene `:`

### Requirement: TipoRelacion y bidireccionalidad por defecto

El crate `domain` MUST exponer `TipoRelacion` con exactamente los valores del modelo [4.6.1](../../../../../docs/4-modelo.md#461-tablas-de-relación): `similar_a`, `sustituible_por`, `complementa` (API `similar`/`substitute`/`complement` se traduce en T-04-05). Parsear un valor fuera de ese conjunto, incluido un valor en inglés del contrato, MUST fallar con `DomainError` `code` `invalid_input`.

El crate `domain` MUST exponer una función pura `default_bidireccional` que, dado un `TipoRelacion`, devuelve:

- `true` para `similar_a`
- `false` para `sustituible_por` y `complementa`

Si el llamador omite el flag bidireccional, el valor MUST ser el de `default_bidireccional`. Un flag explícito MUST prevalecer sobre el default. La función MUST NOT I/O ni consultar un repositorio. MUST NOT persistir. MUST NOT mapear a HTTP.

#### Scenario: similar_a bidireccional por defecto (happy, HU-04-02)

- **WHEN** se evalúa `default_bidireccional` con `tipo` `similar_a`
- **THEN** el resultado es verdadero

#### Scenario: substitute y complement no bidireccionales por defecto (happy)

- **WHEN** se evalúa `default_bidireccional` con `tipo` `sustituible_por`
- **THEN** el resultado es falso
- **WHEN** se evalúa `default_bidireccional` con `tipo` `complementa`
- **THEN** el resultado es falso

#### Scenario: flag explícito prevalece sobre el default (happy)

- **WHEN** se resuelve bidireccionalidad para `tipo` `similar_a` con flag explícito `false`
- **THEN** el resultado es falso
- **WHEN** se resuelve bidireccionalidad para `tipo` `complementa` con flag explícito `true`
- **THEN** el resultado es verdadero

#### Scenario: TipoRelacion en inglés del contrato (unhappy)

- **WHEN** se intenta parsear `"similar"`, `"substitute"` o `"complement"` como `TipoRelacion`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** no se produce un `TipoRelacion`

### Requirement: self_relation rechaza origen igual a destino

Al construir un `Relacion`, `origen` y `destino` MUST ser `Id<Producto>` distintos. Si `origen == destino`, la operación MUST fallar con `DomainError::InvalidInput` con `code` `self_relation` ([5.4](../../../../../docs/5-contract-api.md#54-errores); HU-04-01) y `FieldError.field` `destino`, y MUST NOT producir un `Relacion`. MUST NOT usar `category_cycle` ni `invalid_input` genérico para este caso. MUST NOT persistir. MUST NOT comprobar que origen o destino existan o estén activos (eso es `ENFORCED` / T-04-03).

#### Scenario: Relación consigo mismo (unhappy, HU-04-01, Listo cuando)

- **WHEN** se intenta construir un `Relacion` cuyo `origen` y `destino` son el mismo `Id<Producto>`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `self_relation`
- **AND** el `FieldError.field` es `destino`
- **AND** no se produce un `Relacion`

#### Scenario: origen y destino distintos (happy)

- **WHEN** se construye un `Relacion` con dos `Id<Producto>` distintos y el resto de campos válidos
- **THEN** la construcción termina sin error
- **AND** `origen` y `destino` conservan esos ids

### Requirement: prioridad al menos 1 y nota acotada

Al construir un `Relacion`, `prioridad` MUST ser un entero ≥ 1 (1 = más alta, [4.6.1](../../../../../docs/4-modelo.md#461-tablas-de-relación); OpenAPI `priority` minimum 1). Un valor menor que 1 MUST producir `DomainError` `code` `invalid_input` con `FieldError.field` `prioridad` y MUST NOT producir un `Relacion`.

`nota`, si está presente, MUST tener longitud 1..=500 tras recortar espacios (OpenAPI `note` maxLength 500). Una `nota` de solo espacios MUST tratarse como ausente. Una `nota` con más de 500 caracteres tras recortar MUST producir `DomainError` `code` `invalid_input` con `FieldError.field` `nota` y MUST NOT producir un `Relacion`. MUST NOT persistir. MUST NOT mapear a HTTP.

#### Scenario: prioridad 1 y nota de 500 caracteres (happy)

- **WHEN** se construye un `Relacion` con `prioridad` 1 y `nota` de exactamente 500 caracteres
- **THEN** la construcción termina sin error
- **AND** `prioridad` es 1
- **AND** la `nota` tiene longitud 500

#### Scenario: prioridad menor que 1 (unhappy, Listo cuando)

- **WHEN** se intenta construir un `Relacion` con `prioridad` 0 o negativa
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `prioridad`
- **AND** no se produce un `Relacion`

#### Scenario: nota de más de 500 caracteres (unhappy, Listo cuando)

- **WHEN** se intenta construir un `Relacion` con `nota` de 501 caracteres tras recortar espacios
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `nota`
- **AND** no se produce un `Relacion`

### Requirement: Puerto RelationRepo sin persistencia

El crate `domain` MUST exponer el puerto `RelationRepo` como *trait* con operaciones `create`, `update`, `delete` y `list_for_product`. Los `id` MUST ser `Id<Relacion>` / `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. El consumidor previsto del puerto MUST ser el Recommendation Service (`recommend`, [DC-05](../../../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)), no Catalog.

`create` MUST aceptar `NewRelacion` con `origen`, `destino`, `tipo`, `prioridad`, `nota` y `bidireccional` (`Option<bool>`: ausente = `default_bidireccional(tipo)`). El resultado de éxito MUST ser `Relacion` (arista saliente origen→destino). MUST NOT crear la arista inversa en este ticket (T-04-03).

`update` MUST aceptar `UpdateRelacion` con `id`, `tipo`, `prioridad` opcional y `nota` opcional. `delete` MUST aceptar `tipo` y `id`. `list_for_product` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `Vec<Relacion>` (aristas salientes; el agrupado por tipo y el resumen del destino los hidrata T-04-04).

El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT ejecutar `RELATE`. MUST NOT traducir el índice UNIQUE a `duplicate_relation` (T-04-03). MUST NOT depender de `surrealdb` ni de otros crates del workspace. MUST NOT declarar el puerto en `catalog`.

#### Scenario: El puerto declara create, update, delete y list_for_product (happy)

- **WHEN** se inspecciona el puerto `RelationRepo` en `domain`
- **THEN** expone `create`, `update`, `delete` y `list_for_product`
- **AND** `create` acepta `NewRelacion` y el resultado de éxito usa `Relacion`
- **AND** `update` acepta `UpdateRelacion` y el resultado de éxito usa `Relacion`
- **AND** `delete` acepta `TipoRelacion` e `Id<Relacion>`
- **AND** `list_for_product` acepta `Id<Producto>` y el resultado de éxito usa `Vec<Relacion>`
- **AND** los resultados de error usan `DomainError`

#### Scenario: NewRelacion omite bidireccional y usa el default del tipo (happy, HU-04-02)

- **WHEN** se construye un `NewRelacion` con `tipo` `similar_a` y `bidireccional` ausente
- **THEN** el default resuelto es verdadero
- **WHEN** se construye un `NewRelacion` con `tipo` `complementa` y `bidireccional` ausente
- **THEN** el default resuelto es falso

#### Scenario: domain no persiste relaciones (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`
- **AND** no existe un adaptador que ejecute `RELATE` sobre `similar_a`, `sustituible_por` o `complementa`

#### Scenario: RelationRepo no vive en catalog (unhappy, DC-05)

- **WHEN** se inspecciona el crate `catalog`
- **THEN** no declara el trait `RelationRepo`
- **AND** el puerto está en `domain` para consumo de `recommend`
