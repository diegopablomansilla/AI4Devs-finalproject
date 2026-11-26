# Spec Delta

## Purpose

Define el Index Sync Worker del crate `sync` que propaga eventos de `producto` (y consume `imagen_producto` sin mutar imagen en el MVP) desde el outbox de SurrealDB al índice Meilisearch, con lotes, despertar por notifier y poll de respaldo, avance de cursor tras confirmar la *task*, e idempotencia.

Ticket padre: T-03-07. HU: HU-03-01.

## ADDED Requirements

### Requirement: Worker de productos en sync

El crate `sync` MUST exponer una tarea en segundo plano (Index Sync Worker de Meilisearch) que:

1. Lee el outbox desde `sync_cursor:meilisearch` en lotes de a lo sumo 200 filas, ordenadas por id de evento (ULID), con `id` estrictamente mayor que el cursor actual (`ultimo_evento` vacío = desde el origen).
2. Se despierta cuando `OutboxNotifier` señala un *commit* de outbox, y MUST también reintentar la lectura al menos cada 1 segundo como respaldo (por si el waiter llegó tarde).
3. Para cada evento `entidad = producto`: relee el estado actual del producto (fuente de verdad). Si existe y `activo` es verdadero, MUST construir el documento [4.11.1](../../../../../docs/4-modelo.md#4111-documento) con `build_search_document` y hacer *upsert* en el índice inyectado. Si no existe (`not_found`) o `activo` es falso, MUST *delete* ese id del índice. MUST NOT usar el payload del evento como documento.
4. MUST esperar la *task* de Meilisearch y solo entonces avanzar `sync_cursor:meilisearch.ultimo_evento` al id de ese evento. Si el *upsert*/*delete* falla, MUST NOT avanzar el cursor de ese evento ni de los posteriores del lote.
5. MUST procesar los eventos del lote **en orden**. MAY agrupar escrituras al motor pero el cursor MUST quedar en el último evento cuya *task* confirmó.

El worker MUST recibir por inyección el puerto `TextSearch`, los repositorios (`OutboxRepo`, `ProductRepo`, `AttributeDefRepo`) y un `IndexUid` (los tests usan un uid efímero distinto de `productos`; el uid `productos` lo cablea T-00-27). MUST NOT instanciar el adaptador Meilisearch ni abrir SurrealDB. MUST NOT engancharse en `bin/vortex` (T-00-27). MUST NOT implementar *fan-out*, *backoff* ni `outbox_fallo` (T-03-08, T-03-09).

`imagen` del documento MUST permanecer `null` (T-02-28 / T-03-06).

#### Scenario: Alta de producto aparece en Meilisearch en menos de 1 s (happy, Listo cuando)

- **WHEN** un producto activo se persiste (emite outbox) y el worker procesa el outbox con Meilisearch disponible
- **THEN** un `search` posterior en el índice inyectado incluye un hit cuyo `id` es el del producto
- **AND** esa visibilidad ocurre en menos de 1 segundo desde el *commit*
- **AND** `sync_cursor:meilisearch.ultimo_evento` apunta al evento procesado

#### Scenario: Producto inactivo se elimina del índice (happy)

- **WHEN** un producto activo ya indexado pasa a `activo = false` (evento `producto`/`upsert`) y el worker procesa el evento
- **THEN** un `search` posterior no incluye ese `id` en `hits`
- **AND** el cursor avanza

#### Scenario: Producto ausente se elimina del índice (happy)

- **WHEN** el worker procesa un evento `producto` cuyo `registro` ya no existe (`not_found` al releer)
- **THEN** el worker pide *delete* de ese id
- **AND** el cursor avanza
- **AND** un `search` posterior no incluye ese `id`

#### Scenario: Evento de categoría, marca o atributo no hace fan-out (happy, non-goal T-03-08)

- **WHEN** el worker lee un evento `categoria`, `marca` o `atributo_def`
- **THEN** no reindexa productos afectados
- **AND** el cursor avanza (el lote no se estanca)

#### Scenario: Evento imagen_producto no muta imagen (happy, MVP)

- **WHEN** el worker lee un evento `imagen_producto` (T-02-28 aún no existe)
- **THEN** no cambia el campo `imagen` del documento (sigue ausente/`null`)
- **AND** el cursor avanza
- **AND** el lote no se estanca

#### Scenario: Lote vacío no llama al motor (happy)

- **WHEN** no hay eventos con `id` mayor que el cursor
- **THEN** el ciclo termina sin error
- **AND** no se invoca `upsert_documents` ni `delete_documents`

### Requirement: Idempotencia del worker

Procesar el mismo evento de producto dos veces MUST dejar el índice en el mismo estado que procesarlo una vez ([4.8.1](../../../../../docs/4-modelo.md#481-outbox): se relee la fuente de verdad). MUST NOT duplicar documentos. Un segundo pase con el cursor ya avanzado MUST ser un no-op (lote vacío).

#### Scenario: El mismo evento dos veces deja el índice igual (happy, Listo cuando)

- **WHEN** un evento de alta de producto se procesa y luego se vuelve a aplicar el mismo evento (o se resetea el cursor a antes de ese evento y se drena otra vez)
- **THEN** ambas ejecuciones terminan sin error
- **AND** el índice tiene exactamente un documento para ese producto, con el mismo contenido observable (`id` / `sku`)
- **AND** no aparecen hits duplicados

### Requirement: Cursor no avanza si Meilisearch falla (unhappy)

Si `upsert_documents` o `delete_documents` falla con `DomainError` (`unavailable`, `timeout` o `invalid_input`), el worker MUST dejar `sync_cursor:meilisearch.ultimo_evento` en el valor previo a ese evento. MUST NOT escribir `outbox_fallo` ni aplicar *backoff* exponencial (T-03-09). El siguiente ciclo (notifier o poll 1 s) MUST volver a leer ese evento.

#### Scenario: Motor inalcanzable no pierde el evento (unhappy)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `unavailable`
- **THEN** la operación de ese evento no confirma
- **AND** el cursor **no** avanza
- **AND** el evento sigue pendiente para el siguiente ciclo

#### Scenario: Error permanente no se registra aún (unhappy, non-goal T-03-09)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `invalid_input`
- **THEN** el cursor **no** avanza
- **AND** no se inserta fila en `outbox_fallo`

### Requirement: Tests del worker Meilisearch productos

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo | Motor |
|---|---|---|---|
| `worker_indexes_new_product_within_one_second` | `vortex-core/crates/sync/tests/` | happy, Listo cuando | `#[ignore = "meilisearch"]` (Surreal `test_db` + sidecar) |
| `worker_reprocessing_same_event_is_idempotent` | `vortex-core/crates/sync/tests/` | happy, Listo cuando | `#[ignore = "meilisearch"]` |
| `worker_deletes_inactive_product` | `vortex-core/crates/sync/` | happy | unidad (doble de `TextSearch`) |
| `worker_deletes_missing_product` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_skips_categoria_without_fanout` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_advances_cursor_on_imagen_producto_without_mutating_index` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_empty_outbox_does_not_call_search` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_does_not_advance_cursor_when_search_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_does_not_advance_cursor_on_invalid_input` | `vortex-core/crates/sync/` | unhappy | unidad |

Los tests `#[ignore = "meilisearch"]` MUST usar `MEILISEARCH_URL` (defecto `http://127.0.0.1:7700`) y un uid de índice de prueba distinto de `productos`. `cargo test --workspace` **sin** `--include-ignored` MUST terminar con código 0 aunque el sidecar no esté levantado. `cargo test --workspace -- --include-ignored meilisearch` MUST ejecutar los ignore de esta tabla **y** los ya existentes de `search` / T-09-05.

#### Scenario: Listo cuando contra el motor (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p sync -- --include-ignored meilisearch`
- **THEN** `worker_indexes_new_product_within_one_second` y `worker_reprocessing_same_event_is_idempotent` corren y terminan con código 0

#### Scenario: Unhappy de cursor sin sidecar (unhappy)

- **WHEN** se ejecuta `cargo test -p sync worker_does_not_advance_cursor` **sin** `--include-ignored`
- **THEN** los tests `worker_does_not_advance_cursor_when_search_unavailable` y `worker_does_not_advance_cursor_on_invalid_input` corren y terminan con código 0
