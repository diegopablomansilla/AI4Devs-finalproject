# Spec Delta

Ticket padre: T-04-03. Change: T-04-03. HU: HU-04-01, HU-04-02.

## ADDED Requirements

### Requirement: Adaptador RelationRepo create

El crate `storage` MUST implementar el puerto `RelationRepo` contra las tablas `similar_a`, `sustituible_por` y `complementa` ya migradas (T-04-01) usando `storage::Tx` (T-03-02). Los `id` MUST ser ULID de 26 caracteres **sin** prefijo de tabla (`similar_a:`, `sustituible_por:` o `complementa:`). Los errores MUST ser `DomainError`. MUST NOT mapear a HTTP ni a `problem+json`. MUST NOT emitir evento de outbox (el grafo no está en `OutboxEntity` ni en [4.8.1](../../../../../docs/4-modelo.md#481-outbox); T-04-06 lee SurrealDB). MUST NOT implementar el comportamiento de `update`, `delete` ni `list_for_product` (T-04-04): esas operaciones MUST devolver `DomainError::Unavailable` con `code` `unavailable`. MUST NOT declarar el adaptador en `catalog` ni en `recommend` (DC-05 se cablea en T-04-05). MUST NOT ampliar `DomainError`, `Relacion` ni `NewRelacion`.

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

- **WHEN** se llama `update`, `delete` o `list_for_product` en el adaptador de este ticket
- **THEN** `update` y `delete` fallan con `DomainError` `code` `unavailable`
- **AND** `list_for_product` falla con `DomainError` `code` `unavailable`
- **AND** no se modifican las tablas de relación

## MODIFIED Requirements

### Requirement: UNIQUE y THROW se traducen a DomainError

`Tx::commit` MUST traducir errores del motor a `DomainError` existente (T-00-03) y MUST NOT mapear a HTTP ni `problem+json`. `THROW "stock_negativo"` MUST ser `InvalidInput` con `code` `negative_stock` ([4.5.2](../../../../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción), [5.4](../../../../../docs/5-contract-api.md#54-errores)). Un `THROW` cuyo texto es un `code` de dominio conocido MUST usar esa variante/`code`. Una violación de índice `UNIQUE` MUST ser `Conflict`: `duplicate_sku` si el índice o el mensaje identifican `sku`; `duplicate_slug` si identifican `slug`; `duplicate_relation` si identifican `relation`, los índices `uq_similar_a` / `uq_sustituible_por` / `uq_complementa`, las tablas `similar_a` / `sustituible_por` / `complementa`, o si las sentencias de la `Tx` contienen `RELATE` hacia esas tablas; en cualquier otro UNIQUE, `conflict`. Un fallo de motor no clasificado MUST ser `Unavailable` con `code` `unavailable`. MUST NOT agregar variantes ni `code` nuevos a `DomainError`.

#### Scenario: THROW stock_negativo (unhappy)

- **WHEN** una `Tx` ejecuta `THROW "stock_negativo"` y se confirma
- **THEN** `commit` falla con `DomainError::InvalidInput`
- **AND** el `code` expuesto es `negative_stock`

#### Scenario: UNIQUE sku (unhappy)

- **WHEN** una `Tx` inserta dos filas que violan un índice `UNIQUE` cuyo nombre incluye `sku` y se confirma
- **THEN** `commit` falla con `DomainError::Conflict`
- **AND** el `code` expuesto es `duplicate_sku`
- **AND** no queda la segunda fila ni su evento de outbox (si se había encolado en esa `Tx`)

#### Scenario: UNIQUE de arista de relación (unhappy)

- **WHEN** una `Tx` inserta un segundo `RELATE` que viola `uq_similar_a`, `uq_sustituible_por` o `uq_complementa` y se confirma
- **THEN** `commit` falla con `DomainError::Conflict`
- **AND** el `code` expuesto es `duplicate_relation`

#### Scenario: Error de motor no clasificado (unhappy)

- **WHEN** una `Tx` ejecuta una sentencia que el motor rechaza sin ser `THROW` de código conocido ni UNIQUE (p. ej. SQL inválido)
- **THEN** `commit` falla con `DomainError::Unavailable`
- **AND** el `code` expuesto es `unavailable`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `relation_repo_creates_similar_a_to_b_and_b_to_a` | `storage` (`tests/relacion_repo.rs`) | integración happy (Listo cuando, HU-04-02) |
| `relation_repo_creates_complement_perno_to_arandela` | `storage` (`tests/relacion_repo.rs`) | integración happy (HU-04-01) |
| `relation_repo_similar_explicit_false_is_unidirectional` | `storage` (`tests/relacion_repo.rs`) | integración happy |
| `relation_repo_complement_explicit_true_is_bidirectional` | `storage` (`tests/relacion_repo.rs`) | integración happy |
| `relation_repo_repeat_similar_is_duplicate_relation` | `storage` (`tests/relacion_repo.rs`) | integración unhappy (Listo cuando, HU-04-01) |
| `relation_repo_repeat_complement_is_duplicate_relation` | `storage` (`tests/relacion_repo.rs`) | integración unhappy (HU-04-01) |
| `relation_repo_rejects_self_relation` | `storage` (`tests/relacion_repo.rs`) | integración unhappy (HU-04-01) |
| `relation_repo_missing_destino_is_not_found` | `storage` (`tests/relacion_repo.rs`) | integración unhappy |
| `relation_repo_update_delete_list_are_unavailable` | `storage` (`tests/relacion_repo.rs`) | integración unhappy (T-04-04) |
| `tx_unique_relation_is_duplicate_relation` | `storage` (`tests/tx.rs`) | integración unhappy (mapper) |
