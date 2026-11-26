# Design

## Context

En `develop`: `OutboxEvent` + `Tx` + `OutboxNotifier` (T-03-02); tablas 4.8 y seed de `sync_cursor:meilisearch` (T-03-01); `ProductRepo` / `AttributeDefRepo` en `storage`; `build_search_document` y `SearchDocument` `Serialize` (T-03-06); `MeiliTextSearch` espera *tasks* pero `upsert` manda solo `{ id }` (T-03-04). El crate `sync` es placeholder sin deps; `check-workspace.sh` prohíbe `[dependencies]` ahí. [3.2](../../../docs/3-componentes.md#32-mapa-de-crates-y-regla-de-dependencias): `SYNC → DOM`, `SYNC → STO`; los servicios **no** instancian adaptadores. T-00-27 cablea el worker en el arranque. Ver `proposal.md` para motivación.

Constraints: sin fan-out (T-03-08), sin backoff/`outbox_fallo` (T-03-09), sin `bin/vortex`, MVP del curso, ~2 h. CLI OpenSpec kebab-case (`t-03-07` → carpeta `T-03-07`).

## Goals / Non-Goals

**Goals:**

- Puerto `OutboxRepo` + adaptador SurrealDB
- Worker genérico en `sync` (notify + poll 1 s, lote 200, upsert/delete, cursor tras *task*)
- `upsert_documents` con JSON 4.11.1
- Allowlist de `sync`
- Tests: Listo cuando (ignore Meilisearch) + unidad happy/unhappy sin sidecar

**Non-Goals (diseño):**

- `dyn TextSearch` / `async-trait` (el puerto es RPITIT; el worker es genérico)
- Instanciar `MeiliTextSearch` o `Db::open` fuera de tests / composition root
- Escribir `outbox_fallo` o métricas
- Resolver `imagen_producto` → producto (no hay `ImageRepo`)
- Cambiar el probe T-09-05 ni CI YAML

## Decisions

### 1. Worker en `sync`, genérico, inyectado

**Elección:** `sync::MeiliProductWorker<R, S>` (nombre tentativo) con `R: OutboxRepo + ProductRepo + AttributeDefRepo` y `S: TextSearch`. Constructor: repos, search, `OutboxNotifier`, `IndexUid`, `Duration` de poll (defecto 1 s). Método `run` (loop) y `drain_once` (un lote; tests). `sync` depende de `domain` + `storage` + `tokio`; **no** de `meilisearch-sdk`.

**Por qué:** [3.4.10](../../../docs/3-componentes.md#3410-index-sync-worker) y diagrama 3.2. T-00-27 es el composition root.

**Alternativa:** worker en `search`. Se descartó: mezclaría HTTP del motor con SurrealDB. **Alternativa:** lógica en `storage`. Se descartó: storage no debe conocer `TextSearch`.

### 2. `OutboxRepo` en `domain`; adaptador en `storage`

**Elección:** `OutboxRow { id: Id<OutboxRecord>, event: OutboxEvent }` + trait `OutboxRepo` (`list_after`, `get_cursor`, `advance_cursor`) acotado al consumidor `meilisearch` en este ticket. Adaptador `storage::OutboxStore` (o `impl OutboxRepo for` un newtype sobre `Arc<Db>`). SQL alineado a 4.8.1: `id > $cursor ORDER BY id LIMIT 200`. `registro` se recorta de `tabla:ulid` al ULID. Cursor: `UPDATE sync_cursor:meilisearch SET ultimo_evento = type::thing('outbox', $id)`.

**Por qué:** T-03-02 difería el puerto a T-03-07; 4.12 nombra `OutboxRepo`. `domain` sigue sin SurrealDB.

**Alternativa:** queries sueltas en `sync`. Se descartó: rompe la regla de puertos y acopla `sync` al dialecto SurrealQL.

### 3. Un evento → *task* → cursor (no el lote entero)

**Elección:** para cada fila, aplicar Meilisearch, `wait` de *task*, **luego** `advance_cursor`. Si falla, se corta el lote; los anteriores quedan confirmados.

**Por qué:** un fallo a mitad de lote no re-aplica eventos ya indexados ni pierde los pendientes. El Listo cuando de idempotencia sigue valiendo.

**Alternativa:** confirmar las 200 juntas. Se descartó: un error en la #50 re-procesaría 1–49.

### 4. Entidades no-producto: avanzar sin mutar el índice

**Elección:** `categoria` / `marca` / `atributo_def` / `sinonimo` / `nota_tecnica` → no llamar a `TextSearch`; avanzar cursor. `imagen_producto` igual (no hay tabla/repo; `imagen` sigue `null`).

**Por qué:** si no se avanza, el worker se estanca hasta T-03-08. El ticket pide procesar `imagen_producto`; en MVP eso es **no bloquear**. T-03-08 reemplaza el no-op de categoría/marca/atributo.

**Alternativa:** dejar esos eventos pendientes. Se descartó: el cursor es único; bloquearía productos.

### 5. Errores de `TextSearch`: no avanzar, sin backoff

**Elección:** `Unavailable` / `Timeout` / `InvalidInput` → no cursor, `drain_once` retorna el error; el loop espera notify o 1 s y reintenta el mismo evento. Sin `outbox_fallo`.

**Por qué:** **No hace** = T-03-09. El poll 1 s evita un busy-loop. Un `InvalidInput` puede atascar el cursor hasta T-03-09; se documenta en Risks.

**Alternativa:** saltar `InvalidInput` ya. Se descartó: perdería el evento y T-03-09 no tendría nada que registrar.

### 6. Releer ficha + `AttributeDefRepo::list_all`

**Elección:** `ProductRepo::get` → `FichaProducto` (producto, marca, categoría). `AttributeDefRepo::list_all` una vez por lote (o por evento) para `build_search_document`. `NotFound` o `activo == false` → `delete_documents`.

**Por qué:** el builder exige `&[AtributoDef]` (flags `buscable`/`facetable`); `AtributoFicha` no las trae. 4.8.1: no usar el payload del outbox.

**Alternativa:** `get` por cada `def` del producto. Se descartó: más round-trips; el catálogo de defs es chico en el MVP.

### 7. Serializar `SearchDocument` en el adaptador

**Elección:** `MeiliTextSearch::upsert_documents` manda `&[SearchDocument]` (ya `Serialize`) con primary key `"id"`. Se retira el DTO `{ id }`. Los hits de `search` pueden seguir proyectando solo `id`.

**Por qué:** T-03-06 lo dejó para este ticket. Evita un segundo DTO.

**Alternativa:** `serde_json::Value` a mano. Se descartó: el tipo de dominio ya es el wire 4.11.1.

### 8. Allowlist de `sync` y tests ignore

**Elección:** `[dependencies]`: `domain`, `storage`, `tokio`. `[dev-dependencies]`: `search`, `tokio`. Fixtures en `scripts/fixtures/workspace/`: happy allowlist + unhappy `meilisearch-sdk`/`surrealdb` directos. Integración Listo cuando en `crates/sync/tests/` con `#[ignore = "meilisearch"]`, `test_db()` + `MeiliTextSearch`, índice `t0307-<ulid>`.

**Por qué:** 3.2 permite `SYNC → STO`; el sidecar se inyecta solo en tests. CI `meilisearch` de T-09-05 ya corre `--include-ignored meilisearch`.

**Alternativa:** tests de integración en `search`. Se descartó: el worker es de `sync`.

### 9. Loop notify + `tokio::time::timeout` 1 s

**Elección:** `tokio::select!` entre `notifier.notified()` y `sleep(1s)`, luego `drain_once`. Tests de unidad llaman `drain_once` sin el loop.

**Por qué:** el ticket nombra ambos despertadores. T-03-02 ya avisó que `Notify` no guarda tokens.

## Risks / Trade-offs

- [`InvalidInput` atasca el cursor hasta T-03-09] → Mitigación: escenario unhappy explícito; no se “traga” el error.
- [201 inserts para el test de LIMIT 200] → Mitigación: `test_db()` en memoria; si es lento, emitir 201 `CREATE outbox` en un script.
- [`list_all` de defs en cada evento] → Mitigación: cachear en el lote en apply si hace falta; no es parte de la spec.
- [GET documento Meilisearch para el test 4.11.1] → Mitigación: `client.get_document` vía SDK en el test de `search`, o `search` con `q = sku`.
- [Ámbito ~2 h] → Mitigación: fan-out / retries / wiring ya son otros tickets; no se divide T-03-07.

## Migration Plan

Ninguna migración `.surql` nueva (`0001` ya tiene cursores). Rollback: no arrancar el worker (T-00-27 aún no lo cablea). El índice de tests se borra en el teardown. Allowlist: solo repo/CI.

## Open Questions

Ninguna que altere la spec. El nombre exacto del struct del worker se fija en apply.
