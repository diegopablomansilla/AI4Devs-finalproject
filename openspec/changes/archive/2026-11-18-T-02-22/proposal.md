# Proposal

## Ticket padre

- **Ticket:** [T-02-22](../../../docs/7-tickets.md) — Baja y reactivación de producto
- **Change:** `T-02-22`
- **HU:** [HU-02-06](../../../docs/6-historias.md#hu-02-06--baja-y-reactivación-de-producto)
- **Depende de:** T-02-21 (en `develop`: `POST`/`GET`/`PATCH /api/v1/products`, `ProductGateway` crear/obtener/editar, `PATCH { active }` rechazado con 400)
- **Hace:** `DELETE /api/v1/products/{productId}` (`activo = false` + evento) y reactivación con `PATCH { active: true }`; se conservan ledger, relaciones e imágenes.
- **No hace:** quitar de la búsqueda (lo hace el worker con el evento, T-03-07).
- **Listo cuando:** tests verifican 204, que el ledger sigue intacto y la reactivación.

## Why

HU-02-06 exige baja lógica de un producto discontinuado y reactivación posterior sin perder historial. T-02-21 dejó ficha HTTP y rechaza `PATCH { active }` a propósito. Sin `DELETE` ni `set_activo`, el administrador no puede sacar un SKU de circulación y T-02-34 no tiene acción de baja que consumir.

## What Changes

- `DELETE /api/v1/products/{productId}` → baja lógica (`activo = false`) y **204** sin cuerpo; id desconocido → 404 `not_found`; ya inactivo → 204 idempotente
- `PATCH /api/v1/products/{productId}` con `{ "version": N, "active": true }` → reactiva y **200** `Product` (`active` verdadero); `{ "version": N, "active": false }` es la misma baja que `DELETE` pero con **200** `Product`
- Puerto `ProductRepo::set_activo`, casos de uso `deactivate`/`reactivate` en `catalog`, y fachada `ProductGateway`
- Transacción storage: `UPDATE producto.activo` + `OutboxEvent` `producto`/`upsert`; MUST NOT borrar `movimiento_stock`, aristas ni `imagen_producto`; MUST NOT incrementar `version` ni escribir `stock`
- `GET` de ficha sigue devolviendo productos inactivos (la admin necesita verlos para reactivar)
- Tests: HTTP 204/200 + `assert_conforms`; storage verifica ledger intacto; catalog en memoria

## Non-goals

- Quitar el documento de Meilisearch / dejar de aparecer en búsqueda — **T-03-07** (este ticket solo emite el evento de outbox)
- Dejar de aparecer en recomendaciones — **T-04-06** (las consultas ya filtran `out.activo = true`; no se borran aristas)
- UI listado / baja / reactivación — **T-02-34**
- Listado admin paginado `GET /api/v1/products` — **T-02-27**
- `POST .../stock-movements` — **T-02-25**
- Historial de movimientos — **T-02-26**
- Imágenes — **T-02-28** / **T-02-30** (v2); este ticket MUST NOT borrarlas si existieran
- `DELETE` físico del producto
- Seed / occupancy de categoría por producto inactivo (H-10 ya ignora inactivos en T-02-05)
- Sidecars, e2e Playwright, codegen OpenAPI

## Capabilities

### New Capabilities

- (ninguna)

### Modified Capabilities

- `domain-producto`: `ProductRepo` gana `set_activo(id, activo)` → `FichaProducto`; `UpdateProducto` sigue sin `activo`
- `vortex-storage`: `ProductoRepo::set_activo` en `Tx` con outbox `producto`/`upsert`; conserva ledger, relaciones e imágenes
- `catalog-producto`: casos de uso dar de baja y reactivar; fachada `ProductGateway` las expone
- `vortex-api`: registrar `DELETE`; `PATCH { active }` deja de ser 400 y reactiva/da de baja; tests `assert_conforms`; `503` en `DELETE`

## Impact

- `vortex-core/crates/domain/src/producto.rs` (`ProductRepo::set_activo`)
- `vortex-core/crates/storage/src/producto.rs` + `tests/producto.rs`
- `vortex-core/crates/catalog/src/producto.rs` (casos de uso, gateway, fake en memoria)
- `vortex-core/crates/api/src/products.rs`, `error.rs`, `lib.rs`, `tests/products.rs`, `tests/support/fake_catalog.rs`
- `vortex-core/bin/vortex/tests/serve.rs` (DELETE desconocido → 404, no 503)
- `docs/openapi.yaml`: declarar `503` en `DELETE /api/v1/products/{productId}` (el path ya existe)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-02-22 (Listo cuando: 204, ledger intacto, reactivación)
- Ámbito: mismo molde que marcas/categorías; cabe en ~2 h. No se divide el ticket.
