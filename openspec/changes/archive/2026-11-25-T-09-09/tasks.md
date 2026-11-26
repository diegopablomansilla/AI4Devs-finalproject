# Tasks

## 1. Tests (red) — contrato CI

- [x] 1.1 Crear fixture `scripts/fixtures/ci/scenarios/missing-e2e-job/.github/workflows/ci.yml` con los seis jobs válidos actuales (`fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint`, `meilisearch`, `macos-14`, `pull_request`, comandos § 5, pin Meilisearch) **sin** job `e2e` ni cadena `playwright` — `test -f` del YAML (escenario «Falta el job e2e»)
- [x] 1.2 Crear fixture `scripts/fixtures/ci/scenarios/missing-playwright/.github/workflows/ci.yml` con job `e2e` en `macos-14` que construye `vortex` **sin** invocar Playwright — `test -f` del YAML (escenario «El job e2e no invoca Playwright»)
- [x] 1.3 Extender `scripts/fixtures/ci/run-script-tests.sh`: `assert_fails_with` `missing-e2e-job` needle `e2e`; `missing-playwright` needle `playwright` — `./scripts/fixtures/ci/run-script-tests.sh` sale ≠ 0 porque `check-ci.sh` aún no exige el job (escenarios 1.1–1.2 en rojo)

## 2. Tests (red) — smoke Playwright y axe

- [x] 2.1 Crear `e2e/package.json` + lockfile con `@playwright/test` y `@axe-core/playwright`; `playwright.config.js` (Chromium only, `workers: 1`, `testDir: tests`); `.gitignore` de `e2e/test-results/`, `e2e/playwright-report/`, `e2e/.stack.json` — `test ! -f vectra-web/package.json` y el manifiesto declara Playwright (escenario «e2e existe y vectra-web no toma npm»)
- [x] 2.2 Crear `e2e/tests/smoke.spec.js` con `smoke_app_loads_omnibox_focused` (abre la vista operador, espera `vx-omnibox`, aserta foco en el input de búsqueda) y `smoke_axe_no_critical_serious` (AxeBuilder, fallo si `critical`/`serious`) — `test -f` del spec (escenarios «la app carga y el Omnibox tiene foco», «operador sin violaciones graves»)
- [x] 2.3 Crear helper `e2e/axe.js` y `e2e/tests/axe-helper.spec.js` (`axe_helper_fails_on_serious_violation`) más `e2e/fixtures/axe-violation.html` (p. ej. `<img>` sin `alt`) — `npx playwright test axe-helper` en `e2e/` sale ≠ 0 porque el helper aún no falla en `serious` (escenario «documento con violación serious hace fallar axe» en rojo)
- [x] 2.4 Sin harness/`VORTEX_BIN`, `npx playwright test smoke.spec.js` en `e2e/` sale ≠ 0 (no skipped) (escenarios «sin foco en el Omnibox el smoke falla», «fixture falla si el servidor no arranca»)

## 3. Harness, job y chequeo (green)

- [x] 3.1 Implementar `e2e/harness.mjs` (globalSetup/teardown): puertos libres, TOML temporal (`paths.datos` temp, `paths.web` = `vectra-web/`, Meili en puerto libre), `vortex seed` de `seed/`, arranque Meilisearch (`MEILI_BIN`) y `vortex serve` (`VORTEX_BIN`), espera `GET /health` 200, escribe `e2e/.stack.json`; teardown mata procesos y borra el temp — con `VORTEX_BIN`/`MEILI_BIN` ausentes el setup sale ≠ 0 y no skip (escenarios «fixture arranca stack con dataset», «fixture falla si el servidor no arranca», «fixture falla si el dataset no se puede importar»)
- [x] 3.2 Cablear `playwright.config.js` a `harness.mjs` y `baseURL` desde `.stack.json`; implementar `e2e/axe.js` para que `axe_helper_fails_on_serious_violation` pase — `npx playwright test axe-helper` en `e2e/` sale 0
- [x] 3.3 En `scripts/check-ci.sh`: exigir job `e2e` (`^[[:space:]]{2}e2e:`) y la cadena `playwright` **después** de los chequeos de Meilisearch; no subir el mínimo de `macos-14` ni `actions/cache@v4` por encima de 3; no tratar `playwright`/`e2e` como extra prohibido — `./scripts/check-ci.sh` sobre el repo sale ≠ 0 y nombra `e2e`; las aserciones de 1.3 sobre `missing-e2e-job` pasan
- [x] 3.4 Agregar job `e2e` a `.github/workflows/ci.yml`: `runs-on: macos-14`; cache Cargo; `cargo build --bin vortex`; `actions/setup-node@v4` (Node 22, cache npm de `e2e/package-lock.json`); `npm ci`; `npx playwright install chromium`; descargar Meilisearch del pin; `VORTEX_BIN` + `MEILI_BIN` + `npx playwright test` en `e2e/` — escenario «job e2e presente con Playwright»
- [x] 3.5 `./scripts/check-ci.sh` sobre el repo sale 0 (escenarios «Contrato válido», «job e2e presente con Playwright»)
- [x] 3.6 `./scripts/fixtures/ci/run-script-tests.sh` sale 0 (happy del repo + unhappy 1.1–1.2 + regresiones T-09-01…T-09-05)
- [x] 3.7 Con `VORTEX_BIN` (p. ej. `vortex-core/target/debug/vortex`) y `MEILI_BIN` de la versión fijada, `npx playwright test` en `e2e/` sale 0 (Listo cuando: «la app carga y el Omnibox tiene foco» + axe sin `critical`/`serious`)

## 4. Cierre (regresión, § 5 y non-goals)

- [x] 4.1 `test ! -f vectra-web/package.json`; `git diff` vacío en `docs/openapi.yaml`; el job `test` sigue siendo `cargo test --workspace` **sin** Playwright; no hay job que ejecute `scripts/check-ci.sh`; `.github/workflows/ci.yml` no menciona Ollama ni Whisper (non-goals T-09-10, T-09-13, T-09-18, vectra-web)
- [x] 4.2 `./scripts/check-toolchain.sh`, `./scripts/check-workspace.sh`, `./scripts/check-domain-deps.sh` y `./scripts/check-vectra-web.sh` salen 0
- [x] 4.3 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.4 `openspec validate T-09-09` (o `t-09-09` si el CLI exige minúsculas) sale 0