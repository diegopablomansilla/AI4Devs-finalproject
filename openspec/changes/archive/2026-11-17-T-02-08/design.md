# Design

## Context

En `develop`, `domain` expone `Categoria`, `Slug`, `CategoryRepo`, `DomainError` (incl. `duplicate_slug`), `Id<T>`, `Timestamp` y `OutboxEvent` con `OutboxEntity::Marca`. `storage` abre SurrealDB, aplica `0000`–`0002` (`categoria`), y ofrece `Tx` + mapper UNIQUE/`slug` → `duplicate_slug` + `OutboxNotifier`. El adaptador de categorías vive en `storage/src/categoria.rs` como `CategoriaRepo { db: Arc<Db> }`. No hay tabla, entidad ni puerto de marca. Allowlist de `domain` intacta; `storage` ya usa `serde` para hidratar filas. Ver `proposal.md` para motivación y las specs `domain-marca` / `vortex-storage` para el contrato.

Constraints: sin HTTP (T-02-09); ~2 h; `domain` no gana deps; SurrealDB `=2.7.0`; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Maestro plano `marca` de punta a punta hexagonal: migración + entidad + puerto + adaptador con outbox
- Tests de integración que cierran el Listo cuando (alta, edición, baja lógica) y el unhappy `duplicate_slug`
- Copiar el patrón de `CategoriaRepo` sin lógica de árbol

**Non-Goals (diseño):**

- Extraer `Slug` a `domain/src/slug.rs`
- Casos de uso en `catalog`, HTTP, UI, occupancy
- `OutboxOp::Delete` en baja lógica
- Cambiar `DomainError` o el mapper de `Tx`

## Decisions

### 1. Un módulo `marca.rs` en `domain` y reuso de `Slug`

**Elección:** `vortex-core/crates/domain/src/marca.rs` con `Marca`, `MarcaDraft`, `NewMarca`, `UpdateMarca` y `BrandRepo`. `lib.rs` hace `pub mod marca` y `pub use` de esos tipos. El *slug* es el `Slug` ya definido en `categoria.rs` (mismo patrón OpenAPI `BrandCreate.slug` / `CategoryCreate.slug`).

**Por qué:** T-02-02 diseñó `Slug` para no repetirlo en T-02-08. Extraerlo a un archivo propio no cambia comportamiento y se come el presupuesto de ~2 h.

**Alternativa:** newtype `BrandSlug`. Se descartó: duplicaría transliteración y tests. **Alternativa:** mover `Slug` ahora. Se descartó: refactor de `domain-categoria` fuera de alcance.

### 2. `BrandRepo` completo en este ticket (`create`/`get`/`list_all`/`update`/`set_activo`)

**Elección:** el puerto nace con las cinco operaciones. `NewMarca { nombre, slug: Option<Slug> }`. `UpdateMarca { id, nombre, slug: Option<Slug> }` — si `slug` es `None`, el adaptador regenera con `Slug::from_nombre`. `set_activo` es baja/reactivación lógica. Trait con `impl Future + Send` (edición 2024), dummy en `#[cfg(test)]`.

**Por qué:** el Listo cuando pide alta, edición y baja lógica en un solo ticket (a diferencia de categorías, partidas en T-02-02…T-02-04). Incluir `get`/`list_all` evita un hueco para T-02-09. `slug` opcional alinea `BrandCreate`/`BrandUpdate` de `openapi.yaml` sin implementar HTTP.

**Alternativa:** solo `create` ahora. Se descartó: no cubre el Listo cuando. **Alternativa:** `async-trait`. Se descartó: igual que categorías, rustc 1.99.

### 3. Migración `0003_marca.surql` y ajuste de conteo

**Elección:** archivo embebido con `DEFINE … IF NOT EXISTS` idéntico a 4.4.2 (`uq_marca_slug` UNIQUE sobre `slug`, `CHANGEFEED 3d`). Registrar `MIGRATION_0003_SQL` en `EMBEDDED`. Actualizar `tests/migrate.rs` y `tests/harness.rs` de `len() == 3` a 4 y checksum de `version` 3. Tests de esquema en `tests/marca.rs`.

**Por qué:** el motor de migraciones (T-00-09) exige checksum y orden. Índice global `slug` (no `(padre, slug)`) porque marca es plana.

**Alternativa:** meter `marca` en `0002`. Se descartó: checksum drift y mezcla de tickets.

### 4. Adaptador `MarcaRepo` en `storage/src/marca.rs` con `Arc<Db>`

**Elección:** newtype `MarcaRepo { db: Arc<Db> }` que implementa `domain::BrandRepo`. `lib.rs` hace `mod marca` y `pub use marca::MarcaRepo`. Hidratar con DTO `serde` + strip de `marca:` (mismo mapper de datetime que categorías). Escrituras vía `Tx::run` + `emit` + `commit`; luego `get` para timestamps del motor.

**Por qué:** T-02-06 ya exige `'static` + `Send + Sync` en el adaptador de categorías; copiar `Arc<Db>` evita un préstamo de stack. Un archivo, mismo crate que `Tx`.

**Alternativa:** crate `catalog`. Se descartó: `catalog` es casos de uso (T-02-09); el adaptador pertenece a `storage`.

### 5. UNIQUE `slug` → `duplicate_slug` vía mapper existente

**Elección:** el alta y la edición pasan por `Tx::commit`, que ya mapea UNIQUE + `slug` → `duplicate_slug`. No hace falta `SELECT` previo (el índice es sobre `slug` solo, sin `NONE` de padre). Tests unhappy de alta y edición son obligatorios. Si el mapper devolviera `conflict` genérico, endurecer `map_surreal_message` con el SQL acumulado, sin ampliar `DomainError`.

**Por qué:** T-03-02 ya cubre el caso; marca no tiene el edge de raíces con `padre` NONE.

### 6. Baja lógica = `set_activo(false)` + outbox `upsert`

**Elección:** no se usa `OutboxOp::Delete`. La fila permanece; HU-02-02 exige que los productos (futuros) conserven la referencia. El worker T-03-08 reindexa por `entidad` `marca`. No hay occupancy.

**Por qué:** `DELETE` físico rompería FKs futuras y contradice [5.1](../../docs/5-contract-api.md) (baja lógica en maestros). `motivo: None` basta (igual que categorías).

**Alternativa:** `operacion` `delete`. Se descartó: el registro sigue existiendo.

### 7. Tests de integración en `tests/marca.rs`

**Elección:** archivo nuevo (no mezclar con `categoria.rs`). Harness `storage::testing::test_db()`. Nombres de la tabla «Tests previstos» de las specs. Ajuste de `migrate.rs` / `harness.rs` / `outbox.rs` si asertan el conteo de migraciones.

**Por qué:** un `[[test]]` por agregado; CI `cargo test --workspace` los ejecuta. Sin HTTP.

## Risks / Trade-offs

- **[Conteo de migraciones en tests existentes]** → Mitigación: tarea explícita de ajustar `len() == 3` / `version == 2` a incluir `version` 3.
- **[Mensaje UNIQUE sin la palabra `slug`]** → Mitigación: igual que categorías; endurecer el mapper si el test unhappy falla.
- **[Datetime del motor ≠ RFC 3339 `Z`]** → Mitigación: reutilizar el normalizador de `categoria.rs`.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-08` y rename a `T-02-08` (igual que tickets previos).
- **[Ámbito ~2 h]** → Mitigación: marca es plana; un módulo de dominio + un adaptador + una migración. No se divide el ticket.

## Migration Plan

1. Red: tests unitarios de `Marca`/`BrandRepo` y tests de integración de esquema + repo (no compilan / fallan).
2. Green: `0003_marca.surql` + `marca.rs` en `domain` y `storage`; ajustar conteo de migraciones.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-02-08`.
4. Rollback: quitar migración `0003`, módulos y tests. Bases ya migradas a `0003` conservan la tabla vacía (idempotente). Sin API HTTP.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle de interpolar SQL vs bindings de `Tx` se copia del adaptador de categorías en apply.
