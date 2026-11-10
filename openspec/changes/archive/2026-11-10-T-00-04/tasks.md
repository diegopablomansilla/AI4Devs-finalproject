# Tasks

## 1. Tests (red) — tipos base `Id`, `Decimal`, `Timestamp`

- [x] 1.1 Agregar tests unitarios `id_parses_canonical_ulid`, `id_roundtrip_display_and_json` y `generated_id_matches_ulid_pattern` en `vortex-core/crates/domain` — `cargo test -p domain` no compila o falla (escenarios «Parse de ULID válido», «Ida y vuelta string y JSON», «Id generado es ULID de 26 caracteres»)
- [x] 1.2 Agregar tests unhappy `id_rejects_table_prefix`, `id_rejects_wrong_length` e `id_rejects_invalid_alphabet_and_lowercase` — `cargo test -p domain` sigue en rojo (escenarios «Prefijo de tabla», «ULID de longitud incorrecta», «Caracteres fuera del alfabeto o minúsculas»)
- [x] 1.3 Agregar tests `decimal_parses_contract_examples` y `decimal_roundtrip_display_and_json` — `cargo test -p domain` sigue en rojo (escenarios «Parse de strings del contrato», «Ida y vuelta string y JSON» de Decimal)
- [x] 1.4 Agregar tests unhappy `decimal_rejects_json_number` y `decimal_rejects_invalid_strings` (`abc`, `1e2`, `+120.50`, `.5`, `120.50000`, entero de 13 dígitos) — `cargo test -p domain` sigue en rojo (escenarios «Número JSON», «String decimal inválido»)
- [x] 1.5 Agregar tests `timestamp_parses_rfc3339_z`, `timestamp_normalizes_plus_zero_offset` y `timestamp_roundtrip_display_and_json` — `cargo test -p domain` sigue en rojo (escenarios «Parse UTC con Z», «Offset UTC +00:00 se normaliza a Z», «Ida y vuelta string y JSON» de Timestamp)
- [x] 1.6 Agregar tests unhappy `timestamp_rejects_naive_datetime`, `timestamp_rejects_non_utc_offset` y `timestamp_rejects_garbage_and_json_number` — `cargo test -p domain` sigue en rojo (escenarios «Sin zona horaria», «Offset no UTC», «Cadena o JSON no RFC 3339»)

## 2. Tests (red) — allowlist de `check-workspace.sh`

- [x] 2.1 Fixture temporal (`CHECK_ROOT`) de `crates/domain/Cargo.toml` con `[dependencies] clap.workspace = true` (o `surrealdb`) — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `domain` (escenario «domain declara una dependencia distinta de thiserror»; hoy ya falla por cualquier crate distinta de `thiserror`)
- [x] 2.2 Fixture temporal de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` — el script sale ≠ 0 y nombra ese crate (escenario «Un lib distinto de domain declara dependencias»)
- [x] 2.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (domain todavía solo con `thiserror`; escenario «domain declara thiserror» / «Pins exactos» sigue, aún sin `serde_json` en el manifiesto)

## 3. Harness (green) — pin `serde_json` y allowlist de tipos

- [x] 3.1 En `vortex-core/Cargo.toml` agregar `serde_json = "=1.0.154"` a `[workspace.dependencies]` y `check_pin serde_json 1.0.154` en `scripts/check-workspace.sh` — escenario «Pins exactos» incluye `serde_json`
- [x] 3.2 En `scripts/check-workspace.sh`: permitir en `domain` el subconjunto `{thiserror, ulid, rust_decimal, serde}` en `[dependencies]` y solo `serde_json` en `[dev-dependencies]`, todas con `workspace = true`; seguir rechazando cualquier otra crate, `serde_json` en runtime, y deps en el resto de libs / `bin/vortex`; actualizar el comentario a T-00-04 — una fixture temporal con la allowlist completa sale 0 (escenario «domain declara la allowlist de tipos base»)
- [x] 3.3 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: `ulid` (junto al resto de la allowlist) es happy; el unhappy de `domain` usa `clap` o `surrealdb` — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 3.4 Re-correr 2.1 y 2.2: `clap` en `domain` y deps en `api` siguen ≠ 0 con el needle del crate (unhappy de cargo-workspace en verde)
- [x] 3.5 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en el `Cargo.toml` real de `domain`)

## 4. Implementación (green) — `domain::{Id, Decimal, Timestamp}`

- [x] 4.1 Declarar en `vortex-core/crates/domain/Cargo.toml` `ulid`, `rust_decimal` y `serde` en `[dependencies]` y `serde_json` en `[dev-dependencies]` (`workspace = true`) y actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «domain declara la allowlist de tipos base»)
- [x] 4.2 Implementar `domain::id` (`Id<T>`, parse patrón OpenAPI, `generate`, serde string, `InvalidId`) y reexportar desde `lib.rs` — `cargo test -p domain -- id_` sale 0 (escenarios de Id)
- [x] 4.3 Implementar `domain::decimal` (`Decimal`, gramática OpenAPI, serde solo string, `InvalidDecimal`) y reexportar — `cargo test -p domain -- decimal_` sale 0 (escenarios de Decimal)
- [x] 4.4 Implementar `domain::timestamp` (RFC 3339 UTC, `+00:00` → `Z`, serde string, `InvalidTimestamp`) y reexportar — `cargo test -p domain` sale 0 (todos los escenarios de `domain-types`)
- [x] 4.5 Confirmar que no hay entidades de negocio ni `chrono`/`time` en `domain`, y que ningún tipo implementa HTTP — `rg -n "struct (Categoria|Producto|Movimiento)|chrono|problem\\+json|IntoResponse" vortex-core/crates/domain` sin matches de producto

## 5. Cierre (regresión, § 5 y non-goals)

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde»)
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`ulid` / `rust_decimal` / `serde` no están en la denylist)
- [x] 5.3 `git diff` vacío en `vortex-core/crates/api`, `docs/openapi.yaml` y `.github/workflows/ci.yml` (non-goals: T-00-11, contrato HTTP, CI)
- [x] 5.4 `openspec validate T-00-04` sale 0
