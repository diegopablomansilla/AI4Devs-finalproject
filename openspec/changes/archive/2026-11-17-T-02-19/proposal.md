# Proposal

## Ticket padre

- **Ticket:** [T-02-19](../../../docs/7-tickets.md) — Repositorio de producto: ficha y edición con `version`
- **Change:** `T-02-19`
- **HU:** [HU-02-05](../../../docs/6-historias.md#hu-02-05--edición-con-bloqueo-optimista)
- **Depende de:** T-02-18 (en `develop`: `ProductoRepo::create` transaccional con movimiento `inicial` + outbox; `get` básico con IDs, sin ficha resuelta; puerto `ProductRepo` sin `update`)
- **Hace:** `get` con marca, ruta de categoría y atributos resueltos (clave, etiqueta, unidad); `update` con `WHERE version = $v` y `version += 1`, reemplazando `atributos` y `codigos` completos; sin filas afectadas → `version_conflict`; nunca escribe `stock`.
- **No hace:** imágenes en la ficha (se agregan en T-02-30).
- **Listo cuando:** test de dos ediciones concurrentes con la misma versión: una pasa y la otra da `version_conflict`.

## Why

HU-02-05 exige editar la ficha sin pisar a otra terminal: el cliente envía la `version` leída y el núcleo hace `UPDATE … SET …, version += 1 WHERE version = $v`; cero filas → `version_conflict`. El alta (T-02-18) ya persiste el producto, pero `get` solo hidrata IDs y el puerto no declara `update`. Sin ficha resuelta (marca, ruta de categoría, clave/etiqueta/unidad de atributos) ni bloqueo optimista en SurrealDB, T-02-20/T-02-21 no pueden devolver `GET /products/{id}` ni `PATCH` con 409.

## What Changes

- **BREAKING** (puerto): `ProductRepo::get` pasa a devolver `FichaProducto` (producto + marca opcional hidratada + `Categoria` con `ruta` + atributos con `clave`/`etiqueta`/`unidad`). `create` sigue devolviendo `Producto`
- Puerto `ProductRepo::update` + input `UpdateProducto` (`id`, `version` esperada, campos de ficha; **sin** `stock` ni `activo`)
- Adaptador `ProductoRepo::update` en una `Tx`: `UPDATE … SET …, version += 1 WHERE version = $v`; 0 filas → `DomainError::Conflict` `version_conflict`; `atributos` y `codigos` se reemplazan enteros; MUST NOT escribir `stock`; outbox `producto`/`upsert`
- `get` carga `Marca` (si hay), `Categoria` (ruta) y defs de atributo ya usadas para `Producto::try_new`; MUST NOT unir `imagen_producto`
- Tests de integración en `crates/storage/tests/producto.rs` (Listo cuando: dos `update` con la misma `version`; unhappy `version_conflict`, `not_found`, `duplicate_sku` al cambiar SKU; happy ficha resuelta y stock intacto)
- Tests unitarios del puerto en `domain` (dummy con `update`; `get` → `FichaProducto`)
- TDD: red = tests que no compilan / fallan; green = tipos de ficha + `update` + hidratación

## Non-goals

- Casos de uso en `catalog` (existencia/activo de categoría y marca; merge del PATCH parcial; validación de atributos contra defs cargadas) — **T-02-20**
- HTTP `GET`/`PATCH /api/v1/products/{productId}`, mapeo inglés ↔ español, `problem+json` 409 — **T-02-21**
- Baja/reactivación (`activo`, `set_activo`) — **T-02-22**
- Imágenes en la ficha (`images[]`, `imagen_producto`) — **T-02-30** (v2; No hace del ticket)
- `StockLedger.append` / ingresos y ajustes — **T-02-24**
- Listado admin paginado — **T-02-27** / **T-00-21**
- Worker Meilisearch / aparición en búsqueda — **T-03-07**
- Ampliar `DomainError` (`version_conflict` ya existe en **T-00-03**); no se toca `error.rs` salvo usarlo
- HTTP, `openapi.yaml`, `tracing`, UI, crate `catalog`

## Capabilities

### New Capabilities

- (ninguna carpeta nueva; extiende `domain-producto` y `vortex-storage`)

### Modified Capabilities

- `domain-producto`: `FichaProducto` / `AtributoFicha` / `UpdateProducto`; `ProductRepo::get` hidratado; `ProductRepo::update`; MUST NOT `set_activo` ni listado
- `vortex-storage`: `get` resuelve marca, ruta de categoría y atributos; `update` con bloqueo optimista (`version_conflict`); no escribe `stock`; outbox en la misma `Tx`

## Impact

- **Repo:** `vortex-core/crates/domain/src/producto.rs` (tipos, trait, dummy, reexports en `lib.rs`); `vortex-core/crates/storage/src/producto.rs` (`get` hidratado, `update`); mapper `version_conflict` en `storage/src/tx.rs` si el `THROW` de Surreal lo requiere; tests en `crates/storage/tests/producto.rs` y unitarios de `domain`
- **Verificación:** [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`; `cargo test -p domain producto`; `cargo test -p storage --test producto`; `openspec validate T-02-19`
- **CI:** job `test` existente (T-09-01) ejecuta `cargo test --workspace`
- **Sin** cambios de API HTTP ni de `openapi.yaml` ni de `ConflictCode` (ya incluye `version_conflict`)
- **Ticket:** cierra T-02-19 (Listo cuando: dos ediciones con la misma `version` → una ok, la otra `version_conflict`)
- **Ámbito:** extender el adaptador de T-02-18 con `get` hidratado + un `UPDATE` condicionado; cabe en ~2 h. No se divide el ticket.
