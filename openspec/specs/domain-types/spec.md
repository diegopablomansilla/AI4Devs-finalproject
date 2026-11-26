# domain-types Specification

## Purpose
Define los newtypes de dominio `Id<T>`, `Decimal` y `Timestamp` con parse y serialización de ida y vuelta alineados al contrato (ULID sin prefijo, decimal como string, RFC 3339 UTC), para que las entidades posteriores no reinventen identidad, dinero ni fechas.

## Requirements

### Requirement: Id tipado es un ULID de 26 caracteres sin prefijo de tabla

El crate `domain` MUST exponer un tipo `Id<T>` que representa un ULID Crockford de exactamente 26 caracteres, sin prefijo de tabla (`producto:…` u otro `tabla:`). El alfabeto MUST coincidir con el esquema `Ulid` de `docs/openapi.yaml` (`^[0-9A-HJKMNP-TV-Z]{26}$`). Parsear, mostrar y serializar a JSON MUST usar esa cadena de 26 caracteres. `T` MUST distinguir identidades de entidades distintas en el sistema de tipos (un `Id` de una entidad no es intercambiable con el de otra). El crate MUST NOT incluir el prefijo SurrealDB en el valor. Generar un `Id` MUST producir un ULID válido según las mismas reglas.

#### Scenario: Parse de ULID válido (happy)

- **WHEN** se parsea la cadena `01J9ZKA7M3Q8R2T5V9X1Z4B6D8`
- **THEN** la operación termina sin error
- **AND** el valor mostrado es exactamente esa cadena

#### Scenario: Ida y vuelta string y JSON (happy)

- **WHEN** se parsea un ULID válido, se muestra como string y se serializa/deserializa como JSON
- **THEN** el string mostrado es el mismo ULID de 26 caracteres
- **AND** el JSON es un string (no un objeto ni un número) con ese ULID
- **AND** deserializar ese JSON recupera un `Id` igual al original

#### Scenario: Id generado es ULID de 26 caracteres (happy)

- **WHEN** se genera un `Id`
- **THEN** el valor mostrado tiene longitud 26
- **AND** no contiene `:`
- **AND** coincide con el patrón Crockford en mayúsculas del esquema `Ulid`

#### Scenario: Prefijo de tabla (unhappy)

- **WHEN** se parsea o se deserializa `producto:01J9ZKA7M3Q8R2T5V9X1Z4B6D8` (u otro `tabla:` + ULID)
- **THEN** la operación falla
- **AND** no se produce un `Id`

#### Scenario: ULID de longitud incorrecta (unhappy)

- **WHEN** se parsea una cadena de 25 o 27 caracteres del alfabeto Crockford
- **THEN** la operación falla
- **AND** no se produce un `Id`

#### Scenario: Caracteres fuera del alfabeto o minúsculas (unhappy)

- **WHEN** se parsea un ULID con `I`, `L`, `O`, `U` o letras minúsculas (p. ej. `01j9zka7m3q8r2t5v9x1z4b6d8`)
- **THEN** la operación falla
- **AND** no se produce un `Id`

### Requirement: Decimal se parsea y serializa como string del contrato

El crate `domain` MUST exponer un tipo `Decimal` para cantidades con signo. Parsear desde string y serializar a JSON MUST usar un **string** que cumpla el esquema `Decimal` de `docs/openapi.yaml` (`^-?\d{1,12}(\.\d{1,4})?$`; ejemplos `120.50`, `-3`). MUST NOT aceptar un número JSON de punto flotante. El valor parseado MUST redondear o conservar como máximo 4 decimales según ese patrón. `Decimal` MAY ser negativo (cantidades de movimiento); este requirement MUST NOT exigir no-negatividad (`NonNegativeDecimal` queda fuera).

#### Scenario: Parse de strings del contrato (happy)

- **WHEN** se parsean `120.50`, `-3` y `34`
- **THEN** cada operación termina sin error
- **AND** el valor mostrado vuelve a ser un string que cumple el patrón `Decimal`

#### Scenario: Ida y vuelta string y JSON (happy)

- **WHEN** se parsea `120.50`, se muestra como string y se serializa/deserializa como JSON
- **THEN** el JSON es el string `"120.50"` o un string equivalente que cumple el patrón (mismo valor decimal)
- **AND** deserializar ese JSON recupera un `Decimal` igual al original
- **AND** parsear el string mostrado recupera un `Decimal` igual al original

#### Scenario: Número JSON (unhappy)

- **WHEN** se deserializa el número JSON `120.5` (sin comillas) como `Decimal`
- **THEN** la operación falla
- **AND** no se produce un `Decimal`

#### Scenario: String decimal inválido (unhappy)

- **WHEN** se parsea o deserializa `abc`, `1e2`, `+120.50`, `.5`, `120.50000` (más de 4 decimales) o un entero de 13 dígitos
- **THEN** la operación falla
- **AND** no se produce un `Decimal`

### Requirement: Timestamp es RFC 3339 en UTC

El crate `domain` MUST exponer un tipo `Timestamp` que representa un instante en UTC. Parsear y serializar MUST usar RFC 3339; la forma canónica serializada MUST terminar en `Z` (ejemplo de [5.1](../../../../docs/5-contract-api.md#51-convenciones-generales): `2026-09-24T21:33:00Z`). Un offset `+00:00` MAY aceptarse al parsear y MUST emitirse como `Z`. Cualquier otro offset, ausencia de zona, o cadena que no sea RFC 3339 MUST fallar. Serializar a JSON MUST ser un string, no un número (epoch).

#### Scenario: Parse UTC con Z (happy)

- **WHEN** se parsea `2026-09-24T21:33:00Z`
- **THEN** la operación termina sin error
- **AND** el valor serializado (string y JSON) es `2026-09-24T21:33:00Z`

#### Scenario: Offset UTC +00:00 se normaliza a Z (happy)

- **WHEN** se parsea `2026-09-24T21:33:00+00:00`
- **THEN** la operación termina sin error
- **AND** el valor serializado termina en `Z` y representa el mismo instante

#### Scenario: Ida y vuelta string y JSON (happy)

- **WHEN** se parsea un timestamp UTC válido, se muestra como string y se serializa/deserializa como JSON
- **THEN** el JSON es un string RFC 3339 con sufijo `Z`
- **AND** deserializar ese JSON recupera un `Timestamp` igual al original

#### Scenario: Sin zona horaria (unhappy)

- **WHEN** se parsea `2026-09-24T21:33:00` (sin `Z` ni offset)
- **THEN** la operación falla
- **AND** no se produce un `Timestamp`

#### Scenario: Offset no UTC (unhappy)

- **WHEN** se parsea `2026-09-24T21:33:00-03:00`
- **THEN** la operación falla
- **AND** no se produce un `Timestamp`

#### Scenario: Cadena o JSON no RFC 3339 (unhappy)

- **WHEN** se parsea una cadena vacía, `2026/09/24` o se deserializa un número JSON (epoch)
- **THEN** la operación falla
- **AND** no se produce un `Timestamp`
