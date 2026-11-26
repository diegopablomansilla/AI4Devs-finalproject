# Proposal

## Ticket padre

- **Ticket:** [T-04-05](../../../docs/7-tickets.md) — Endpoints de relaciones
- **Change:** `T-04-05`
- **HU:** [HU-04-01](../../../docs/6-historias.md#hu-04-01--crear-relación-entre-productos), [HU-04-02](../../../docs/6-historias.md#hu-04-02--relaciones-bidireccionales), [HU-04-03](../../../docs/6-historias.md#hu-04-03--gestionar-las-relaciones-de-un-producto)
- **Depende de:** T-04-04 (en `develop`: `RelationRepo` create/update/delete/list saliente), T-09-04 (`assert_conforms`)
- **Hace:** casos de uso en `recommend` y `POST /api/v1/relations`, `PATCH`/`DELETE /api/v1/relations/{relationType}/{relationId}`, `GET /api/v1/products/{productId}/relations`.
- **No hace:** UI.
- **Listo cuando:** escenarios de HU-04-01 y HU-04-02 como tests de `api` validados contra `openapi.yaml`.

## Why

HU-04-01 y HU-04-02 exigen crear una relación (complementario, `self_relation`, `duplicate_relation`) y un par bidireccional cuyo PATCH/DELETE aplique a ambas aristas. T-04-04 dejó el puerto persistente; el crate `recommend` sigue vacío y el Gateway no registra `/relations`. Sin HTTP ni casos de uso, T-04-09 no puede pintar la pestaña y las HUs de administración no tienen contrato ejecutable.

## What Changes

- Casos de uso en `recommend` (`RelationService` / `RelationGateway` object-safe) sobre `RelationRepo` + `ProductRepo` (existencia del producto en el GET; DC-05: no viven en `catalog`)
- `POST /api/v1/relations` → 201 `{ items }` (1 arista, o 2 si bidireccional); `bidirectional` omitido usa el default del tipo (`similar` sí, el resto no)
- `PATCH` / `DELETE /api/v1/relations/{relationType}/{relationId}` → el grupo bidireccional se actualiza o borra entero (ya lo hace el repo)
- `GET /api/v1/products/{productId}/relations` → 200 `{ items }` admin (incluye destinos inactivos y sin stock); query `type` y `direction` (`outgoing` defecto, `incoming`); producto inexistente → 404
- DTO inglés ↔ dominio ([5.3](../../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)): `similar`/`substitute`/`complement`, `sourceProductId`/`targetProductId`, `priority`/`note`/`bidirectionalGroup`
- `target` como `ProductSummary` mínimo desde `DestinoResumen` (`productType` `""`, `price`/`stock` `"0"`, `brand` null, `categoryPath` `[]`, `imageUrl` null)
- Allowlist: `recommend` puede depender de `domain`; `api` y `vortex` pueden depender de `recommend`
- `docs/openapi.yaml`: declarar `503` en las cuatro operaciones (mismo patrón que el resto de `/api/v1`)
- Tests de `api` con fake de `RelationGateway` + `assert_conforms`; tests de `recommend` contra repos en memoria
- Composition root: `vortex serve` inyecta `RelationService` (`RelacionRepo` + `ProductoRepo`)

## Non-goals

- UI de ficha, panel o carga rápida — **T-04-09**, **T-04-08**, **T-04-10** (carga rápida es v2; No hace del ticket)
- Consulta de recomendaciones (4.6.2), `destacar_sustitutos`, filtro `activo`/`stock > 0`, `GET /api/v1/products/{id}/recommendations` — **T-04-06**, **T-04-07**
- `vortex seed` de relaciones — **T-04-11**
- Hidratar `ProductSummary` completo (`productType`, `price`, `stock` reales, marca, ruta) — el GET admin usa el resumen de T-04-04; stock de destinos es T-04-06
- Reabrir `RelationRepo::create` / esquema `0007` / reglas de dominio (`self_relation`, prioridad, nota)
- Evento de outbox, Meilisearch, tracing de negocio extra, crate `catalog`

## Capabilities

### New Capabilities

- `vortex-recommend`: casos de uso de ABM de relaciones (crear, actualizar, borrar, listar) sobre `RelationRepo`, con 404 de producto en el listado vía `ProductRepo`; fachada object-safe para el Gateway; sin HTTP ni SurrealDB

### Modified Capabilities

- `vortex-api`: registrar los cuatro endpoints de relaciones; DTO 5.3; `AppState` + `vortex serve` inyectan `RelationGateway`; tests `assert_conforms`; OpenAPI declara `503`; MUST NOT registrar `/recommendations`
- `cargo-workspace`: `recommend` MAY `domain` (+ `tokio` en dev); `api` MAY path `recommend`; `vortex` MAY path `recommend`
- `domain-relacion`: el puerto gana `list_incoming_for_product` (T-04-04 difería `direction=incoming` a este ticket)
- `vortex-storage`: implementar `list_incoming_for_product` (aristas con `out` = el producto, resumen del origen)

## Impact

- `vortex-core/crates/recommend/` (casos de uso; deja de ser lib vacío)
- `vortex-core/crates/api/` (`src/relations.rs`, `AppState`, router, `error.rs` remap de campos, `tests/relations.rs`, fake)
- `vortex-core/crates/domain/src/relacion.rs` (método `list_incoming_for_product`)
- `vortex-core/crates/storage/src/relacion.rs` + `tests/relacion_repo.rs`
- `vortex-core/bin/vortex/src/serve.rs` y `Cargo.toml`
- `scripts/check-workspace.sh` + fixture de allowlist
- `docs/openapi.yaml` (`503` en las cuatro operaciones)
- Verificación: [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5
- Ticket: cierra T-04-05 (Listo cuando: HU-04-01 y HU-04-02 como tests de `api` con `assert_conforms`)
- Ámbito: mapper HTTP delgado + servicio en `recommend` + listado incoming diferido por T-04-04; cabe en ~2 h. No se divide el ticket.
