# Tasks

## 1. Tests (red) — contrato CI

- [x] 1.1 Crear fixture `scripts/fixtures/ci/scenarios/missing-meilisearch-job/.github/workflows/ci.yml` con cinco jobs válidos (`fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint`, `macos-14`, `pull_request`, comandos § 5, caché, `redocly.yaml` si hace falta en esa raíz) **sin** job `meilisearch` ni `--include-ignored` — `test -f` del YAML (escenario «Falta el job meilisearch»)
- [x] 1.2 Crear fixture `scripts/fixtures/ci/scenarios/missing-include-ignored/.github/workflows/ci.yml` con los seis jobs incluyendo `meilisearch` que descarga un binario **sin** `--include-ignored` — `test -f` del YAML (escenario «El job no incluye los tests ignorados»)
- [x] 1.3 Crear fixture `scripts/fixtures/ci/scenarios/unpinned-meilisearch/` con `scripts/meilisearch-version` = `v1.58.0` y un job `meilisearch` con `--include-ignored` que **no** menciona ese tag — `test -f` del YAML y del pin (escenario «Falta el pin o el workflow usa otra versión»)
- [x] 1.4 Crear fixture `scripts/fixtures/ci/scenarios/ollama-extra/.github/workflows/ci.yml` (esqueleto de `meilisearch-extra` + `openapi-lint`, step que invoca Ollama) — `test -f` del YAML (escenario «Extra de sidecar v2 fuera de alcance»)
- [x] 1.5 Extender `scripts/fixtures/ci/run-script-tests.sh`: `assert_fails_with` `missing-meilisearch-job` needle `meilisearch`; `missing-include-ignored` needle `--include-ignored`; `unpinned-meilisearch` needle `meilisearch-version` o `v1.58.0`; `ollama-extra` needle `Ollama` o `ollama`; **quitar** `assert_fails_with` de `meilisearch-extra` — `./scripts/fixtures/ci/run-script-tests.sh` sale ≠ 0 porque `check-ci.sh` aún no exige el job ni deja pasar Meilisearch (escenarios de 1.1–1.4 en rojo)

## 2. Tests (red) — probe de integración

- [x] 2.1 Crear `vortex-core/crates/search/tests/meilisearch_health.rs` con `#[ignore = "meilisearch"] fn meilisearch_health_is_available` que hace `GET {MEILISEARCH_URL}/health` (default `http://127.0.0.1:7700`) y exige HTTP 200 + JSON `status == "available"`; **sin** `meilisearch-sdk` — `test -f` del archivo (escenarios «Probe con Meilisearch disponible» y «Probe con Meilisearch caído»)
- [x] 2.2 Con Meilisearch **detenido**, `cargo test --workspace -- --include-ignored meilisearch` en `vortex-core/` sale ≠ 0 y el stderr nombra conexión o `/health` (escenario «Probe con Meilisearch caído»)
- [x] 2.3 Con Meilisearch **detenido**, `cargo test --workspace` en `vortex-core/` sale 0 y el probe figura como ignorado (escenario «La suite por defecto no exige el motor»; § 5)

## 3. Harness, pin y job (green)

- [x] 3.1 Agregar `scripts/meilisearch-version` con una línea `v1.58.0` — `test -f` y el contenido coincide (escenario «Pin presente y alineado con el workflow»)
- [x] 3.2 En `scripts/check-ci.sh`: quitar `Meilisearch` y `meilisearch` de la denylist de extras; exigir job `meilisearch` (`^[[:space:]]{2}meilisearch:`), las cadenas `--include-ignored`, `meilisearch-macos-apple-silicon` y el tag leído de `scripts/meilisearch-version`, y el archivo de pin **después** de `openapi-lint` y de los extras que siguen (Ollama, Whisper, `check-ci.sh`) — `./scripts/check-ci.sh` sobre el repo sale ≠ 0 y nombra `meilisearch`; las aserciones de 1.5 sobre `missing-meilisearch-job` pasan
- [x] 3.3 Agregar job `meilisearch` a `.github/workflows/ci.yml`: `runs-on: macos-14`; `working-directory: vortex-core` en el step de cargo; `rustup show`; `actions/cache@v4` (key `macos-14-meilisearch-` + hash de `Cargo.lock`); descargar el asset `meilisearch-macos-apple-silicon` del tag del pin; arrancar en `127.0.0.1:7700` con `MEILI_NO_ANALYTICS=true`; esperar `GET /health`; `cargo test --workspace -- --include-ignored meilisearch` — escenario «Job presente con motor y filtro ignore»
- [x] 3.4 `./scripts/check-ci.sh` sobre el repo sale 0 (escenarios «Contrato válido», «Job presente con motor y filtro ignore», «Workflow con Meilisearch y sin motores v2»)
- [x] 3.5 `./scripts/fixtures/ci/run-script-tests.sh` sale 0 (happy del repo + unhappy 1.1–1.4 + regresiones T-09-01/T-00-02/T-09-03)
- [x] 3.6 Con Meilisearch `v1.58.0` escuchando en `127.0.0.1:7700`, `cargo test --workspace -- --include-ignored meilisearch` en `vortex-core/` sale 0 (escenario «Probe con Meilisearch disponible» / Listo cuando)

## 4. Cierre (regresión, § 5 y non-goals)

- [x] 4.1 `git diff` vacío en `docs/openapi.yaml` y sin `meilisearch-sdk` / `reqwest` en `vortex-core/crates/search/Cargo.toml`; no hay `models/manifest.toml` ni job que ejecute `scripts/check-ci.sh`; el job `test` sigue siendo `cargo test --workspace` **sin** `--include-ignored` (non-goals T-03-04, T-00-24, T-09-13, T-09-18)
- [x] 4.2 `./scripts/check-toolchain.sh`, `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` salen 0
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 `.github/workflows/ci.yml` no menciona Ollama ni Whisper (non-goals T-09-13, T-09-18)
- [x] 4.5 `openspec validate T-09-05` (o `t-09-05` si el CLI exige minúsculas) sale 0