# Proposal

## Ticket padre

- **Ticket:** [T-00-06](../../../docs/7-tickets.md) — Logs estructurados con rotación
- **Change:** `T-00-06`
- **HU:** [HU-00-09](../../../docs/6-historias.md#hu-00-09--logs-estructurados)
- **Depende de:** T-00-05 (cerrado en `develop`)
- **Hace:** `tracing-subscriber` con capa JSON a `<datos>/logs/vortex.log`, rotación diaria con `tracing-appender` y retención configurable (defecto 14 archivos); salida legible en consola en modo desarrollo; nivel por configuración
- **No hace:** métricas (T-00-14)
- **Listo cuando:** un test genera logs en un directorio temporal y verifica el formato JSON y la retención

## Why

Vortex Core ya carga `Config` (T-00-05) con `logs.nivel`, `logs.retencion` y `paths.datos`, pero el proceso no emite trazas: no hay subscriber, ni archivo JSON, ni rotación. HU-00-09 pide diagnosticar sin llenar el disco; T-00-12 (spans por petición), T-00-20 (arranque ordenado) y el resto de crates van a `tracing::info!` / spans contra este subscriber. Hay que instalarlo ahora, comprobable en un temp dir, antes de la CLI y del servidor.

## What Changes

- Agregar en `bin/vortex` un módulo de logging que instala `tracing-subscriber`: capa JSON hacia `<paths.datos>/logs/` con prefijo `vortex.log`, rotación diaria (`tracing-appender`) y retención `logs.retencion` (el example ya usa 14)
- Crear el directorio `logs/` si falta; el nivel del filtro sale de `logs.nivel`
- En modo desarrollo, una segunda capa fmt legible a stderr; el archivo JSON se escribe siempre
- TDD: happy = emitir un evento en un directorio temporal, parsear JSON y comprobar que al superar la retención solo quedan N archivos; unhappy = no se puede crear/escribir el directorio de logs
- Pin `tracing-subscriber = "=0.3.23"` en `[workspace.dependencies]` (el ticket nombra la crate; T-00-01 pinneó `tracing` y `tracing-appender` pero no el subscriber)
- Ampliar la allowlist de `bin/vortex` en `check-workspace.sh` con `tracing`, `tracing-subscriber` y `tracing-appender` (`workspace = true`); `serde_json` solo en `[dev-dependencies]` para parsear líneas en tests
- **No** inicializar el subscriber en `main` todavía: `vortex --config config.example.toml` seguiría intentando escribir en `/var/lib/vortex/datos/logs` y rompería los tests de T-00-05. T-00-20 (secuencia configuración → logs) lo engancha al arranque de `serve`

## Non-goals

Este change **no** cubre:

- Métricas Prometheus / `GET /metrics` — **No hace** del ticket; **T-00-14**
- Spans por petición HTTP (método, ruta, estado, duración) — **T-00-12** (este change solo instala el subscriber para que esos spans se graben)
- CLI `clap`, subcomando `serve`, arranque ordenado — **T-00-07**, **T-00-20**
- Recarga en caliente de config o del nivel de log — T-00-05 No hace
- HTTPS, `launchd`, sidecars, Storage, Axum, `GET /health` — tickets posteriores
- Instrumentar crates lib (`domain`, `storage`, …) con `tracing` — cada ticket de producto; aquí no se tocan libs
- Overlay `RUST_LOG` como fuente de verdad (el nivel es `logs.nivel`)
- `DomainError` / `problem+json` — fallos de logging son error de proceso en el binario
- Cambios al esquema TOML (`logs.nivel` / `logs.retencion` ya existen en `vortex-config`)
- Cambios a `docs/openapi.yaml`

## Capabilities

### New Capabilities

- `vortex-logging`: subscriber `tracing` de Vortex Core (JSON rotado bajo `<datos>/logs/vortex.log*`, retención, consola pretty en desarrollo, nivel desde `Config`); tests en directorio temporal cubren JSON y retención

### Modified Capabilities

- `cargo-workspace`: `[workspace.dependencies]` agrega el pin exacto `tracing-subscriber = "=0.3.23"`. `bin/vortex` MAY declarar `tracing`, `tracing-subscriber` y `tracing-appender` además de `toml` / `serde` / `thiserror` (todas `workspace = true`). MAY declarar `serde_json` en `[dev-dependencies]` con `workspace = true`. MUST NOT declarar `clap`, `metrics` ni otras crates. `cargo test --workspace` MUST ejecutar también los tests de logging del package `vortex`.

## Impact

- **Repo:** `vortex-core/bin/vortex/` (`Cargo.toml`, módulo `logging`, tests), `vortex-core/Cargo.toml` / `Cargo.lock` (pin `tracing-subscriber`), `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** tests del package `vortex` (JSON + retención + error de I/O), más [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core`; `check-workspace.sh` en verde con la allowlist ampliada
- **Ticket:** cierra T-00-06 (Listo cuando: test en temp dir verifica JSON y retención)
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `bin/vortex` declara `tracing` / `tracing-subscriber` / `tracing-appender` o `serde_json` en `[dev-dependencies]`; sigue fallando si declara `clap`, `metrics` u otra crate fuera de la lista
