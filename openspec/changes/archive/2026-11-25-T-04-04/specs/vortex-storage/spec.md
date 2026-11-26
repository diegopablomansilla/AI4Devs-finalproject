# Spec Delta

Ticket padre: T-04-04. Change: T-04-04. HU: HU-04-02, HU-04-03.

## ADDED Requirements

### Requirement: Adaptador RelationRepo update, delete y list_for_product

El crate `storage` MUST implementar `RelationRepo::update`, `delete` y `list_for_product` contra las tablas `similar_a`, `sustituible_por` y `complementa` usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla. Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT emitir evento de outbox. MUST NOT reindexar Meilisearch. MUST NOT cambiar el esquema `0007`. MUST NOT declarar el adaptador en `catalog` ni en `recommend` (DC-05 se cablea en T-04-05). MUST NOT filtrar destinos inactivos ni por `stock` (eso es T-04-06). MUST NOT listar aristas **entrantes**.

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

## MODIFIED Requirements

### Requirement: Adaptador RelationRepo create

El crate `storage` MUST implementar el puerto `RelationRepo` contra las tablas `similar_a`, `sustituible_por` y `complementa` ya migradas (T-04-01) usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla (`similar_a:`, `sustituible_por:` o `complementa:`). Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT emitir evento de outbox (el grafo no está en `OutboxEntity` ni en [4.8.1](../../../../../docs/4-modelo.md#481-outbox); T-04-06 lee SurrealDB). `update`, `delete` y `list_for_product` MUST comportarse según el requirement «Adaptador RelationRepo update, delete y list_for_product» (T-04-04); MUST NOT devolver `unavailable` para esas operaciones. MUST NOT declarar el adaptador en `catalog` ni en `recommend` (DC-05 se cablea en T-04-05). MUST NOT ampliar `DomainError`, `Relacion` ni `NewRelacion`.

`create` MUST persistir `NewRelacion` en una sola transacción:

1. Validar el input con las reglas de dominio (`self_relation`, `prioridad` ≥ 1, `nota` ≤ 500) **antes** de abrir la `Tx`. Si fallan, MUST NOT ejecutar `RELATE`.
2. Resolver bidireccionalidad con `resolve_bidireccional(tipo, bidireccional)` (T-04-02): ausente = default del tipo (`similar_a` verdadero; `sustituible_por` y `complementa` falso). Un flag explícito MUST prevalecer.
3. Si no es bidireccional: un `RELATE` origen→destino en la tabla de `tipo`, **sin** `grupo_bidireccional`.
4. Si es bidireccional: dos `RELATE` (origen→destino y destino→origen) en la **misma** transacción, mismo `grupo_bidireccional` (ULID de 26 caracteres sin prefijo), misma `prioridad` y `nota` ([4.6.1](../../../../../docs/4-modelo.md#461-tablas-de-relación)). Si el segundo `RELATE` falla, MUST NOT quedar la primera arista.
5. El resultado de éxito MUST ser el `Relacion` de la arista saliente origen→destino. Si el par es bidireccional, `grupo_bidireccional` MUST estar presente en esa arista.

Una violación del índice único `(in, out)` (`uq_similar_a`, `uq_sustituible_por` o `uq_complementa`) MUST fallar con `DomainError::Conflict` y `code` `duplicate_relation`, y MUST NOT dejar una segunda arista del mismo par y tipo. Un destino u origen inexistente (`ENFORCED`) MUST fallar con `DomainError` `code` `not_found` y MUST NOT insertar aristas. `origen == destino` MUST ser `self_relation` (dominio) y MUST NOT llegar al motor.

MUST NOT reindexar Meilisearch. MUST NOT cambiar el esquema `0007`.

#### Scenario: A similar B crea A→B y B→A (happy, HU-04-02, Listo cuando)

- **WHEN** existen productos `A` y `B` distintos y se llama `create` con `tipo` `similar_a`, `origen` `A`, `destino` `B`, `prioridad` 1 y `bidireccional` ausente
- **THEN** la operación termina sin error
- **AND** existe la arista `A -> similar_a -> B`
- **AND** existe la arista `B -> similar_a -> A`
- **AND** ambas aristas comparten el mismo `grupo_bidireccional`
- **AND** el `id` devuelto no contiene `:`
- **AND** el `Relacion` devuelto tiene `origen` `A` y `destino` `B`
- **AND** no existe fila nueva en `outbox`

#### Scenario: Complement Perno M8 a Arandela M8 es unidireccional (happy, HU-04-01)

- **WHEN** existen "Perno M8" y "Arandela M8" y se llama `create` con `tipo` `complementa`, origen el perno, destino la arandela, `prioridad` 1 y `bidireccional` ausente
- **THEN** la operación termina sin error
- **AND** existe solo la arista perno→arandela en `complementa`
- **AND** `grupo_bidireccional` está ausente
- **AND** no existe la arista inversa arandela→perno

#### Scenario: Flag explícito false en similar crea una sola arista (happy)

- **WHEN** se llama `create` con `tipo` `similar_a` y `bidireccional` `false` entre dos productos existentes
- **THEN** la operación termina sin error
- **AND** existe solo la arista origen→destino
- **AND** `grupo_bidireccional` está ausente

#### Scenario: Flag explícito true en complementa crea el par (happy)

- **WHEN** se llama `create` con `tipo` `complementa` y `bidireccional` `true` entre dos productos existentes
- **THEN** la operación termina sin error
- **AND** existen ambas aristas
- **AND** ambas comparten el mismo `grupo_bidireccional`

#### Scenario: Repetir A similar B da duplicate_relation (unhappy, HU-04-01, Listo cuando)

- **WHEN** ya existe el par bidireccional `A similar B` y se llama `create` otra vez con `tipo` `similar_a`, origen `A`, destino `B` y `bidireccional` ausente
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_relation`
- **AND** `record_count("similar_a")` sigue siendo 2
- **AND** no queda una tercera ni cuarta arista

#### Scenario: Complement duplicado da duplicate_relation (unhappy, HU-04-01)

- **WHEN** ya existe `A -> complementa -> B` y se llama `create` otra vez con el mismo par y tipo
- **THEN** la operación falla
- **AND** el error es `DomainError::Conflict` con `code` `duplicate_relation`
- **AND** `record_count("complementa")` sigue siendo 1

#### Scenario: Relación consigo mismo (unhappy, HU-04-01)

- **WHEN** se llama `create` con el mismo `Id<Producto>` en `origen` y `destino`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `self_relation`
- **AND** no se inserta ninguna arista

#### Scenario: Destino inexistente (unhappy)

- **WHEN** se llama `create` con un `destino` que no existe en `producto`
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `not_found`
- **AND** no se inserta ninguna arista

#### Scenario: update, delete y list_for_product no persisten (unhappy)

- **WHEN** se llama `update`, `delete` o `list_for_product` en el adaptador
- **THEN** esas operaciones MUST NOT devolver `DomainError` `code` `unavailable` por falta de implementación
- **AND** el comportamiento MUST ser el del requirement «Adaptador RelationRepo update, delete y list_for_product» (T-04-04)

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `relation_repo_update_priority_from_b_updates_both_edges` | `storage` (`tests/relacion_repo.rs`) | integración happy (Listo cuando, HU-04-02) |
| `relation_repo_update_unidirectional_nota_leaves_no_inverse` | `storage` (`tests/relacion_repo.rs`) | integración happy |
| `relation_repo_delete_from_b_removes_both_edges` | `storage` (`tests/relacion_repo.rs`) | integración happy (HU-04-02) |
| `relation_repo_delete_unidirectional_leaves_other_types` | `storage` (`tests/relacion_repo.rs`) | integración happy |
| `relation_repo_list_for_product_groups_by_type_with_destino_resumen` | `storage` (`tests/relacion_repo.rs`) | integración happy (HU-04-03) |
| `relation_repo_list_for_product_empty_when_no_edges` | `storage` (`tests/relacion_repo.rs`) | integración happy |
| `relation_repo_update_unknown_id_is_not_found` | `storage` (`tests/relacion_repo.rs`) | integración unhappy |
| `relation_repo_delete_unknown_id_is_not_found` | `storage` (`tests/relacion_repo.rs`) | integración unhappy |
| `relation_repo_update_rejects_prioridad_below_one` | `storage` (`tests/relacion_repo.rs`) | integración unhappy |
| `relation_repo_update_rejects_nota_over_500` | `storage` (`tests/relacion_repo.rs`) | integración unhappy |
