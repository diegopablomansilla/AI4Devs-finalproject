# Design

## Context

En `develop`, `bin/vortex` es un `fn main` vacío y `check-workspace.sh` rechaza cualquier `[dependencies]` ahí. `[workspace.dependencies]` pinnea `serde`, `thiserror` y `clap`, pero **no** `toml`. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: no recarga en caliente (No hace); no `clap` (T-00-07); no logs/Storage/Axum; ~2 h; HU-00-06 escenario «Configuración inválida»; MVP HTTP en `localhost` ([2.7](../../docs/2-arquitectura.md#27-red-local-y-seguridad-mínima)).

## Goals / Non-Goals

**Goals:**

- `Config` parseable desde TOML con las siete secciones; errores con clave + motivo
- Resolución `--config` > `VORTEX_CONFIG`; proceso 0 / ≠ 0 según Listo cuando
- `config.example.toml` y allowlist mínima en `check-workspace.sh` para `toml`/`serde`/`thiserror` en `vortex`

**Non-Goals (diseño):**

- `clap` Parser / subcomandos / `--help`
- `figment`, crate `config`, overlay por clave de entorno
- Crear directorios o verificar que existan
- Mapear `ConfigError` a `DomainError`
- Tests con `assert_cmd` / `predicates` (no pinnear crates de test)

## Decisions

### 1. `Config` vive en `bin/vortex`, no en `domain` ni en un crate nuevo

**Elección:** módulo `vortex-core/bin/vortex/src/config.rs` (structs `Config`, `Paths`, `Server`, …) usado por `main.rs`. Tests unitarios en el mismo módulo; tests de proceso en `vortex-core/bin/vortex/tests/config_cli.rs` con `env!("CARGO_BIN_EXE_vortex")`. No se agrega `[lib]`.

**Por qué:** [3.4.13](../../docs/3-componentes.md#3413-plataforma-binvortex) ya asigna la carga de configuración al composition root. T-00-01 no tiene un crate `config`; `domain` no puede I/O ni TOML (T-00-02).

**Alternativa:** crate lib nuevo. Se descartó: no está en [2.8](../../docs/2-arquitectura.md#28-estructura-de-repositorio) y excedería ~2 h. **Alternativa:** `[lib]` en el package `vortex`. Se descartó: los unitarios en `src/config.rs` bastan; el CLI se cubre con `tests/` del binario.

### 2. `toml` 1.2.1 + `serde` derive + `thiserror`; argv a mano

**Elección:** pin `toml = "=1.2.1"` (crates.io `1.2.1+spec-1.1.0`; Cargo ignora el build metadata con `=`). En `bin/vortex`: `toml`, `serde = { workspace = true, features = ["derive"] }`, `thiserror`. Parser de argv mínimo: `--config PATH` y `--config=PATH`. Sin `clap`.

**Por qué:** el ticket pide TOML, no CLI. `clap` es T-00-07 (reusa `--config`). `toml` 1.2.1 es la estable al planificar (2026-11-06); default features incluyen `serde` + `parse`.

**Alternativa:** `clap` ahora con un `Parser` de un flag. Se descartó: mezcla tickets. **Alternativa:** `figment`. Se descartó: el ticket no pide capas ni env por clave.

### 3. Claves TOML en el idioma del ticket; `deny_unknown_fields`

**Elección:** nombres de sección/campo literales del Hace (`paths.datos`, `negocio.moneda`, `backup.hora`, `backup.retencion`, `logs.nivel`, …). `#[serde(deny_unknown_fields)]` en cada struct. Tras el deserialize, validar formato (`SocketAddr` para `bind`, `^[A-Z]{3}$` para moneda, `HH:MM` 24 h, `url` con esquema `http`/`https`, `nivel` enum, `retencion >= 1`). No usar `reqwest::Url` ni crates extra.

**Por qué:** el Listo cuando exige **clave** en el mensaje; `deny_unknown_fields` convierte un typo en esa clave. Validar URLs a mano evita declarar `reqwest` en el binario.

**Alternativa:** claves inglés (`paths.data`, `business.currency`). Se descartó: el ticket ya nombra `negocio` y `datos`. **Alternativa:** exigir que las rutas existan. Se descartó: el primer arranque (T-00-08 / T-00-20) crea el árbol.

### 4. `ConfigError` con `key` + `reason` a stderr

**Elección:** `enum`/`struct` `ConfigError { key: String, reason: String }` (`thiserror`). `Display`: `config: {key}: {reason}`. Claves especiales: `toml` (sintaxis), `--config` / `VORTEX_CONFIG` (path ausente), el path literal (I/O). `main` imprime en **stderr** y `process::exit(1)`. Éxito: `exit(0)` sin servir. No hay recarga, ni `notify`, ni `SIGHUP`.

**Por qué:** cierra el Listo cuando y el escenario HU-00-06 sin T-00-06 (logs JSON). T-00-06 puede reenviar el mismo error a `tracing`.

**Alternativa:** `DomainError::InvalidInput`. Se descartó: no hay `field` de API; el arranque no es un caso de uso HTTP (T-00-11).

### 5. Allowlist de `vortex` en `check-workspace.sh` + fixture `clap`

**Elección:** función `vortex_deps_allowed` análoga a `domain_deps_allowed`: `[dependencies]` ⊆ `{toml, serde, thiserror}` todas `workspace = true`; rechazar `[dev-dependencies]` y cualquier otra crate. `check_pin toml 1.2.1`. Actualizar `scripts/fixtures/workspace/run-script-tests.sh`: happy = repo con allowlist; unhappy de `vortex` = `clap` en el binario (reemplaza «cualquier deps»).

**Por qué:** T-00-01-03 dejó el binario sin deps a propósito para este ticket. `clap` es el unhappy obvio (existe el pin, T-00-07 lo usará).

**Alternativa:** permitir cualquier crate del workspace en `vortex`. Se descartó: T-00-07 / T-00-08 deben ampliar la lista de forma explícita.

### 6. Example y defaults de MVP en el archivo, no en código

**Elección:** `vortex-core/config.example.toml` con `bind = "127.0.0.1:8080"`, `hostname = "localhost"`, `moneda = "ARS"`, Meilisearch `http://127.0.0.1:7700`, Ollama `http://127.0.0.1:11434`, `backup.hora = "03:00"`, retenciones `14`, `logs.nivel = "info"`. El código no rellena defaults: falta una clave obligatoria → error. Solo `meilisearch.api_key` es opcional (`#[serde(default)]`).

**Por qué:** el example es la documentación; un default silencioso ocultaría typos. Puerto 8080 no está en el backlog; se asume para localhost MVP (HTTPS/`vectra.local` son v2). `14` alinea T-00-06.

**Alternativa:** defaults en Rust si falta la sección `logs`. Se descartó: el ticket lista `logs` como sección de `Config`, no como opcional.

## Risks / Trade-offs

- **[Parser argv frágil vs clap]** → Mitigación: solo `--config` / `--config=`; T-00-07 lo sustituye por `clap` global. Unhappy: flag sin valor.
- **[toml 1.x vs 0.8 API]** → Mitigación: `toml::from_str` + serde; el primer test red fija el import. Pin exacto.
- **[macOS case-insensitive: `openspec new change T-00-05`]** → Mitigación: CLI pide kebab lowercase; la carpeta del change se llama `T-00-05` (ticket = rama).
- **[main sale 0 y T-00-07 necesita subcomandos]** → Mitigación: documentado; `serve` no arranca aquí. `--help` sin config queda para T-00-07.
- **[Claves en español en Rust]** → Mitigación: `#[serde(rename = "datos")]` si los campos Rust van en snake_case inglés (`data`), **o** campos Rust `datos` iguales al TOML. Preferir campos iguales al TOML para que el `key` del error coincida sin tabla de rename.

## Migration Plan

1. Red: tests unitarios de parse/validación (no compilan).
2. Red: tests de binario (`CARGO_BIN_EXE_vortex`) para exit 0 / ≠ 0.
3. Red: fixture `clap` en `bin/vortex` — `check-workspace.sh` ≠ 0 (hoy ya ≠ 0 por cualquier deps).
4. Green: pin `toml`, allowlist, `Config`, `main`, example, fixtures.
5. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-05`.
6. Rollback: revertir `bin/vortex`, el pin, el example y el bloque de allowlist. Sin datos ni API.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El puerto de ejemplo `8080` vs `80` queda fijado en el example (`8080`); T-00-10 puede documentar otro bind sin cambiar el esquema.
