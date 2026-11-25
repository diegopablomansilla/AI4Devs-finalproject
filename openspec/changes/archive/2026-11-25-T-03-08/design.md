# Design

## Context

En `develop` (tras T-03-07): `MeiliProductWorker` drena el outbox, sincroniza `producto` (relee ficha + `build_search_document`) y **avanza el cursor sin reindexar** en `categoria` / `marca` / `atributo_def` (`worker_skips_categoria_without_fanout`). `ProductRepo::list` ya filtra rama completa y marca. `productos_index_settings` vive en `search`; `sync` no puede depender de `search` en `[dependencies]` (allowlist). Storage ya emite un outbox `categoria` por cada nodo de la rama al renombrar/mover ([4.4.1](../../../docs/4-modelo.md#441-categoria--árbol)). Ver `proposal.md` para motivación.

Constraints: sinónimos = T-03-27; backoff/`outbox_fallo` = T-03-09; sin `bin/vortex`; MVP del curso; ~2 h. CLI OpenSpec kebab-case (`t-03-08` → carpeta `T-03-08`).

## Goals / Non-Goals

**Goals:**

- *Fan-out* en el worker existente (mismos genéricos e inyección)
- `productos_index_settings` en `domain` + reexport en `search`
- Tests: Listo cuando rename Fijaciones (ignore Meilisearch) + unidad happy/unhappy sin sidecar

**Non-Goals (diseño):**

- Nuevo método en `ProductRepo` para listar por `atributo_def`
- Inyectar `CategoryRepo` / `BrandRepo` (el `registro` del evento *es* el id de filtro)
- `dyn TextSearch` / `async-trait`
- Wiring, métricas, `outbox_fallo`, `synonyms`

## Decisions

### 1. Extender `MeiliProductWorker`, no un segundo worker

**Elección:** el mismo `drain_once` despacha `categoria` / `marca` / `atributo_def` a *fan-out*. `producto` e `imagen_producto` no cambian.

**Por qué:** un solo cursor `sync_cursor:meilisearch`; 3.4.10 es un Index Sync Worker. Un segundo loop duplicaría notify/poll.

**Alternativa:** worker aparte por entidad. Se descartó: dos cursores o contención del mismo.

### 2. Rama y marca vía `ProductRepo::list`; atributo vía list+get

**Elección:**

- `categoria`: `FiltroProducto { categoria: Some(registro), marca: None, activo: true }` — el adaptador ya aplica `categoria = $c OR categoria.ancestros CONTAINS $c`.
- `marca`: `FiltroProducto { marca: Some(registro), …, activo: true }`.
- `atributo_def`: paginar `list` de activos y `get` para ver si `atributos[*].def` coincide. Página `take = 200` (mismo tope que el outbox).

**Por qué:** no se toca el puerto ni los dobles de `catalog`/`api` (caber en ~2 h). El catálogo piloto es chico; T-03-11 cubre reconstrucción masiva.

**Alternativa:** `list_ids_using_atributo` en `ProductRepo`. Se pospuso: 6 `impl` + delta `domain-producto` / `vortex-storage` sin ganar el Listo cuando (categoría).

### 3. `atributo_def`: siempre `apply_settings` con el snapshot actual

**Elección:** no comparar “si cambió `facetable`”. Tras el evento se arma el perfil con las defs **activas y facetables** de `list_all` y se llama `apply_settings`, **luego** se reindexan los usuarios. El outbox no trae el valor anterior (4.8.1: se relee la fuente de verdad).

**Por qué:** idempotente; un no-op de settings (mismas listas) es barato. Si `facetable` pasó a false, el perfil deja de incluir `attr_<clave>` y el *upsert* deja de emitir ese campo.

**Alternativa:** `motivo` en el outbox. Se descartó: storage de `atributo_def` no lo setea; acoplaría el worker al payload.

### 4. Un evento de fan-out → todas las tasks → un avance de cursor

**Elección:** para un evento, completar settings (si aplica) y todos los *upsert*/*delete* de afectados; **después** `advance_meilisearch_cursor`. Si falla a mitad, el cursor no avanza; el reintento vuelve a aplicar (idempotente). MAY mandar varios documentos en un solo `upsert_documents`.

**Por qué:** coherente con T-03-07 (cursor = último evento confirmado). Un avance prematuro dejaría productos de la rama con `lvl0` viejo.

**Alternativa:** un cursor por producto afectado. Se descartó: no hay filas de outbox por producto en un rename.

### 5. Mover `productos_index_settings` a `domain`

**Elección:** el constructor puro pasa a `domain`; `search` reexporta. El worker llama `domain::productos_index_settings`. Allowlist de `sync` intacta.

**Por qué:** 3.2 `SYNC → DOM`, no `SYNC → SEARCH`. Las listas 4.11.2 son conocimiento del documento de dominio.

**Alternativa:** `search` en `[dependencies]` de `sync`. Se descartó: `cargo-workspace` y T-03-07 lo prohíben.

### 6. Listo cuando: Surreal + Meilisearch + lectura del documento

**Elección:** test ignore `worker_fanout_categoria_rename_updates_lvl0_within_one_second`: sembrar Fijaciones → Pernos + producto, indexar, persistir rename (o `UPDATE` de `ruta`/`nombre` + outbox `categoria`), `drain_once`, leer `categoria.lvl0` del documento. `TextSearch::search` solo devuelve `id`; el test usa un helper público del adaptador (`MeiliTextSearch::fetch_document` o GET JSON vía el `Client` interno) **sin** ampliar el puerto. Índice `t0308-<ulid>`.

**Por qué:** el ticket nombra `categoria.lvl0`, no solo “el hit existe”.

**Alternativa:** filtrar `search` por faceta. Se descartó: T-03-12 aún no manda filtros.

### 7. Dobles de unidad: `list`/`get` reales, `RecordingSearch` con settings

**Elección:** extender `FakeProducts` para `list` (filtro rama/marca) y un mapa de fichas; `RecordingSearch` registra `apply_settings` y documentos (para `lvl0` / `attr_*`). Reemplazar `worker_skips_categoria_without_fanout`.

**Por qué:** el Listo cuando de HU-02-01 se cubre con sidecar; rama/marca/atributo y unhappy no deben exigir Meilisearch.

## Risks / Trade-offs

- [Scan O(n) de productos en `atributo_def`] → Mitigación: MVP chico; documentado; no se abre un ticket `-01`.
- [Rename emite N outbox de categoría (nodo + descendientes) → N fan-outs de rama] → Mitigación: *upsert* idempotente; latencia < 1 s en el dataset de prueba (2 nodos).
- [`InvalidInput` en un producto del fan-out atasca el cursor hasta T-03-09] → Mitigación: mismo contrato que T-03-07; escenario unhappy explícito.
- [GET documento no está en `TextSearch`] → Mitigación: helper solo en el adaptador / test; no se cambia el puerto.
- [Ámbito ~2 h] → Mitigación: sin nuevo método de repo; sin wiring; sinónimos y retries ya son otros tickets.

## Migration Plan

Ninguna migración `.surql`. Rollback: el worker vuelve al no-op de T-03-07 (índice stale hasta T-03-11). Tests borran el índice efímero. Reexport de `search::productos_index_settings` evita romper T-03-05.

## Open Questions

Ninguna que altere la spec. El nombre exacto del helper de lectura del documento se fija en apply.
