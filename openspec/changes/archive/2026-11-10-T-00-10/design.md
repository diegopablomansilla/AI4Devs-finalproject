# Design

## Context

En `develop`, `crates/api` es un `lib.rs` vacío sin `[dependencies]`; `check-workspace.sh` falla si cualquier lib distinto de `domain`/`storage` declara deps. `bin/vortex` parsea clap y despacha **todos** los subcomandos (incluido `serve`) a `no implementado` / exit 1. `Config::from_path` existe pero `main` no lo llama (`#![allow(dead_code)]`). `axum = "=0.8.9"` y `tokio = "=1.54.1"` ya están en `[workspace.dependencies]`. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: HTTP en `localhost` (MVP); no TLS (`axum-server` es T-00-16); no `problem+json` (T-00-11); no `init_logging` ni Storage (T-00-20); ~2 h.

## Goals / Non-Goals

**Goals:**

- Router vacío + `AppState` clonable en `api`; `vortex serve` escucha HTTP y apaga con SIGTERM/SIGINT
- Allowlist de `api` (`axum` + `tokio` de test) y de `vortex` (`api`, `tokio`, `axum`)
- Tests de crate (404) y de binario (404, señales, config, bind ocupado)

**Non-Goals (diseño):**

- `tower` / `reqwest` / `assert_cmd` como deps nuevas (no están en la tabla de pins)
- `#[tokio::main]` en todo `main` (solo el camino `serve` entra al runtime)
- Traits dummy de catálogo/salud en `AppState`
- `env = "VORTEX_CONFIG"` en clap; la resolución sigue en código (`--config` / `std::env`)

## Decisions

### 1. `api` construye el router; `bin/vortex` enlaza y apaga

**Elección:** `api::build_router(AppState) -> axum::Router`. El binario resuelve config, hace `tokio::net::TcpListener::bind(server.bind)`, `axum::serve(listener, router).with_graceful_shutdown(shutdown_signal())`. `AppState` es `Clone` sobre `Arc<AppStateInner>` vacío. Sin rutas: el fallback de Axum es 404.

**Por qué:** el ticket nombra `build_router` y `AppState`; [3.2](../../../docs/3-componentes.md) deja el composition root en `bin/vortex`. Inner-`Arc` deja agregar `Arc<dyn Trait>` en T-00-13+ sin cambiar el tipo público.

**Alternativa:** `api::serve(addr, state)` encapsula bind+señales. Se descartó: mezcla I/O de proceso con el crate de HTTP y complica tests de config en el binario. **Alternativa:** `axum-server`. Se descartó: T-00-16 (TLS).

### 2. Runtime Tokio solo en `serve`; señales Unix

**Elección:** `Commands::Serve` llama a `serve::run(cli.config)` que crea el runtime (`macros` no hace falta en `main`). `shutdown_signal`: `tokio::signal::ctrl_c` y, en Unix, `SignalKind::terminate`. Exit 0 tras `serve(...).await`. Errores de config/bind: stderr + exit ≠ 0 (`ConfigError` ya formatea `config: {key}: {reason}`).

**Por qué:** stubs no deben pagar el runtime ni tocar el TOML. El Nodo Maestro es macOS; `cfg(unix)` cubre SIGTERM. `SIGINT` = Ctrl+C.

**Alternativa:** `#[tokio::main]` en `main`. Se descartó: los stubs y `--help` no necesitan async. **Alternativa:** crate `nix` para `kill` en tests. Se descartó: `kill -TERM` / `kill -INT` vía `std::process::Command` basta.

### 3. Resolución de config en el camino `serve`

**Elección:** helper `resolve_config_path(flag: Option<&Path>)`: si hay `--config`, ese path; si no, `VORTEX_CONFIG`; si no, `ConfigError` con clave `--config` / `VORTEX_CONFIG`. Luego `Config::from_path`. Se quita `#![allow(dead_code)]` de `config.rs`. No se llama `init_logging`.

**Por qué:** T-00-07 dejó el algoritmo documentado y el stub sin I/O. T-00-20 engancha logs y Storage encima de este bind.

**Alternativa:** clap `env = "VORTEX_CONFIG"`. Se pospone: duplicaría el contrato ya testeado en T-00-05 y mezclaría parseo con carga.

### 4. Allowlist y fixtures

**Elección:** `api_deps_allowed`: `[dependencies]` solo `axum` (`workspace = true`); `[dev-dependencies]` solo `tokio` (`workspace = true`; features `macros` / `rt` / `rt-multi-thread` / `net`). `vortex_deps_allowed` suma `api` (path a `crates/api`), `tokio`, `axum`. Fixture `api con dependencies` pasa a crate prohibida (`serde`/`clap`); se agrega happy `api con allowlist axum`. Unhappy de `vortex` sigue siendo `metrics`.

**Por qué:** mismo patrón que `storage`/`clap`. Sin pin nuevo de `tower`: los tests del router usan `axum::serve` + `TcpStream` y una petición HTTP/1.1 mínima, o el binario real.

**Alternativa:** declarar `tower` en workspace para `ServiceExt::oneshot`. Se descartó: no está en la tabla de versiones y no hace falta para el Listo cuando.

### 5. Tests: puerto efímero + proceso real para señales

**Elección:** tests de `api` en `crates/api/tests/router.rs` (`build_router_unknown_path_returns_404`). Tests de binario en `bin/vortex/tests/serve.rs` (tabla de la spec). Puerto: `TcpListener::bind("127.0.0.1:0")`, leer `local_addr`, soltar, escribir ese `host:port` en un TOML temporal derivado del example. Señales: spawn `CARGO_BIN_EXE_vortex`, esperar 404, `kill -TERM`/`-INT`, assert exit 0 y que el puerto vuelve a bindar. Bind ocupado: mantener el listener en el test y spawn `serve`. Timeout de espera para no colgar CI.

**Por qué:** el Listo cuando es observable en el proceso. `:0` dentro del TOML no publica el puerto elegido al test.

**Alternativa:** parsear "listening on" en logs. Se descartó: este change no instala `init_logging`. **Alternativa:** dejar los tests `vortex_serve_prints_unimplemented`. Se descartó: **BREAKING** del stub.

## Risks / Trade-offs

- **[Carrera al reutilizar el puerto 0]** → Mitigación: si el happy path pierde el puerto, reintentar bind; el unhappy de bind ocupado no suelta el listener.
- **[Test de SIGTERM se cuelga]** → Mitigación: timeout al `wait`; si vence, `kill -KILL` y fallo del test.
- **[macOS case-insensitive: `openspec new change t-00-10`]** → Mitigación: rename a `T-00-10` (igual que T-00-07).
- **[404 sin `problem+json`]** → Aceptado: T-00-11. Los tests solo aserten status 404.
- **[Fixture `api con dependencies` deja de fallar si no se reescribe]** → Mitigación: tarea explícita en tasks.md; unhappy nuevo con `serde`/`clap`.

## Migration Plan

1. Red: tests de `router.rs` / `serve.rs` no compilan o fallan; tests de stub de `serve` se reescriben.
2. Red: `check-workspace.sh` aún rechaza `axum` en `api`.
3. Green: allowlist + deps + `build_router` + `serve::run` + shutdown + fixtures.
4. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-10`.
5. Rollback: revertir crate `api`, módulo `serve`, allowlist, fixtures y tests. Sin datos de usuario ni contrato OpenAPI.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El 404 de Axum (cuerpo vacío o texto) queda como detalle de implementación hasta T-00-11.
