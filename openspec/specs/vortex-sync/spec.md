# vortex-sync Specification

## Purpose

Define el Index Sync Worker del crate `sync` que propaga eventos de `producto` y hace *fan-out* de `categoria`, `marca` y `atributo_def` desde el outbox de SurrealDB al índice Meilisearch, con lotes, despertar por notifier y poll de respaldo, avance de cursor tras confirmar la *task*, e idempotencia. Consume `imagen_producto` sin mutar imagen en el MVP.

Ticket padre: T-03-07, T-03-08. HU: HU-03-01, HU-02-01.

## Requirements

### Requirement: Worker de productos en sync

El crate `sync` MUST exponer una tarea en segundo plano (Index Sync Worker de Meilisearch) que:

1. Lee el outbox desde `sync_cursor:meilisearch` en lotes de a lo sumo 200 filas, ordenadas por id de evento (ULID), con `id` estrictamente mayor que el cursor actual (`ultimo_evento` vacío = desde el origen).
2. Se despierta cuando `OutboxNotifier` señala un *commit* de outbox, y MUST también reintentar la lectura al menos cada 1 segundo como respaldo (por si el waiter llegó tarde).
3. Para cada evento `entidad = producto`: relee el estado actual del producto (fuente de verdad). Si existe y `activo` es verdadero, MUST construir el documento [4.11.1](../../../../../docs/4-modelo.md#4111-documento) con `build_search_document` y hacer *upsert* en el índice inyectado. Si no existe (`not_found`) o `activo` es falso, MUST *delete* ese id del índice. MUST NOT usar el payload del evento como documento.
4. MUST esperar la *task* de Meilisearch y solo entonces avanzar `sync_cursor:meilisearch.ultimo_evento` al id de ese evento. Si el *upsert*/*delete* (o el *fan-out* del evento) falla, MUST NOT avanzar el cursor de ese evento ni de los posteriores del lote.
5. MUST procesar los eventos del lote **en orden**. MAY agrupar escrituras al motor pero el cursor MUST quedar en el último evento cuya *task* confirmó.

El worker MUST recibir por inyección el puerto `TextSearch`, los repositorios (`OutboxRepo`, `ProductRepo`, `AttributeDefRepo`) y un `IndexUid` (los tests usan un uid efímero distinto de `productos`; el uid `productos` lo cablea T-00-27). MUST NOT instanciar el adaptador Meilisearch ni abrir SurrealDB. MUST NOT engancharse en `bin/vortex` (T-00-27). MUST implementar *fan-out* de `categoria`, `marca` y `atributo_def` según el requirement «Fan-out de categoría, marca y atributo_def». MUST NOT implementar *backoff* ni `outbox_fallo` (T-03-09). MUST NOT aplicar `synonyms` (T-03-27).

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
| `worker_indexes_new_product_within_one_second` | `vortex-core/crates/sync/tests/` | happy, Listo cuando T-03-07 | `#[ignore = "meilisearch"]` (Surreal `test_db` + sidecar) |
| `worker_reprocessing_same_event_is_idempotent` | `vortex-core/crates/sync/tests/` | happy, Listo cuando T-03-07 | `#[ignore = "meilisearch"]` |
| `worker_fanout_categoria_rename_updates_lvl0_within_one_second` | `vortex-core/crates/sync/tests/` | happy, Listo cuando T-03-08 | `#[ignore = "meilisearch"]` |
| `worker_deletes_inactive_product` | `vortex-core/crates/sync/` | happy | unidad (doble de `TextSearch`) |
| `worker_deletes_missing_product` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_fanout_categoria_reindexes_full_branch` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_fanout_marca_reindexes_products` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_fanout_atributo_def_reindexes_users_and_applies_settings` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_fanout_empty_affected_advances_cursor` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_sinonimo_still_skips_without_mutating_index` | `vortex-core/crates/sync/` | happy, non-goal T-03-27 | unidad |
| `worker_advances_cursor_on_imagen_producto_without_mutating_index` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_empty_outbox_does_not_call_search` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_does_not_advance_cursor_when_search_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_does_not_advance_cursor_on_invalid_input` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_fanout_does_not_advance_cursor_when_search_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_fanout_does_not_advance_cursor_when_apply_settings_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_fanout_invalid_registro_does_not_advance` | `vortex-core/crates/sync/` | unhappy | unidad |

Los tests `#[ignore = "meilisearch"]` MUST usar `MEILISEARCH_URL` (defecto `http://127.0.0.1:7700`) y un uid de índice de prueba distinto de `productos`. `cargo test --workspace` **sin** `--include-ignored` MUST terminar con código 0 aunque el sidecar no esté levantado. `cargo test --workspace -- --include-ignored meilisearch` MUST ejecutar los ignore de esta tabla **y** los ya existentes de `search` / T-09-05.

MUST NOT conservar `worker_skips_categoria_without_fanout` (reemplazado por los tests `worker_fanout_*`).

#### Scenario: Listo cuando contra el motor (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p sync -- --include-ignored meilisearch`
- **THEN** `worker_indexes_new_product_within_one_second`, `worker_reprocessing_same_event_is_idempotent` y `worker_fanout_categoria_rename_updates_lvl0_within_one_second` corren y terminan con código 0

#### Scenario: Unhappy de cursor sin sidecar (unhappy)

- **WHEN** se ejecuta `cargo test -p sync worker_does_not_advance_cursor` **sin** `--include-ignored`
- **THEN** los tests `worker_does_not_advance_cursor_when_search_unavailable`, `worker_does_not_advance_cursor_on_invalid_input`, `worker_fanout_does_not_advance_cursor_when_search_unavailable` y `worker_fanout_does_not_advance_cursor_when_apply_settings_unavailable` corren y terminan con código 0

### Requirement: Fan-out de categoría, marca y atributo_def

Cuando el worker procesa un evento cuyo `entidad` es `categoria`, `marca` o `atributo_def`, MUST reindexar los productos afectados releyendo la fuente de verdad (mismo *upsert*/*delete* que un evento `producto`). MUST NOT usar el payload del outbox como documento. MUST NOT aplicar `synonyms` ni procesar *fan-out* de `sinonimo` (T-03-27: el evento `sinonimo` MUST seguir avanzando el cursor sin llamar a `TextSearch`).

**`categoria`:** MUST listar productos **activos** de toda la rama del `registro` ([4.4.1](../../../../../docs/4-modelo.md#441-categoria--árbol): `categoria = $c OR categoria.ancestros CONTAINS $c`, vía `ProductRepo::list` con `FiltroProducto.categoria`). Para cada uno MUST construir el documento 4.11.1 y hacer *upsert* (o *delete* si al releer está inactivo o ausente). Una categoría sin productos (o `registro` que no matchea filas) MUST avanzar el cursor sin error.

**`marca`:** MUST listar productos **activos** con `FiltroProducto.marca` igual al `registro` y reindexarlos igual. Una marca sin productos MUST avanzar el cursor.

**`atributo_def`:** MUST, **antes** de reindexar:

1. Leer `AttributeDefRepo::list_all`.
2. Construir el perfil `productos` con las `clave` de las definiciones que tienen `facetable = true` y `activo = true`.
3. Llamar `apply_settings` en el índice inyectado (actualiza `filterableAttributes` con `attr_<clave>`). Si `apply_settings` falla, MUST NOT reindexar ni avanzar el cursor.

Luego MUST reindexar los productos activos que **usan** esa definición (`atributos[*].def` = `registro`). En el MVP el worker MAY descubrirlos paginando `ProductRepo::list` (activos) y `get`. Cero productos MUST igual aplicar settings y avanzar el cursor.

El cursor MUST avanzar solo cuando **todas** las *tasks* de ese evento (settings + documentos) confirmaron. Si un *upsert*/*delete* falla a mitad del *fan-out*, MUST NOT avanzar el cursor. Un `registro` que no es ULID de 26 caracteres MUST fallar con `DomainError` `code` `invalid_input` y MUST NOT avanzar el cursor.

El *fan-out* MUST ser idempotente: reprocesar el mismo evento deja el índice y los settings en el mismo estado observable.

#### Scenario: Renombrar Fijaciones actualiza categoria.lvl0 en menos de 1 s (happy, Listo cuando HU-02-01)

- **WHEN** existen las categorías "Fijaciones" (raíz) y "Pernos" (hija) y un producto activo en "Pernos" ya indexado
- **AND** se persiste el rename de "Fijaciones" a "Tornillería" (outbox `categoria` por la rama) y el worker drena el outbox con Meilisearch disponible
- **THEN** el documento de ese producto en el índice tiene `categoria.lvl0` igual a `"Tornillería"`
- **AND** esa visibilidad ocurre en menos de 1 segundo desde el *commit*
- **AND** el cursor avanza

#### Scenario: Fan-out de categoría reindexa toda la rama (happy)

- **WHEN** el worker procesa un evento `categoria`/`upsert` cuyo `registro` es una categoría con productos en el nodo y en una descendiente
- **THEN** pide *upsert* de los documentos de esos productos activos (nodo y descendientes)
- **AND** el cursor avanza

#### Scenario: Fan-out de marca reindexa sus productos (happy)

- **WHEN** el worker procesa un evento `marca`/`upsert` y existen productos activos con esa marca
- **THEN** pide *upsert* de esos productos
- **AND** productos de otra marca no se reindexan por ese evento
- **AND** el cursor avanza

#### Scenario: Fan-out de atributo_def aplica filterableAttributes y reindexa usuarios (happy)

- **WHEN** el worker procesa un evento `atributo_def`/`upsert` de una definición `facetable` activa con clave `medida` usada por un producto activo
- **THEN** llama `apply_settings` con un perfil `productos` cuyo `filterableAttributes` incluye `attr_medida`
- **AND** pide *upsert* de ese producto
- **AND** el cursor avanza

#### Scenario: Evento sin productos afectados avanza el cursor (happy)

- **WHEN** el worker procesa un evento `categoria` o `marca` cuyo `registro` no tiene productos activos
- **THEN** no falla
- **AND** el cursor avanza

#### Scenario: sinonimo no hace fan-out (happy, non-goal T-03-27)

- **WHEN** el worker lee un evento `sinonimo`
- **THEN** no llama a `upsert_documents`, `delete_documents` ni `apply_settings`
- **AND** el cursor avanza

#### Scenario: Motor inalcanzable en fan-out no pierde el evento (unhappy)

- **WHEN** el worker procesa un evento `categoria` y `TextSearch` falla con `code` `unavailable` al *upsert*
- **THEN** la operación de ese evento no confirma
- **AND** el cursor **no** avanza
- **AND** el evento sigue pendiente para el siguiente ciclo

#### Scenario: apply_settings inalcanzable no avanza el cursor (unhappy)

- **WHEN** el worker procesa un evento `atributo_def` y `apply_settings` falla con `code` `unavailable`
- **THEN** no reindexa productos de ese evento
- **AND** el cursor **no** avanza

#### Scenario: registro inválido en fan-out (unhappy)

- **WHEN** el worker procesa un evento `categoria`, `marca` o `atributo_def` cuyo `registro` no es un ULID de 26 caracteres
- **THEN** la operación falla
- **AND** el error es `DomainError` con `code` `invalid_input`
- **AND** el cursor **no** avanza
