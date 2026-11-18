# Design

## Context

En `develop`, `crates/api` traduce `DomainError` a `problem+json` (T-00-11) y sirve POST de productos y stock, pero **no** hay helper de cursor ni `GET /products` / historial. OpenAPI ya define `Cursor`, `Limit` (default 50, max 100) y `nextCursor`. `domain::Id<T>` parsea ULID Crockford de 26 caracteres y rechaza minúsculas y prefijo de tabla. La allowlist de `api` no incluye `base64`. Ver `proposal.md` para motivación y `specs/vortex-api/spec.md` para el contrato.

Constraints: MVP del curso; ~2 h; Listo cuando = tests unitarios de codificación y límites; no HTTP de listados (T-02-27 / T-02-26); no offset (T-03-13).

## Goals / Non-Goals

**Goals:**

- Módulo de paginación en `api` reusable por T-02-27 (y T-02-26 en v2)
- Encode/decode ULID ↔ base64url, `limit` 1..=100 (default 50), `nextCursor` nulo al final
- Unhappy path `invalid_input` en `cursor` / `limit` alineado a `ApiError`

**Non-Goals (diseño):**

- Extractor Axum de query (`?cursor=&limit=`) — lo cablean T-02-27 / T-02-26
- Dirección de orden (`id > cursor` vs `id < cursor`) — la define cada listado
- Permitir `base64` / `data-encoding` en la allowlist
- Tests de contrato `assert_conforms` contra un path HTTP (no hay path nuevo)

## Decisions

### 1. Helper en `api/src/pagination.rs`, no en `domain`

**Elección:** módulo `pagination` en el crate `api`, reexportado (`pub use pagination::{…}` según lo que los tests necesiten). Encode/decode y `limit` devuelven `Result<_, ApiError>` (o `DomainError` que `ApiError::from` ya mapea). Campos de error en inglés de OpenAPI: `cursor`, `limit`.

**Por qué:** el ticket dice «helper en `api`». El cursor es un detalle de wire (base64url + `nextCursor`), no un invariante de dominio. `domain` no debe conocer HTTP ni encoding de query.

**Alternativa:** poner encode/decode en `domain::Id`. Se descartó: T-00-03 / allowlist de `domain` no contemplan codecs de API; el cursor no se persiste. **Alternativa:** crate `api` llama a `catalog`. Se descartó: no hay persistencia ni caso de uso.

### 2. Base64url sin padding, implementado en el módulo

**Elección:** RFC 4648 §5, **sin** `=` de padding. Encoder/decoder local sobre los 26 bytes ASCII del ULID. MUST NOT declarar `base64` en `Cargo.toml` ni tocar `scripts/check-workspace.sh`.

**Por qué:** query strings y `=` conviven mal; el payload es siempre 26 bytes (tamaño fijo). Sumar `base64` obliga a pin en workspace, allowlist y fixtures (~fuera de ~2 h y del Listo cuando).

**Alternativa:** crate `base64` `URL_SAFE_NO_PAD`. Se descartó: harness de allowlist. **Alternativa:** cursor = ULID crudo. Se descartó: 5.6 y el ticket piden opaco en base64url (el cliente no debe aritmetizar el id).

### 3. Página con patrón `limit + 1`

**Elección:** `paginate(items, limit, id_of)` recibe una secuencia **ya ordenada** de longitud ≤ `limit + 1`. Si `items.len() > limit`, corta a `limit` y pone `nextCursor = encode(id del último retenido)`; si no, `nextCursor = None`. El llamador (futuro GET) hace `WHERE id > $cursor ORDER BY id LIMIT $limit+1` o el equivalente descendente.

**Por qué:** un solo corte cubre «hay más páginas» sin un count extra. El helper no habla SQL ni Surreal.

**Alternativa:** devolver siempre `nextCursor` del último ítem y que el cliente descubra el final con una página vacía. Se descartó: OpenAPI / 5.6 piden `nextCursor = null` al final, y HU-02-08 «cargar más hasta el final» necesita esa señal.

### 4. Tests unitarios en el módulo, no HTTP de listado

**Elección:** `#[cfg(test)]` en `pagination.rs` (nombres de la tabla de la spec). El escenario `problem+json` construye el `ApiError` del helper y llama `IntoResponse` (mismo patrón que `internal_error_does_not_leak_cause`), sin registrar rutas nuevas.

**Por qué:** Listo cuando = «tests unitarios de codificación y límites». Un GET de productos sería T-02-27.

**Alternativa:** sonda `GET /_probe/page` en tests. Se descartó: no aporta más que `IntoResponse` y ensucia el router de test.

### 5. ULID vía `Id<()>` / parseo existente

**Elección:** tras base64url, parsear el payload con `Id` (`FromStr`) para reusar rechazo de minúsculas, longitud y prefijo `tabla:`. El helper no reimplementa Crockford.

**Por qué:** un solo alfabeto ULID en el repo (`domain::id`).

**Alternativa:** aceptar cualquier 26 chars. Se descartó: un cursor «válido» que no sirve como `id > $cursor` en Surreal.

## Risks / Trade-offs

- **[T-02-27 parsea `limit` como string de query y el helper solo ve `Option<u32>`]** → Mitigación: el extractor de T-02-27 mapea no-entero a `invalid_input` field `limit`; este change documenta el contrato numérico.
- **[Decoder casero con bug en padding/resto]** → Mitigación: roundtrip del ULID canónico de 26 bytes (resto 2) más unhappy de `=` / `+` / `/`; payload de tamaño distinto a 26 → `invalid_input`.
- **[Alguien usa el helper para búsqueda offset]** → Mitigación: spec MUST NOT offset; T-03-13 sigue con Meilisearch.

## Migration Plan

1. Red: tests de `pagination_*` no compilan.
2. Green: módulo + `mod pagination` en `lib.rs`; constructores de error si hacen falta.
3. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-21`.
4. Rollback: borrar el módulo; no hay datos ni paths HTTP que revertir.

## Open Questions

Ninguna que cambie specs, enfoque o desglose.
