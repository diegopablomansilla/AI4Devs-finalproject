# Proposal

## Ticket padre

- **Ticket:** [T-03-11](../../../docs/7-tickets.md) — `vortex reindex` (Meilisearch) con intercambio atómico
- **Change / rama:** `T-03-11` (CLI OpenSpec: `t-03-11` → carpeta `T-03-11`)
- **HU:** [HU-03-02](../../../docs/6-historias.md#hu-03-02--reindexación-total) (MVP: con el servidor detenido, sin canal por socket)
- **Depende de:** T-03-07 (worker + `build_search_document` + `TextSearch::swap_indexes`; T-03-05/T-03-06/T-00-27 ya en `develop`)
- **Hace:** registra el último evento de outbox existente, construye `productos_nuevo` desde las tablas maestras con la configuración 4.11.2, `swapIndexes` atómico, borra el índice viejo y ubica `sync_cursor:meilisearch` en el evento registrado; en el MVP se ejecuta con el servidor detenido.
- **No hace:** vectores (T-05-09).
- **Listo cuando:** test que busca en bucle durante el reindex sin recibir errores ni resultados vacíos.

## Why

`vortex reindex` sigue siendo stub (`no implementado`) y el índice `productos` solo se actualiza por el outbox. HU-03-02 exige reconstruir Meilisearch desde SurrealDB y cambiar de índice de forma atómica para recuperar la búsqueda ante inconsistencias. Ahora, porque el worker, el builder 4.11.1, `swap_indexes` y el recableo MVP ([7.5.1](../../../docs/7-tickets.md): modo directo, servidor detenido, sin T-00-22) ya están en `develop`; es el único ticket MVP pendiente.

## What Changes

- Orquestador de reindexación Meilisearch en `sync` (función de biblioteca): snapshot del último id de `outbox`, crea `productos_nuevo` con el perfil 4.11.2, recorre productos **activos** vía `ProductRepo::list` + `get` + `build_search_document`, `upsert` por lotes, `swap_indexes(productos_nuevo, productos)`, borra el uid que quedó con los datos viejos, avanza `sync_cursor:meilisearch` al evento registrado
- `vortex reindex` deja de ser stub: carga `Config` (`--config` / `VORTEX_CONFIG`), abre Storage + migraciones en `<paths.datos>/db` con el servidor **detenido**, conecta Meilisearch por URL configurada, ejecuta el orquestador y termina 0 si confirma
- La búsqueda HTTP de `vortex serve` no corre durante el comando (lock RocksDB, H-01); el **Listo cuando** busca en bucle contra el índice vivo `productos` del motor (Meilisearch sigue arriba) mientras se construye `productos_nuevo`
- **BREAKING** (CLI): `vortex reindex` ya no imprime `no implementado` ni ignora `--config`; sin config, lock de RocksDB o Meilisearch caído → exit ≠ 0
- TDD: happy (rebuild + swap + cursor; búsqueda concurrente sin error ni página vacía) + unhappy (config ausente, base bloqueada, Meilisearch inalcanzable, fallo de documento **antes** del swap sin mover el cursor)
- MUST NOT recalcular vectores, MUST NOT usar el canal Unix socket, MUST NOT arrancar HTTP

## Non-goals

- Reindexación de vectores / `sync_cursor:vectores` / `--force` — **T-05-09** (**No hace** del ticket; v2)
- Canal de administración Unix socket / reindex con `vortex serve` vivo — **T-00-22** (v2)
- Purga del outbox — **T-03-10** (v2)
- Sinónimos en settings — **T-03-27** (v2)
- Imagen principal en el documento — **T-02-28** (v2; `imagen` sigue `null`)
- Supervisor que levante Meilisearch — **T-00-25** (v2); el comando consume la URL ya configurada
- `GET /search`, OpenAPI, UI, facetas HTTP
- Cambiar el puerto `TextSearch` (create/swap/delete ya existen) ni el esquema SurrealDB
- Crates nuevas en el workspace

## Capabilities

### New Capabilities

- `vortex-reindex`: `vortex reindex` reconstruye el índice Meilisearch `productos` desde las tablas maestras con intercambio atómico y alinea `sync_cursor:meilisearch`, abriendo SurrealDB en modo directo

### Modified Capabilities

- `vortex-cli`: `vortex reindex` deja de ser stub «no implementado»; el resto de subcomandos no implementados no cambia
- `vortex-sync`: el crate `sync` expone el orquestador de reindexación total (además del worker incremental)

## Impact

- **Repo:** `vortex-core/crates/sync/` (orquestador + tests), `vortex-core/bin/vortex/` (`reindex` en CLI, tests de stub y de comando)
- **APIs / OpenAPI:** ninguna
- **CLI:** `vortex reindex`; exige `--config` o `VORTEX_CONFIG` como `serve`/`seed`
- **Verificación:** `cargo test -p sync`, `cargo test -p vortex --test cli --test reindex`, tests `#[ignore = "meilisearch"]` (job CI T-09-05), [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`, `openspec validate T-03-11`
- **Ticket:** cierra T-03-11 (Listo cuando: búsqueda en bucle durante el swap sin error ni hits vacíos)
- **Ámbito:** ~2 h; reutiliza `ProductRepo::list`/`get`, `build_search_document`, `productos_index_settings` y `TextSearch::{create_index,apply_settings,upsert_documents,swap_indexes,delete_index}`. No se divide el ticket.
