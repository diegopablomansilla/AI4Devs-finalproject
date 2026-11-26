# Spec Delta

Ticket padre: T-04-05. Change: T-04-05. HU: HU-04-03.

## MODIFIED Requirements

### Requirement: Puerto RelationRepo sin persistencia

El crate `domain` MUST exponer el puerto `RelationRepo` como *trait* con operaciones `create`, `update`, `delete`, `list_for_product` y `list_incoming_for_product`. Los `id` MUST ser `Id<Relacion>` / `Id<Producto>` sin prefijo de tabla. Los errores MUST ser `DomainError`. El consumidor previsto del puerto MUST ser el Recommendation Service (`recommend`, [DC-05](../../../../../docs/3-componentes.md#36-decisiones-de-diseño-transversales-a-los-componentes)), no Catalog.

`create` MUST aceptar `NewRelacion` con `origen`, `destino`, `tipo`, `prioridad`, `nota` y `bidireccional` (`Option<bool>`: ausente = `default_bidireccional(tipo)`). El resultado de éxito MUST ser `Relacion` (arista saliente origen→destino). MUST NOT crear la arista inversa en el crate `domain` (persistencia en `storage`).

`update` MUST aceptar `UpdateRelacion` con `id`, `tipo`, `prioridad` opcional y `nota` opcional. El resultado de éxito MUST ser `Relacion`. `delete` MUST aceptar `tipo` y `id`. `list_for_product` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `RelacionesDeProducto` (aristas **salientes** agrupadas por tipo, cada ítem con `DestinoResumen`; el adaptador en `storage` hidrata). `list_incoming_for_product` MUST aceptar `Id<Producto>` y el resultado de éxito MUST ser `RelacionesDeProducto` (aristas **entrantes** agrupadas por tipo; cada `DestinoResumen` es el **origen** de la arista).

El crate `domain` MUST NOT implementar el adaptador SurrealDB. MUST NOT mapear a HTTP. MUST NOT ejecutar `RELATE`. MUST NOT traducir el índice UNIQUE a `duplicate_relation`. MUST NOT depender de `surrealdb` ni de otros crates del workspace. MUST NOT declarar el puerto en `catalog`.

#### Scenario: El puerto declara create, update, delete y list_for_product (happy)

- **WHEN** se inspecciona el puerto `RelationRepo` en `domain`
- **THEN** expone `create`, `update`, `delete` y `list_for_product`
- **AND** `create` acepta `NewRelacion` y el resultado de éxito usa `Relacion`
- **AND** `update` acepta `UpdateRelacion` y el resultado de éxito usa `Relacion`
- **AND** `delete` acepta `TipoRelacion` e `Id<Relacion>`
- **AND** `list_for_product` acepta `Id<Producto>` y el resultado de éxito usa `RelacionesDeProducto`
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

#### Scenario: El puerto declara list_incoming_for_product (happy, T-04-05)

- **WHEN** se inspecciona el puerto `RelationRepo` en `domain`
- **THEN** expone `list_incoming_for_product`
- **AND** acepta `Id<Producto>`
- **AND** el resultado de éxito usa `RelacionesDeProducto`
