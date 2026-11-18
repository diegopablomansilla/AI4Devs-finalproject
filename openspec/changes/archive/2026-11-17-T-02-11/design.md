# Design

## Context

En `develop`, `domain` expone `Categoria`, `Marca`, `Slug`, `CategoryRepo`, `BrandRepo`, `DomainError` (incl. `conflict`), `Id<T>`, `Timestamp` y `OutboxEvent` con `OutboxEntity::AtributoDef`. `storage` abre SurrealDB, aplica `0000`–`0003` (`marca`), y ofrece `Tx` + mapper UNIQUE → `ConflictCode` + `OutboxNotifier`. Los adaptadores de categorías y marcas viven en `storage` como `*Repo { db: Arc<Db> }`. No hay tabla, entidad ni puerto de `atributo_def`. Allowlist de `domain` intacta. Ver `proposal.md` para motivación y las specs `domain-atributo-def` / `vortex-storage` para el contrato.

Constraints: sin HTTP (T-02-12); sin `attribute_in_use` (T-02-23); ~2 h; `domain` no gana deps; SurrealDB `=2.7.0`; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Maestro plano `atributo_def` de punta a punta hexagonal: migración + entidad + puerto + adaptador con outbox
- Tests que cierran el Listo cuando: enum sin opciones → `invalid_input`; clave duplicada → `conflict` (409)
- Copiar el patrón de `MarcaRepo` (T-02-08) con validación extra de `tipo` / `opciones` / `clave`

**Non-Goals (diseño):**

- Nuevo `code` `duplicate_key` en `DomainError` / `openapi.yaml`
- Casos de uso en `catalog`, HTTP, UI, occupancy
- `OutboxOp::Delete` en baja lógica
- Mapeo inglés API ↔ español modelo (T-02-12)

## Decisions

### 1. Un módulo `atributo_def.rs` en `domain` y `TipoAtributo`

**Elección:** `vortex-core/crates/domain/src/atributo_def.rs` con `AtributoDef`, `AtributoDefDraft`, `NewAtributoDef`, `UpdateAtributoDef`, `TipoAtributo` (`Texto` / `Numero` / `Booleano` / `Enum` serializados como `texto` / `numero` / `booleano` / `enum`) y `AttributeDefRepo`. `lib.rs` hace `pub mod atributo_def` y `pub use` de esos tipos. Campos en español ([5.3](../../docs/5-contract-api.md#53-correspondencia-api--modelo-de-datos)).

**Por qué:** el modelo 4.4.3 y Surreal usan español; T-02-12 traduce a `key`/`label`/`text`/`searchable`. Un enum Rust impide `tipo` inválido en dominio; el `ASSERT` de la migración cubre escrituras crudas.

**Alternativa:** strings sueltas para `tipo`. Se descartó: no cierra el unhappy de esquema vs dominio. **Alternativa:** nombres de API en la entidad. Se descartó: rompe la convención de `Categoria`/`Marca`.

### 2. `AttributeDefRepo` completo (`create`/`get`/`list_all`/`update`/`set_activo`)

**Elección:** el puerto nace con las cinco operaciones, igual que `BrandRepo`. `NewAtributoDef` / `UpdateAtributoDef` llevan `clave`, `etiqueta`, `tipo`, `unidad`, `opciones`, `facetable`, `buscable`, `orden` (sin PATCH parcial; T-02-12 fusiona el body OpenAPI). `set_activo` es baja/reactivación lógica. Trait con `impl Future + Send` (edición 2024), dummy en `#[cfg(test)]`. `list_all` ordena por `orden` ascendente y `clave` como desempate.

**Por qué:** el Listo cuando pide validación de alta; incluir get/list/update/baja evita un hueco para T-02-12. OpenAPI describe el listado ordenado por `order`; hacerlo en el puerto ahorra lógica en `catalog`. Permitir cambiar `clave`/`tipo` ahora (sin productos) deja T-02-23 para `attribute_in_use`.

**Alternativa:** solo `create`. Se descartó: no habilita T-02-12. **Alternativa:** `async-trait`. Se descartó: igual que marcas, rustc 1.99.

### 3. Validación de `clave` y `opciones` en dominio

**Elección:** `clave` MUST matchear `^[a-z][a-z0-9_]*$`. `enum` exige `opciones` con ≥ 1 valor no vacío; `None` o `[]` → `invalid_input` con `FieldError` en `opciones`. Tipos no-enum con opciones no vacías → `invalid_input`. `etiqueta` 1..=60; `unidad` opcional ≤ 16. Trim de `etiqueta` y de cada opción.

**Por qué:** el ticket nombra explícitamente `snake_case` y "enum sin opciones". El patrón OpenAPI es más estricto que un `snake_case` laxo (empieza por letra); alinearlo ahora evita doble validación en T-02-12.

**Alternativa:** validar solo en el adaptador. Se descartó: el Listo cuando es de dominio (`invalid_input`) y T-02-16 reutilizará la entidad.

### 4. Migración `0004_atributo_def.surql` y ajuste de conteo

**Elección:** archivo embebido con `DEFINE … IF NOT EXISTS` idéntico a 4.4.3 (`uq_atributo_clave` UNIQUE sobre `clave`, `CHANGEFEED 3d`, `ASSERT` de `tipo`). Registrar `MIGRATION_0004_SQL` en `EMBEDDED`. Actualizar `tests/migrate.rs` y `tests/harness.rs` de `len() == 4` a 5 y checksum de `version` 4. Tests de esquema en `tests/atributo_def.rs`.

**Por qué:** el motor de migraciones (T-00-09) exige checksum y orden. No mezclar con `0003_marca`.

**Alternativa:** meter `atributo_def` en `0003`. Se descartó: checksum drift y mezcla de tickets.

### 5. Adaptador `AtributoDefRepo` en `storage/src/atributo_def.rs` con `Arc<Db>`

**Elección:** newtype `AtributoDefRepo { db: Arc<Db> }` que implementa `domain::AttributeDefRepo`. `lib.rs` hace `mod atributo_def` y `pub use atributo_def::AtributoDefRepo`. Hidratar con DTO `serde` + strip de `atributo_def:`. Escrituras vía `Tx::run` + `emit` + `commit`; luego `get` para timestamps del motor.

**Por qué:** copiar `MarcaRepo` evita un préstamo de stack y mantiene el adaptador en `storage`.

**Alternativa:** crate `catalog`. Se descartó: `catalog` es casos de uso (T-02-12).

### 6. UNIQUE `clave` → `conflict` vía mapper existente (sin `duplicate_key`)

**Elección:** el Listo cuando dice "clave duplicada → 409". En dominio eso es `DomainError::Conflict` con `code` `conflict` (tabla [5.4](../../docs/5-contract-api.md#54-errores) / `ErrorCode`). `Tx::commit` ya mapea UNIQUE sin token `sku`/`slug`/`relation` al `else` → `ConflictCode::Conflict`. T-02-12 mapeará `conflict` a HTTP 409. No se añade `duplicate_key` a `DomainError` ni a `openapi.yaml`. Si el mapper devolviera `unavailable`, endurecer `map_surreal_message` para reconocer `clave` / `uq_atributo_clave` y devolver `Conflict`, sin ampliar la taxonomía.

**Por qué:** no hay `duplicate_key` en el contrato. Inventarlo sería un change de `domain-error` + OpenAPI fuera de alcance. Reusar `duplicate_slug` mentiría el `code`.

**Alternativa:** nuevo `code` `duplicate_key`. Se descartó: Non-goal; rompe T-00-03. **Alternativa:** `duplicate_slug`. Se descartó: el índice no es `slug`.

### 7. Baja lógica = `set_activo(false)` + outbox `upsert`

**Elección:** no se usa `OutboxOp::Delete`. La fila permanece. El worker T-03-08 reindexa por `entidad` `atributo_def`. No hay occupancy.

**Por qué:** `DELETE` físico rompería FKs futuras (`producto.atributos[*].def`) y contradice [5.1](../../docs/5-contract-api.md) (baja lógica en maestros).

### 8. Tests de integración en `tests/atributo_def.rs`

**Elección:** archivo nuevo (no mezclar con `marca.rs`). Harness `storage::testing::test_db()`. Nombres de la tabla «Tests previstos» de las specs. Ajuste de `migrate.rs` / `harness.rs` si asertan el conteo de migraciones.

**Por qué:** un `[[test]]` por agregado; CI `cargo test --workspace` los ejecuta. Sin HTTP.

## Risks / Trade-offs

- **[Conteo de migraciones en tests existentes]** → Mitigación: tarea explícita de ajustar `len() == 4` / incluir `version` 4.
- **[Mensaje UNIQUE sin la palabra `clave`]** → Mitigación: el `else` del mapper ya produce `conflict`; si el unhappy falla, reconocer `uq_atributo_clave` en `map_surreal_message`.
- **[Datetime del motor ≠ RFC 3339 `Z`]** → Mitigación: reutilizar el normalizador de `marca.rs` / `categoria.rs`.
- **[`openspec new change` exige kebab-case]** → Scaffold `t-02-11` y rename a `T-02-11` (igual que tickets previos).
- **[Ámbito ~2 h]** → Mitigación: mismo molde que T-02-08 más validación de tipo/opciones. No se divide el ticket.

## Migration Plan

1. Red: tests unitarios de `AtributoDef`/`AttributeDefRepo` y tests de integración de esquema + repo (no compilan / fallan).
2. Green: `0004_atributo_def.surql` + `atributo_def.rs` en `domain` y `storage`; ajustar conteo de migraciones.
3. Cierre: § 5 en `vortex-core/`, `check-domain-deps.sh` en 0, `openspec validate T-02-11`.
4. Rollback: quitar migración `0004`, módulos y tests. Bases ya migradas a `0004` conservan la tabla vacía (idempotente). Sin API HTTP.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El detalle de interpolar SQL vs bindings de `Tx` se copia del adaptador de marcas en apply.
