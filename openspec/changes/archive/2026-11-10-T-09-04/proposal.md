# Proposal

## Ticket padre

- **Ticket:** [T-09-04](../../../docs/7-tickets.md) — Validador de respuestas contra `openapi.yaml`
- **Change:** `T-09-04`
- **HU:** [HU-09-02](../../../docs/6-historias.md#hu-09-02--validación-del-contrato-de-api)
- **Depende de:** T-09-03 (lint Redocly en CI), T-00-13 (`GET /health`) — ambos en `develop`
- **Hace:** helper de tests en `crates/api/tests/support/openapi.rs`: carga `docs/openapi.yaml`, resuelve `$ref`, obtiene el esquema por método + ruta + estado + *content type* y valida con `jsonschema` (draft 2020-12); `assert_conforms(...)`
- **No hace:** generación de código
- **Listo cuando:** el test de `GET /health` pasa y un campo requerido omitido a propósito lo hace fallar

## Why

PAPI-04 y HU-09-02 piden que los tests de `api` validen cuerpos HTTP contra `docs/openapi.yaml`, sin codegen. El lint de T-09-03 ya rechaza un YAML roto, pero un handler puede devolver JSON que no cumple el esquema y la suite sigue verde. `GET /health` (T-00-13) es el primer endpoint real contra el que se puede cerrar ese contrato; los tickets de catálogo (T-02-06 en adelante) reutilizarán el mismo helper.

## What Changes

- Agregar `vortex-core/crates/api/tests/support/openapi.rs` (módulo `support`) que carga `docs/openapi.yaml`, resuelve `$ref` locales, localiza el schema por método + ruta + estado + content type y valida con `jsonschema` draft 2020-12
- Exponer `assert_conforms(...)` (y un `conforms(...) -> Result` para el unhappy sin `should_panic`)
- Cablear el helper en el test de integración `GET /health` (200; también 503, mismo schema `Health`)
- Test unhappy: un JSON de `Health` sin un campo `required` hace fallar `assert_conforms` / `conforms`
- Permitir `jsonschema` y `serde_yaml` **solo** en `[dev-dependencies]` de `api` (pins exactos; `jsonschema` sin features HTTP/TLS)
- TDD: red = tests que no compilan / allowlist que rechaza las crates; green = helper + allowlist + Listo cuando

## Non-goals

Este change **no** cubre:

- Codegen desde OpenAPI (tipos, clients, utoipa, `openapi-generator`) — **No hace** del ticket / PAPI-04
- Validar **todas** las respuestas actuales del crate `api` (`GET /metrics` `text/plain`, `problem+json` de T-00-11, 404) — HU-09-02 se cierra por endpoint en tickets posteriores (T-02-06, T-02-09, …)
- Mutar `docs/openapi.yaml`, [5.13](../../../docs/5-contract-api.md#513-especificación-openapi-31) o el handler de `/health`
- Job de CI nuevo (el job `test` de T-09-01 ya corre `cargo test --workspace`)
- Lint Redocly — **T-09-03** (ya en `develop`)
- Endpoints de negocio — T-02-06 y siguientes **dependen** de este helper
- Sidecars reales en CI — **T-09-05**, **T-09-13**, **T-09-18**

## Capabilities

### New Capabilities

- `openapi-conformance`: helper de tests que valida un cuerpo JSON contra el schema OpenAPI 3.1 correspondiente (método + ruta + estado + content type, `$ref` resueltos, draft 2020-12); `GET /health` conforme; un required omitido falla

### Modified Capabilities

- `cargo-workspace`: el crate `api` MAY declarar `jsonschema` y `serde_yaml` en `[dev-dependencies]` (pins exactos); MUST NOT declararlas en `[dependencies]`

## Impact

- **Repo:** `vortex-core/crates/api/tests/support/` (nuevo), `crates/api/tests/health.rs` y un test del helper, `crates/api/Cargo.toml` (dev-deps), `vortex-core/Cargo.toml` (pins), `scripts/check-workspace.sh` + fixture happy de allowlist. No toca `docs/openapi.yaml` ni el runtime de `GET /health`
- **APIs / OpenAPI:** ninguna mutación del contrato; solo se **consume** el YAML vigente en tests
- **CI / verificación:** mismos comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) (`cargo fmt --check`, `clippy -D warnings`, `cargo test --workspace`) más `scripts/check-workspace.sh`; `openspec validate T-09-04` (o `t-09-04` si el CLI exige minúsculas)
- **Ticket:** cierra T-09-04 (Listo cuando: `GET /health` pasa la validación y un required omitido falla)
- **Dependientes:** T-02-06 / T-02-09 / T-02-12 / T-02-21 / T-03-13 / T-04-05 podrán llamar `assert_conforms` sin reescribir el loader
