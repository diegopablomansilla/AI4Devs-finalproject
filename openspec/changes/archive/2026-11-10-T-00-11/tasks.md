# Tasks

## 1. Tests (red) — DomainError → problem+json

- [x] 1.1 Agregar `vortex-core/crates/api/tests/problem.rs` (o módulo de test en el crate) con `domain_error_variants_map_to_problem_json`: un `DomainError` por cada `code` de dominio → respuesta con status de la fila 5.4, `Content-Type: application/problem+json`, `code` y `title` de la tabla — `cargo test -p api -- domain_error_variants` no compila o falla (escenarios «Cada code de dominio se traduce», «NotFound a 404», «InvalidInput a 400 con errors[]», «Conflict version_conflict a 409», «Unavailable incluye Retry-After», «Timeout a 504»)
- [x] 1.2 En el mismo archivo, asertar que `Conflict` no incluye la clave `errors` y que `InvalidInput` con un `FieldError` sí la incluye — sigue en rojo (escenarios «Conflict no expone errors», «InvalidInput a 400 con errors[]»)

## 2. Tests (red) — 404, JSON y 500

- [x] 2.1 Extender `build_router_unknown_path_returns_404` para exigir `Content-Type: application/problem+json`, `code` `not_found` e `instance` `/no-such-route` — `cargo test -p api --test router -- unknown_path` falla (escenarios «ruta inexistente es problem+json not_found», «ruta inexistente responde 404»)
- [x] 2.2 Agregar `json_rejection_returns_400_invalid_input`: sonda de test `POST` con `AppJson`, cuerpo `{` y cuerpo JSON de tipo incorrecto y petición sin `Content-Type` → 400, `code` `invalid_input`, `errors` con `field` no vacío — `cargo test -p api -- json_rejection` no compila o falla (escenarios «JSON malformado», «JSON con campo inválido», «Content-Type ausente en JSON»)
- [x] 2.3 Agregar `internal_error_does_not_leak_cause`: `ApiError::internal` con un error cuyo `Display` contiene un marcador (`secret=hunter2`) → 500, `code` `internal`, el cuerpo no contiene el marcador — `cargo test -p api -- internal_error` no compila o falla (escenario «500 internal no filtra la causa»)
- [x] 2.4 Extender `vortex_serve_starts_and_returns_404` para exigir `Content-Type: application/problem+json` y `code` `not_found` — `cargo test -p vortex --test serve -- starts_and_returns` falla (escenario «vortex serve arranca y responde 404»)

## 3. Tests (red) — allowlist

- [x] 3.1 Fixture temporal de `crates/api/Cargo.toml` con `clap` (o `tower-http`) en `[dependencies]` — `./scripts/check-workspace.sh` sale ≠ 0 y nombra `api` (escenario «api declara una dependencia fuera de la allowlist»; **no** usar `serde`, pasará a estar permitido)
- [x] 3.2 Fixture temporal de `crates/api/Cargo.toml` con la allowlist completa (`axum`, path `domain`, `serde`, `serde_json`, `thiserror`) — hoy el script sale ≠ 0 (escenario «api declara la allowlist» aún en rojo)
- [x] 3.3 `./scripts/check-workspace.sh` sobre el repo sale 0 (api todavía sin `domain`/`serde`; escenario previo a este change)

## 4. Harness (green) — allowlist `api`

- [x] 4.1 En `scripts/check-workspace.sh`: permitir en `api` `axum`, `domain` (path), `serde`, `serde_json` y `thiserror`; `[dev-dependencies]` sigue solo `tokio`; `tower-http` sigue prohibido — el fixture 3.2 sale 0 y el 3.1 sigue ≠ 0 nombrando `api` (escenarios «api declara la allowlist», «api declara una dependencia fuera de la allowlist»)
- [x] 4.2 Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy `api` con allowlist axum (sigue MAY); happy con allowlist completa; unhappy `api` con `clap`/`tower-http` (reemplaza `api con serde`) — `./scripts/fixtures/workspace/run-script-tests.sh` sale 0
- [x] 4.3 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (todavía sin las deps nuevas en el `Cargo.toml` real de `api`)

## 5. Implementación (green) — `ApiError`

- [x] 5.1 Declarar en `vortex-core/crates/api/Cargo.toml` path `domain` y `serde` / `serde_json` / `thiserror` (`workspace = true`); actualizar `Cargo.lock` — `./scripts/check-workspace.sh` sobre el repo sale 0 (escenario «api declara la allowlist»)
- [x] 5.2 Implementar `ApiError` (`From<DomainError>`, `IntoResponse`, títulos, `type` kebab, `Retry-After: 5` en `Unavailable`, `internal` que descarta la causa) — `cargo test -p api -- domain_error_variants internal_error` sale 0 (escenarios 1.1–1.2 y 2.3 / Listo cuando)

## 6. Implementación (green) — fallback y `AppJson`

- [x] 6.1 Instalar fallback 404 `problem+json` en `build_router` con `instance` = path — `cargo test -p api --test router -- unknown_path` sale 0 (escenarios 2.1)
- [x] 6.2 Implementar extractor `AppJson<T>` que mapea `JsonRejection` a `400 invalid_input` con `errors[{ field, message }]` (`field` = ruta serde o `body`) — `cargo test -p api -- json_rejection` sale 0 (escenarios 2.2)
- [x] 6.3 Confirmar `vortex serve` hereda el fallback — `cargo test -p vortex --test serve -- starts_and_returns` sale 0 (escenario 2.4 / Listo cuando)
- [x] 6.4 Confirmar que no hay rutas `/health`, `/metrics` ni `/api/v1`, que `api` no declara `tower-http` y que `docs/openapi.yaml` no cambió — `rg` / `git diff` (non-goals T-00-12, T-00-13, T-00-14, T-02-06)

## 7. Cierre (regresión y § 5)

- [x] 7.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «Build, tests y clippy en verde» / tabla de tests de `vortex-api`)
- [x] 7.2 `./scripts/check-workspace.sh`, `./scripts/check-domain-deps.sh` y `./scripts/check-toolchain.sh` salen 0 (`api` depende de `domain`, no al revés)
- [x] 7.3 `openspec validate T-00-11` (o `t-00-11` si el CLI exige minúsculas) sale 0
