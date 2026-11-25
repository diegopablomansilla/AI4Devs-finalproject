# Design

## Context

En `develop`: `.github/workflows/ci.yml` tiene `fmt`, `clippy`, `test`, `domain-deps` y `openapi-lint` en `macos-14`. `scripts/check-ci.sh` **prohíbe** las cadenas `Meilisearch` y `meilisearch` a propósito, para que este ticket las enganche. La fixture `scripts/fixtures/ci/scenarios/meilisearch-extra` es el unhappy de T-09-03. No existe `scripts/meilisearch-version` ni tests `#[ignore = "meilisearch"]`. El crate `search` es un placeholder. `meilisearch-sdk = "=0.33.0"` está en el workspace pero ningún crate lo declara. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: runner `macos-14` (Apple Silicon, [7.2](../../docs/7-tickets.md)); Meilisearch se consume por URL en el MVP ([2.7](../../docs/2-arquitectura.md#27-red-local-y-seguridad-mínima)); recableo § 7.5.1: sin manifiesto de `vortex models` (T-00-24). Compatibilidad SDK: `meilisearch-sdk` 0.33 garantiza Meilisearch 1.x; el pin se toma del último tag estable `v1.58.0` (binario `meilisearch-macos-apple-silicon`).

## Goals / Non-Goals

**Goals:**

- Un job de CI independiente que levanta la versión fijada y corre exactamente `cargo test --workspace -- --include-ignored meilisearch`
- Un pin de versión compartido entre desarrollo y CI, sin `models/manifest.toml`
- Un probe mínimo para que el Listo cuando no sea un filtro cargo vacío
- Un solo harness local (`check-ci.sh`) que falle si falta el job, el pin o `--include-ignored`

**Non-Goals (diseño):**

- Docker / Colima / `brew install meilisearch` en el runner
- SHA-256 del binario (entra con T-00-24)
- `meilisearch-sdk` ni `reqwest` en el crate `search` (el probe es HTTP mínimo; el adaptador es T-03-04)
- Master key / multi-tenant; `/health` es público
- Subir el mínimo global de `runs-on: macos-14` o `actions/cache@v4` por encima de 3 (rompe el orden de error de las fixtures T-09-01)
- Parser YAML propio; se sigue usando grep POSIX
- Job que ejecute `check-ci.sh` en GitHub
- `act` para simular macos-14

## Decisions

### 1. Job `meilisearch` en paralelo, no un step de `test`

**Elección:** `jobs.meilisearch` con `runs-on: macos-14`, en el mismo workflow (`pull_request` + `push` a `develop`/`main`).

**Por qué:** T-09-01 diseñó el YAML para que T-09-05 agregue **jobs**. El Listo cuando queda como check independiente: un fallo del motor no se confunde con un panic unitario. El job `test` sigue siendo § 5 sin sidecar.

**Alternativa:** un step al final de `test`. Se descartó: mezcla señales y contradice el wording del ticket (*job*). Además `cargo test --workspace` del job `test` MUST NOT llevar `--include-ignored` (los laptops sin Meilisearch y el job barato seguirían rotos).

### 2. Binario oficial de GitHub Releases, no Docker

**Elección:** el job descarga `https://github.com/meilisearch/meilisearch/releases/download/<tag>/meilisearch-macos-apple-silicon`, `chmod +x`, arranca en background (`MEILI_NO_ANALYTICS=true`, `--http-addr 127.0.0.1:7700`) y espera `GET /health` antes de `cargo test`.

**Por qué:** `macos-14` es arm64; el asset coincide. Docker en runners macOS de GitHub es Colima y suele ser más lento/frágil. Homebrew no fija el tag del pin.

**Alternativa:** `getmeili/meilisearch-action`. Se descartó: acción de terceros y menos control del pin. **Alternativa:** imagen Docker `getmeili/meilisearch:<tag>`. Se descartó por el runner.

### 3. Pin en `scripts/meilisearch-version`, no en `models/manifest.toml`

**Elección:** un archivo de una línea `v1.58.0`. El job interpola o embebe ese tag; `check-ci.sh` exige que el workflow contenga el mismo string que el archivo. T-00-24 (v2) podrá absorber el pin en el manifiesto.

**Por qué:** el recableo § 7.5.1 lo pide explícitamente. Una sola fuente evita que CI y desarrollo diverjan. `v1.58.0` es el último 1.x al planificar (compatible con `meilisearch-sdk` 0.33).

**Alternativa:** constante solo en el YAML. Se descartó: desarrollo local no tiene dónde leerla. **Alternativa:** pin más viejo (`v1.14.0`, citado en el changelog del SDK 0.32). Se descartó: el ticket pide «la misma que se usa en desarrollo»; no hay instalación previa, así que se fija la estable actual.

### 4. Probe HTTP en `search`, sin SDK

**Elección:** `vortex-core/crates/search/tests/meilisearch_health.rs` con `#[ignore = "meilisearch"]` y nombre que contiene `meilisearch`. Cliente HTTP mínimo (`std::net::TcpStream` o equivalente) a `GET /health`. URL: `MEILISEARCH_URL` o default `http://127.0.0.1:7700`. Éxito = HTTP 200 y JSON `status == "available"`. Fallo de conexión = test failed, no skip.

**Por qué:** el Listo cuando exige que la CI **corra** un test contra el motor real. `cargo test -- --include-ignored meilisearch` sobre un workspace sin tests que matcheen sale 0 en vacío. T-03-04 aún no existe; meter `meilisearch-sdk` aquí duplicaría el adaptador.

**Alternativa:** no agregar test y solo el job. Se descartó: no cumple el Listo cuando. **Alternativa:** el probe en `scripts/` con `curl`. Se descartó: el ticket nombra `#[ignore = "meilisearch"]`.

### 5. Filtro cargo `meilisearch` + `--include-ignored`

**Elección:** comando exacto `cargo test --workspace -- --include-ignored meilisearch`. Convención: todo test de motor real lleva `#[ignore = "meilisearch"]` **y** `meilisearch` en el nombre (Rust no filtra por el string del atributo `ignore`).

**Por qué:** el **Hace** nombra `--include-ignored`. El filtro evita re-ejecutar toda la suite en el job caro. T-03-04 reutilizará la misma convención.

**Alternativa:** `cargo test -- --ignored` (solo ignorados, cualquier razón). Se descartó: cuando existan ignores de Ollama/Whisper, el job de Meilisearch los arrastraría.

### 6. Extender `check-ci.sh`; el extra pasa a Ollama

**Elección:** quitar `Meilisearch` y `meilisearch` de la denylist; exigir job `meilisearch` (`^[[:space:]]{2}meilisearch:`), las cadenas `--include-ignored`, `meilisearch-macos-apple-silicon` y el tag de `scripts/meilisearch-version`; exigir el archivo de pin. Mantener `>= 3` caches y `macos-14` para no romper el orden de error de las fixtures T-09-01. El job nuevo se comprueba **después** de `openapi-lint` y de los extras que siguen (Ollama, Whisper, `check-ci.sh`).

**Por qué:** un solo comando local cubre el contrato CI, igual que T-09-03. Si se exigiera `runs-on`/`cache` `>= 6` *antes* del nombre del job, la fixture de cinco jobs fallaría con «macos-14» en vez de nombrar `meilisearch`.

**Alternativa:** `scripts/check-meilisearch.sh`. Se descartó: duplicaría el grep de workflow.

Orden de chequeos (relevante para TDD):

1. Archivo, `pull_request`, jobs `fmt`/`clippy`/`test`, no ubuntu, `macos-14` `>= 3`, cache `>= 3`, `vortex-core`, comandos § 5
2. Extras prohibidos **sin** Meilisearch (sigue Ollama, Whisper, `check-ci.sh`)
3. Job `domain-deps` + `scripts/check-domain-deps.sh`
4. Job `openapi-lint` + `@redocly/cli` + `lint docs/openapi.yaml` + `redocly.yaml`
5. Job `meilisearch` + `--include-ignored` + asset + tag + `scripts/meilisearch-version`

Así `missing-openapi-job` sigue fallando por `openapi-lint`, no por el job nuevo.

### 7. El job `meilisearch` sí cachea Cargo; `check-ci` no cuenta un sexto cache

**Elección:** el job incluye `rustup show` y `actions/cache@v4` (mismo patrón que `test`, key `macos-14-meilisearch-…`). `check-ci.sh` **no** sube el mínimo de caches de 3 a 6.

**Por qué:** el job compila tests. Exigir 6 caches en el harness haría fallar `missing-meilisearch-job` (cinco jobs, 3–4 caches) por «cache» en vez de por el nombre del job.

**Alternativa:** sin caché, como `openapi-lint`. Se descartó: ese job no compila Rust.

### 8. Unhappy del probe = motor caído, no fixture HTTP

**Elección:** el escenario «Meilisearch caído» se corre en apply con `--include-ignored` **antes** de levantar el binario (expect ≠ 0). El happy se corre después, en local si hay binario o en el job de CI. No se usa `act`.

**Por qué:** el probe debe fallar de verdad sin el sidecar; un mock HTTP no demostraría el Listo cuando. El aislamiento (`cargo test --workspace` verde sin motor) cubre que el job `test` no se rompe.

**Alternativa:** mock de `/health` con un listener de test. Se descartó: no es «Meilisearch real».

## Risks / Trade-offs

- **[Grep del YAML]** → un comentario `meilisearch:` podría falsear. Mitigación: el archivo real es corto; el patrón ancla indentación de job.
- **[Filtro por nombre, no por `#[ignore]`]** → un test `meilisearch_*` no ignorado también correría en el job. Mitigación: la spec exige ambos; T-03-04 sigue la convención.
- **[Pin `v1.58.0` vs SDK 0.33]** → el SDK declara compatibilidad 1.x, no cada minor. Si 1.58 rompe el SDK, T-03-04 lo verá; este ticket solo pega `/health`. Mitigación: el pin es un archivo de una línea, fácil de bajar.
- **[Descarga de GitHub Releases en CI]** → rate limit / asset movido. Mitigación: tag y nombre de asset fijos; fallo visible.
- **[Listo cuando real es GitHub]** → el harness cubre contrato YAML; el probe local cubre el motor; el PR de `T-09-05` es la prueba operativa del job.
- **[`openspec new change` rechaza `T-09-05`]** → scaffold en `t-09-05` y rename (igual que T-09-01 / T-09-03). Nota operativa, no bloquea.
- **[Alcance ~2 h]** → job + pin + probe + harness. No se divide el backlog.

## Migration Plan

1. Red: `check-ci.sh` exige el job `meilisearch` → el repo actual (cinco jobs) sale ≠ 0.
2. Fixtures YAML sin el job / sin `--include-ignored` / con tag distinto: salen ≠ 0 y nombran el motivo.
3. Red probe: test ignorado con motor caído sale ≠ 0; `cargo test --workspace` sigue en 0.
4. Green: agregar pin, job y probe; `check-ci.sh` sobre el repo sale 0.
5. Reemplazar `meilisearch-extra` por `ollama-extra` en `run-script-tests.sh`.
6. Rollback: revertir el job, el pin, el probe y la denylist de `check-ci.sh`. No hay datos.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre del job (`meilisearch`) y el pin `v1.58.0` quedan fijados aquí.
