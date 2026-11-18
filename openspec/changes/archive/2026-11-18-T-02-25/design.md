# Design

## Context

En `develop`, T-02-24 dejó `MovimientoStockRepo: StockLedger` (`append` transaccional 4.5.2 + `list_by_producto`) y T-02-21/T-02-22 el molde HTTP de ficha (`ProductGateway` en `AppState`, DTO 5.3, `X-Vectra-Terminal`, `assert_conforms`). `StockLedger` usa `impl Future` (no es `dyn`). No hay caso de uso de stock en `catalog` ni ruta `POST .../stock-movements`. OpenAPI ya declara el POST (`StockMovementCreate` solo `inbound`/`adjustment`) y el GET de historial (v2). Ver `proposal.md` para motivación y `specs/` para el contrato.

Constraints: MVP del curso; ~2 h; `api` no depende de `storage`; `domain` no gana crates; sin GET (T-02-26); sin UI (T-02-38).

## Goals / Non-Goals

**Goals:**

- HTTP `POST` de ingreso/ajuste con DTO 5.3, `X-Vectra-Terminal` y `assert_conforms` (Listo cuando HU-02-07)
- Caso de uso delgado en `catalog` que restringe a `ingreso`/`ajuste` y reenvía a `StockLedger::append`
- Inyección de `StockGateway` en `AppState` y `vortex serve`

**Non-Goals (diseño):**

- Extraer `parse_terminal_header` a un módulo compartido si copiar 15 líneas alcanza (MAY extraer si el linter lo pide)
- Hacer `StockLedger` object-safe (`async-trait`)
- Extraer el movimiento `inicial` de `ProductRepo::create` a este caso de uso
- Paginación / `list_by_producto` en HTTP
- Tests de `storage` nuevos (T-02-24 ya cubre `append`)

## Decisions

### 1. `StockGateway` aparte de `ProductGateway`

**Elección:** `catalog/src/stock.rs` con `StockService<L: StockLedger>` y trait object-safe `StockGateway { register(NewMovimientoStock) -> MovimientoStock }`. `AppState` gana `Option<Arc<dyn StockGateway>>` (`with_stock`). El handler vive en `api/src/stock.rs`, no se hincha `products.rs`.

**Por qué:** registrar stock no necesita categoría/marca/`AtributoDef`. Meter `StockLedger` en `ProductService` sumaría un genérico a todos los tests de ficha. El puerto no es `dyn`.

**Alternativa:** `ProductGateway::register_stock`. Se descartó: acopla inventario a la ficha y obliga a tocar el fake de productos. **Alternativa:** `api` llama a `StockLedger` directo. Se descartó: rompe 3.2 (`api` no depende de `storage`; el trait no es object-safe sin wrapper).

### 2. Rechazo de `inicial`/`venta` en catalog, no en el puerto

**Elección:** `StockService::register` falla con `invalid_input` field `tipo` si el tipo no es `ingreso` ni `ajuste`, **antes** de `append`. `StockLedger` sigue aceptando todos los tipos (POS T-08-03, alta T-02-18).

**Por qué:** el Hace del ticket es «solo inbound y adjustment». Cambiar el puerto rompería el adaptador y el POS futuro.

**Alternativa:** filtrar solo en HTTP (enum serde). Se descartó: el caso de uso quedaría callable con `venta` desde otro crate. HTTP **también** rechaza `initial`/`sale` por el enum OpenAPI.

### 3. Fake HTTP vs ledger real

**Elección:** tests de `api` (`tests/stock_movements.rs`) usan un fake de `StockGateway` (y el fake de producto ya existente si el escenario de `negative_stock` relee `GET` stock). El fake recuerda el último `NewMovimientoStock` (para asertar `terminal`) y puede devolver `negative_stock` / `not_found` / `invalid_input`. Persistencia real: T-02-24 + `vortex_serve_post_stock_unknown_product_returns_404`.

**Por qué:** `#[cfg(test)]` de `catalog` no es visible desde `api`. Surreal en cada test HTTP acoplaría a `storage`.

**Alternativa:** reexportar `InMemoryStockLedger`. Se descartó: no es público; el fake HTTP es el molde de T-02-21.

### 4. Remap `cantidad`/`motivo` en el handler de stock

**Elección:** extender `remap_api_field` con `cantidad`→`quantity` y `motivo`→`reason`. El handler de stock usa el remap global (`tipo`→`type` es correcto aquí; no reescribir a `productType`). Path inválido → `productId`. `X-Vectra-Terminal` se genera en el Gateway.

**Por qué:** 5.3 es la tabla única; `tipo` ya significa `type` salvo en ficha de producto.

**Alternativa:** remap contextual por ruta. Se descartó: las claves nuevas no colisionan.

### 5. GET / PATCH / DELETE no se registran

**Elección:** solo `post` en `/api/v1/products/{product_id}/stock-movements`. Un `GET` no devuelve `StockMovementPage` (405 o fallback). No hay path `{movementId}`.

**Por qué:** No hace = historial T-02-26. HU-02-07 inmutabilidad = no editar ni borrar. Registrar GET vacío sería implementar el path a medias.

### 6. OpenAPI `503` y decimales como string

**Elección:** agregar `'503': Unavailable` a `POST .../stock-movements`. No tocar `GET`, `StockMovementCreate` ni `StockMovement`. `quantity` JSON number → rechazo serde → `400 invalid_input`.

**Por qué:** T-09-04 falla si respondemos 503 sin schema. El enum OpenAPI ya limita `type` a `inbound`/`adjustment`.

### 7. Composition root

**Elección:** `serve.rs` construye `StockService::new(MovimientoStockRepo::new(Arc::clone(&db)))` y `AppState::with_stock`. Reusa el mismo `Arc<Db>` que `ProductoRepo`.

**Por qué:** el adaptador ya existe; no hay que abrir una segunda sesión.

### 8. Ledger en memoria para tests de catalog

**Elección:** un `StockLedger` dummy en `catalog` (`#[cfg(test)]`) que guarda movimientos, suma stock y emite `negative_stock` / `not_found`. No depende de `storage`.

**Por qué:** misma regla que `catalog-producto`. El Listo cuando de este ticket es HTTP; los tests de catalog cubren el rechazo de `inicial`/`venta` sin Surreal.

### 9. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-25` y rename a `T-02-25`.

## Risks / Trade-offs

- **[Fake HTTP no escribe Surreal]** → `vortex_serve_post_stock_unknown_product_returns_404` + tests T-02-24 cubren persistencia y `negative_stock` real. Mitigación: el escenario HTTP de stock intacto usa `GET` al fake de producto (stock `"5"`), no al ledger.
- **[`tipo` dual en remap]** → stock usa el remap global `tipo`→`type`; productos siguen reescribiendo a `productType` antes. Mitigación: test `stock_problem_field_is_reason_not_motivo`.
- **[GET colección 405 vs 404]** → spec solo prohíbe `StockMovementPage`; no exige un status concreto. T-02-26 registrará GET.
- **[Alcance ~2 h]** → HTTP + wrapper catalog + wiring. No se divide el ticket.

## Migration Plan

N/A (endpoint nuevo; no hay datos que migrar). Rollback = revertir el PR a `develop`.

## Open Questions

Ninguna que cambie specs o tareas. El path GET permanece en `openapi.yaml` como contrato v2 sin implementación.
