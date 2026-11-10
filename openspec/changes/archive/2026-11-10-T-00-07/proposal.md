# Proposal

## Ticket padre

- **Ticket:** [T-00-07](../../../docs/7-tickets.md) — CLI con subcomandos
- **Change:** `T-00-07`
- **HU:** [HU-00-06](../../../docs/6-historias.md#hu-00-06--servicio-del-sistema-y-arranque-ordenado)
- **Depende de:** T-00-05 (cerrado en `develop`; T-00-06 también está en `develop`)
- **Hace:** `clap` en `bin/vortex` con `serve`, `reindex`, `backup`, `cert`, `models`, `seed`, `verify` y `service install|uninstall`; opción global `--config`. Los no implementados responden "no implementado" con código 1
- **No hace:** la lógica de cada comando
- **Listo cuando:** `vortex --help` lista todos los subcomandos

## Why

Vortex Core ya carga `Config` (T-00-05) y tiene logging (T-00-06), pero `bin/vortex` no expone subcomandos: no hay `serve`, ni esqueleto para `seed` / `reindex` / `backup`. HU-00-06 y [3.4.13](../../../docs/3-componentes.md#3413-plataforma-binvortex) exigen una CLI operativa; T-00-10 engancha el servidor Axum a `vortex serve`. Hay que materializar `clap` ahora, con todos los nombres del ticket y stubs, antes de implementar ningún comando.

## What Changes

- Agregar `clap` (`=4.6.7`, ya pinneado) en `bin/vortex` con subcomandos `serve`, `reindex`, `backup`, `cert`, `models`, `seed`, `verify` y `service` anidado (`install` | `uninstall`)
- Opción global `--config` (reemplaza el parser manual de argv); `VORTEX_CONFIG` sigue resolviendo el path cuando un comando cargue `Config` (T-00-10+)
- Cada subcomando reconocido y aún sin lógica imprime `no implementado` en stderr y termina con código 1
- `vortex --help` lista los ocho subcomandos de primer nivel y sale 0 **sin** exigir archivo de configuración (Listo cuando)
- **BREAKING** (CLI): `vortex` / `vortex --config <válido>` sin subcomando ya no cargan el TOML ni salen 0; clap exige un subcomando. Los tests de `config_cli` que cubrían ese contrato de T-00-05 se reescriben
- Ampliar la allowlist de `check-workspace.sh`: `bin/vortex` MAY declarar `clap` (`workspace = true`, feature `derive`). El unhappy de fixture pasa de `clap` a `metrics` (T-00-14)
- TDD: happy = `--help` lista todos + `--config` aparece como flag global + `service --help` lista `install`/`uninstall`; unhappy = comando desconocido o subcomando faltante → ≠ 0 y no es el mensaje de stub; stub = `no implementado` y exit 1

## Non-goals

Este change **no** cubre:

- La lógica de cada comando — **No hace** del ticket
- `vortex serve` escuchando HTTP, `AppState`, apagado ordenado — **T-00-10**
- `vortex cert`, HTTPS, `vectra.local` — T-00-15 … T-00-18 (v2)
- `vortex models` (Meilisearch / embeddings / Ollama) — T-00-24, T-05-05, T-05-12 (v2)
- `vortex seed` / `reindex` / `verify` / `backup` reales — T-02-42+, T-03-11, T-02-45, T-00-28 (varios v2)
- `vortex service install|uninstall` con `launchd` — **T-00-30** (v2)
- Cargar `Config` ni instalar el subscriber de logs al despachar un stub — T-00-10 / T-00-20 enganchan `serve`
- Recarga en caliente del TOML — T-00-05 No hace
- Métricas, Axum, Storage, OpenAPI — T-00-08, T-00-11+, T-00-14, T-09-03
- Relajar `[dependencies]` en libs; no tocar `check-domain-deps.sh` más que confirmar que sigue en verde
- Cambios a `docs/openapi.yaml`

## Capabilities

### New Capabilities

- `vortex-cli`: esqueleto `clap` de Vortex Core (subcomandos del ticket, `--config` global, `--help` completo, stubs `no implementado` con exit 1)

### Modified Capabilities

- `cargo-workspace`: `bin/vortex` MAY declarar `clap` además de `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender}` (todas `workspace = true`; `clap` MAY activar feature `derive`). MUST NOT declarar `metrics` ni otras crates. `cargo test --workspace` MUST ejecutar también los tests de CLI del package `vortex`.
- `vortex-config`: `--config` pasa a ser flag global de clap (`--config <path>` / `--config=<path>`). El proceso **ya no** carga el TOML ni sale 0 solo por tener un archivo válido: hace falta un subcomando. `Config::from_path` y el esquema TOML no cambian; la resolución `--config` / `VORTEX_CONFIG` queda para cuando un comando implementado cargue config (T-00-10).

## Impact

- **Repo:** `vortex-core/bin/vortex/` (`Cargo.toml`, `src/main.rs`, módulo CLI, tests de binario), `vortex-core/Cargo.lock`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** tests del package `vortex` (help + stubs + argv inválido), más [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core`; `check-workspace.sh` en verde con `clap` en la allowlist
- **Ticket:** cierra T-00-07 (Listo cuando: `vortex --help` lista todos los subcomandos)
- **BREAKING** (CLI y harness interno): sin subcomando el proceso ya no sale 0; `check-workspace.sh` deja de fallar si `bin/vortex` declara `clap`; sigue fallando si declara `metrics` u otra crate fuera de la lista
