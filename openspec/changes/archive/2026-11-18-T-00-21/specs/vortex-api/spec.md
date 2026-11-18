# Spec Delta

Ticket padre: T-00-21. Change: T-00-21. HU: HU-02-08, HU-02-10.

## ADDED Requirements

### Requirement: Paginación por cursor opaco en el Gateway

El crate `api` MUST exponer un helper de paginación por cursor para listados de administración, alineado con [5.6](../../../../../docs/5-contract-api.md#56-paginación) y con los parámetros `Cursor` / `Limit` de `docs/openapi.yaml`. MUST NOT registrar `GET /api/v1/products` ni `GET /api/v1/products/{productId}/stock-movements`. MUST NOT implementar paginación por *offset* (`offset`, `estimatedTotalHits`).

El cursor MUST ser opaco: internamente es el último ULID visto (26 caracteres Crockford, sin prefijo de tabla), serializado en **base64url sin padding** (RFC 4648 §5). Un cursor ausente MUST significar la primera página. Codificar un ULID canónico y decodificar el cursor resultante MUST devolver el mismo ULID. El valor codificado MUST NOT ser el ULID en texto plano.

`limit` ausente MUST equivaler a **50**. `limit` MUST aceptarse solo en el rango **1..=100**. Un `limit` menor que 1 o mayor que 100 MUST ser `invalid_input` con `errors[].field` `limit`.

Un cursor presente que no sea base64url sin padding, que decodifique a bytes que no sean un ULID canónico de 26 caracteres, o que esté vacío MUST ser `invalid_input` con `errors[].field` `cursor`. Ese error MUST traducirse vía `ApiError` a HTTP **400**, `Content-Type: application/problem+json` y `code` `invalid_input`.

Dada una secuencia de ítems ya ordenada por el llamador (hasta `limit + 1` elementos) y un `limit` válido: si hay más ítems que `limit`, la página MUST devolver exactamente `limit` ítems y `nextCursor` MUST ser el cursor del último ítem devuelto; si hay `limit` ítems o menos (incluida la secuencia vacía), `nextCursor` MUST ser `null`. El helper MUST NOT imponer dirección de orden (ascendente o descendente): eso lo define el listado consumidor (T-02-27, T-02-26).

MUST NOT agregar crates a la allowlist de `api` para esta codificación.

#### Scenario: Roundtrip ULID y cursor opaco (happy, Listo cuando)

- **WHEN** se codifica el ULID canónico `01J9ZKA7M3Q8R2T5V9X1Z4B6D8` y se decodifica el cursor resultante
- **THEN** el ULID recuperado es `01J9ZKA7M3Q8R2T5V9X1Z4B6D8`
- **AND** el cursor no es igual al ULID en texto plano
- **AND** el cursor solo usa el alfabeto base64url (`A–Z`, `a–z`, `0–9`, `-`, `_`) y no contiene `=`

#### Scenario: Cursor ausente es primera página (happy)

- **WHEN** se decodifica un cursor ausente
- **THEN** el resultado es éxito sin ULID (primera página)

#### Scenario: limit por defecto 50 y tope 100 (happy, Listo cuando)

- **WHEN** se interpreta `limit` ausente
- **THEN** el valor efectivo es 50
- **WHEN** se interpreta `limit` 1 y `limit` 100
- **THEN** ambos se aceptan con esos valores

#### Scenario: nextCursor del último ítem si hay más páginas (happy)

- **WHEN** se arma una página con `limit` 2 a partir de tres ítems con ULID `A`, `B`, `C` (en ese orden)
- **THEN** la página contiene exactamente los ítems `A` y `B`
- **AND** `nextCursor` decodifica al ULID de `B`

#### Scenario: nextCursor null al final (happy)

- **WHEN** se arma una página con `limit` 2 a partir de dos ítems
- **THEN** `nextCursor` es `null`
- **WHEN** se arma una página con `limit` 2 a partir de cero ítems
- **THEN** `nextCursor` es `null` y la lista de ítems está vacía

#### Scenario: cursor inválido no es base64url (unhappy)

- **WHEN** se decodifica el cursor `%%%`
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: cursor con padding o alfabeto estándar (unhappy)

- **WHEN** se decodifica un cursor base64url con padding `=` o con alfabeto estándar (`+` o `/`)
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: cursor que no decodifica a ULID (unhappy)

- **WHEN** se decodifica un cursor base64url válido cuyo payload no es un ULID canónico de 26 caracteres (p. ej. texto `not-a-ulid`, ULID en minúsculas, o ULID con prefijo de tabla)
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: cursor vacío (unhappy)

- **WHEN** se decodifica el cursor `""`
- **THEN** el error es `invalid_input`
- **AND** `errors` incluye `field` `cursor`

#### Scenario: limit fuera de rango (unhappy, Listo cuando)

- **WHEN** se interpreta `limit` 0
- **THEN** el error es `invalid_input` con `errors[].field` `limit`
- **WHEN** se interpreta `limit` 101
- **THEN** el error es `invalid_input` con `errors[].field` `limit`

#### Scenario: cursor inválido se traduce a problem+json (unhappy)

- **WHEN** el helper rechaza un cursor inválido y el error se traduce con `ApiError`
- **THEN** la respuesta tiene estado HTTP 400
- **AND** `Content-Type` es `application/problem+json`
- **AND** el JSON tiene `code` `invalid_input`
- **AND** `errors` incluye `field` `cursor`

## Tests previstos (TDD)

| Test | Crate | Tipo |
|---|---|---|
| `pagination_encode_decode_roundtrip` | `api` | unitario, happy (Listo cuando) |
| `pagination_absent_cursor_is_first_page` | `api` | unitario, happy |
| `pagination_limit_default_is_50` | `api` | unitario, happy (Listo cuando) |
| `pagination_limit_1_and_100_are_accepted` | `api` | unitario, happy (Listo cuando) |
| `pagination_next_cursor_when_more_remain` | `api` | unitario, happy |
| `pagination_next_cursor_null_on_last_page` | `api` | unitario, happy |
| `pagination_next_cursor_null_on_empty_page` | `api` | unitario, happy |
| `pagination_invalid_cursor_not_base64url_is_invalid_input` | `api` | unitario, unhappy |
| `pagination_invalid_cursor_padding_or_std_alphabet_is_invalid_input` | `api` | unitario, unhappy |
| `pagination_invalid_cursor_non_ulid_is_invalid_input` | `api` | unitario, unhappy |
| `pagination_empty_cursor_is_invalid_input` | `api` | unitario, unhappy |
| `pagination_limit_zero_is_invalid_input` | `api` | unitario, unhappy (Listo cuando) |
| `pagination_limit_101_is_invalid_input` | `api` | unitario, unhappy (Listo cuando) |
| `pagination_invalid_cursor_maps_to_problem_json` | `api` | unitario (`IntoResponse`), unhappy |
