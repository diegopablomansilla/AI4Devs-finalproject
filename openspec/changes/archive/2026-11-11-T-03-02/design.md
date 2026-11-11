# Design

## Context

`storage` abre SurrealDB, aplica `0000` + `0001_outbox.surql` y expone `Db::query` / `record_count`. No hay `Tx`, ni tipos de outbox en `domain`, ni dependencia `storage → domain`. `check-workspace.sh` solo permite `{surrealdb, thiserror, sha2}` en `[dependencies]` de `storage`. Ver `proposal.md` para motivación.

SurrealDB `=2.7.0`. `DomainError` (T-00-03) ya tiene `negative_stock`, `duplicate_sku` / `duplicate_slug` / `duplicate_relation` y `conflict`. `Id<T>` rechaza prefijo `tabla:`.

## Goals / Non-Goals

**Goals:** helper atómico dato+outbox; mapper UNIQUE/THROW → `DomainError` existente; `Notify` post-commit; allowlist de `storage` para `domain` + `tokio`.

**Non-Goals:** repositorios, worker, cursores, métricas, HTTP. Ver Non-goals del proposal.

## Decisions

### 1. `OutboxEvent` en `domain` con enums + ULID sin prefijo

`OutboxEntity` y `OutboxOp` como enums (`producto`…`imagen_producto`, `upsert`/`delete`). `registro` se valida como ULID Crockford de 26 caracteres (reutilizar la regla de `Id<T>`). `motivo: Option<String>`. Constructor `try_new` (o equivalente) falla en prefijo `tabla:` y en parse de entidad/operación.

**Por qué:** el ticket pide el tipo en `domain`; `registro` tipo `record` en Surreal es `tabla:ulid` y el ULID sin prefijo es la convención de `Id`. Incluir valores v2 en el enum igual que el `ASSERT` de `0001` (T-03-01) evita un cambio de tipo cuando lleguen T-03-24 / T-02-28.

**Alternativa:** `registro: String` opaco `producto:ulid`. Se descarta: filtra el prefijo Surreal al dominio. **Alternativa:** solo entidades MVP. Se descarta: el esquema 4.8 ya lista las siete.

### 2. `storage` depende de `domain`; `Tx` vive en `storage`

`crates/storage/Cargo.toml`: `domain = { path = "../domain" }`. `Db::tx()` (o `Tx::new(&db)`) acumula `(sql, bindings)` y `Vec<OutboxEvent>`. `commit` arma un único script `BEGIN TRANSACTION; … CREATE outbox …; COMMIT TRANSACTION;` con parámetros bound (`$entidad`, `$registro`, …). El `CREATE` usa `type::thing($entidad, $registro)` (o equivalente 2.7) para el `record`. Id del evento: ULID (`CREATE type::thing('outbox', $eid)` o `rand::ulid()` en SurrealQL) para preservar el orden de 4.8.

**Por qué:** el diagrama 3.2 ya tiene `STO → DOM`; el ticket nombra `storage::Tx`. Un query único con BEGIN/COMMIT es el patrón de [4.5.2](../../../docs/4-modelo.md#452-registro-de-un-movimiento-transacción).

**Alternativa:** puerto `OutboxRepo` en domain. Se descarta: **No hace** repositorios; T-03-07 lee el outbox. **Alternativa:** N queries sueltas sin BEGIN. Se descarta: no es atómico.

`Db::query` (tests de esquema) sigue devolviendo `OpenError`. Solo `Tx::commit` habla `DomainError`.

### 3. Mapper UNIQUE / THROW sin ampliar `DomainError`

Función interna `map_surreal_error(&SurrealError) -> DomainError`:

| Señal | Resultado |
|---|---|
| texto `stock_negativo` (THROW 4.5.2) | `InvalidInput` / `negative_stock` |
| THROW cuyo texto es un `DomainCode` conocido (`negative_stock`, `category_cycle`, …) | esa variante |
| UNIQUE + `sku` en índice o mensaje | `Conflict` / `duplicate_sku` |
| UNIQUE + `slug` | `duplicate_slug` |
| UNIQUE + `relation` | `duplicate_relation` |
| UNIQUE resto | `Conflict` / `conflict` |
| otro | `Unavailable` / `unavailable` |

Los tests de UNIQUE definen una tabla auxiliar `DEFINE INDEX … UNIQUE` cuyo nombre incluye `sku` (no `producto`). THROW se prueba con `THROW "stock_negativo"` en la misma `Tx` que un `CREATE` auxiliar + `emit`.

**Por qué:** [5.4](../../../docs/5-contract-api.md#54-errores) ya fija el origen de esos códigos. T-02-24 reutilizará el mismo THROW.

**Alternativa:** THROW JSON `{ code: "…" }`. Se descarta: 4.5.2 usa string. **Alternativa:** nuevas variantes. Se descarta: T-00-03 cerrado.

### 4. `OutboxNotifier` en `storage`, no en `domain`

`Arc<OutboxNotifier>` dentro de `Db` (`tokio::sync::Notify`). Tras `COMMIT` OK: `notify_waiters()`. Si `commit` falla, no se llama. Tests: `tokio::sync::Notify` + `timeout` corto para el unhappy (el waiter no debe completar).

**Por qué:** el ticket pide `tokio::sync::Notify`; `domain` no puede depender de tokio (infra). T-03-07 en `sync` ya depende de `storage`.

**Alternativa:** trait `OutboxPort::notify` en domain. Se descarta: sobre-diseño para ~2 h; el ticket nombra el tipo concreto. **Alternativa:** canal `mpsc`. Se descarta: `Notify` basta (el worker relee el outbox; no necesita el payload).

`tokio` en `[dependencies]` de `storage` con feature `sync`; se mantiene en `[dev-dependencies]` para `macros` / `rt-multi-thread`.

### 5. Tests contra `test_db()`, tabla auxiliar

`tests/tx.rs` usa `storage::testing::test_db()`. Happy: `DEFINE TABLE scratch SCHEMAFULL` + campo + `Tx` CREATE + emit. Unhappy UNIQUE: índice `uq_scratch_sku` UNIQUE. No se tocan `categoria`/`producto`. Tests de `OutboxEvent` son unidad en `domain` (sin Surreal).

Allowlist: ampliar `storage_deps_allowed` y el fixture happy de `scripts/fixtures/workspace/` (el `Cargo.toml` real de `storage` pasará a declarar `domain` + `tokio`). Unhappy sigue siendo `axum`.

### 6. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-03-02` y rename a `T-03-02` (igual que T-03-01).

## Risks / Trade-offs

- **[Dialect SurrealQL 2.7: BEGIN/COMMIT + `type::thing` + mensaje UNIQUE]** → Mitigación: el test de integración es la prueba; ajustar SQL/parser del mapper sin cambiar la spec si el texto del motor difiere (`already exists`, nombre de índice, etc.).
- **[Notify espurio si un waiter llega tarde]** → Mitigación: `Notify` no guarda tokens; T-03-07 combina notify + poll 1 s. Los tests arrancan el waiter **antes** del commit (o usan `notified()` previo).
- **[Allowlist: `tokio` en dependencies y dev-dependencies]** → Mitigación: `storage_deps_allowed` acepta `tokio` en ambos; el script de fixtures happy copia o replica el `Cargo.toml` real.
- **[Ámbito ~2 h]** → Mitigación: sin repositorios, sin worker, tabla auxiliar; no se divide el ticket.

## Migration Plan

Ninguna migración `.surql` nueva. Bases con `0001` ya aplicado siguen. `storage` pasa a enlazar `domain`; `vortex serve` no cambia comportamiento (aún no usa `Tx`). Rollback de esquema: N/A. El change de allowlist es local al repo (CI `check-workspace.sh`).
