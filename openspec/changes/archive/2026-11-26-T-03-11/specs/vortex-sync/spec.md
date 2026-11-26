# Spec Delta

Ticket padre: T-03-11. HU: HU-03-02.

## ADDED Requirements

### Requirement: Orquestador de reindexación total en sync

El crate `sync` MUST exponer, además del Index Sync Worker incremental, una función de biblioteca que reconstruye el índice Meilisearch desde las tablas maestras según la capability `vortex-reindex` (snapshot de outbox, índice `productos_nuevo`, perfil 4.11.2, productos activos, `swap_indexes`, borrado del uid viejo, avance de `sync_cursor:meilisearch`). MUST recibir por inyección `TextSearch`, `ProductRepo`, `AttributeDefRepo` y `OutboxRepo`. MUST NOT instanciar el adaptador Meilisearch ni abrir SurrealDB: el composition root `vortex reindex` MUST construirlos e inyectar los uids `productos` y `productos_nuevo`. MUST NOT drenar el outbox evento a evento (eso sigue siendo el worker). MUST NOT recalcular vectores.

#### Scenario: sync expone reindexación total (happy)

- **WHEN** se invoca la reindexación total de `sync` con puertos inyectados y Meilisearch disponible
- **AND** hay al menos un producto activo
- **THEN** la operación termina sin error
- **AND** un `search` posterior en el uid vivo incluye ese producto
- **AND** el worker incremental no se arranca como parte de esa invocación

#### Scenario: reindex no consume el outbox como el worker (happy)

- **WHEN** hay eventos de outbox pendientes detrás del cursor
- **AND** se ejecuta la reindexación total
- **THEN** el índice vivo queda alineado con los productos activos de las tablas maestras
- **AND** `sync_cursor:meilisearch` queda en el último evento existente al snapshot (los pendientes no se procesan uno a uno)
