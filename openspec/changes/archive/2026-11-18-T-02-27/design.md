# Design

## Context

En `develop`, T-02-21/T-02-22 dejaron `ProductGateway` (`create`/`get`/`update`/`deactivate`/`reactivate`) inyectado en `AppState`, DTO 5.3 de ficha y `GET /api/v1/products` **sin registrar** (el test `get_products_list_is_not_summary_page` lo cubre). T-00-21 dejó `api::pagination` (`decode_cursor`, `resolve_limit`, `paginate`, `encode_cursor`). `ProductRepo` no declara `list`. OpenAPI ya contrata `GET /api/v1/products` → `ProductSummaryPage` con `categoryId`/`brandId`/`active`/`cursor`/`limit`; no declara `503` en ese GET. El modelo [4.4.1](../../../docs/4-modelo.md#441-categoria--árbol) fija la rama: `categoria = $c OR categoria.ancestros CONTAINS $c`. Ver `proposal.md` para motivación.

Constraints: MVP del curso; ~2 h; `api` no depende de `storage`; `domain` no gana crates; sin búsqueda (T-03-13); sin UI (T-02-34); sin imágenes (T-02-28).

## Goals / Non-Goals

**Goals:**

- HTTP `GET /api/v1/products` con filtros de rama/marca/activo, orden por ID, cursor T-00-21 y `assert_conforms` (Listo cuando HU-02-10)
- `ProductRepo::list` + adaptador SurrealDB + caso de uso delgado en `catalog`
- DTO `ProductSummary` 5.3; `imageUrl: null`

**Non-Goals (diseño):**

- Tipo `ResumenProducto` que hidrate atributos/códigos (eso es ficha)
- Hacer `ProductRepo` object-safe (`async-trait`)
- Índice extra en `producto` para el listado (el IDX de `categoria.ancestros` ya existe en la tabla `categoria`)
- Extraer un mapper 5.3 genérico
- Tests e2e Playwright

## Decisions

### 1. `list` en `ProductRepo`, no un scan en catalog

**Elección:** `ProductRepo::list(FiltroProducto, after, take) -> Vec<ResumenProducto>`. Catalog reenvía. Storage ejecuta el SELECT. El Gateway pide `take = limit + 1` y llama `paginate`.

**Por qué:** maestros (`list_all`) caben en memoria; el catálogo admin no. Un `list_all` de productos rompería el tope de ~2 h en cuanto haya seed (T-02-41, ~200 SKUs) y no usaría el cursor.

**Alternativa:** catalog llama `get` N veces. Se descartó: N+1 y no filtra en el motor. **Alternativa:** `api` consulta SurrealDB. Se descartó: rompe 3.2.

### 2. `ResumenProducto` liviano, no `FichaProducto`

**Elección:** tipo de dominio con `id`, `sku`, `nombre`, `tipo`, `precio`, `stock`, `activo`, `marca_nombre`, `categoria_ruta`. Storage hidrata marca.nombre y categoria.ruta **sin** atributos ni códigos.

**Por qué:** `ProductSummary` no pide ficha completa; hidratar `AtributoDef` por fila es costo muerto. El mapper HTTP es 1:1 (`brand` = `marca_nombre`, `categoryPath` = `categoria_ruta`, `imageUrl = null`).

**Alternativa:** devolver `FichaProducto` y recortar en API. Se descartó: obliga joins de atributos en un listado. **Alternativa:** armar el DTO en storage. Se descartó: nombres en inglés en el adaptador.

### 3. Filtros AND; categoría/marca desconocida = página vacía

**Elección:** `activo` siempre se aplica (default HTTP `true`). `categoria`/`marca` ausentes no filtran. ULID de categoría o marca que no matchea → `200 { items: [], nextCursor: null }`, no `404`.

**Por qué:** OpenAPI del GET colección solo declara 200 y 400 (más 503 que agregamos). Un filtro es una predicado, no un lookup de recurso.

**Alternativa:** 404 si `categoryId` no existe. Se descartó: no está en el contrato y obliga un GET extra a `CategoryRepo` antes de listar.

### 4. Query de rama en el motor, no expansión en catalog

**Elección:** SurrealQL `WHERE categoria = $c OR categoria.ancestros CONTAINS $c` ([4.4.1](../../../docs/4-modelo.md#441-categoria--árbol)). El fake en memoria de catalog/api replica la misma regla con `Categoria.ancestros`.

**Por qué:** el modelo ya denormaliza `ancestros`; no hay que materializar el subárbol en el caso de uso.

**Alternativa:** catalog lista todas las categorías descendientes y hace `IN (...)`. Se descartó: duplica lógica que el record link ya resuelve.

### 5. Handler en `products.rs`; reusar helper

**Elección:** `GET` en la misma ruta `/api/v1/products` (hoy solo `post`). Query struct `category_id`, `brand_id`, `active: Option<bool>`, `cursor`, `limit`. `decode_cursor` / `resolve_limit` / `paginate` / `encode_cursor`. Remap de parseo: ULID inválido → `categoryId`/`brandId`; el helper ya pone `cursor`/`limit`.

**Por qué:** un módulo `product_list.rs` partiría el hexágono de producto sin ganancia. T-00-21 existe para no reimplementar base64url.

**Alternativa:** Axum 405 hasta que T-02-34. Se descartó: este ticket **es** el GET.

### 6. Fake HTTP vs Surreal

**Elección:** tests de `api` (`tests/products.rs`) extienden el fake de `ProductGateway` con `list` in-memory (HashMap de productos + categorías con `ancestros`). Persistencia de rama: `storage/tests/producto.rs`. Cableado: `vortex_serve_get_products_empty_returns_200` (200 vacío ≠ 503).

**Por qué:** mismo molde T-02-21/T-02-25. `#[cfg(test)]` de catalog no es visible desde `api`.

### 7. `q` se ignora

**Elección:** no declarar `q` en el query struct; Axum descarta el param. El test compara `?q=perno` vs sin `q`.

**Por qué:** No hace = búsqueda por texto. Rechazar `q` con 400 sería un contrato extra que OpenAPI no tiene.

### 8. OpenAPI `503`

**Elección:** agregar `'503': Unavailable` a `GET /api/v1/products`. No tocar esquemas `ProductSummary` / `ProductSummaryPage`.

**Por qué:** T-09-04 falla si respondemos 503 sin schema. El GET de ficha ya lo tiene.

### 9. Impls de `ProductRepo` que ganan `list`

Hay dummies en `domain`, `catalog` (`#[cfg(test)]`), `api/tests/support/fake_catalog.rs` y el adaptador `storage`. Todos MUST implementar `list` o no compilan. El fake HTTP filtra en memoria; no abre Surreal.

### 10. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-27` y rename a `T-02-27`.

## Risks / Trade-offs

- **[Surreal `categoria.ancestros CONTAINS` en record link]** → el modelo lo documenta; si el motor no recorre el link, el test de storage de rama falla en rojo y se ajusta a `type::thing` / fetch explícito. Mitigación: el escenario `storage_list_branch_includes_descendants` es el canario.
- **[Fake HTTP no escribe Surreal]** → `vortex_serve_get_products_empty_returns_200` + tests de `storage` cubren el camino real.
- **[Call sites de ProductRepo]** → el compilador obliga a tocar cada impl. Mitigación: `list` que devuelve `Ok(vec![])` en dummies de tests que no ejercitan listado.
- **[Alcance ~2 h]** → HTTP + `list` en tres capas. No se divide el ticket: T-00-21 ya paginó y T-02-21 ya cableó el Gateway.
- **[`get_products_list_is_not_summary_page`]** → se reemplaza por los tests 200; dejarlo rompe el Listo cuando.

## Migration Plan

N/A (endpoint nuevo sobre path ya contratado; no hay datos que migrar). Rollback = revertir el PR a `develop`. El test previo que prohibía el GET deja de aplicar.

## Open Questions

Ninguna que cambie specs o tareas. `imageUrl` queda `null` hasta T-02-30.
