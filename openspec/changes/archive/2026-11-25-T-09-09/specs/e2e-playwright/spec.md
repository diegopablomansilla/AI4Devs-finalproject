# Spec Delta

Ticket padre: T-09-09. Change: T-09-09. HU: HU-09-03.

## Purpose

Define el *setup* de tests de extremo a extremo de la SPA Vectra: un proyecto de desarrollo con navegador real, una fixture que arranca Vortex contra datos temporales y el dataset de desarrollo, un smoke de carga con foco en el Omnibox, un chequeo de accesibilidad, y el job de CI que lo ejecuta, para que T-09-10 y siguientes cuelguen escenarios de negocio de un stack reproducible.

## ADDED Requirements

### Requirement: Proyecto e2e de desarrollo en la raíz

El repositorio MUST contener el directorio `e2e/` en la raíz, con un manifiesto de dependencias de desarrollo que declara Playwright y el puente de accesibilidad `@axe-core/playwright`, un lockfile, y al menos un archivo de especificación bajo `e2e/tests/`. `vectra-web/` MUST NOT ganar `package.json` ni dependencias npm: la SPA sigue cargando como ES Modules servidos por el Gateway. Playwright MUST ser herramienta de desarrollo (no se distribuye con el binario `vortex` ni con `vectra-web`).

#### Scenario: e2e existe y vectra-web no toma npm (happy)

- **WHEN** se inspecciona la raíz del repositorio
- **THEN** existe `e2e/` con manifiesto, lockfile y al menos un spec bajo `e2e/tests/`
- **AND** el manifiesto declara Playwright y `@axe-core/playwright`
- **AND** no existe `vectra-web/package.json`

#### Scenario: falta e2e o el manifiesto (unhappy)

- **WHEN** no existe `e2e/`, o falta el manifiesto, o el manifiesto no declara Playwright
- **THEN** la verificación del contrato e2e termina con código distinto de 0
- **AND** el mensaje nombra `e2e` o Playwright

### Requirement: Fixture de vortex serve con datos temporales y dataset de desarrollo

Antes de correr los specs, una fixture MUST: (1) crear un directorio de datos temporal (no `paths.datos` de una instalación local); (2) escribir un TOML válido apuntando `paths.datos` a ese directorio, `paths.web` a `vectra-web/` del checkout, `server.bind` a `127.0.0.1` en un puerto libre, y `meilisearch.url` a un Meilisearch de la versión fijada en `scripts/meilisearch-version` también en un puerto libre; (3) ejecutar `vortex seed` contra el directorio `seed/` del repositorio **con el servidor detenido**; (4) arrancar Meilisearch y después `vortex serve --config` de ese TOML; (5) esperar a que `GET /health` responda HTTP 200. Al terminar la suite, MUST detener esos procesos y borrar el directorio temporal. MUST NOT reutilizar `data.ms/` ni `vortex-core/data.ms/` del working tree. MUST NOT lanzar Ollama ni Whisper.

Si falta el binario `vortex`, el dataset `seed/`, el pin de Meilisearch, o `GET /health` no responde a tiempo, la fixture MUST terminar con código distinto de 0 y MUST NOT marcar los tests como skipped.

#### Scenario: fixture arranca stack con dataset (happy)

- **WHEN** se ejecuta la suite e2e sobre un checkout que cumple `vortex-seed`, `seed-dataset` y `ci-meilisearch`
- **THEN** `vortex seed` importa `seed/` contra un `paths.datos` temporal
- **AND** `vortex serve` escucha en el bind de la fixture
- **AND** `GET /health` responde HTTP 200 antes de abrir el navegador

#### Scenario: fixture falla si el servidor no arranca (unhappy)

- **WHEN** el binario `vortex` no existe, o el bind no queda escuchando, o `GET /health` no responde a tiempo
- **THEN** la fixture termina con código distinto de 0
- **AND** los specs no se marcan skipped

#### Scenario: fixture falla si el dataset no se puede importar (unhappy)

- **WHEN** `seed/` no existe o `vortex seed` termina con código distinto de 0
- **THEN** la fixture termina con código distinto de 0
- **AND** `vortex serve` no queda escuchando con una base vacía a propósito

### Requirement: Smoke — la app carga y el Omnibox tiene foco

La suite MUST incluir un spec cuyo título o nombre estable identifique «la app carga y el Omnibox tiene foco» (Listo cuando del ticket). Ese spec MUST abrir la vista del operador (`#/` o la URL raíz que el *router* pinta como `data-view="operator"`) servida por el `vortex serve` de la fixture. MUST esperar a que exista el Omnibox. MUST asertar que el campo de búsqueda del Omnibox tiene el foco de teclado (el mismo contrato de T-01-08, ahora en navegador real). MUST NOT tipear consultas, MUST NOT abrir el panel de recomendaciones, MUST NOT crear productos ni navegar a admin (T-09-10, T-09-11).

#### Scenario: la app carga y el Omnibox tiene foco (happy, Listo cuando)

- **WHEN** la fixture dejó `vortex serve` escuchando con el dataset de desarrollo
- **AND** el navegador abre la vista del operador
- **THEN** el documento carga (no página de error del navegador)
- **AND** existe el Omnibox
- **AND** el campo de búsqueda del Omnibox tiene el foco de teclado

#### Scenario: sin foco en el Omnibox el smoke falla (unhappy)

- **WHEN** la vista del operador no monta el Omnibox, o el campo de búsqueda no recibe el foco
- **AND** corre el spec de smoke
- **THEN** el spec termina en fallo (no skipped)
- **AND** el mensaje nombra el Omnibox o el foco

### Requirement: Chequeo de accesibilidad con axe en el smoke

El spec de smoke MUST ejecutar un barrido de accesibilidad sobre la vista del operador con `@axe-core/playwright`, incluyendo el árbol dentro de Shadow DOM. MUST fallar si hay violaciones de impacto `critical` o `serious` (WCAG 2.2 A/AA). MUST NOT fallar la suite solo por impacto `moderate` o `minor` en este ticket. MUST NOT exigir Lighthouse.

Un helper de axe MUST, contra un documento de fixture con una violación `serious` conocida (p. ej. imagen sin alternativa de texto), terminar en fallo.

#### Scenario: operador sin violaciones graves (happy)

- **WHEN** corre el smoke sobre la vista del operador servida por la fixture
- **THEN** el barrido axe termina sin violaciones `critical` ni `serious`

#### Scenario: documento con violación serious hace fallar axe (unhappy)

- **WHEN** se ejecuta el helper de axe contra un documento de fixture que contiene una violación de impacto `serious`
- **THEN** el helper termina en fallo
- **AND** el mensaje nombra la regla o el impacto

### Requirement: Job de CI e2e

`.github/workflows/ci.yml` MUST definir un job llamado `e2e` en el mismo workflow que `fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint` y `meilisearch`. El job MUST dispararse en `pull_request`. El job MUST usar el runner `macos-14`. El job MUST: instalar Node, instalar dependencias de `e2e/` desde el lockfile, instalar el navegador Chromium de Playwright, construir el binario `vortex` del workspace `vortex-core/`, descargar Meilisearch de la versión fijada (mismo pin que `ci-meilisearch`), y ejecutar la suite Playwright (`npx playwright test` o equivalente contra `e2e/`). El job MUST NOT instalar Ollama ni Whisper. El job MUST NOT ejecutar `scripts/check-ci.sh`. Los jobs `fmt`/`clippy`/`test` MUST seguir siendo los comandos de [docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) § 5, sin Playwright.

#### Scenario: job e2e presente con Playwright (happy)

- **WHEN** se inspecciona `.github/workflows/ci.yml`
- **THEN** declara un job llamado `e2e`
- **AND** ese job declara `runs-on: macos-14`
- **AND** un step invoca Playwright contra `e2e/`
- **AND** el job menciona Chromium
- **AND** el job construye o invoca el binario `vortex`

#### Scenario: falta el job e2e (unhappy)

- **WHEN** `.github/workflows/ci.yml` declara `fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint` y `meilisearch` pero no el job `e2e`
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra `e2e`

#### Scenario: el job no invoca Playwright (unhappy)

- **WHEN** el job `e2e` existe pero no invoca Playwright
- **THEN** la verificación del contrato de CI termina con código distinto de 0
- **AND** el mensaje nombra Playwright o `playwright`

### Requirement: Tests previstos del setup e2e

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios.

| Test | Ubicación | Camino |
|---|---|---|
| `smoke_app_loads_omnibox_focused` | `e2e/tests/smoke.spec.js` (o `.ts`) | happy (Listo cuando) |
| `smoke_axe_no_critical_serious` | mismo spec o helper invocado desde el smoke | happy |
| `axe_helper_fails_on_serious_violation` | `e2e/tests/` | unhappy |
| Fixture: `GET /health` 200 tras seed | harness e2e | happy |
| Fixture: binario o seed ausente → exit ≠ 0, no skip | harness e2e | unhappy |
| `scripts/check-ci.sh` sobre el repo | `scripts/check-ci.sh` | happy |
| Fixture YAML sin job `e2e` | `scripts/fixtures/ci/scenarios/missing-e2e-job/` | unhappy |
| Fixture YAML con job `e2e` sin Playwright | `scripts/fixtures/ci/scenarios/missing-playwright/` | unhappy |
| `scripts/fixtures/ci/run-script-tests.sh` | harness CI | happy + unhappy (regresión T-09-01…T-09-05) |

`npx playwright test` en `e2e/` MUST ejecutar el Listo cuando. `./scripts/check-ci.sh` MUST verificar el job. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) § 5). `./scripts/check-vectra-web.sh` MUST seguir en verde y MUST NOT exigir `vectra-web/package.json`.

#### Scenario: suite de setup cubre happy y unhappy (happy)

- **WHEN** se ejecutan `npx playwright test` en `e2e/`, `./scripts/check-ci.sh` y `./scripts/fixtures/ci/run-script-tests.sh`
- **THEN** el smoke de Omnibox y axe pasan
- **AND** el helper de axe falla contra el documento con violación `serious`
- **AND** las fixtures `missing-e2e-job` y `missing-playwright` fallan nombrando el motivo
- **AND** los comandos § 5 sobre `vortex-core/` terminan con código 0
