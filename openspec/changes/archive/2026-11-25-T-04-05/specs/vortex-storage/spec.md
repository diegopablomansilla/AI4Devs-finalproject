# Spec Delta

Ticket padre: T-04-05. Change: T-04-05. HU: HU-04-03.

## ADDED Requirements

### Requirement: Adaptador RelationRepo list_incoming_for_product

El crate `storage` MUST implementar `RelationRepo::list_incoming_for_product` contra las tablas `similar_a`, `sustituible_por` y `complementa`. MUST devolver `RelacionesDeProducto` con las aristas **entrantes** (`out` = el producto pedido) de las tres tablas, agrupadas por tipo, ordenadas por `prioridad` ascendente (1 = más alta) y, a igualdad, por `id`. Cada ítem MUST ser `RelacionConDestino` cuyo `DestinoResumen` es el producto **origen** (`in`: `id`, `sku`, `nombre`, `activo`). MUST incluir orígenes inactivos y con `stock` 0. Un producto sin aristas entrantes, o un `Id<Producto>` que no existe, MUST devolver los tres grupos vacíos (el 404 de producto lo aplica `recommend` en T-04-05). MUST NOT hidratar `ProductSummary` completo. MUST NOT filtrar por `activo` ni `stock` (T-04-06). MUST NOT mapear a HTTP. MUST NOT cambiar el esquema `0007`.

#### Scenario: Listado incoming con resumen del origen (happy)

- **WHEN** existe una arista `A complement B` y se llama `list_incoming_for_product` con B
- **THEN** la operación termina sin error
- **AND** `complementa` contiene esa arista
- **AND** el `DestinoResumen` es A (`sku` y `nombre` de A)
- **AND** `list_for_product(B)` no incluye esa arista (sigue siendo solo salientes)

#### Scenario: Incoming vacío (happy)

- **WHEN** un producto no tiene aristas entrantes y se llama `list_incoming_for_product`
- **THEN** la operación termina sin error
- **AND** los tres grupos están vacíos

## MODIFIED Requirements

### Requirement: Adaptador RelationRepo update, delete y list_for_product

El crate `storage` MUST implementar `RelationRepo::update`, `delete` y `list_for_product` contra las tablas `similar_a`, `sustituible_por` y `complementa` usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT emitir evento de outbox. MUST NOT reindexar Meilisearch. MUST NOT cambiar el esquema `0007`. MUST NOT declarar el adaptador en `catalog` ni en `recommend` (DC-05 se cablea en T-04-05). MUST NOT filtrar destinos inactivos ni por `stock` (eso es T-04-06). El listado **entrante** MUST vivir en `list_incoming_for_product` (requirement «Adaptador RelationRepo list_incoming_for_product»); `list_for_product` MUST seguir devolviendo solo salientes.

`update` MUST aplicar `UpdateRelacion` (`id`, `tipo`, `prioridad` opcional, `nota` opcional) en una sola transacción:

1. Resolver la arista `tipo`+`id`. Si no existe, MUST fallar con `DomainError` `code` `not_found` y MUST NOT modificar tablas.
2. Validar el parche con las reglas de dominio **antes** de abrir la `Tx`: `prioridad` presente MUST ser ≥ 1; `nota` presente MUST cumplir 1..=500 tras recortar (o ausente si solo espacios). Si fallan, MUST NOT persistir.
3. Campos omitidos (`None`) MUST conservar el valor actual de la arista.
4. Si la arista tiene `grupo_bidireccional`, el mismo parche MUST aplicarse a **ambas** aristas del grupo en esa `Tx` ([4.6.1](../../../../../docs/4-modelo.md#461-tablas-de-relación); HU-04-02). Si no tiene grupo, MUST actualizar solo esa arista.
5. El resultado de éxito MUST ser el `Relacion` de la arista identificada por `id` (no el par). `actualizado_en` MUST refrescarse. MUST NOT cambiar `origen`, `destino`, `tipo` ni `grupo_bidireccional`.

`delete` MUST borrar **físicamente** (no baja lógica) la arista `tipo`+`id`. Si no existe, MUST fallar con `code` `not_found`. Si tiene `grupo_bidireccional`, MUST borrar ambas aristas del grupo en la misma `Tx`. Si no tiene grupo, MUST borrar solo esa arista. MUST NOT dejar aristas huérfanas del grupo. MUST NOT tocar `producto`.

`list_for_product` MUST devolver `RelacionesDeProducto` con las aristas **salientes** (`in` = el producto) de las tres tablas, agrupadas en `similar_a`, `sustituible_por` y `complementa`. Cada grupo MUST ordenarse por `prioridad` ascendente (1 = más alta) y, a igualdad, por `id`. Cada ítem MUST ser `RelacionConDestino` con el `Relacion` de la arista y `DestinoResumen` del `out` (`id`, `sku`, `nombre`, `activo`). MUST incluir destinos inactivos y con `stock` 0. Un producto sin aristas salientes, o un `Id<Producto>` que no existe, MUST devolver los tres grupos vacíos (el 404 de producto es T-04-05). MUST NOT hidratar `ProductSummary` completo.

#### Scenario: Cambiar prioridad desde B modifica ambas aristas (happy, HU-04-02, Listo cuando)

- **WHEN** existe el par bidireccional `A similar B` y se llama `update` con el `id` de la arista `B → A`, `tipo` `similar_a` y `prioridad` 3
- **THEN** la operación termina sin error
- **AND** la arista `B → A` tiene `prioridad` 3
- **AND** la arista `A → B` tiene `prioridad` 3
- **AND** ambas conservan el mismo `grupo_bidireccional`
- **AND** el `Relacion` devuelto es la arista `B → A`
- **AND** no existe fila nueva en `outbox`

#### Scenario: Editar nota de un complement unidireccional no toca inversa (happy)

- **WHEN** existe solo `Perno → Arandela` en `complementa` y se llama `update` con esa arista, `nota` `"Arandela plana M8"` y `prioridad` ausente
- **THEN** la operación termina sin error
- **AND** esa arista tiene la nota nueva y la misma `prioridad`
- **AND** no existe arista inversa
- **AND** `record_count("complementa")` sigue siendo 1

#### Scenario: Eliminar el grupo bidireccional desde B borra ambas (happy, HU-04-02)

- **WHEN** existe el par bidireccional `A similar B` y se llama `delete` con el `id` de la arista `B → A` y `tipo` `similar_a`
- **THEN** la operación termina sin error
- **AND** no existe la arista `A → B`
- **AND** no existe la arista `B → A`
- **AND** `record_count("similar_a")` es 0

#### Scenario: Eliminar complement unidireccional no borra otras aristas (happy)

- **WHEN** existe `Perno → Arandela` en `complementa` y un par `similar_a` entre los mismos productos
- **AND** se llama `delete` sobre la arista `complementa`
- **THEN** la operación termina sin error
- **AND** `record_count("complementa")` es 0
- **AND** el par `similar_a` permanece

#### Scenario: Listado agrupado con resumen del destino (happy, HU-04-03)

- **WHEN** un producto tiene al menos una arista saliente de cada tipo (incl. un destino inactivo)
- **AND** se llama `list_for_product` con ese producto
- **THEN** la operación termina sin error
- **AND** `similar_a`, `sustituible_por` y `complementa` contienen esas aristas salientes
- **AND** cada ítem expone `prioridad`, `nota`, `grupo_bidireccional` y `DestinoResumen` con `id`, `sku`, `nombre` y `activo`
- **AND** el destino inactivo aparece con `activo` falso
- **AND** no aparecen aristas entrantes
- **AND** cada grupo está ordenado por `prioridad` ascendente

#### Scenario: Producto sin relaciones lista grupos vacíos (happy)

- **WHEN** existe un producto sin aristas y se llama `list_for_product`
- **THEN** la operación termina sin error
- **AND** los tres grupos están vacíos

#### Scenario: update id desconocido (unhappy)

- **WHEN** se llama `update` con un `id` que no existe en la tabla de `tipo`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** no se modifican las tablas de relación

#### Scenario: delete id desconocido (unhappy)

- **WHEN** se llama `delete` con un `id` que no existe en la tabla de `tipo`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** no se modifican las tablas de relación

#### Scenario: update con prioridad menor que 1 (unhappy)

- **WHEN** existe una arista y se llama `update` con `prioridad` 0
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `prioridad`
- **AND** la arista conserva su `prioridad` anterior

#### Scenario: update con nota de más de 500 caracteres (unhappy)

- **WHEN** existe una arista y se llama `update` con `nota` de 501 caracteres
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el `FieldError.field` es `nota`
- **AND** la arista conserva su `nota` anterior
