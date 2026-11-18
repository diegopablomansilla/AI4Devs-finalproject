# Tasks

## 1. Tests (red) — encoding, limit y página (happy)

- [x] 1.1 Agregar tests unitarios `pagination_encode_decode_roundtrip` y `pagination_absent_cursor_is_first_page` en `vortex-core/crates/api` (`src/pagination.rs` `#[cfg(test)]` o `tests/pagination.rs`): ULID `01J9ZKA7M3Q8R2T5V9X1Z4B6D8` roundtrip, cursor ≠ ULID plano, alfabeto base64url sin `=`; cursor ausente = primera página — `cargo test -p api pagination_encode_decode pagination_absent_cursor` no compila o falla (escenarios «Roundtrip ULID y cursor opaco», «Cursor ausente es primera página» / Listo cuando)
- [x] 1.2 Agregar `pagination_limit_default_is_50` y `pagination_limit_1_and_100_are_accepted` — `cargo test -p api pagination_limit_default pagination_limit_1_and_100` no compila o falla (escenario «limit por defecto 50 y tope 100» / Listo cuando)
- [x] 1.3 Agregar `pagination_next_cursor_when_more_remain`, `pagination_next_cursor_null_on_last_page` y `pagination_next_cursor_null_on_empty_page` — `cargo test -p api pagination_next_cursor` no compila o falla (escenarios «nextCursor del último ítem si hay más páginas», «nextCursor null al final»)

## 2. Tests (red) — cursor y limit inválidos (unhappy)

- [x] 2.1 Agregar `pagination_invalid_cursor_not_base64url_is_invalid_input`, `pagination_invalid_cursor_padding_or_std_alphabet_is_invalid_input`, `pagination_invalid_cursor_non_ulid_is_invalid_input` y `pagination_empty_cursor_is_invalid_input` — `cargo test -p api pagination_invalid_cursor pagination_empty_cursor` no compila o falla (escenarios «cursor inválido no es base64url», «cursor con padding o alfabeto estándar», «cursor que no decodifica a ULID», «cursor vacío»)
- [x] 2.2 Agregar `pagination_limit_zero_is_invalid_input` y `pagination_limit_101_is_invalid_input` — `cargo test -p api pagination_limit_zero pagination_limit_101` no compila o falla (escenario «limit fuera de rango» / Listo cuando)
- [x] 2.3 Agregar `pagination_invalid_cursor_maps_to_problem_json`: el error del helper vía `ApiError`/`IntoResponse` → 400, `Content-Type: application/problem+json`, `code` `invalid_input`, `errors[].field` `cursor` — `cargo test -p api pagination_invalid_cursor_maps_to_problem_json` no compila o falla (escenario «cursor inválido se traduce a problem+json»)

## 3. Implementación (green) — helper de paginación

- [x] 3.1 Implementar `vortex-core/crates/api/src/pagination.rs` (encode/decode base64url sin padding, parseo de ULID con `domain::Id`, `limit` default 50 / rango 1..=100, `paginate` con patrón `limit+1`) y `mod pagination` en `lib.rs`; MUST NOT agregar crates ni tocar `check-workspace.sh` — `cargo test -p api pagination_encode_decode pagination_absent_cursor pagination_limit_default pagination_limit_1_and_100 pagination_next_cursor` sale 0 (capability `vortex-api` / Listo cuando encoding y límites happy)
- [x] 3.2 Mapear fallos de cursor/`limit` a `invalid_input` con `errors[].field` `cursor` o `limit` (constructores en `error.rs` o `DomainError` + `ApiError::from`) — `cargo test -p api pagination_invalid_cursor pagination_empty_cursor pagination_limit_zero pagination_limit_101 pagination_invalid_cursor_maps_to_problem_json` sale 0 (unhappy / Listo cuando límites)
- [x] 3.3 Confirmar que no hay `GET /api/v1/products` de listado, no hay `GET .../stock-movements`, `docs/openapi.yaml` no cambió y `api/Cargo.toml` no declara `base64` — `rg` / `git diff` (non-goals T-02-27, T-02-26, T-03-13)

## 4. Cierre (regresión y § 5)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` salen 0 (`api` sin crate `base64`; `domain` sin encoding de cursor)
- [x] 4.3 `openspec validate T-00-21` (o `t-00-21` si el CLI exige minúsculas) sale 0