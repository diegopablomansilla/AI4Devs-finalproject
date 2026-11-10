# Proposal

## Ticket padre

- **Ticket:** [T-00-05](../../../docs/7-tickets.md) — Configuración TOML con validación
- **Change:** `T-00-05`
- **HU:** [HU-00-06](../../../docs/6-historias.md#hu-00-06--servicio-del-sistema-y-arranque-ordenado)
- **Depende de:** T-00-01 (cerrado en `develop`)
- **Hace:** `Config` con secciones `paths` (datos, modelos, backups, web), `server` (bind, hostname), `negocio` (moneda), `meilisearch`, `ollama`, `backup` (hora, retención) y `logs`; carga desde `--config` o `VORTEX_CONFIG`; `config.example.toml`
- **No hace:** recarga en caliente
- **Listo cuando:** una configuración inválida produce un mensaje que indica clave y motivo, y el proceso termina con código distinto de 0

## Why

Vortex Core todavía no tiene un archivo de configuración: `bin/vortex` es un `main` vacío y el arranque no puede fallar temprano si el operador entrega un TOML roto. HU-00-06 exige que una configuración inválida impida el arranque e indique el error; T-00-06, T-00-07 y T-00-08 van a leer rutas, bind, sidecars y logs desde este `Config`. Hay que materializarlo ahora, con validación por clave, antes de la CLI y de Storage.

## What Changes

- Agregar `Config` en `bin/vortex` (composition root de [3.4.13](../../../docs/3-componentes.md#3413-plataforma-binvortex)) con las siete secciones del ticket
- Cargar el archivo desde `--config <path>` (gana) o, si no hay flag, `VORTEX_CONFIG`; sin path → error de proceso
- Validar claves y valores; si falla, escribir a stderr un mensaje con **clave** y **motivo** y salir ≠ 0
- Si el TOML es válido, el proceso termina 0 (aún no hay `serve`; T-00-07 agrega subcomandos)
- Publicar `vortex-core/config.example.toml` válido contra el mismo esquema
- Pin `toml = "=1.2.1"` en `[workspace.dependencies]`; `bin/vortex` declara `toml`, `serde` (feature `derive`) y `thiserror` con `workspace = true`
- Relajar `scripts/check-workspace.sh` **solo** para esa allowlist en `vortex`; el resto de libs (salvo `domain`) siguen sin deps; **no** se declara `clap` (T-00-07)
- TDD: happy = TOML completo + example + `--config` / `VORTEX_CONFIG` → exit 0; unhappy = TOML inválido, clave faltante/valor ilegal, archivo inexistente, path ausente → mensaje con clave+motivo y exit ≠ 0

## Non-goals

Este change **no** cubre:

- Recarga en caliente / *watch* del archivo — **No hace** del ticket
- CLI `clap` con subcomandos (`serve`, `reindex`, …) ni `--help` completo — **T-00-07** (reutilizará `--config`)
- Logs estructurados, rotación y consola de desarrollo — **T-00-06** (lee `logs` / `paths.datos`)
- Apertura de SurrealDB, migraciones, Axum, `GET /health` — T-00-08 … T-00-13
- Supervisor de sidecars, `launchd`, HTTPS / `vectra.local` — T-00-15+ (v2); las secciones `meilisearch` / `ollama` / `backup` **sí** se parsean ahora para no cambiar el esquema después
- Crear directorios (`datos`, `modelos`, …) ni exigir que existan en disco
- Overlay de variables de entorno por clave (solo `VORTEX_CONFIG` como path); `figment` / crate `config`
- `DomainError` / `problem+json` — errores de arranque son `ConfigError` en el binario (T-00-03 / T-00-11)
- Relajar `[dependencies]` en libs distintas de `domain`; no tocar `check-domain-deps.sh` más que confirmar que sigue en verde
- Cambios a `docs/openapi.yaml`

## Capabilities

### New Capabilities

- `vortex-config`: carga y validación del TOML de Vortex Core (`Config` + `config.example.toml` + resolución `--config` / `VORTEX_CONFIG`); configuración inválida nombra clave y motivo y el proceso sale ≠ 0

### Modified Capabilities

- `cargo-workspace`: `[workspace.dependencies]` agrega el pin exacto `toml = "=1.2.1"`. `bin/vortex` deja de estar vacío: MAY declarar en `[dependencies]` únicamente `toml`, `serde` y `thiserror` (todas `workspace = true`; `serde` con feature `derive`). MUST NOT declarar `clap` ni otras crates. Los libs distintos de `domain` siguen sin `[dependencies]` / `[dev-dependencies]`. El requirement «binario vacío» se reemplaza por «binario con Config». `cargo test --workspace` MUST ejecutar también los tests de carga/validación de `vortex`.

## Impact

- **Repo:** `vortex-core/bin/vortex/` (`Cargo.toml`, `src/main.rs`, módulo `config`), `vortex-core/config.example.toml`, `vortex-core/Cargo.toml` / `Cargo.lock` (pin `toml`), `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** tests del package `vortex` (unitarios + binario), más [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sobre `vortex-core`; `check-workspace.sh` en verde con la allowlist de `vortex`
- **Ticket:** cierra T-00-05 (Listo cuando: inválido → clave+motivo y exit ≠ 0)
- **BREAKING** (harness interno): `check-workspace.sh` deja de fallar si `bin/vortex` declara `toml` / `serde` / `thiserror`; sigue fallando si declara otra crate (p. ej. `clap`) o si un lib distinto de `domain` declara deps
