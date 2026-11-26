# Proposal

## Ticket padre

- **Ticket:** [T-00-02](../../../docs/7-tickets.md) — Verificación automática de la regla de dependencias de `domain`
- **Change:** `T-00-02-01`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Hace (ticket, recorte de este ZZ):** `scripts/check-domain-deps.sh`, que analiza `cargo tree -p domain -e normal,build` y falla si aparece `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort`, `whisper-rs` o cualquier crate del workspace
- **Listo cuando (ticket, recorte de este ZZ):** el script sale ≠ 0 si se agrega `surrealdb` (u otro ítem de la lista) al grafo de `domain`. El **Listo cuando** completo del ticket (fallo en CI de un PR) queda para **T-00-02-02**

## Why

HU-00-01 y [3.2](../../../docs/3-componentes.md#32-mapa-de-crates-y-regla-de-dependencias) exigen que `domain` no se acoplue a motores ni a otros crates del workspace. T-00-01 deja el crate vacío y `check-workspace.sh` rechaza *cualquier* `[dependencies]` en los libs; eso se romperá en cuanto T-00-03/T-00-04 agreguen `thiserror`, `ulid` o `rust_decimal`. Hace falta un chequeo del *árbol* (`cargo tree`), no del manifiesto, **antes** de que el dominio deje de estar vacío.

## What Changes

- Agregar `scripts/check-domain-deps.sh` (POSIX, `CHECK_ROOT` opcional como los scripts de T-00-01)
- El script corre `cargo tree -p domain -e normal,build` sobre `vortex-core/Cargo.toml` y falla si el árbol incluye SDKs prohibidos o cualquier package del workspace distinto de `domain`
- TDD con fixtures temporales: happy path sobre el repo; unhappy path (SDK directo, crate del workspace, dependencia transitiva) sin ensuciar `develop`

## Non-goals

Este ZZ **no** cubre:

- Job de GitHub Actions ni el **Listo cuando** «un PR con `surrealdb` en `domain` falla la CI» → **T-00-02-02** (tras T-09-01)
- CI base (`fmt` / `clippy` / `test`) → **T-09-01**
- Reglas equivalentes para otros crates (ticket **No hace**)
- Relajar `check-workspace.sh` para que `domain` declare dependencias de dominio (`thiserror`, `ulid`, …) → **T-00-03** / **T-00-04**
- Entidades, errores o tipos de dominio → **T-00-03**, **T-00-04**
- SurrealDB embebido o migraciones → **T-00-08**, **T-00-09**

## Capabilities

### New Capabilities

- `domain-deps`: verificación automática de que el árbol `normal,build` de `domain` no incluye SDKs de motores ni crates del workspace

### Modified Capabilities

- Ninguna. `cargo-workspace` sigue describiendo el workspace vacío de T-00-01; no se cambia su contrato en este ZZ.

## Impact

- **Repo:** `scripts/check-domain-deps.sh` (nuevo). No toca `vortex-core/crates/domain` en el árbol git; las violaciones se prueban con fixtures
- **APIs / OpenAPI:** ninguna
- **Build:** el script invoca `cargo tree`; no agrega dependencias al workspace
- **Ticket:** cubre el **Hace** del script; no cierra T-00-02 (falta el job de CI en T-00-02-02)
- **BREAKING:** no. Script nuevo; `check-workspace.sh` no cambia
