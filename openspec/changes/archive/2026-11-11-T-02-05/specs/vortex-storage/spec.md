# Spec Delta

## ADDED Requirements

### Requirement: CategoryRepo set_activo

El adaptador `CategoryRepo` en `storage` MUST implementar `set_activo(id, activo)`. MUST actualizar solo el campo `activo` (y `actualizado_en`) de esa fila en una `Tx` con un `OutboxEvent` `categoria`/`upsert`. MUST NOT recalcular `ancestros`/`ruta` ni tocar hijas. `id` desconocido MUST ser `not_found`. MUST NOT reindexar productos (T-03-08). MUST NOT consultar `producto`.

#### Scenario: Baja lógica persistida (happy)

- **WHEN** existe una categoría activa y `set_activo` se llama con `false`
- **THEN** `get` devuelve `activo` falso
- **AND** existe un evento outbox `categoria`/`upsert` adicional

#### Scenario: set_activo de id desconocido (unhappy)

- **WHEN** `set_activo` se llama con un `id` que no existe
- **THEN** la operación falla con `code` `not_found`
- **AND** no se añade evento outbox

#### Scenario: Suite de integración set_activo (happy)

- **WHEN** se ejecuta `cargo test -p storage --test categoria` filtrando `category_repo_set_activo_persists_inactive` y `category_repo_set_activo_unknown_id_is_not_found`
- **THEN** ambos pasan
