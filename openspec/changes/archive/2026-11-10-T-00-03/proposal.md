# Proposal

## Ticket padre

- **Ticket:** [T-00-03](../../../docs/7-tickets.md) — Taxonomía de errores de dominio
- **Change:** `T-00-03`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Depende de:** T-00-01 (cerrado en `develop`)
- **Hace:** `domain::error::DomainError` con `NotFound`, `Conflict`, `InvalidInput` (con lista de `FieldError { field, message }`), `Unavailable` y `Timeout`; enums de `code` alineados con [5.4](../../../docs/5-contract-api.md#54-errores) (`negative_stock`, `category_cycle`, `self_relation`, `version_conflict`, `duplicate_sku`, `duplicate_slug`, `duplicate_relation`, `attribute_in_use`, …)
- **No hace:** traducción a HTTP (T-00-11)
- **Listo cuando:** tests unitarios cubren la construcción de cada variante y su `code`

## Why

El crate `domain` está vacío: no hay una taxonomía compartida para que servicios, repositorios y (más adelante) el Gateway hablen el mismo error. T-00-08, T-00-11, T-02-02, T-03-03 y T-04-02 dependen de `DomainError`. Hay que definirla ahora, con `code` estable alineado a [5.4](../../../docs/5-contract-api.md#54-errores), sin filtrar HTTP al dominio.

## What Changes

- Agregar `domain::error::DomainError` (`thiserror`) con las cinco variantes del ticket y `FieldError { field, message }` en `InvalidInput`
- Enums de `code` de dominio alineados a la tabla de [5.4](../../../docs/5-contract-api.md#54-errores) / `ErrorCode` de `docs/openapi.yaml`, **sin** códigos solo de Gateway (`payload_too_large`, `unsupported_media_type`, `internal`)
- `DomainError::code()` (o equivalente) que devuelve el `code` snake_case de cada construcción
- Declarar `thiserror` en `crates/domain` con `workspace = true`
- Relajar `scripts/check-workspace.sh` **solo** para que `domain` pueda declarar `thiserror`; el resto de libs y `bin/vortex` siguen sin `[dependencies]`
- TDD: happy = construir cada variante y leer su `code`; unhappy = `code` desconocido, códigos HTTP-only y `FieldError` inválido

## Non-goals

Este change **no** cubre:

- Traducción a HTTP / RFC 9457 (`type`, `title`, `status`, `instance`, `Content-Type: application/problem+json`) — **T-00-11** (No hace del ticket)
- Códigos solo de Gateway (`payload_too_large`, `unsupported_media_type`, `internal`) — T-00-11 / T-00-12
- Mensajes de UI por `code` — **T-01-07**
- Tipos base `Id`, `Decimal`, `Timestamp` — **T-00-04**
- Entidades, puertos (`CategoryRepo`, `TextSearch`, …) ni adaptadores — tickets de fase 2+
- `serde` de `DomainError` ni cambios a `docs/openapi.yaml` — el contrato HTTP no se toca
- Relajar `[dependencies]` en otros crates ni en `bin/vortex` — T-00-04 (`ulid` / `rust_decimal`), T-00-07 (`clap`)
- Ampliar la denylist de `check-domain-deps.sh` — `thiserror` no es un SDK de motor

## Capabilities

### New Capabilities

- `domain-error`: taxonomía `DomainError` / `FieldError` / `code` de dominio en el crate `domain`, construible y consultable sin HTTP

### Modified Capabilities

- `cargo-workspace`: `domain` MAY declarar `thiserror` desde `[workspace.dependencies]`; los demás libs y `bin/vortex` siguen sin `[dependencies]` / `[dev-dependencies]`. El pin `thiserror = "=2.0.21"` no cambia.

## Impact

- **Repo:** `vortex-core/crates/domain/` (`Cargo.toml`, `src/lib.rs`, módulo `error`), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`
- **APIs / OpenAPI:** ninguna (los `code` se alinean al enum existente; no se implementa el contrato HTTP)
- **CI / verificación:** `cargo test -p domain`, más [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core`; `check-workspace.sh` y `check-domain-deps.sh` en verde con `thiserror` en `domain`
- **Ticket:** cierra T-00-03 (Listo cuando: cada variante + su `code`)
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `domain` declara `thiserror`; sigue fallando si `domain` declara otra crate o si otro lib declara `[dependencies]`
