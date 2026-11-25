# Spec Delta

Ticket padre: T-00-27. Change: T-00-27. HU: HU-00-06.

## MODIFIED Requirements

### Requirement: Worker de productos en sync

El crate `sync` MUST exponer una tarea en segundo plano (Index Sync Worker de Meilisearch) que:

1. Lee el outbox desde `sync_cursor:meilisearch` en lotes de a lo sumo 200 filas, ordenadas por id de evento (ULID), con `id` estrictamente mayor que el cursor actual (`ultimo_evento` vacío = desde el origen).
2. Se despierta cuando `OutboxNotifier` señala un *commit* de outbox, y MUST también reintentar la lectura al menos cada 1 segundo como respaldo (por si el waiter llegó tarde), salvo cuando aplica el *backoff* del requirement «Backoff exponencial en errores transitorios».
3. Para cada evento `entidad = producto`: relee el estado actual del producto (fuente de verdad). Si existe y `activo` es verdadero, MUST construir el documento [4.11.1](../../../../../docs/4-modelo.md#4111-documento) con `build_search_document` y hacer *upsert* en el índice inyectado. Si no existe (`not_found`) o `activo` es falso, MUST *delete* ese id del índice. MUST NOT usar el payload del evento como documento.
4. MUST esperar la *task* de Meilisearch y solo entonces avanzar `sync_cursor:meilisearch.ultimo_evento` al id de ese evento cuando la operación confirma. Si el *upsert*/*delete* (o el *fan-out*) falla con `unavailable` o `timeout`, MUST NOT avanzar el cursor de ese evento ni de los posteriores del lote. Si falla con `invalid_input`, MUST registrar `outbox_fallo` y avanzar según el requirement «Error permanente registra outbox_fallo y avanza».
5. MUST procesar los eventos del lote **en orden**. MAY agrupar escrituras al motor pero el cursor MUST quedar en el último evento confirmado o marcado `fallido`.

El worker MUST recibir por inyección el puerto `TextSearch`, los repositorios (`OutboxRepo`, `ProductRepo`, `AttributeDefRepo`), un `IndexUid` (los tests del crate `sync` usan un uid efímero distinto de `productos`) y un puerto de telemetría (requirement «Telemetría de sync»). MUST NOT instanciar el adaptador Meilisearch ni abrir SurrealDB: el composition root `vortex serve` (T-00-27) MUST construir el adaptador, abrir Storage y **inyectar** el uid `productos`. MUST implementar *fan-out* de `categoria`, `marca` y `atributo_def` según el requirement «Fan-out de categoría, marca y atributo_def». MUST implementar *backoff* y `outbox_fallo` según los requirements de T-03-09. MUST NOT aplicar `synonyms` (T-03-27). MUST NOT reprocesar a mano filas `fallido`.

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
- **THEN** reindexa los productos afectados según el requirement «Fan-out de categoría, marca y atributo_def» (T-03-08: este escenario deja de ser no-op)
- **AND** el cursor avanza cuando el *fan-out* confirma (el lote no se estanca)

#### Scenario: Evento imagen_producto no muta imagen (happy, MVP)

- **WHEN** el worker lee un evento `imagen_producto` (T-02-28 aún no existe)
- **THEN** no cambia el campo `imagen` del documento (sigue ausente/`null`)
- **AND** el cursor avanza
- **AND** el lote no se estanca

#### Scenario: Lote vacío no llama al motor (happy)

- **WHEN** no hay eventos con `id` mayor que el cursor
- **THEN** el ciclo termina sin error
- **AND** no se invoca `upsert_documents` ni `delete_documents`

#### Scenario: vortex serve inyecta el índice productos (happy)

- **WHEN** `vortex serve` arranca el Index Sync Worker
- **THEN** el `IndexUid` inyectado es `productos`
- **AND** el crate `sync` no abre SurrealDB ni instancia el cliente Meilisearch
