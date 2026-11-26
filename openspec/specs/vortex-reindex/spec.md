# vortex-reindex Specification

## Purpose

Reconstruye el índice Meilisearch `productos` desde las tablas maestras de SurrealDB con un índice temporal, intercambio atómico y cursor de outbox alineado, invocable con `vortex reindex` y el servidor detenido.

## Requirements

### Requirement: Reindexación total con intercambio atómico

El crate `sync` MUST exponer una función de biblioteca de reindexación Meilisearch que recibe el puerto `TextSearch`, `ProductRepo`, `AttributeDefRepo`, `OutboxRepo` y los uids `productos` (vivo) y `productos_nuevo` (temporal). MUST NOT instanciar el cliente Meilisearch ni abrir SurrealDB (eso es el composition root `vortex reindex`). MUST NOT recalcular vectores ni tocar `sync_cursor:vectores` (T-05-09). MUST NOT aplicar `synonyms` (T-03-27). `imagen` del documento MUST permanecer `null`.

Antes de escribir el índice temporal, MUST registrar el id del último evento de `outbox` existente (o ausencia de eventos). MUST crear el índice `productos_nuevo` (si ya existía de una corrida interrumpida, MUST borrarlo y crearlo de nuevo), MUST aplicar el perfil 4.11.2 (`productos_index_settings` con claves facetables activas) y MUST indexar **solo** productos `activo = true` leyendo las tablas maestras (`ProductRepo::list` + `get` + `build_search_document`), no el outbox. MUST asegurar que el uid vivo `productos` exista (vacío permitido) **antes** del intercambio, para que `swap_indexes` no dependa de un uid ausente.

Tras confirmar los documentos del índice temporal, MUST intercambiar `productos_nuevo` y `productos` de forma atómica (`swap_indexes`), MUST borrar el uid que quedó con los datos viejos (`productos_nuevo` post-swap) y MUST ubicar `sync_cursor:meilisearch` en el evento registrado. Si no había eventos de outbox, MUST dejar el cursor sin `ultimo_evento` (NONE). MUST NOT avanzar el cursor si el intercambio no confirmó.

Productos inactivos MUST NOT aparecer en el índice vivo después de un reindex exitoso (el índice nuevo se construye solo con activos). Una segunda invocación sobre la misma base MUST dejar el mismo conjunto de documentos activos (idempotencia observable).

#### Scenario: rebuild desde maestros y swap (happy)

- **WHEN** hay al menos un producto activo en SurrealDB y Meilisearch está disponible
- **AND** se ejecuta la reindexación
- **THEN** la operación termina sin error
- **AND** un `search` posterior en el uid `productos` incluye el `id` de ese producto
- **AND** el uid `productos_nuevo` ya no existe
- **AND** `sync_cursor:meilisearch.ultimo_evento` es el último id de `outbox` que existía al empezar (o NONE si no había eventos)

#### Scenario: inactivos no quedan en el índice nuevo (happy)

- **WHEN** hay un producto activo y otro inactivo
- **AND** se ejecuta la reindexación
- **THEN** un `search` posterior en `productos` incluye el activo
- **AND** no incluye el inactivo

#### Scenario: búsqueda en bucle durante el reindex (happy, Listo cuando)

- **WHEN** el índice vivo `productos` ya tiene al menos un documento de un producto activo
- **AND** un cliente busca en bucle ese índice (mismo `q` que lo encuentra) mientras corre la reindexación
- **THEN** ninguna búsqueda de ese bucle falla
- **AND** ninguna búsqueda de ese bucle devuelve `hits` vacío
- **AND** al terminar, ese producto sigue en `productos`

#### Scenario: segunda ejecución idempotente (happy)

- **WHEN** se reindexa dos veces seguidas sobre la misma base y el mismo Meilisearch
- **THEN** ambas ejecuciones terminan sin error
- **AND** el conjunto de `id` en `productos` coincide con los productos activos
- **AND** no hay hits duplicados para un mismo `id`

#### Scenario: catálogo vacío (happy)

- **WHEN** no hay productos activos
- **AND** se ejecuta la reindexación
- **THEN** la operación termina sin error
- **AND** un `search` posterior en `productos` no incluye hits de producto
- **AND** el uid `productos_nuevo` ya no existe

#### Scenario: Meilisearch inalcanzable no mueve el cursor (unhappy)

- **WHEN** el puerto `TextSearch` falla con `unavailable` al crear o escribir `productos_nuevo`
- **AND** `sync_cursor:meilisearch` tenía un valor (o NONE) conocido
- **THEN** la reindexación falla con `DomainError` `code` `unavailable`
- **AND** el cursor no cambia
- **AND** no se llama a `swap_indexes`

#### Scenario: documento inválido aborta antes del swap (unhappy)

- **WHEN** al construir un documento 4.11.1 un producto activo produce `invalid_input`
- **THEN** la reindexación falla con `DomainError` `code` `invalid_input`
- **AND** no se llama a `swap_indexes`
- **AND** `sync_cursor:meilisearch` no cambia
- **AND** el índice vivo `productos` conserva los documentos que tenía

### Requirement: vortex reindex abre la base en modo directo

`vortex reindex` MUST dejar de responder `no implementado`. MUST cargar `Config` con `--config` o `VORTEX_CONFIG` (mismo criterio que `vortex serve` y `vortex seed`). MUST abrir Storage en `<paths.datos>/db`, aplicar migraciones, conectar Meilisearch con `meilisearch.url` / `api_key` y ejecutar la reindexación total. MUST terminar con código 0 si la reindexación confirma. MUST NOT arrancar el servidor HTTP ni sidecars. MUST NOT usar el canal Unix socket (T-00-22 es v2). MUST NOT recalcular vectores.

Si faltan `--config` y `VORTEX_CONFIG`, MUST terminar con código distinto de 0 y MUST NOT escribir `no implementado`. Si el archivo de configuración no existe o es inválido, MUST terminar ≠ 0 y MUST NOT reindexar. Si no se puede abrir la base (lock de RocksDB porque `vortex serve` está corriendo, o ruta inaccesible), MUST terminar ≠ 0, MUST informar en stderr que no se pudo abrir la base, y MUST NOT escribir `no implementado`. Si Meilisearch no está `up` en la URL configurada, MUST terminar ≠ 0, MUST informar el fallo de Meilisearch en stderr, MUST NOT escribir `no implementado`, y MUST NOT avanzar el cursor.

#### Scenario: reindex sale 0 (happy)

- **WHEN** existe un TOML válido, `paths.datos` escribible, el servidor no está corriendo
- **AND** Meilisearch responde en la URL configurada
- **AND** hay al menos un producto activo
- **AND** se ejecuta `vortex --config <toml> reindex`
- **THEN** el proceso termina con código 0
- **AND** stderr no contiene `no implementado`
- **AND** un `search` posterior en el uid `productos` incluye ese producto

#### Scenario: reindex sin config (unhappy)

- **WHEN** se ejecuta `vortex reindex` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: reindex con config inexistente (unhappy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml reindex`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: reindex con base bloqueada (unhappy)

- **WHEN** SurrealDB en `<paths.datos>/db` ya está abierta por otro proceso (servidor en ejecución)
- **AND** se ejecuta `vortex --config <toml> reindex`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr indica que no se pudo abrir la base
- **AND** stderr no contiene `no implementado`

#### Scenario: reindex con Meilisearch caído (unhappy)

- **WHEN** el TOML apunta a una URL donde no hay Meilisearch escuchando
- **AND** la base se puede abrir
- **AND** se ejecuta `vortex --config <toml> reindex`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `reindex_rebuilds_productos_from_masters_and_swaps` | `sync` (`tests/meili_reindex.rs`) | integración `#[ignore = "meilisearch"]` happy |
| `reindex_omits_inactive_products` | `sync` | integración meilisearch happy |
| `reindex_search_loop_never_empty_or_error` | `sync` | integración meilisearch happy (Listo cuando) |
| `reindex_second_run_idempotent` | `sync` | integración meilisearch happy |
| `reindex_empty_catalog_succeeds` | `sync` | integración meilisearch happy |
| `reindex_unavailable_does_not_advance_cursor` | `sync` (`src/…` o tests unitarios con fake) | unitario unhappy |
| `reindex_invalid_document_aborts_before_swap` | `sync` | unitario unhappy |
| `vortex_reindex_exit_0` | `vortex` (`tests/reindex.rs`) | integración meilisearch happy |
| `vortex_reindex_without_config_is_error` | `vortex` (`tests/cli.rs`) | integración unhappy |
| `vortex_reindex_missing_config_file_is_not_unimplemented` | `vortex` (`tests/cli.rs`) | integración unhappy |
| `vortex_reindex_locked_db_is_error` | `vortex` (`tests/reindex.rs`) | integración unhappy |
| `vortex_reindex_meilisearch_down_is_error` | `vortex` (`tests/reindex.rs`) | integración unhappy |
