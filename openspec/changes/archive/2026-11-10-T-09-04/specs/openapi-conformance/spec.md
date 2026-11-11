# Spec Delta

Ticket padre: T-09-04. Change: T-09-04. HU: HU-09-02.

## Purpose

Define el helper de tests que valida cuerpos JSON de la API contra el schema OpenAPI 3.1 de `docs/openapi.yaml` (sin codegen), para que `GET /health` y los endpoints posteriores no divergjan del contrato.

## ADDED Requirements

### Requirement: Helper de conformidad en tests de api

El crate `api` MUST exponer un helper de tests (no de runtime) que carga `docs/openapi.yaml`, resuelve `$ref` locales del documento, obtiene el schema JSON Schema por método HTTP + ruta + estado + content type, y valida un valor JSON contra ese schema con JSON Schema draft 2020-12. MUST exponer `assert_conforms(method, path, status, content_type, body)` que falla el test si el cuerpo no cumple el schema o si no hay schema para esa combinación. MUST NOT generar código Rust ni tipos a partir del YAML. MUST NOT formar parte de `[dependencies]` del crate `api`.

#### Scenario: GET /health 200 conforme (happy)

- **WHEN** se envía `GET /health` al router de `build_router` con agregado `ok`
- **AND** se llama a `assert_conforms` con método `GET`, ruta `/health`, estado 200, content type `application/json` y el cuerpo JSON de la respuesta
- **THEN** la aserción no falla
- **AND** el test termina en verde

#### Scenario: GET /health 503 conforme (happy)

- **WHEN** se envía `GET /health` al router de `build_router` con agregado `starting` o `down`
- **AND** se llama a `assert_conforms` con método `GET`, ruta `/health`, estado 503, content type `application/json` y el cuerpo JSON de la respuesta
- **THEN** la aserción no falla
- **AND** `Content-Type` no es `application/problem+json`

#### Scenario: campo required omitido hace fallar (unhappy)

- **WHEN** se valida contra el schema de `GET /health` 200 `application/json` un JSON que omite a propósito un campo `required` del esquema `Health` (p. ej. `version`)
- **THEN** `assert_conforms` (o `conforms`) falla
- **AND** el mensaje nombra el campo omitido o el keyword `required`

#### Scenario: operación o content type desconocidos (unhappy)

- **WHEN** se pide el schema para un método + ruta + estado + content type que no existen en `docs/openapi.yaml` (p. ej. `POST /health` 200 `application/json`)
- **THEN** `assert_conforms` (o `conforms`) falla
- **AND** el mensaje nombra la combinación buscada

### Requirement: Carga de docs/openapi.yaml y resolución de $ref

El helper MUST leer el archivo `docs/openapi.yaml` del repositorio (el mismo path que [5.13](../../../../docs/5-contract-api.md#513-especificación-openapi-31) y T-09-03). MUST resolver `$ref` JSON Pointer locales (`#/components/schemas/...`, `#/components/responses/...`) antes o durante la validación, de modo que el schema `Health` (con `ComponentHealth` / `ComponentStatus` anidados) se aplique completo. MUST coincidir la ruta del request con un path item por igualdad exacta o sustituyendo un segmento por cada parámetro `{name}`. El content type MUST compararse por tipo MIME sin parámetros (p. ej. `application/json; charset=utf-8` equivale a `application/json`). MUST NOT mutar `docs/openapi.yaml`.

#### Scenario: el YAML vigente se carga (happy)

- **WHEN** un test de `api` invoca el helper por primera vez en ese proceso
- **THEN** se lee `docs/openapi.yaml` relativo a la raíz del repositorio
- **AND** el schema de `GET /health` 200 `application/json` es el esquema `Health` (tras resolver `$ref`)

#### Scenario: falta el archivo OpenAPI (unhappy)

- **WHEN** el helper no puede leer `docs/openapi.yaml`
- **THEN** el test falla
- **AND** el mensaje nombra `openapi.yaml`

## Tests previstos

| Test | Crate / archivo | Tipo | Escenarios |
|---|---|---|---|
| `get_health_ok_returns_200` + `assert_conforms` | `crates/api/tests/health.rs` | integración | GET /health 200 conforme |
| `get_health_starting_returns_503` (o equivalente) + `assert_conforms` | `crates/api/tests/health.rs` | integración | GET /health 503 conforme |
| JSON `Health` sin `version` (u otro required) | `crates/api/tests/openapi_conformance.rs` | unitario del helper | campo required omitido hace fallar |
| `POST /health` 200 `application/json` | `crates/api/tests/openapi_conformance.rs` | unitario del helper | operación o content type desconocidos |
| `scripts/check-workspace.sh` sobre el repo | script | happy | allowlist openapi (dev-deps) |
| Fixture `api` con `jsonschema` en `[dependencies]` | `scripts/fixtures/workspace` | unhappy | crate de validación en runtime |

Este change MUST NOT modificar `docs/openapi.yaml`, el handler de `GET /health` ni agregar codegen. Los tests de `problem+json` y `GET /metrics` no se recablean aquí.
