# vortex-sync Specification

## Purpose

Define el Index Sync Worker del crate `sync` que propaga eventos de `producto` y hace *fan-out* de `categoria`, `marca` y `atributo_def` desde el outbox de SurrealDB al índice Meilisearch, con lotes, despertar por notifier y poll de respaldo, avance de cursor tras confirmar la *task*, e idempotencia. Consume `imagen_producto` sin mutar imagen en el MVP.

Ticket padre: T-03-07, T-03-08. HU: HU-03-01, HU-02-01.

## Requirements

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

### Requirement: Idempotencia del worker

Procesar el mismo evento de producto dos veces MUST dejar el índice en el mismo estado que procesarlo una vez ([4.8.1](../../../../../docs/4-modelo.md#481-outbox): se relee la fuente de verdad). MUST NOT duplicar documentos. Un segundo pase con el cursor ya avanzado MUST ser un no-op (lote vacío).

#### Scenario: El mismo evento dos veces deja el índice igual (happy, Listo cuando)

- **WHEN** un evento de alta de producto se procesa y luego se vuelve a aplicar el mismo evento (o se resetea el cursor a antes de ese evento y se drena otra vez)
- **THEN** ambas ejecuciones terminan sin error
- **AND** el índice tiene exactamente un documento para ese producto, con el mismo contenido observable (`id` / `sku`)
- **AND** no aparecen hits duplicados

### Requirement: Cursor no avanza si Meilisearch falla (unhappy)

Si `upsert_documents`, `delete_documents` o `apply_settings` falla con `DomainError` `code` `unavailable` o `timeout`, el worker MUST dejar `sync_cursor:meilisearch.ultimo_evento` en el valor previo a ese evento. MUST NOT escribir `outbox_fallo`. El siguiente ciclo (tras *backoff*) MUST volver a leer ese evento. El caso `invalid_input` MUST seguir el requirement «Error permanente registra outbox_fallo y avanza», no este.

#### Scenario: Motor inalcanzable no pierde el evento (unhappy)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `unavailable`
- **THEN** la operación de ese evento no confirma
- **AND** el cursor **no** avanza
- **AND** el evento sigue pendiente para el siguiente ciclo

#### Scenario: Error permanente no se registra aún (unhappy, non-goal T-03-09)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `invalid_input`
- **THEN** se registra `outbox_fallo` con `estado` `fallido` y `consumidor` `meilisearch` (T-03-09: este escenario deja de ser non-goal)
- **AND** el cursor **avanza**
- **AND** el detalle vive en el requirement «Error permanente registra outbox_fallo y avanza»

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
| `worker_fanout_does_not_advance_cursor_when_search_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_fanout_does_not_advance_cursor_when_apply_settings_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |

Los tests `#[ignore = "meilisearch"]` MUST usar `MEILISEARCH_URL` (defecto `http://127.0.0.1:7700`) y un uid de índice de prueba distinto de `productos`. `cargo test --workspace` **sin** `--include-ignored` MUST terminar con código 0 aunque el sidecar no esté levantado. `cargo test --workspace -- --include-ignored meilisearch` MUST ejecutar los ignore de esta tabla **y** los ya existentes de `search` / T-09-05.

MUST NOT conservar `worker_does_not_advance_cursor_on_invalid_input` ni `worker_fanout_invalid_registro_does_not_advance` (reemplazados por los tests T-03-09). Los tests de reintentos/fallos/telemetría están en el requirement «Tests de reintentos, fallos y telemetría (T-03-09)».

#### Scenario: Listo cuando contra el motor (happy)

- **WHEN** Meilisearch de la versión fijada responde en la URL configurada
- **AND** se ejecuta `cargo test -p sync -- --include-ignored meilisearch`
- **THEN** `worker_indexes_new_product_within_one_second`, `worker_reprocessing_same_event_is_idempotent` y `worker_fanout_categoria_rename_updates_lvl0_within_one_second` corren y terminan con código 0

#### Scenario: Unhappy de cursor sin sidecar (unhappy)

- **WHEN** se ejecuta `cargo test -p sync worker_does_not_advance_cursor` **sin** `--include-ignored`
- **THEN** los tests `worker_does_not_advance_cursor_when_search_unavailable`, `worker_fanout_does_not_advance_cursor_when_search_unavailable` y `worker_fanout_does_not_advance_cursor_when_apply_settings_unavailable` corren y terminan con código 0
- **AND** `worker_does_not_advance_cursor_on_invalid_input` ya no existe (reemplazado por `worker_records_fallido_and_advances_on_invalid_input`)

### Requirement: Fan-out de categoría, marca y atributo_def

Cuando el worker procesa un evento cuyo `entidad` es `categoria`, `marca` o `atributo_def`, MUST reindexar los productos afectados releyendo la fuente de verdad (mismo *upsert*/*delete* que un evento `producto`). MUST NOT usar el payload del outbox como documento. MUST NOT aplicar `synonyms` ni procesar *fan-out* de `sinonimo` (T-03-27: el evento `sinonimo` MUST seguir avanzando el cursor sin llamar a `TextSearch`).

**`categoria`:** MUST listar productos **activos** de toda la rama del `registro` ([4.4.1](../../../../../docs/4-modelo.md#441-categoria--árbol): `categoria = $c OR categoria.ancestros CONTAINS $c`, vía `ProductRepo::list` con `FiltroProducto.categoria`). Para cada uno MUST construir el documento 4.11.1 y hacer *upsert* (o *delete* si al releer está inactivo o ausente). Una categoría sin productos (o `registro` que no matchea filas) MUST avanzar el cursor sin error.

**`marca`:** MUST listar productos **activos** con `FiltroProducto.marca` igual al `registro` y reindexarlos igual. Una marca sin productos MUST avanzar el cursor.

**`atributo_def`:** MUST, **antes** de reindexar:

1. Leer `AttributeDefRepo::list_all`.
2. Construir el perfil `productos` con las `clave` de las definiciones que tienen `facetable = true` y `activo = true`.
3. Llamar `apply_settings` en el índice inyectado (actualiza `filterableAttributes` con `attr_<clave>`). Si `apply_settings` falla con `unavailable` o `timeout`, MUST NOT reindexar ni avanzar el cursor.

Luego MUST reindexar los productos activos que **usan** esa definición (`atributos[*].def` = `registro`). En el MVP el worker MAY descubrirlos paginando `ProductRepo::list` (activos) y `get`. Cero productos MUST igual aplicar settings y avanzar el cursor.

El cursor MUST avanzar solo cuando **todas** las *tasks* de ese evento (settings + documentos) confirmaron, **salvo** `invalid_input` (requirement «Error permanente registra outbox_fallo y avanza»). Si un *upsert*/*delete* falla a mitad del *fan-out* con `unavailable` o `timeout`, MUST NOT avanzar el cursor. Un `registro` que no es ULID de 26 caracteres MUST tratarse como `invalid_input` permanente (fila `fallido` + avance).

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
- **THEN** se registra `outbox_fallo` con `estado` `fallido` y `consumidor` `meilisearch`
- **AND** el cursor **avanza**

### Requirement: Backoff exponencial en errores transitorios

Cuando `drain` de un lote falla con `DomainError` `code` `unavailable` o `timeout`, el loop del worker MUST esperar un *backoff* exponencial antes del siguiente intento: `min(30s, 1s × 2^n)` donde `n` es el número de fallos transitorios consecutivos ya observados (n = 0 → 1 s; tope 30 s). MUST NOT avanzar el cursor. MUST NOT escribir `outbox_fallo`. Durante el *backoff* MUST NOT acortar la espera por `OutboxNotifier` (evita martillar Meilisearch caído). Un `drain` que termina `Ok` (lote vacío o eventos confirmados / permanentes ya registrados) MUST reiniciar `n` a 0 y volver al poll de 1 s + notifier. Otros `DomainError` (p. ej. `not_found` de Storage) MUST NOT clasificarse como permanentes ni transitorios de Meilisearch: MUST NOT avanzar el cursor de ese evento y MUST NOT escribir `outbox_fallo`.

#### Scenario: Meilisearch caído no pierde eventos y al volver se aplican (happy, Listo cuando HU-03-01)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `unavailable`
- **AND** el cursor no avanza
- **AND** un `drain` posterior ve el mismo evento con `TextSearch` ya disponible
- **THEN** el documento queda visible en el índice
- **AND** el cursor avanza
- **AND** no se inserta fila en `outbox_fallo`

#### Scenario: Timeout no avanza el cursor (unhappy)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `timeout`
- **THEN** el cursor **no** avanza
- **AND** no se inserta fila en `outbox_fallo`
- **AND** el evento sigue pendiente

#### Scenario: Backoff se acota a 30 s (happy)

- **WHEN** el worker acumula fallos transitorios consecutivos suficientes para superar 30 s en la fórmula
- **THEN** la espera entre intentos es 30 segundos
- **AND** no supera 30 segundos

#### Scenario: Lote vacío no entra en backoff (happy)

- **WHEN** no hay eventos posteriores al cursor
- **THEN** el ciclo termina `Ok`
- **AND** el intervalo de poll sigue siendo 1 s (no *backoff*)

### Requirement: Error permanente registra outbox_fallo y avanza

Cuando el procesamiento de un evento (producto o *fan-out*) falla con `DomainError` `code` `invalid_input`, el worker MUST:

1. Escribir (o actualizar) una fila en `outbox_fallo` con `consumidor` `meilisearch`, `estado` `fallido`, `ultimo_error` no vacío y `evento` = id de la fila de outbox.
2. Avanzar `sync_cursor:meilisearch` a ese evento.
3. Continuar con el siguiente evento del lote (MUST NOT abortar el lote ni aplicar *backoff*).
4. Incrementar la telemetría `outbox_failed_total`.

MUST NOT dejar el cursor detrás de un `invalid_input` ya registrado. Un segundo `invalid_input` del mismo `(evento, consumidor)` MUST ser idempotente (índice único; no duplica filas). MUST NOT implementar reproceso manual de fallidos.

#### Scenario: Error permanente de HU-03-01 (unhappy, Listo cuando)

- **WHEN** el worker procesa un evento `producto` y `TextSearch` falla con `code` `invalid_input`
- **THEN** existe una fila `outbox_fallo` con `estado` `fallido` y `consumidor` `meilisearch` para ese evento
- **AND** el cursor **avanza**
- **AND** `outbox_failed_total` se incrementa
- **AND** el evento **no** queda pendiente para el siguiente ciclo

#### Scenario: registro inválido en fan-out se marca fallido (unhappy)

- **WHEN** el worker procesa un evento `categoria`, `marca` o `atributo_def` cuyo `registro` no es un ULID de 26 caracteres
- **THEN** se registra `outbox_fallo` `fallido`
- **AND** el cursor avanza
- **AND** el lote no se estanca

#### Scenario: InvalidInput no aborta el resto del lote (happy)

- **WHEN** un lote tiene un evento que falla con `invalid_input` y un evento posterior procesable
- **THEN** el primero queda `fallido` y su cursor avanza
- **AND** el segundo se procesa en el mismo `drain`

### Requirement: Telemetría de sync

El worker MUST publicar, vía un puerto de telemetría inyectado (el crate `sync` MUST NOT depender de `api` ni del recorder Prometheus):

| Nombre | Tipo | Labels | Cuándo |
|---|---|---|---|
| `outbox_lag` | gauge | `consumidor` = `meilisearch` | Tras cada `drain` (éxito o error): eventos de `outbox` con id > cursor `meilisearch` |
| `outbox_failed_total` | counter | — | Al registrar un `fallido` |
| `sync_propagation_seconds` | histogram | — | Al confirmar un evento (avance de cursor por éxito, no por `fallido`): segundos entre `creado_en` del outbox y la confirmación |

`outbox_lag` MUST NOT usar el consumidor `vectores` (T-05-08). Un doble de prueba MUST registrar las llamadas; la implementación Prometheus vive en `api` (`prometheus-metrics`).

#### Scenario: Lag se publica tras un drain (happy)

- **WHEN** hay eventos pendientes detrás del cursor `meilisearch` y el worker ejecuta un `drain`
- **THEN** la telemetría recibe `outbox_lag` con label `consumidor` `meilisearch` igual a esa cantidad

#### Scenario: Propagación se observa en éxito (happy)

- **WHEN** el worker confirma un evento `producto` con Meilisearch disponible
- **THEN** observa `sync_propagation_seconds` ≥ 0
- **AND** no incrementa `outbox_failed_total`

#### Scenario: Lag no incluye vectores (happy)

- **WHEN** el worker publica `outbox_lag`
- **THEN** la label `consumidor` es `meilisearch`
- **AND** no publica lag del cursor `vectores`

### Requirement: Tests de reintentos, fallos y telemetría (T-03-09)

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Tipo | Motor |
|---|---|---|---|
| `worker_retries_after_unavailable_and_does_not_lose_events` | `vortex-core/crates/sync/` | happy, Listo cuando HU-03-01 | unidad (`TextSearch` falso: falla → recupera) |
| `worker_timeout_does_not_advance_cursor` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_backoff_caps_at_thirty_seconds` | `vortex-core/crates/sync/` | happy | unidad (función de delay, sin sleep real de 30 s) |
| `worker_empty_batch_does_not_backoff` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_records_fallido_and_advances_on_invalid_input` | `vortex-core/crates/sync/` | unhappy, Listo cuando HU-03-01 | unidad |
| `worker_fanout_invalid_registro_records_fallido_and_advances` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_invalid_input_continues_remaining_batch` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_does_not_write_fallo_on_unavailable` | `vortex-core/crates/sync/` | unhappy | unidad |
| `worker_publishes_outbox_lag_after_drain` | `vortex-core/crates/sync/` | happy | unidad |
| `worker_observes_propagation_on_success` | `vortex-core/crates/sync/` | happy | unidad |

MUST NOT conservar `worker_does_not_advance_cursor_on_invalid_input` ni `worker_fanout_invalid_registro_does_not_advance` (reemplazados). `cargo test --workspace` **sin** `--include-ignored` MUST ejecutar los de esta tabla y terminar 0.

#### Scenario: Listo cuando HU-03-01 (happy + unhappy)

- **WHEN** se ejecuta `cargo test -p sync worker_retries_after_unavailable worker_records_fallido`
- **THEN** `worker_retries_after_unavailable_and_does_not_lose_events` y `worker_records_fallido_and_advances_on_invalid_input` corren y terminan con código 0
