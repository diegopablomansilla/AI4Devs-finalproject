# Design

## Context

En `develop`, `vortex reindex` es stub (T-00-07). El puerto `TextSearch` ya declara `create_index` / `swap_indexes` / `delete_index` / `apply_settings` / `upsert_documents` (T-03-03/T-03-04); `build_search_document` y `productos_index_settings` existen; el worker de `sync` ya pagina `ProductRepo::list(activo: true)` y relee fichas. `vortex seed` es el precedente de CLI en modo directo (config → `Db::open` + migrate → lógica → exit). Recableo MVP §7.5.1: reindex abre la base con el servidor detenido (H-01, sin T-00-22). Ver `proposal.md` para motivación y `specs/` para el contrato.

Constraints: No hace = vectores; ~2 h; members fijos; `openspec new change` exige kebab-case (`t-03-11` → rename `T-03-11`).

## Goals / Non-Goals

**Goals:**

- Orquestador en `sync` inyectando puertos (tests con fake + Meilisearch real = Listo cuando)
- `vortex reindex` composition root: config → Storage → Meili → orquestador → exit 0
- Fail-closed: lock RocksDB, Meili caído o documento inválido → exit ≠ 0 / `DomainError`, sin `swap`, sin mover cursor, sin `no implementado`

**Non-Goals (diseño):**

- Ampliar `OutboxRepo` con `latest_event` (se recorre `list_after_meilisearch` hasta vaciar)
- Extraer crate `boot` compartido con `serve`/`seed`
- Renombrar índices en Meilisearch (no hay API de rename; el swap es el contrato)
- Re-sonda periódica de Meilisearch ni arrancar el worker desde `reindex`

## Decisions

### 1. Orquestador en `sync`; composition root en `bin/vortex`

**Elección:** `sync::reindex_meilisearch(...)` (nombre MAY variar) recibe `&impl TextSearch`, `&impl ProductRepo`, `&impl AttributeDefRepo`, `&impl OutboxRepo` y los dos `IndexUid`. Tests unitarios usan fakes como el worker. Tests `#[ignore = "meilisearch"]` usan `MeiliTextSearch` + `storage` (mismo patrón que `sync/tests/meili_product_worker.rs`). `bin/vortex` abre `Db`, espera Meili `up` (sin degradar: si no hay `up`, aborta), construye repos/adaptador e inyecta uids `productos` / `productos_nuevo`.

**Por qué:** HU-03-02 vive junto al worker; `sync` ya depende de `domain`+`storage` y tiene `search` solo en dev-dependencies. El binario ya declara `search` y `sync`.

**Alternativa:** lógica solo en `bin/vortex`. Se descartó: el Listo cuando (bucle de search) quedaría atado al proceso CLI y duplicaría fakes. **Alternativa:** crate nuevo. Se descartó: members fijos y ~2 h.

### 2. Snapshot del último outbox con `list_after_meilisearch`, no puerto nuevo

**Elección:** paginar desde `after = None` con `OUTBOX_BATCH_LIMIT` hasta lote vacío; el último id visto es el snapshot. Si no hay filas, el cursor destino es NONE (no llamar `advance_meilisearch_cursor`).

**Por qué:** `advance_meilisearch_cursor` ya exige que el id exista; no hace falta cambiar `domain`. En MVP (~200 SKUs, pocos eventos) son 1–2 lecturas.

**Alternativa:** `SELECT id FROM outbox ORDER BY id DESC LIMIT 1` en un método nuevo. Se descartó: cambia el puerto fuera del Hace. T-05-09 podrá añadirlo si el volumen lo pide.

### 3. Blue-green: `productos_nuevo` → swap → borrar el uid viejo

**Elección:** si `productos_nuevo` existe (corrida interrumpida), `delete_index` y recrear. Crear `productos_nuevo`, `apply_settings` 4.11.2, upsert de activos por lotes (`OUTBOX_BATCH_LIMIT`). Si `productos` no existe, `create_index` (ignorar already-exists como `serve::bootstrap_productos_index`). `swap_indexes(productos_nuevo, productos)` y luego `delete_index(productos_nuevo)` (post-swap ese uid tiene los datos viejos). Fallos **antes** del swap: no avanzar cursor; MAY dejar `productos_nuevo` (la siguiente corrida lo borra).

**Por qué:** Meilisearch exige dos uids existentes para el swap; el uid vivo nunca se borra. La búsqueda contra `productos` sigue sirviendo documentos viejos hasta el swap atómico (Listo cuando).

**Alternativa:** vaciar `productos` y rellenar in-place. Se descartó: corta la búsqueda (rompe HU-03-02). **Alternativa:** swap y no borrar. Se descartó: deja un índice huérfano.

### 4. Solo productos activos; no drenar el outbox

**Elección:** `FiltroProducto { activo: true, categoria: None, marca: None }` + `get` + `build_search_document`. Inactivos no se escriben al índice nuevo (desaparecen del vivo tras el swap). El cursor salta al snapshot: el worker no tiene que rehacer esos eventos.

**Por qué:** [4.8.2](../../docs/4-modelo.md#482-sync_cursor-y-outbox_fallo) — la reindexación total no usa el outbox. El worker (T-03-07) ya trata inactivo como delete.

**Alternativa:** upsert todos y delete de inactivos en el índice nuevo. Se descartó: trabajo extra en un índice vacío.

### 5. Listo cuando busca en Meilisearch, no en HTTP de `serve`

**Elección:** el test de bucle llama `TextSearch::search` sobre el uid `productos` en paralelo con el orquestador. `vortex serve` está detenido (lock RocksDB). Meilisearch sigue arriba.

**Por qué:** recableo MVP + H-01. HU-03-02 «la búsqueda no se interrumpe» se cumple en el motor, no en Axum.

**Alternativa:** levantar `serve` y buscar `/search` durante el reindex. Se descartó: el lock impide abrir la base; T-00-22 es v2.

### 6. CLI: mismo contrato de config que `seed`; Meili down aborta

**Elección:** `Commands::Reindex` en `main` (como `Seed`). Reusar `resolve_config_path` + `Config::from_path`. Tokio, `Db::open` + `migrate`, cliente Meili, `health`/`wait` corto: si no hay `up`, exit ≠ 0 (a diferencia de `serve`, que degrada). Tests: quitar `reindex` del loop de stubs en `cli.rs` y de `config_cli.rs` (`vortex_reindex_with_*_still_stub`); el stub de `--config` inexistente pasa a `backup`. Lock = mismo patrón que `tests/seed.rs`. Happy CLI y Meili-down: `#[ignore = "meilisearch"]` / URL con puerto cerrado.

**Por qué:** reindex sin motor no puede cumplir HU-03-02; degradar dejaría el índice intacto y un exit 0 mentiroso.

**Alternativa:** degradar como `serve`. Se descartó: el comando no tiene otro trabajo útil. **Alternativa:** `--force`. Se descartó: es T-05-09 (vectores).

## Risks / Trade-offs

- **[Swap atómico vs ventana de build]** → Mitigación: las búsquedas van al uid vivo hasta el swap; el Listo cuando cubre esa ventana. El swap en sí es una task de Meilisearch ya esperada por el adaptador.
- **[Test de bucle flaky]** → Mitigación: sembrar el uid vivo **antes** de lanzar el orquestador; `q` que matchea un SKU conocido; fallar el test al primer error o `hits.is_empty()`.
- **[Lock RocksDB en macOS (H-01)]** → Mitigación: mismo patrón que `seed.rs` / `storage/tests/open.rs`.
- **[Índice `productos_nuevo` huérfano si el proceso muere post-swap pre-delete]** → Mitigación: la siguiente corrida borra `productos_nuevo` al inicio. El uid vivo ya tiene los datos nuevos.
- **[Documento inválido a mitad de upsert]** → Mitigación: abortar sin swap; el vivo no cambia; cursor intacto.
- **[kebab-case del CLI OpenSpec]** → Scaffold `t-03-11` y rename (ya hecho).

## Migration Plan

No hay datos de producción. Rollback: revertir el PR; `vortex reindex` vuelve a stub. Quien haya reindexado conserva el índice `productos` (derivado). Tras `vortex seed`, el operador corre `vortex reindex` con Meilisearch arriba y `serve` detenido.

## Open Questions

Ninguna que altere specs o tareas. Snapshot por paginación de outbox, abortar si Meili está down y el Listo cuando contra el motor (no HTTP) quedan fijados arriba.
