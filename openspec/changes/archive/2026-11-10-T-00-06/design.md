# Design

## Context

En `develop`, `bin/vortex` carga y valida `Config` (T-00-05) y sale 0; `logs.nivel` / `logs.retencion` / `paths.datos` ya existen. `[workspace.dependencies]` pinnea `tracing = "=0.1.44"` y `tracing-appender = "=0.2.5"` pero **no** `tracing-subscriber`. `check-workspace.sh` solo permite `{toml, serde, thiserror}` en el binario y rechaza `[dev-dependencies]`. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: no métricas (T-00-14); no spans HTTP (T-00-12); no `clap` / `serve` (T-00-07); no enganchar logging en `main` (los tests CLI de T-00-05 usan `config.example.toml` con `datos = "/var/lib/vortex/datos"`); ~2 h; MVP.

## Goals / Non-Goals

**Goals:**

- Módulo `logging` en `bin/vortex` que, dado un `Config`, escribe JSON rotado bajo `<datos>/logs/vortex.log*` y aplica retención
- Tests en temp dir: JSON parseable, filtro de nivel, span en el JSON, retención, pretty vs production, I/O unhappy
- Pin `tracing-subscriber` y allowlist alineada en `check-workspace.sh`

**Non-Goals (diseño):**

- `set_global_default` desde `main` (T-00-20)
- `assert_cmd` / crates de test nuevas (parsear JSON con `serde_json` en `[dev-dependencies]`)
- Reloj inyectable para simular el cambio de día (la retención se prueba con archivos prefabricados)
- Mapear `LoggingError` a `DomainError`

## Decisions

### 1. Módulo `logging` en `bin/vortex`; tests con subscriber scoped, no global

**Elección:** `vortex-core/bin/vortex/src/logging.rs`. API de tests: construir un subscriber + guardia de flush y ejecutar el cuerpo con `tracing::subscriber::with_default` (no `set_global_default`). `init_logging(&Config)` queda para T-00-20; en este change **no** se llama desde `main.rs`. Sin `[lib]`.

**Por qué:** cargo corre unitarios en el mismo proceso: un subscriber global rompe tests en paralelo. `with_default` es scoped y cumple el Listo cuando. Llamar `init` desde `main` escribiría en `/var/lib/vortex/datos/logs` con el example y rompería `config_cli`.

**Alternativa:** `set_global_default` + `--test-threads=1`. Se descartó: frágil y acoplado al harness. **Alternativa:** crate `logging` nuevo. Se descartó: [3.4.13](../../../docs/3-componentes.md#3413-plataforma-binvortex) asigna logs al composition root; ~2 h.

### 2. `tracing-subscriber` 0.3.23 + `tracing-appender` 0.2.5 + `tracing` 0.1.44

**Elección:** pin nuevo `tracing-subscriber = "=0.3.23"` (crates.io al planificar, 2026-11-10). En `bin/vortex`: `tracing`, `tracing-subscriber = { workspace = true, features = ["json", "env-filter"] }`, `tracing-appender`, más las deps de T-00-05. Capa JSON: `fmt::layer().json()` con `RollingFileAppender::builder().rotation(Rotation::DAILY).filename_prefix("vortex.log").max_log_files(retencion)`. Writer no bloqueante en el camino de `init_logging`; en tests, writer bloqueante (o drop de `WorkerGuard`) para flush determinista.

**Por qué:** el ticket nombra esas crates. El Builder de 0.2.5 expone `max_log_files`. Prefijo `vortex.log` produce `vortex.log.YYYY-MM-DD` (familia `<datos>/logs/vortex.log*` del ticket). `env-filter` no lee `RUST_LOG`: se construye `EnvFilter::new(&config.logs.nivel)`.

**Alternativa:** `log` + `env_logger`. Se descartó: el stack y DC-04 son `tracing`. **Alternativa:** rotación por tamaño. Se descartó: el ticket pide diaria.

### 3. Retención: `max_log_files` + poda explícita al iniciar

**Elección:** función `retain_log_files(dir, keep: u32)` que lista archivos cuyo nombre empieza por `vortex.log`, ordena (nombre ISO / mtime) y borra los más viejos hasta dejar `keep`. Se invoca al inicializar. Además `max_log_files` en el appender para el prune en rotación.

**Por qué:** `tracing-appender` solo poda al rotar; un test no puede esperar al cambio de día. El Listo cuando exige verificar retención en temp dir. Poda al iniciar cubre bajar `logs.retencion` en el TOML.

**Alternativa:** `Rotation::MINUTELY` solo en tests. Se descartó: no es el comportamiento de producción. **Alternativa:** crate `file-rotate`. Se descartó: el ticket fija `tracing-appender`.

### 4. Modo desarrollo vía `VORTEX_ENV` (y TTY si falta)

**Elección:** pretty `fmt::layer()` a stderr si `VORTEX_ENV` es `development`/`dev`, o si la variable está ausente **y** stderr es un terminal (`IsTerminal`). Si vale `production`/`prod`, no se instala esa capa. El JSON a archivo no depende del modo. En tests, un `MakeWriter` inyectable captura stderr.

**Por qué:** el TOML no tiene `mode` (T-00-05 cerró el esquema). `debug_assertions` no sirve en un binario release local. `launchd` (T-00-29, v2) pondrá `VORTEX_ENV=production` o no tendrá TTY.

**Alternativa:** siempre pretty a stderr. Se descartó: el ticket limita la consola legible al modo desarrollo. **Alternativa:** clave TOML nueva. Se descartó: cambiaría `vortex-config`.

### 5. `LoggingError { key, reason }`; no reusar `ConfigError` en el tipo

**Elección:** `thiserror` con `Display` `logging: {key}: {reason}`. Claves: `paths.datos` (no es directorio / I/O al crear el árbol), `logs` (appender). `create_dir_all` de `<datos>/logs`. Unhappy: `datos` es un archivo regular.

**Por qué:** mismo contrato de mensaje que T-00-05 (clave + motivo) sin mezclar parseo TOML con I/O de logs. T-00-20 podrá imprimir ambos en stderr.

**Alternativa:** reusar `ConfigError`. Se descartó: acopla el módulo al parser. **Alternativa:** `DomainError`. Se descartó: no hay caso de uso HTTP (T-00-11).

### 6. Allowlist: tracing* en deps; `serde_json` solo en dev-deps

**Elección:** `vortex_deps_allowed` acepta `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender}` en `[dependencies]` (`workspace = true`) y `{serde_json}` en `[dev-dependencies]`. Unhappy de fixture sigue siendo `clap` (T-00-07); se documenta que `metrics` también falla (T-00-14). `check_pin tracing-subscriber 0.3.23`.

**Por qué:** `serde_json` ya está pinneado y solo sirve para parsear líneas en tests (igual que `domain`). No meter `metrics` ni `clap`.

**Alternativa:** parsear JSON a mano. Se descartó: el Listo cuando pide verificar formato JSON. **Alternativa:** `serde_json` en `[dependencies]`. Se descartó: el binario no serializa logs a mano.

## Risks / Trade-offs

- **[Nombre `vortex.log.YYYY-MM-DD` vs `vortex.log` literal]** → Mitigación: spec habla de prefijo `vortex.log*`; el ticket cita el path de familia. Sin symlink.
- **[macOS case-insensitive: `openspec new change T-00-06`]** → Mitigación: scaffold en `t-00-06` y rename a `T-00-06` (igual que T-00-05).
- **[`with_default` vs `init` global]** → Mitigación: T-00-20 llama `init_logging` una vez al arrancar `serve` y guarda el `WorkerGuard`. Este change deja la función lista y sin usarla en `main`.
- **[Permisos de «no escribible» en CI macOS]** → Mitigación: el unhappy principal es `datos` como archivo regular (portable). El caso chmod se intenta; si el runner ignora permisos, no es el único unhappy.
- **[Pretty a stderr vs captura en tests]** → Mitigación: `MakeWriter` inyectable en la API de test; no depender de heredar stderr del proceso cargo.

## Migration Plan

1. Red: tests de JSON / filtro / span / retención / pretty / I/O en `src/logging.rs` (no compilan).
2. Red: fixture `clap` sigue ≠ 0; el repo aún sin `tracing-subscriber` en el pin listo del script.
3. Green: pin, allowlist, deps de `vortex`, `logging.rs`, fixtures.
4. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-06`.
5. Rollback: revertir módulo, pin, allowlist y fixtures. Sin datos de usuario ni API.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. `VORTEX_ENV` queda fijado aquí; T-00-20 / T-00-29 solo tienen que setearlo al arrancar el servicio.
