# Proposal

## Ticket padre

- **Ticket:** [T-00-20](../../../docs/7-tickets.md) — Arranque ordenado de `vortex serve` (paso 1)
- **Change:** `T-00-20`
- **HU:** [HU-00-06](../../../docs/6-historias.md#hu-00-06--servicio-del-sistema-y-arranque-ordenado) (MVP parcial: queda `launchd` en v2; este ticket cubre configuración inválida y el camino hasta `/health` `ok` con Storage)
- **Depende de:** T-00-09, T-00-13, T-00-19 (cerrados en `develop`)
- **Hace:** secuencia configuración → logs → Storage y migraciones → registro de salud → API por HTTP en `localhost` (MVP del curso; HTTPS se suma en v2 con T-00-16); `/health` en `starting` hasta terminar; cada fase registra su duración; si falla Storage o la configuración, no se expone la API
- **No hace:** supervisor ni worker (T-00-27)
- **Listo cuando:** un test de integración arranca el binario, espera `ok` y verifica que con una base inaccesible el proceso termina con error

## Why

`vortex serve` ya carga `Config`, escucha HTTP y publica `/health` con todos los componentes `disabled` (agregado `ok`), pero **no** instala el subscriber de logs ni abre SurrealDB. HU-00-06 y [3.4.13](../../../docs/3-componentes.md#3413-plataforma-binvortex) exigen el orden configuración → Storage → API: sin eso, un TOML válido deja la API arriba con Storage apagado, y T-00-27 no tiene un composition root sobre el que enganchar Meilisearch y el worker. Hay que cablear el paso 1 ahora, en HTTP `localhost`, antes del supervisor (v2) y del paso 2.

## What Changes

- `vortex serve` sigue el orden **configuración → `init_logging` → Storage (`Db::open` en `<paths.datos>/db` + `Db::migrate`) → registro de salud → bind HTTP**
- El `HealthRegistry` publica `storage = starting` **antes** de abrir la base y `storage = up` **después** de migrar; el resto de componentes sigue `disabled`. El listener TCP MUST NOT aceptarse hasta que Storage y migraciones terminen OK, así un fallo de config, logs o Storage no deja la API expuesta. El primer `GET /health` observable es entonces `ok` con `components.storage.status = up` (Listo cuando)
- Cada fase emite un span / evento `tracing` con su duración (`boot.config`, `boot.logging`, `boot.storage`, `boot.health`, `boot.api`)
- El handle `Db` permanece vivo en el composition root mientras el proceso sirve (no se inyecta en `AppState` todavía: no hay rutas de negocio)
- Allowlist: `bin/vortex` MAY declarar path `storage`
- TDD: happy = binario arranca, logs de fases, espera `/health` `ok` + `storage` `up`; unhappy = config inválida (ya cubierto), logs no escribibles, base inaccesible → exit ≠ 0 y el puerto no queda escuchando
- **BREAKING** (proceso): `vortex serve` ahora abre RocksDB y escribe `<datos>/logs/`; un `paths.datos` no escribible o un `db` ilegible abortan antes de bind (antes el proceso escuchaba igual)
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `vortex` declara path `storage`

## Non-goals

Este change **no** cubre:

- Supervisor de sidecars, worker de índice, verificación de Meilisearch por URL — **No hace** del ticket; **T-00-27** (paso 2) y **T-00-23** / **T-00-25** (v2)
- HTTPS, `rustls`, redirección :80, `vectra.local`, `launchd` — **T-00-16**, **T-00-17**, **T-00-30** (v2)
- Canal de administración Unix socket — **T-00-22** (v2)
- Inyectar `Db` en `AppState` o rutas de catálogo — T-02-06+
- Probes HTTP a Meilisearch/Ollama ni estados `meilisearch`/`llm`/`sync` distintos de `disabled` — T-00-27 / adaptadores
- Tablas de negocio nuevas (el motor ya aplica `0000_migracion`) — T-02-01, T-03-01
- Cambiar `docs/openapi.yaml` (`GET /health` ya está)
- Crates nuevas en `[workspace.dependencies]`

## Capabilities

### New Capabilities

- (ninguna — el arranque vive en el composition root de `vortex serve`, ya especificado)

### Modified Capabilities

- `vortex-api`: `vortex serve` instala logging, abre Storage + migraciones, publica salud (`storage` `starting` → `up`) y **después** escucha HTTP; fallo de config/logs/Storage aborta sin bind; cada fase registra duración
- `vortex-logging`: `vortex serve` **sí** llama a `init_logging` (T-00-06 lo dejó sin enganchar a `main`); un fallo de logs aborta sin Storage ni HTTP
- `cargo-workspace`: `bin/vortex` MAY declarar path `storage`; `cargo test --workspace` MUST cubrir el boot (espera `ok` + base inaccesible)

## Impact

- **Repo:** `vortex-core/bin/vortex/` (`Cargo.toml`, `serve.rs`, `logging.rs` deja de marcar `init_logging` como dead_code, `tests/serve.rs`), `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** sin paths nuevos; `GET /health` pasa a reportar `storage: up` en el proceso real (el contrato no cambia)
- **CI / verificación:** [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core/` (`cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace`); `scripts/check-workspace.sh`; `openspec validate T-00-20` (o `t-00-20` si el CLI exige minúsculas)
- **Ticket:** cierra T-00-20 (Listo cuando: integración espera `ok`; base inaccesible → error)
- **BREAKING** (proceso y harness): ver What Changes
