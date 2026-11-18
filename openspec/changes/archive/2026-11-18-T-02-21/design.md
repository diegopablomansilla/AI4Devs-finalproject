# Design

## Context

En `develop`, T-02-20 dejó `ProductService` / `ProductGateway` (`create` / `get` / `update`, `PatchProducto` sin `stock` ni `activo`) y fakes en memoria. T-02-18 persistió alta + movimiento `inicial` **sin** `terminal` (lo diferió a este ticket). T-02-09/T-02-12 fijaron el molde HTTP: `AppState` con `Option<Arc<dyn …>>`, DTO 5.3, `assert_conforms`, `503` sin servicio, composition root en `vortex serve`. OpenAPI ya declara `POST/GET/PATCH /products`, `ProductCreate`/`ProductUpdate`/`Product`, `Location` y `X-Vectra-Terminal`; no declara `503` en esos paths. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; sin listado (T-02-27); sin `DELETE`/`active` (T-02-22); `api` no depende de `storage`; `domain` no gana crates.

## Goals / Non-Goals

**Goals:**

- HTTP `POST`/`GET`/`PATCH` de ficha con DTO 5.3, `Location` y `assert_conforms`
- Plomería `X-Vectra-Terminal` → `NewProducto.terminal` → `movimiento_stock.terminal` en el alta
- Inyección de `ProductGateway` en `AppState` y `vortex serve`

**Non-Goals (diseño):**

- Merge de parche en el handler (ya está en `catalog::PatchProducto`)
- `async-trait`
- Extraer un mapper 5.3 genérico compartido con maestros
- Implementar `StockLedger` o movimientos `inbound`/`adjustment`
- Hidratar `images` (siempre `[]`)

## Decisions

### 1. Handlers delgados sobre `ProductGateway`

**Elección:** `vortex-core/crates/api/src/products.rs` mapea JSON inglés → `NewProducto` / `PatchProducto`, llama a `Arc<dyn ProductGateway>`, serializa `FichaProducto` → `Product`. `AppState` guarda `Option<Arc<dyn ProductGateway>>` con `with_products`. Sin servicio → `503 unavailable`.

**Por qué:** mismo hexágono que categorías/marcas/atributos. T-02-20 ya dejó la fachada object-safe.

**Alternativa:** `api` llama a `ProductoRepo`. Se descartó: rompe 3.2. **Alternativa:** merge de PATCH en el handler. Se descartó: T-02-20 asignó el merge a `catalog`.

### 2. `NewProducto.terminal` en domain, no un argumento extra en el Gateway

**Elección:** campo `terminal: Option<String>` en `NewProducto`. Catalog reenvía el struct. Storage escribe `terminal` en el `CREATE movimiento_stock` si `stock_inicial > 0`. El Gateway recorta espacios, rechaza `len > 64`, y pone `None` si falta o queda vacío.

**Por qué:** T-02-18 lo dejó explícitamente para T-02-21. Un segundo parámetro en `ProductGateway::create` duplicaría el DTO del puerto.

**Alternativa:** header solo en HTTP y olvidar el ledger hasta T-02-25. Se descartó: el Hace del ticket nombra `X-Vectra-Terminal` para el movimiento inicial. **Alternativa:** validar 64 chars en `domain`. Se descartó: el límite es del parámetro OpenAPI, no del modelo 4.5.1.

### 3. Fake HTTP vs fake `catalog`

**Elección:** tests de `api` (`tests/products.rs`) usan un fake de `ProductGateway` en `tests/support/fake_catalog.rs` que recuerda el último `NewProducto` (para asertar `terminal`) y semilla categoría/marca/defs vía el mismo árbol de fakes de maestros. Tests de `storage` cubren persistencia de `terminal` en Surreal. Tests de `catalog` cubren que `terminal` no se pierde al crear.

**Por qué:** `#[cfg(test)]` de `catalog` no es visible desde `api`. Surreal en cada test HTTP acoplaría a `storage`. El binario `vortex_serve_get_unknown_product_returns_404` cubre el cableado real (404 ≠ 503).

**Alternativa:** reexportar `InMemoryProductRepo`. Se descartó: no es público y mezclaría crates.

### 4. Remap de `errors[].field` sin romper `tipo` → `type`

**Elección:** extender `remap_api_field` con claves que no colisionan: `categoria`→`categoryId`, `marca`→`brandId`, `descripcion`→`description`, `unidad_medida`→`unitOfMeasure`, `precio`→`price`, `ubicacion`→`location`, `stock_inicial`→`initialStock`. El handler de productos, **antes** de `from_domain`, reescribe `FieldError.field == "tipo"` a `productType` (el remap global sigue siendo `tipo`→`type` para attribute-definitions). Path inválido → `productId`. `active` y `X-Vectra-Terminal` se generan en el Gateway, no en domain.

**Por qué:** 5.3 es la tabla única; `tipo` ya significa `type` en defs.

**Alternativa:** remap contextual por ruta. Se descartó: un `match` extra en productos es más local.

### 5. `PATCH { active }` se rechaza

**Elección:** si `ProductUpdate.active` es `Some(_)`, responder `400 invalid_input` field `active` sin llamar al servicio. No ignorar el campo.

**Por qué:** T-02-22 es el dueño de reactivación. Un ignore silencioso haría que `{ version, active: true }` parezca éxito sin reactivar.

**Alternativa:** ignorar `active`. Se descartó: UX engañosa. **Alternativa:** implementarlo aquí. Se descartó: No hace / T-02-22.

### 6. `GET /api/v1/products` no se registra

**Elección:** solo `post` en `/api/v1/products`; `get`+`patch` en `/{productId}`. Un `GET` a la colección no devuelve `ProductSummaryPage` (Axum 405 o el fallback del método). `DELETE` tampoco se registra.

**Por qué:** No hace = listado T-02-27. Registrar GET vacío sería implementar el path a medias.

### 7. OpenAPI `503` y decimales como string

**Elección:** agregar `'503': Unavailable` a POST/GET/PATCH de producto (como marcas). No tocar `ProductCreate`/`Product`. `price` JSON number → rechazo serde → `400 invalid_input` (T-00-11). `images: []`. Enums se traducen en el handler (`unit`↔`unidad`, `manufacturer`↔`fabricante`).

**Por qué:** T-09-04 falla si respondemos 503 sin schema. `NonNegativeDecimal` ya es string.

### 8. Composition root

**Elección:** `serve.rs` construye `ProductService::new(ProductoRepo, CategoriaRepo, MarcaRepo, AtributoDefRepo)` (todos con `Arc<Db>`) y `AppState::with_products`. Occupancy de categorías sigue el stub actual.

**Por qué:** los cuatro puertos ya existen; `ProductService` es genérico sobre ellos.

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-21` y rename a `T-02-21`.

## Risks / Trade-offs

- **[`tipo` dual en remap]** → handler de productos reescribe a `productType` antes del remap global. Mitigación: test `product_problem_field_is_category_id_not_categoria` + un caso de `productType` si domain emite `tipo`.
- **[Fake HTTP no escribe Surreal]** → `vortex_serve_get_unknown_product_returns_404` + test de `storage` con `terminal` cubren persistencia.
- **[Call sites de `NewProducto`]** → hay que rellenar `terminal: None` en tests de catalog/storage. Mitigación: campo `Option`, compilación obliga a tocarlo.
- **[GET colección 405 vs 404]** → spec solo prohíbe `ProductSummaryPage`; no exige un status concreto. T-02-27 registrará GET.
- **[Alcance ~2 h]** → HTTP + mapeo es el núcleo; `terminal` es un campo y un literal SQL. No se divide el ticket.
