# Tasks

## 1. Tests (red)

- [x] 1.1 Confirmar que `scripts/check-domain-deps.sh` no existe (`test ! -e scripts/check-domain-deps.sh`) — escenario «Script presente» en rojo
- [x] 1.2 Fixture temporal (`CHECK_ROOT`) con path crate `name = "surrealdb"` en el árbol `normal,build` de `domain` — `cargo tree -p domain -e normal,build` lista el package `surrealdb` (oráculo del escenario «surrealdb aparece en el árbol»)
- [x] 1.3 Fixture con path crate `reqwest` (u otro SDK de la lista distinto de `surrealdb`) en el árbol de `domain` — `cargo tree` lista ese package (escenario «otro SDK prohibido aparece en el árbol»)
- [x] 1.4 Fixture transitiva: `domain` → crate intermedio → path crate `reqwest`, sin declarar `reqwest` en `domain/Cargo.toml` — `cargo tree -e normal,build` lista `reqwest` (escenario «SDK prohibido solo como dependencia transitiva»)
- [x] 1.5 Fixture con `domain` dependiendo de un member `storage` (mini-workspace, no el repo) — `cargo tree` lista `storage` (escenario «domain depende de un crate del workspace»)
- [x] 1.6 Fixture cuyo workspace no tiene package `domain` — `cargo tree -p domain` falla (oráculo del escenario «falta el package domain»)
- [x] 1.7 Fixture con `surrealdb` solo en `[dev-dependencies]` de `domain` — `cargo tree -p domain -e normal,build` no lista `surrealdb` (oráculo del escenario «SDK prohibido solo en dev-dependencies»)

## 2. Script (green)

- [x] 2.1 Crear `scripts/check-domain-deps.sh` POSIX (`CHECK_ROOT` opcional, `fail`, `cargo tree -p domain -e normal,build` sobre `vortex-core/Cargo.toml`, denylist de SDKs, packages del workspace excepto `domain`, match por nombre exacto) — `test -x scripts/check-domain-deps.sh` (escenario «Script presente»)
- [x] 2.2 `./scripts/check-domain-deps.sh` sobre el repo sale 0 (escenarios «domain limpio» y «domain es la raíz del árbol»)
- [x] 2.3 Fixture 1.2 — el script sale ≠ 0 y el stderr nombra `surrealdb`
- [x] 2.4 Fixture 1.3 — el script sale ≠ 0 y el stderr nombra el SDK
- [x] 2.5 Fixture 1.4 — el script sale ≠ 0 y el stderr nombra `reqwest`
- [x] 2.6 Fixture 1.5 — el script sale ≠ 0 y el stderr nombra `storage`
- [x] 2.7 Fixture 1.6 — el script sale ≠ 0 y el mensaje indica que falta `domain`
- [x] 2.8 Fixture 1.7 — el script sale 0 (aristas `dev` fuera del chequeo)
- [x] 2.9 `./scripts/check-workspace.sh` sobre el repo sigue saliendo 0 (non-goal: no se modifica ese harness)

## 3. Cierre (non-goals)

- [x] 3.1 `vortex-core/crates/domain/Cargo.toml` no declara `[dependencies]` ni `[dev-dependencies]` (`git diff` vacío en ese crate)
- [x] 3.2 No hay workflow ni job de CI que ejecute este script (T-00-02-02) — no existe `.github/workflows/` nuevo en este change
