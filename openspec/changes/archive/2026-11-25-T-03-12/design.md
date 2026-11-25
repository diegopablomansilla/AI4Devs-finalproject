# Design

## Context

En `develop`, `domain` expone `TextSearch`, `SearchQuery` (`q` ≤ 200, `offset` ≤ 1000, `limit` 1..=50), `SearchFilters` (struct público: `categoria_ids`, `marca_id`, `con_stock`, `atributos` sin prefijo `attr_`) y `SearchHit { id }`. `MeiliTextSearch::search` envía solo `q`/`offset`/`limit` y deserializa `MeiliDocument { id }` — filtros, `orden` y `_formatted` se ignoran (vortex-search lo deja explícito a T-03-12). El índice `productos` ya tiene `filterableAttributes` / `sortableAttributes` 4.11.2 + H-03/H-04. El crate `search` puede depender de `domain`, `meilisearch-sdk`, `tokio`, `serde`, `serde_json`. `catalog` orquesta CRUD; no hay caso de uso de búsqueda. Ver `proposal.md` para motivación.

Constraints: MVP del curso; ~2 h; sin HTTP (T-03-13); sin facetas (T-03-18); sin respaldo (T-03-30); Listo cuando = `TextSearch` falso, sin sidecar.

## Goals / Non-Goals

**Goals:**

- `SearchService<S: TextSearch>` en el crate `search` que arma `SearchQuery` y llama al puerto
- Traductor puro `SearchFilters`/`SearchOrden` → `filter`/`sort` de Meilisearch + `attributesToHighlight`
- `SearchHit` de dominio con campos 4.11.1 opcionales/por defecto y `resaltado`
- Fake que registra el `SearchQuery` (Listo cuando) + unitarios del traductor sin motor
- `SearchGateway` object-safe para T-03-13

**Non-Goals (diseño):**

- Query string OpenAPI (`categoryId`, `attr[clave]`, `sort=relevance`) en este crate
- `async-trait` / `dyn TextSearch` (el puerto sigue RPITIT; la fachada boxea futures como `ProductGateway`)
- Tests `#[ignore = "meilisearch"]` nuevos (T-03-14 cubre ranking contra el motor)
- Cambiar allowlist de `search` o `domain`
- Rama de categoría: el caso de uso recibe `categoria_ids` ya resueltos; T-03-13 mapea un `categoryId` (incluye descendientes vía `categoria_ids` del documento)

## Decisions

### 1. Caso de uso en `search`, no en `catalog`

**Elección:** `vortex-core/crates/search/src/query.rs` (nombre tentativo) con `SearchService<S: TextSearch>` e `IndexUid` inyectado. `catalog` no se toca.

**Por qué:** el ticket dice «en `search`». EP-03 es el componente Search ([3.4.4](../../../docs/3-componentes.md#344-search-service)). El servicio solo depende del *trait* de `domain`; el adaptador Meilisearch vive al lado pero los tests del caso de uso no lo instancian.

**Alternativa:** `catalog::SearchService`. Se descartó: mezcla EP-02 con búsqueda y contradice el Hace. **Alternativa:** crate nuevo. Se descartó: el member `search` ya existe y T-03-04 lo reservó para esto.

### 2. Dos capas de «traducción», una fuente de verdad para Meilisearch

**Elección:**

1. **Caso de uso → puerto:** arma `SearchQuery` / `SearchFilters` de dominio. El fake registra ese struct (Listo cuando: cada filtro).
2. **Adaptador → motor:** funciones puras `meili_filter(&SearchFilters) -> Option<String>` y `meili_sort(SearchOrden) -> Option<Vec<String>>` usadas por `search_inner`. Highlight fijo `["nombre", "descripcion"]`.

**Por qué:** hexagonal: el caso de uso no conoce la sintaxis `attr_<clave> = "M8"`. El ticket pide ambos (armar consulta con `attr_<clave>` **y** tests con `TextSearch` falso). Las funciones puras cubren la sintaxis sin sidecar.

**Alternativa:** que el caso de uso emita el string de filtro. Se descartó: filtraría Meilisearch en la capa de aplicación. **Alternativa:** solo adaptar en `search_inner` sin funciones extraídas. Se descartó: no se puede TDD sin motor.

### 3. Sintaxis de filtro y sort

**Elección:**

| Dominio | Meilisearch |
|---|---|
| `categoria_ids = [id, …]` | `categoria_ids IN ["id", …]` |
| `marca_id = Some(id)` | `marca_id = "id"` |
| `con_stock = Some(true/false)` | `con_stock = true` / `false` |
| `atributos["medida"] = "M8"` | `attr_medida = "M8"` |
| varios | unidos con ` AND ` |
| vacío | `None` → no llamar `with_filter` |
| `Relevancia` | no `with_sort` |
| `PrecioAsc` / `PrecioDesc` | `precio:asc` / `precio:desc` |
| `StockDesc` | `stock_disponible:desc` |

Escapar valores string del filtro (reemplazar `\` y `"`) para no romper la gramática si un valor de atributo trae comillas.

**Por qué:** coinciden con `filterableAttributes` / `sortableAttributes` de T-03-05. `categoria_ids IN` aprovecha que el documento 4.11.1 guarda ancestros + self: un id de rama matchea descendientes sin que este ticket recorra el árbol.

**Alternativa:** `categoria.lvl0 = "…"`. Se descartó: el dominio filtra por id, no por nombre de nivel. **Alternativa:** `sort` también en relevancia (`stock_disponible:desc`). Se descartó: eso ya es `rankingRules`; duplicarlo cambia el contrato de «relevancia».

### 4. Validar claves de atributo en `SearchQuery::try_new`

**Elección:** al construir `SearchQuery`, cada clave de `filtros.atributos` pasa la misma regla que `productos_index_settings` (minúsculas ASCII, dígitos, `_`, primer carácter letra, **sin** `attr_`). Fallo → `invalid_input`, no se llama al puerto.

**Por qué:** el prefijo `attr_` es detalle de Meilisearch; si llegara al puerto, el adaptador produciría `attr_attr_medida`. Reusar la regla evita un segundo validador divergente. Extraer un helper `domain` compartido si el duplicado duele; no es obligatorio.

**Alternativa:** validar solo en el adaptador. Se descartó: el fake no lo vería y el Listo cuando pide el caso de uso. **Alternativa:** `SearchFilters` con constructor propio. Se descartó: hoy es struct público; validar en `SearchQuery::try_new` basta y no rompe los tests que arman `SearchFilters` a mano y luego `try_new`.

### 5. `SearchHit` crece; el dummy sigue compilando

**Elección:** añadir a `SearchHit` los campos 4.11.1 usados por OpenAPI `SearchHit` (sku, nombre, tipo, marca, codigos, atributos_texto, precio, unidad_medida, stock_disponible, con_stock, ubicacion, imagen) con `Default` (strings vacías, `None`, 0, `false`) y `resaltado: Option<SearchHighlight { nombre, descripcion }>`. Tests existentes de `SearchHit { id }` se actualizan con `..Default::default()` o un constructor `SearchHit::from_id`. El adaptador deserializa el documento + `_formatted` (SDK: `formatted_result` / `SearchResult.formatted_result`) y rellena.

**Por qué:** T-03-13 debe mapear `highlight` desde `_formatted` y campos required de OpenAPI. Si este ticket deja solo `id`, T-03-13 o reconsulta Surreal o reabre el adaptador. Hidratar aquí es el costo mínimo (el JSON ya viene en el hit).

**Alternativa:** DTO de aplicación en `search` distinto de `domain::SearchHit`. Se descartó: el puerto ya devuelve `SearchPage`. **Alternativa:** hidratar con `ProductRepo`. Se descartó: latencia y desactualización respecto del índice.

Meilisearch envuelve coincidencias en `<em>` por defecto; T-03-13 / T-03-16 hablan de `<mark>`. **Elección:** pedir `highlight_pre_tag` / `highlight_post_tag` `<mark>` y `</mark>` en la consulta para no reescribir HTML en el caso de uso.

### 6. `SearchGateway` object-safe, sin facetas

**Elección:** trait con future boxed (`Pin<Box<dyn Future + Send>>`), método `search`. `SearchService` lo implementa. Sin `facet_search` ni fallback.

**Por qué:** mismo molde que T-02-20 (`ProductGateway`) para `AppState` en T-03-13. El puerto `TextSearch` sigue RPITIT.

**Alternativa:** omitir la fachada hasta T-03-13. Se descartó: reabriría `search` en el ticket HTTP. **Alternativa:** `dyn TextSearch`. Se descartó: RPITIT no es object-safe; el worker ya usa genéricos.

### 7. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-03-12` y rename a `T-03-12`.

## Risks / Trade-offs

- **[SearchHit { id } rompe tests de domain/search/sync]** → Mitigación: constructor `from_id` / `Default` en campos nuevos; actualizar matches en el mismo change. No es un split de ticket: el compilador lista los sitios.
- **[Suite ignore de T-03-04 no cubre filtros reales]** → Mitigación: aceptable; T-03-14 corre contra el motor. Este ticket verifica traducción pura + fake.
- **[`<em>` vs `<mark>`]** → Mitigación: tags en la consulta. Si el SDK 0.33 no expone pre/post tag, mapear en el adaptador (`<em>` → `<mark>`) y documentarlo al aplicar.
- **[Ámbito ~2 h]** → Mitigación: sin HTTP, sin sidecar, sin facetas. Si hidratar 4.11.1 se alarga, el mínimo Listo cuando es fake + `resaltado`; el resto de campos de hit sigue siendo este ticket (T-03-13 no debe reabrir el adaptador).

## Migration Plan

Sin migración `.surql`. Despliegue = merge a `develop`. Rollback = revertir `search` (query + `search_inner`) y el shape de `SearchHit`; el índice no cambia.

## Open Questions

Ninguna que altere spec, enfoque o tasks.
