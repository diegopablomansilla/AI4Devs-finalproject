# Proposal

## Ticket padre

- **Ticket:** [T-09-09](../../../docs/7-tickets.md) — *Setup* de tests e2e con Playwright
- **Change:** `T-09-09`
- **HU:** [HU-09-03](../../../docs/6-historias.md#hu-09-03--tests-de-extremo-a-extremo-de-la-spa)
- **Depende de:** T-02-41 (cerrado en `develop`: dataset `seed/` ~200 SKUs)
- **Hace:** carpeta `e2e/` con Playwright, *fixture* que levanta `vortex serve` con datos temporales y el dataset de desarrollo, chequeo de accesibilidad con `@axe-core/playwright` y *job* de CI
- **No hace:** escenarios de negocio
- **Listo cuando:** un test "la app carga y el Omnibox tiene foco" pasa en CI

## Why

HU-09-03 pide tests de extremo a extremo de la SPA como la ve el operador. `vectra-web` se verifica hoy con `node --test` y un stub de DOM (sin navegador, sin `vortex serve`, sin dataset). T-09-10 y T-09-11 (búsqueda, recomendaciones, alta) no tienen dónde colgarse. El **Hace** de este ticket es el *setup*: Playwright, fixture de stack y un smoke + axe; el **No hace** deja los flujos H1 para esos tickets.

## What Changes

- Crear `e2e/` como proyecto Node de *dev-only* (`package.json` + lockfile): Playwright y `@axe-core/playwright`. `vectra-web/` sigue sin `package.json` ni bundler
- Fixture (global setup / teardown) que: reserva puertos libres, escribe un TOML contra directorios temporales, importa `seed/` con `vortex seed` (servidor detenido), levanta Meilisearch de la versión fijada y `vortex serve`, espera `GET /health`, y limpia al terminar
- Un único spec de Playwright: `#/` carga, el input del Omnibox tiene el foco, y axe no reporta violaciones `critical`/`serious`
- Job de CI `e2e` en `macos-14` (paralelo a `meilisearch`): build del binario `vortex`, npm ci, Chromium de Playwright, Meilisearch pinneado, `npx playwright test`
- Extender `scripts/check-ci.sh` para exigir el job `e2e` y la invocación de Playwright; fixtures YAML unhappy si falta
- TDD: happy = smoke + axe + contrato CI; unhappy = job ausente / sin Playwright, stack que no arranca, seed inválido, app sin foco en Omnibox, violaciones axe
- **BREAKING** (harness interno): `check-ci.sh` pasa a exigir el job `e2e`

## Non-goals

Este change **no** cubre:

- Búsqueda con typo, tarjetas con stock, navegación por teclado → **T-09-10** (**No hace:** escenarios de negocio)
- Panel de recomendaciones y alta de producto en e2e → **T-09-11**
- e2e Modo IA / voz / visión+POS → **T-09-17**, **T-09-21**, **T-09-24** (v2)
- Filtros por atributo y stock en e2e → v2 (T-03-19 / recableo T-09-10)
- `package.json`, Playwright o npm **dentro de** `vectra-web/` (la spec `vectra-web` lo prohíbe)
- Firefox / WebKit; Lighthouse; benchmarks de latencia con Playwright → **T-09-08** (v2)
- Ollama / Whisper en CI → **T-09-13**, **T-09-18**
- Un job que ejecute `scripts/check-ci.sh` en GitHub (sigue siendo harness local)
- Cambios a `docs/openapi.yaml` o a crates de dominio

## Capabilities

### New Capabilities

- `e2e-playwright`: proyecto `e2e/` con Playwright, fixture de `vortex serve` + dataset de desarrollo + Meilisearch, smoke «app carga y Omnibox con foco», chequeo axe, y job de CI que lo ejecuta

### Modified Capabilities

- `ci-base`: `scripts/check-ci.sh` MUST exigir el job `e2e` que invoca Playwright; MUST NOT tratar `playwright` / `e2e/` como extra fuera de alcance; MUST seguir rechazando Ollama, Whisper y un job que ejecute `check-ci.sh` en GitHub

## Impact

- **Repo:** `e2e/` (nuevo), `.github/workflows/ci.yml` (job nuevo), `scripts/check-ci.sh` y fixtures de CI, `.gitignore` (`playwright-report`, `test-results`). No toca `vectra-web/package.json` (no existe) ni `docs/openapi.yaml`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** séptimo check en cada PR; comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sin cambio en `fmt`/`clippy`/`test`; el job nuevo añade Playwright. Verificación local: `npx playwright test` en `e2e/` con stack de la fixture. `openspec validate T-09-09` (o `t-09-09` si el CLI exige minúsculas)
- **Ticket:** cierra T-09-09; desbloquea T-09-10. El Listo cuando en GitHub se confirma en el PR de esta rama
- **Alcance ~2 h:** setup + un smoke + axe + job. No se divide el backlog
