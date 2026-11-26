# Proposal

## Ticket padre

- **Ticket:** [T-04-04](../../../docs/7-tickets.md) — Repositorio: editar, eliminar y listar
- **Change:** `T-04-04`
- **HU:** [HU-04-02](../../../docs/6-historias.md#hu-04-02--relaciones-bidireccionales), [HU-04-03](../../../docs/6-historias.md#hu-04-03--gestionar-las-relaciones-de-un-producto)
- **Depende de:** T-04-03 (en `develop`: `RelacionRepo::create` simple o bidireccional; `update` / `delete` / `list_for_product` aún `Unavailable`)
- **Hace:** `update` y `delete` que aplican a todo el grupo bidireccional; `list_for_product` con las aristas salientes agrupadas por tipo, con resumen del destino.
- **No hace:** recomendaciones (T-04-06).
- **Listo cuando:** test: cambiar la prioridad desde B modifica ambas aristas.

## Why

HU-04-02 exige que editar o borrar una arista de un par bidireccional aplique a ambas; HU-04-03 necesita ver las aristas salientes de un producto agrupadas por tipo, con prioridad, nota y un resumen del destino (incluidos inactivos). `RelationRepo::create` ya persiste el grafo; sin `update` / `delete` / `list_for_product` no hay persistencia hexagonal para T-04-05.

## What Changes

- Completar el adaptador `RelacionRepo` en `storage`: implementar `update`, `delete` y `list_for_product` (dejar de devolver `unavailable`)
- `update`: parche de `prioridad` y/o `nota` sobre la arista `tipo`+`id`; si tiene `grupo_bidireccional`, la misma `Tx` aplica el parche a **ambas** aristas del grupo ([4.6.1](../../../docs/4-modelo.md#461-tablas-de-relación); HU-04-02)
- `delete`: baja **física** de la arista; si tiene `grupo_bidireccional`, borra ambas en la misma `Tx` (OpenAPI / [5.2](../../../docs/5-contract-api.md): DELETE físico solo en imágenes y relaciones)
- `list_for_product`: aristas **salientes** de las tres tablas, agrupadas por tipo, ordenadas por `prioridad` (1 = más alta), con resumen del destino (`id`, `sku`, `nombre`, `activo`); incluye destinos inactivos y sin stock (admin; T-04-06 filtra)
- Tipos de listado en `domain` (`DestinoResumen`, `RelacionConDestino`, `RelacionesDeProducto`) y firma de `list_for_product` alineada (T-04-02 difería el agrupado y el resumen a este ticket)
- Tests de integración en `crates/storage/tests/relacion_repo.rs` (happy Listo cuando: cambiar prioridad desde B modifica ambas aristas; unhappy: id desconocido, `prioridad`/`nota` inválidas)
- TDD: red = tests que fallan contra los stubs `Unavailable`; green = `update` / `delete` / `list_for_product`

## Non-goals

- Consulta de recomendaciones (4.6.2), `destacar_sustitutos`, filtro `activo`/`stock > 0` — **T-04-06** (No hace del ticket)
- Endpoint `GET /api/v1/products/{id}/recommendations` — **T-04-07**
- Endpoints HTTP de relaciones, casos de uso en `recommend` — **T-04-05**
- UI panel de recomendaciones, ficha y carga rápida — **T-04-08**, **T-04-09**, **T-04-10** (carga rápida es v2)
- `vortex seed` de relaciones — **T-04-11**
- Listado de aristas **entrantes** (`direction=incoming` del OpenAPI) — **T-04-05**
- Hidratar `ProductSummary` completo (`productType`, `brand`, `categoryPath`, `price`, `stock`, `imageUrl`) — **T-04-05** mapea el resumen mínimo al contrato
- Cambiar `create`, el esquema `0007`, `DomainError` o `NewRelacion` / `UpdateRelacion`
- Evento de outbox, Meilisearch, HTTP, `openapi.yaml`, `tracing`, UI, crate `catalog` / `recommend`

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `vortex-storage` y `domain-relacion`)

### Modified Capabilities

- `vortex-storage`: `RelationRepo::update` / `delete` / `list_for_product` (grupo bidireccional en una `Tx`; listado saliente agrupado con resumen del destino; REPLACE del stub `unavailable` de T-04-03)
- `domain-relacion`: `list_for_product` hidrata agrupado por tipo y `DestinoResumen` (trabajo diferido en T-04-02)

## Impact

- **Repo:** `vortex-core/crates/storage/src/relacion.rs` (y tests `tests/relacion_repo.rs`); `vortex-core/crates/domain/src/relacion.rs` (tipos de listado + firma del puerto + dummy de test)
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p storage --test relacion_repo`; `cargo test -p domain relacion`; `openspec validate T-04-04`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml`
- **Ticket:** cierra T-04-04 (Listo cuando: cambiar la prioridad desde B modifica ambas aristas)
- **Ámbito:** tres operaciones del puerto ya declarado + tipos de listado; cabe en ~2 h. No se divide el ticket.
