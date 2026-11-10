# Design

## Context

En `develop`, `bin/vortex` parsea `--config` a mano, carga `Config` y sale 0 si el TOML es válido (`tests/config_cli.rs`). `clap = "=4.6.7"` ya está en `[workspace.dependencies]` pero **prohibido** en el crate `vortex` (`check-workspace.sh` usa `clap` como fixture unhappy). `init_logging` no se llama desde `main` (T-00-20). Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: no lógica de comandos (T-00-10+); no cargar TOML ni logs en stubs; ~2 h; MVP (los stubs de `cert` / `models` / `service` / `verify` / `backup` se declaran ahora aunque su lógica sea v2).

## Goals / Non-Goals

**Goals:**

- `clap` derive en `bin/vortex`: ocho subcomandos de primer nivel + `service install|uninstall`, `--config` global
- Tests de binario: `--help` lista todos; stubs `no implementado` + exit 1; argv inválido ≠ stub
- Allowlist de `vortex` incluye `clap`; el unhappy de fixture pasa a `metrics`

**Non-Goals (diseño):**

- `assert_cmd` / crates de test nuevas (`std::process::Command` como `config_cli.rs`)
- Cargar `Config` o `init_logging` desde `main` (T-00-10 / T-00-20)
- Completar `clap` `env = "VORTEX_CONFIG"` como requisito de este change: el flag global basta; T-00-10 puede cablear el env al cargar
- Mapear errores de CLI a `DomainError`

## Decisions

### 1. Módulo `cli` en `bin/vortex` con `clap` derive; `main` solo despacha stubs

**Elección:** `vortex-core/bin/vortex/src/cli.rs` con `Parser` / `Subcommand`. Enum de primer nivel: `Serve`, `Reindex`, `Backup`, `Cert`, `Models`, `Seed`, `Verify`, `Service { command: ServiceCmd }`. `ServiceCmd`: `Install`, `Uninstall`. `#[arg(long, global = true)] config: Option<PathBuf>` acepta `--config <path>` y `--config=<path>`. Tras parsear, cualquier variante llama a un helper que escribe `no implementado` en stderr y `process::exit(1)`. `--help` / `-h` los resuelve clap (exit 0) **antes** de despachar.

**Por qué:** el ticket nombra `clap` y esos subcomandos. Derive es el estilo de clap 4 y evita un builder largo. `global = true` cumple «opción global `--config`» sin repetir el flag. Sin `[lib]`: los tests de contrato son de binario (`CARGO_BIN_EXE_vortex`).

**Alternativa:** `clap` builder. Se descartó: más ruido para el mismo contrato. **Alternativa:** crate `cli` nuevo. Se descartó: [3.4.13](../../../docs/3-componentes.md#3413-plataforma-binvortex) asigna la CLI al composition root; ~2 h.

### 2. Stubs no cargan `Config`; se elimina el parser manual de argv

**Elección:** `main` deja de llamar `resolve_config_path` / `Config::from_path`. Se borra `parse_config_flag`. Los tests de proceso en `tests/config_cli.rs` que esperan exit 0 (o error de TOML) **sin subcomando** se reescriben o se mueven: el esquema sigue cubierto por unitarios de `config.rs`; el proceso se cubre en `tests/cli.rs`. `VORTEX_CONFIG` no se lee en este change (nadie carga el archivo). T-00-10 usará `cli.config` y, si es `None`, `VORTEX_CONFIG`, reutilizando el algoritmo de `vortex-config`.

**Por qué:** el Listo cuando es `vortex --help` sin TOML. El ticket dice que los no implementados responden `no implementado` con código 1: si se validara el TOML primero, `vortex serve` sin archivo fallaría con `config: --config: …` y rompería ese contrato. Cargar en stubs también escribiría en `/var/lib/vortex/datos` en cuanto T-00-20 enganche logs.

**Alternativa:** cargar Config y después imprimir el stub. Se descartó: mezcla T-00-05 con T-00-07 y exige TOML para un comando vacío. **Alternativa:** subcomando implícito que solo valida Config. Se descartó: clap exige subcomando y `--help` es el happy path.

### 3. Mensaje exacto `no implementado`, código 1, stderr

**Elección:** `eprintln!("no implementado"); process::exit(1);` para **todas** las variantes reconocidas (incluido `serve`). Comando desconocido / falta de subcomando / `service` sin acción: clap `error-context` (código ≠ 0, típicamente 2) y el texto MUST NOT contener `no implementado`. Tests buscan el substring en stderr.

**Por qué:** el ticket fija el string y el código 1. Reservar el mensaje a nombres conocidos evita que un typo se lea como stub a implementar.

**Alternativa:** `todo!()` / `unimplemented!()`. Se descartó: panic no es código 1 estable ni el texto pedido. **Alternativa:** código 2 para stubs. Se descartó: el ticket dice 1.

### 4. Allowlist: `clap` en deps; fixture unhappy = `metrics`

**Elección:** `vortex_deps_allowed` acepta `{toml, serde, thiserror, tracing, tracing-subscriber, tracing-appender, clap}` en `[dependencies]` (`workspace = true`; `clap` MAY `features = ["derive"]`). En `bin/vortex/Cargo.toml`: `clap = { workspace = true, features = ["derive"] }`. El pin `clap = "=4.6.7"` **ya existe**; no se toca la tabla de versiones. Fixture unhappy de `run-script-tests.sh` declara `metrics` en `vortex` (T-00-14) en lugar de `clap`.

**Por qué:** T-00-06 dejó `clap` como crate prohibida a propósito para este ticket. `metrics` es el próximo candidato del binario y sigue fuera de alcance.

**Alternativa:** `clap` sin `derive` (macro `clap_derive` aparte). Se descartó: en 4.6 `features = ["derive"]` es el camino documentado. **Alternativa:** feature `env`. Se pospone a T-00-10 para no mezclar resolución de path con stubs.

### 5. Tests de binario en `tests/cli.rs`; TDD red → green

**Elección:** `vortex-core/bin/vortex/tests/cli.rs` spawnea `env!("CARGO_BIN_EXE_vortex")` como `config_cli.rs`. Cobertura: `--help` / `-h` / `service --help`; `--config` en la ayuda; cada stub (un test parametrizado o tabla para los seis de primer nivel + `serve` + `service install|uninstall`); sin subcomando; subcomando desconocido; `service` solo. `TRY_PARSE` unitario opcional en `cli.rs` para `--config` / `--config=`. Verificación: [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) en `vortex-core/`.

**Por qué:** el Listo cuando es observable en el binario, no en un parser suelto. Un crate de tests de integración ve el `exit(1)` real.

**Alternativa:** solo `Command::try_parse_from` en unitarios. Se descartó: no cubre el código de salida ni stderr del proceso. **Alternativa:** `assert_cmd`. Se descartó: nueva crate fuera de la allowlist.

## Risks / Trade-offs

- **[macOS case-insensitive: `openspec new change t-00-07`]** → Mitigación: scaffold en `t-00-07` y rename a `T-00-07` (igual que T-00-05 / T-00-06).
- **[Regresión de `config_cli`: exit 0 sin subcomando]** → Mitigación: reescribir esos tests en el mismo change; unitarios de `Config::from_path` no se tocan.
- **[clap 4 sale 2 en usage error vs 1 del stub]** → Mitigación: la spec exige 1 solo para reconocidos; unknown/missing solo ≠ 0 y sin el string de stub.
- **[`--help` a stdout vs stderr]** → Mitigación: los tests buscan los nombres en stdout+stderr concatenados.
- **[Stubs de v2 en el MVP]** → Mitigación: el ticket MVP los lista; no implementar lógica (T-00-15, T-00-24, T-00-30, T-02-45, T-00-28).

## Migration Plan

1. Red: `tests/cli.rs` (help + stubs + unhappy) no compila o falla; `config_cli` que espera exit 0 se marca para reescribir.
2. Red: fixture `metrics` en `vortex` ≠ 0; el repo aún sin `clap` en `bin/vortex`.
3. Green: allowlist + `clap` en `Cargo.toml` + `cli.rs` + despacho en `main` + fixtures.
4. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-07`.
5. Rollback: revertir módulo CLI, allowlist, fixtures y tests. Sin datos de usuario ni API.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. T-00-10 decide cuándo llamar a `Config::from_path` (tras parsear `serve`) y si `clap` `env = "VORTEX_CONFIG"` sustituye el helper.
