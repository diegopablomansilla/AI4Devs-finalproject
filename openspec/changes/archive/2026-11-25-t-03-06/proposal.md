# Proposal

## Ticket padre

- **Ticket:** [T-03-06](../../../docs/7-tickets.md) — Construcción del documento de búsqueda
- **Change:** `t-03-06` (rama Git `T-03-06`)
- **HU:** [HU-03-01](../../../docs/6-historias.md) (solo el documento; el envío a Meilisearch es T-03-07)
- **Depende de:** T-02-19 (cerrado: `Producto`, `Marca`, `Categoria`, `AtributoDef`)
- **Hace:** función pura `build_search_document(producto, marca, categoría, defs)` según [4.11.1](../../../docs/4-modelo.md#4111-documento) más `marca_id` y `categoria_ids` (H-03); `atributos_texto` solo con defs `buscable`; `attr_<clave>` solo con `facetable`; `ubicacion` como `P3 · E-B · 12`; `imagen` en `null` (MVP sin T-02-28).
- **No hace:** envío a Meilisearch, worker, cambios en el adaptador `MeiliDocument`, `openapi.yaml`.
- **Listo cuando:** test de *snapshot* con el producto de ejemplo de 4.4.4 / 4.11.1.

## Why

El adaptador T-03-04 solo serializa `id` en `upsert`. T-03-05 dejó el índice configurado pero sin documento 4.11.1. H-03 exige `marca_id` y `categoria_ids` filtrables en el JSON. T-03-07 necesita un builder puro antes de llamar `TextSearch::upsert_documents`.

## What Changes

- Ampliar `domain::SearchDocument` con los campos 4.11.1 (serialización JSON alineada a Meilisearch: `precio` numérico, `imagen` null).
- `domain::build_search_document` (archivo `search_document.rs` o módulo en `text_search`).
- Tests de unidad happy + unhappy en `domain`; **Listo cuando** = `build_search_document_snapshot_4_4_4`.
- `SearchDocument::try_new` / `try_from_ulid_str` siguen válidos (resto en default/null).

## Non-goals

- Worker Index Sync, outbox, `upsert` completo en `search` → **T-03-07**
- Imagen principal en el documento → **T-02-28** (v2)
- Consulta SurrealDB / repos para cargar marca-categoría-defs
- `GET /search`, ranking, resaltado → **T-03-12+**
- Cambiar `MeiliTextSearch::upsert_documents` para enviar el cuerpo completo (T-03-07)

## Capabilities

### Modified Capabilities

- `domain-text-search`: documento 4.11.1 + `build_search_document`; levanta MUST NOT de T-03-03/T-03-06 en specs maestras (delta en este change).

## Impact

- **Repo:** `vortex-core/crates/domain/src/` (`text_search.rs`, nuevo `search_document.rs`, tests).
- **CI:** `cargo test -p domain`, `cargo test --workspace`; `scripts/check-domain-deps.sh`.
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`.