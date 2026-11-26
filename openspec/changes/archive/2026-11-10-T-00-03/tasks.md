# Tasks

## 1. Tests (red) — taxonomía `DomainError`

- [x] 1.1 Agregar tests unitarios en `vortex-core/crates/domain` que construyan las cinco variantes y un `InvalidInput` con `FieldError { field: "attributes[1].value", message }` y con lista vacía + `invalid_input` — `cargo test -p domain` no compila o falla (escenarios «Cada variante se construye», «InvalidInput conserva FieldError», «InvalidInput sin errores de campo», «DomainError es std::error::Error»)
- [x] 1.2 Agregar tests que aserten `code()` (o `as_str`) para `NotFound` → `not_found`, `Timeout` → `timeout`, `version_conflict`, `negative_stock`, `ai_unavailable` y un caso por cada `code` de la tabla 5.4 de la spec — `cargo test -p domain` sigue en rojo (escenarios «Code de cada variante», «Todos los codes de dominio son construibles»)
- [x] 1.3 Agregar tests unhappy: `FromStr`/`parse` de `no_existe`; de `payload_too_large`, `unsupported_media_type` e `internal`; `FieldError`/`InvalidInput` con `field` vacío y con `message` vacío; construcción `InvalidInput` + `version_conflict` (no compila o `Err`) — `cargo test -p domain` sigue en rojo (escenarios «Code desconocido», «Code solo de Gateway», «Code de otra variante», «Field vacío», «Message vacío»)

## 2. Tests (red) — allowlist de `check-workspace.sh`

- [x] 2.1 Fixture temporal (`CHECK_ROOT`) de `crates/domain/Cargo.toml` con `[dependencies] ulid.workspace = true` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `domain` (escenario «domain declara una dependencia distinta de thiserror»; hoy ya falla por cualquier `[dependencies]`)
- [x] 2.2 Fixture temporal de un lib que no es `domain` (p. ej. `api`) con `[dependencies]` — el script sale ≠ 0 y nombra ese crate (escenario «Un lib distinto de domain declara dependencias»)
- [x] 2.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (domain todavía sin deps; escenario «Pins exactos» sigue)

## 3. Harness (green) — `thiserror` en `domain`

- [x] 3.1 En `scripts/check-workspace.sh`: permitir `[dependencies]` en `domain` solo si la única crate es `thiserror` con `workspace = true`; seguir rechazando `[dev-dependencies]` en `domain`, cualquier otra crate en `domain`, y `[dependencies]` en el resto de libs / `bin/vortex`; actualizar el comentario a T-00-03 — una fixture temporal con solo `thiserror` en `domain` sale 0 (escenario «domain declara thiserror»)
- [x] 3.2 Re-correr 2.1 y 2.2: `ulid` en `domain` y deps en `api` siguen ≠ 0 con el needle del crate (unhappy de cargo-workspace en verde)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0

## 4. Implementación (green) — `domain::error`

- [x] 4.1 Declarar `thiserror = { workspace = true }` en `vortex-core/crates/domain/Cargo.toml` y actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0
- [x] 4.2 Implementar `domain::error` (`DomainError` con las cinco variantes, enums de `code` por fila 5.4, `DomainCode`, `FieldError::try_new`, `code()` / `FromStr`) y reexportar desde `lib.rs` — `cargo test -p domain` sale 0 (todos los escenarios de `domain-error`)
- [x] 4.3 Confirmar que `InvalidInput { code: version_conflict }` no es representable (no compila o el test 1.3 de mismatch pasa) y que ningún tipo de `domain` implementa traducción HTTP — `rg -n "problem\\+json|IntoResponse|StatusCode" vortex-core/crates/domain` sin matches de producto

## 5. Cierre (regresión, § 5 y non-goals)

- [x] 5.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde»)
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`thiserror` no está en la denylist)
- [x] 5.3 `git diff` vacío en `vortex-core/crates/api`, `docs/openapi.yaml` y `.github/workflows/ci.yml` (non-goals: T-00-11, contrato HTTP, CI)
- [x] 5.4 `openspec validate T-00-03` sale 0
