# Proposal

## Ticket padre

- **Ticket:** [T-00-04](../../../docs/7-tickets.md) — Tipos base de dominio (ID, decimal, fecha)
- **Change:** `T-00-04`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Depende de:** T-00-01 (cerrado en `develop`; T-00-03 ya dejó `thiserror` en `domain`)
- **Hace:** `Id<T>` (ULID de 26 caracteres, sin prefijo de tabla), `Decimal` (`rust_decimal`, serializado como string), `Timestamp` (UTC, RFC 3339)
- **No hace:** entidades de negocio
- **Listo cuando:** tests de *parse*/serialización de ida y vuelta, incluidos casos inválidos

## Why

El crate `domain` ya tiene `DomainError` pero no los tipos con los que T-02-02, T-02-16 y T-02-17 van a modelar identidades, dinero/stock y fechas. El contrato ([5.1](../../../docs/5-contract-api.md#51-convenciones-generales), esquemas `Ulid` / `Decimal` / `Timestamp` en `docs/openapi.yaml`) ya fija ULID Crockford de 26 caracteres **sin** prefijo de tabla, decimales como string y RFC 3339 en UTC. Hay que materializarlos ahora como newtypes parseables, antes de cualquier entidad.

## What Changes

- Agregar `domain::Id<T>`, `domain::Decimal` y `domain::Timestamp` (módulos dedicados; reexport en `lib.rs`)
- `Id<T>`: ULID Crockford de 26 caracteres, sin prefijo `tabla:`; parse, `Display` y serde como string; `PhantomData<T>` para no mezclar IDs de entidades distintas
- `Decimal`: newtype sobre `rust_decimal::Decimal`; parse y serde **solo** como string, alineados al patrón OpenAPI (`^-?\d{1,12}(\.\d{1,4})?$`)
- `Timestamp`: instante UTC; parse y serde RFC 3339; serialización canónica con sufijo `Z`
- Declarar en `crates/domain` (con `workspace = true`): `ulid`, `rust_decimal`, `serde`; `serde_json` solo en `[dev-dependencies]` para los tests de ida y vuelta JSON
- Ampliar `scripts/check-workspace.sh` y la fixture `scripts/fixtures/workspace/run-script-tests.sh` para esa allowlist; el resto de libs y `bin/vortex` siguen sin deps
- TDD: happy = parse/serialización ida y vuelta (string y JSON); unhappy = prefijo de tabla, ULID inválido, JSON number, decimal/timestamp fuera de contrato

## Non-goals

Este change **no** cubre:

- Entidades de negocio (`Categoria`, `Producto`, `MovimientoStock`, …) ni puertos (`CategoryRepo`, `StockLedger`) — **T-02-02**, **T-02-16**, **T-02-17** (No hace del ticket)
- Newtype `NonNegativeDecimal` (precio/stock ≥ 0) — validación de entidad en tickets de catálogo / stock
- Prefijo SurrealDB `tabla:<ulid>` en el tipo de dominio — lo traduce `storage` al persistir (T-00-08 / repositorios)
- `chrono` / `time` como crate nueva del stack — T-00-01 no las pinnea; el parse UTC queda en `domain`
- Traducción HTTP / `problem+json` — **T-00-11**
- Configuración TOML, CLI, SurrealDB, migraciones, Axum — T-00-05 … T-00-10
- Relajar `[dependencies]` en otros crates ni en `bin/vortex`
- Ampliar la denylist de `check-domain-deps.sh` — `ulid` / `rust_decimal` / `serde` no son SDKs de motor
- Cambios a `docs/openapi.yaml` — los tipos se alinean a esquemas ya existentes

## Capabilities

### New Capabilities

- `domain-types`: newtypes `Id<T>`, `Decimal` y `Timestamp` en el crate `domain`, con parse/serialización de ida y vuelta (string y JSON) y rechazo de entradas inválidas

### Modified Capabilities

- `cargo-workspace`: `domain` MAY declarar `thiserror`, `ulid`, `rust_decimal` y `serde` en `[dependencies]`, y `serde_json` en `[dev-dependencies]`, todas con `workspace = true`. Los demás libs y `bin/vortex` siguen sin `[dependencies]` / `[dev-dependencies]`. Los pins existentes no cambian; se agrega el pin exacto de `serde_json` (solo para tests de `domain`).

## Impact

- **Repo:** `vortex-core/crates/domain/` (`Cargo.toml`, `src/lib.rs`, módulos de tipos), `vortex-core/Cargo.toml` / `Cargo.lock` (pin `serde_json`), `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna (alineación a `Ulid`, `Decimal`, `Timestamp` ya definidos; no se implementa el contrato HTTP)
- **CI / verificación:** `cargo test -p domain`, más [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core`; `check-workspace.sh` y `check-domain-deps.sh` en verde con las nuevas deps de `domain`
- **Ticket:** cierra T-00-04 (Listo cuando: parse/serialización ida y vuelta, incluidos inválidos)
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `domain` declara `ulid` / `rust_decimal` / `serde` (y `serde_json` en dev); sigue fallando si `domain` declara otra crate o si otro lib declara deps
