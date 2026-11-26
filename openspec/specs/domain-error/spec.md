# domain-error Specification

## Purpose

Define la taxonomía compartida de errores de dominio (`NotFound`, `Conflict`, `InvalidInput`, `Unavailable`, `Timeout`) con `code` estable alineado al contrato, para que servicios y adaptadores fallen igual sin filtrar HTTP al crate `domain`.

Ticket padre: T-00-03. Change: T-00-03. HU: HU-00-01.

## Requirements

### Requirement: Variantes de DomainError

El crate `domain` MUST exponer `domain::error::DomainError` con exactamente estas variantes: `NotFound`, `Conflict`, `InvalidInput`, `Unavailable` y `Timeout`. `InvalidInput` MUST incluir una lista de `FieldError` con `field` y `message`. `DomainError` MUST implementar `std::error::Error`. El crate MUST NOT mapear esas variantes a estado HTTP, `problem+json` ni `Content-Type`.

#### Scenario: Cada variante se construye

- **WHEN** se construye cada una de las cinco variantes
- **THEN** la construcción termina sin error
- **AND** el valor es esa variante

#### Scenario: InvalidInput conserva FieldError

- **WHEN** se construye `InvalidInput` con un `FieldError` cuyo `field` es `attributes[1].value` y un `message` no vacío
- **THEN** la lista expuesta contiene exactamente ese par `field` / `message`

#### Scenario: InvalidInput sin errores de campo

- **WHEN** se construye `InvalidInput` con lista de `FieldError` vacía y `code` `invalid_input`
- **THEN** la construcción termina sin error
- **AND** la lista expuesta está vacía

#### Scenario: DomainError es std::error::Error

- **WHEN** se usa un `DomainError` construido como `std::error::Error`
- **THEN** el valor implementa ese trait (se puede tratar como error de biblioteca)

### Requirement: Codes de dominio alineados con el contrato 5.4

Cada `DomainError` MUST exponer un `code` en snake_case. El `code` MUST coincidir con un valor de dominio de la tabla de [5.4](../../../docs/5-contract-api.md#54-errores) / esquema `ErrorCode` de `docs/openapi.yaml`, según esta asignación:

| Variante | `code` admitidos |
|---|---|
| `InvalidInput` | `invalid_input`, `self_relation`, `category_cycle`, `negative_stock` |
| `NotFound` | `not_found` |
| `Conflict` | `conflict`, `version_conflict`, `duplicate_sku`, `duplicate_slug`, `duplicate_relation`, `attribute_in_use` |
| `Unavailable` | `unavailable`, `ai_unavailable`, `speech_unavailable` |
| `Timeout` | `timeout` |

El conjunto de `code` de dominio MUST NOT incluir `payload_too_large`, `unsupported_media_type` ni `internal`. Un `code` MUST NO poder asociarse a una variante distinta de la fila de la tabla.

#### Scenario: Code de cada variante (happy)

- **WHEN** se construye `NotFound`, `Timeout`, `Conflict` con `version_conflict`, `InvalidInput` con `negative_stock` y `Unavailable` con `ai_unavailable`
- **THEN** el `code` expuesto es `not_found`, `timeout`, `version_conflict`, `negative_stock` y `ai_unavailable` respectivamente

#### Scenario: Todos los codes de dominio son construibles

- **WHEN** se construye un `DomainError` por cada `code` de la tabla de este requirement
- **THEN** cada construcción termina sin error
- **AND** el `code` expuesto es exactamente el valor pedido

#### Scenario: Code desconocido (unhappy)

- **WHEN** se intenta obtener un `code` de dominio desde la cadena `no_existe`
- **THEN** la operación falla
- **AND** no se produce un `DomainError`

#### Scenario: Code solo de Gateway (unhappy)

- **WHEN** se intenta obtener un `code` de dominio desde `payload_too_large`, `unsupported_media_type` o `internal`
- **THEN** la operación falla
- **AND** no se produce un `DomainError`

#### Scenario: Code de otra variante (unhappy)

- **WHEN** se intenta construir `InvalidInput` con `code` `version_conflict` (u otro `code` que la tabla asigna a `Conflict`)
- **THEN** la construcción es imposible o falla
- **AND** no se produce un `InvalidInput`

### Requirement: FieldError rechaza campo o mensaje vacío

Un `FieldError` MUST tener `field` y `message` no vacíos (tras recortar espacios). Construir `InvalidInput` (o el `FieldError`) con `field` vacío o `message` vacío MUST fallar y MUST NOT producir un `DomainError`.

#### Scenario: Field vacío (unhappy)

- **WHEN** se intenta construir un `FieldError` o un `InvalidInput` con `field` vacío o solo espacios
- **THEN** la operación falla
- **AND** no se produce un `DomainError`

#### Scenario: Message vacío (unhappy)

- **WHEN** se intenta construir un `FieldError` o un `InvalidInput` con `message` vacío o solo espacios
- **THEN** la operación falla
- **AND** no se produce un `DomainError`
