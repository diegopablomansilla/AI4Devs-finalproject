# Design

## Context

En `develop`: no existe `e2e/`. `vectra-web/` se verifica con `scripts/check-vectra-web.sh` + `node --test` y un stub de DOM (T-01-08 cubre foco del Omnibox **sin** navegador). `vortex serve` sirve la SPA desde `paths.web`, arranca en degradado si Meilisearch está caído, e indexa el outbox cuando el motor está `up`. `vortex seed` importa `seed/` (~200 SKUs, T-02-41) en modo directo con el servidor detenido. CI: seis jobs en `macos-14` (`fmt`, `clippy`, `test`, `domain-deps`, `openapi-lint`, `meilisearch`); `scripts/check-ci.sh` no menciona e2e. `.gitignore` ya ignora `node_modules/`. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: runner `macos-14`; Meilisearch por URL ([2.7](../../docs/2-arquitectura.md#27-red-local-y-seguridad-mínima)); Playwright es herramienta de desarrollo ([7.2](../../docs/7-tickets.md)); `vectra-web` MUST NOT tener `package.json`; `check-ci.sh` sigue siendo grep POSIX y **no** sube el mínimo de `macos-14`/`actions/cache@v4` por encima de 3 (mismo truco que T-09-05).

## Goals / Non-Goals

**Goals:**

- Un proyecto `e2e/` aislado de la SPA, reutilizable por T-09-10+
- Una fixture que deja un Nodo Maestro temporal (SurrealDB + seed + Meilisearch + `vortex serve`) listo antes del primer spec
- Un smoke que demuestra el Listo cuando en Chromium, más axe `critical`/`serious`
- Un job de CI paralelo que no ensucia `cargo test --workspace`

**Non-Goals (diseño):**

- Firefox / WebKit / `playwright test --ui`
- `webServer` de Playwright apuntando a un `vortex serve` de desarrollo del usuario (puerto 8080 fijo)
- Docker / Colima para Meilisearch (mismo criterio que T-09-05)
- Compilar `vortex` en *release* en CI (debug alcanza; el job ya cachea `target/`)
- Parser YAML propio; se sigue usando grep POSIX
- Job que ejecute `check-ci.sh` en GitHub
- `act` para simular macos-14
- Arreglar violaciones axe `moderate`/`minor` preexistentes (fuera del Listo cuando)

## Decisions

### 1. `e2e/` con `package.json`; `vectra-web/` intacto

**Elección:** `e2e/package.json` + `e2e/package-lock.json` con `@playwright/test` y `@axe-core/playwright` pinneados. Specs en JavaScript (`e2e/tests/*.spec.js`) para no introducir TypeScript en el repo. `vectra-web/` no recibe manifiesto npm.

**Por qué:** el ticket nombra `e2e/`. La spec `vectra-web` prohíbe `package.json` en la SPA (ES Modules servidos por el Gateway). Playwright no se distribuye ([7.2](../../docs/7-tickets.md)).

**Alternativa:** Playwright en la raíz. Se descartó: contaminaría el checkout y chocaría con el harness `check-vectra-web.sh`. **Alternativa:** TypeScript. Se descartó: Playwright lo soporta, pero el resto del front es JS vanilla.

### 2. Fixture propia (globalSetup/teardown), no `webServer` de Playwright

**Elección:** `e2e/harness.mjs` exportado como `globalSetup` / `globalTeardown` de `playwright.config.js`. El harness: reserva dos puertos libres (Vortex y Meilisearch); crea un dir temp; escribe `vortex.toml` (`paths.datos` = temp, `paths.web` = `<repo>/vectra-web`, `meilisearch.url` = Meili local); resuelve `VORTEX_BIN` o `vortex-core/target/debug/vortex`; corre `vortex seed <repo>/seed`; arranca Meilisearch (`MEILI_BIN` o el binario que dejó el job); arranca `vortex serve --config`; espera `GET /health` HTTP 200; escribe `e2e/.stack.json` (`baseURL`, pids, temp dir) que el config lee como `use.baseURL`. Teardown: SIGTERM a vortex y Meili, `rm -rf` del temp, borra `.stack.json`.

**Por qué:** hay que **seed con el servidor detenido** (`vortex-seed`) y luego servir. `webServer` de Playwright asume un único proceso HTTP. Puertos libres evitan chocar con un `vortex serve` local en `:8080` o Meili en `:7700`.

**Alternativa:** `webServer.command = "vortex serve"`. Se descartó: no cubre seed ni Meili ni temp `datos`. **Alternativa:** un `scripts/e2e-up.sh` invocado a mano. Se descartó: el Listo cuando exige CI desatendida.

### 3. Meilisearch en la fixture, no solo Vortex degradado

**Elección:** la fixture **siempre** levanta Meilisearch de `scripts/meilisearch-version` (mismo asset `meilisearch-macos-apple-silicon` que T-09-05). El job de CI lo descarga a un path y exporta `MEILI_BIN`. En local, `MEILI_BIN` o `meilisearch` en `PATH`.

**Por qué:** `vortex serve` *puede* arrancar degradado, pero el dataset no se indexa y T-09-10 (búsqueda) heredaría una fixture inútil. Seed no habla con Meili; el Index Sync Worker sí, al subir `serve`. Arrancar Meili **antes** de `serve` evita el wait de 20 s del boot degradado.

**Alternativa:** Meili solo en T-09-10. Se descartó: el **Hace** pide dataset de desarrollo en la fixture; un stack a medias no es el Nodo Maestro que ve el operador.

### 4. Un spec de smoke + helper axe; Chromium only

**Elección:** `e2e/tests/smoke.spec.js` con dos tests (o un test con dos aserciones nombradas): carga `#/` y foco del `<input>` del Omnibox (vía `vx-omnibox` + `shadowRoot`); `AxeBuilder` con `withTags(['wcag2a', 'wcag2aa', 'wcag22aa'])` y fallo si `impact` ∈ {`critical`,`serious`}. Helper `e2e/axe.js` testeado contra un HTML estático `e2e/fixtures/axe-violation.html` (p. ej. `<img>` sin `alt`). `projects: [{ name: 'chromium' }]`.

**Por qué:** Listo cuando = un test. El **Hace** pide axe. Shadow DOM es el Omnibox real (T-01-08). `moderate`/`minor` en custom elements / glassmorphism no deben bloquear el setup. Un helper con fixture HTML cubre el unhappy de axe **sin** romper la SPA a propósito.

**Alternativa:** fallar en cualquier violación axe. Se descartó: riesgo de ruido fuera del Listo cuando y de ~2 h. **Alternativa:** `page.locator('vx-omnibox input')` sin entrar al shadow. Se descartó: el input **está** en el shadow; Playwright lo perfora con `locator('vx-omnibox').locator('input')`.

Foco: `expect(omniboxInput).toBeFocused()` después de `goto(baseURL)` (el *router* default es operador). Unhappy del smoke: el mismo spec; no hay un segundo archivo que simule admin.

### 5. Job `e2e` en paralelo, no un step de `test`

**Elección:** `jobs.e2e` con `runs-on: macos-14`. Steps: checkout; `rustup show`; `actions/cache@v4` (key `macos-14-e2e-` + hash de `Cargo.lock`); `cargo build --bin vortex` en `vortex-core/`; `actions/setup-node@v4` (Node 22, cache npm sobre `e2e/package-lock.json`); `npm ci` en `e2e/`; `npx playwright install chromium`; descargar Meilisearch como el job `meilisearch` (leer `scripts/meilisearch-version`); `MEILI_BIN` + `VORTEX_BIN=vortex-core/target/debug/vortex` + `npx playwright test` en `e2e/`. Timeout del job ≥ 20 min (seed ~200 SKUs + boot).

**Por qué:** el ticket dice *job*. Compilar y npm no deben mezclarse con `cargo test --workspace`. El job `meilisearch` ya ocupa `:7700` en **otra** VM; este job usa puertos libres de la fixture, no hace `needs: meilisearch`.

**Alternativa:** `needs: [test, meilisearch]`. Se descartó: alarga el critical path y no comparte procesos. **Alternativa:** `playwright install --with-deps`. Se descartó: es receta de Ubuntu; en `macos-14` basta Chromium.

`check-ci.sh` **no** sube el mínimo de caches/`macos-14` de 3 a 7 (si no, `missing-e2e-job` fallaría por «cache» en vez de por `e2e`). Orden de chequeos nuevos **después** de Meilisearch: job `e2e` (`^[[:space:]]{2}e2e:`), cadena `playwright`.

### 6. Harness local del contrato: grep, no Playwright en `check-ci.sh`

**Elección:** `check-ci.sh` exige el YAML. Las fixtures `missing-e2e-job` y `missing-playwright` viven en `scripts/fixtures/ci/scenarios/`. El smoke real se corre con Playwright (local o job), no con grep.

**Por qué:** igual que T-09-05 (el probe HTTP no lo simula `check-ci`). `check-ci` tiene que seguir siendo POSIX y barato.

### 7. Artefactos de Playwright gitignored; lockfile commiteado

**Elección:** `.gitignore` añade `e2e/test-results/`, `e2e/playwright-report/`, `e2e/blob-report/`, `e2e/.cache/`, `e2e/.stack.json`. `e2e/package-lock.json` **sí** se versiona. `playwright.config.js` pone `fullyParallel: false` y `workers: 1` (un solo `vortex serve` con RocksDB embebido).

**Por qué:** SurrealDB embebido no es seguro para N procesos `serve` sobre el mismo `datos`. T-09-10 podrá relajar workers si aísla dirs; este ticket no.

## Risks / Trade-offs

- **[Seed de ~200 SKUs en cada run]** → 10–60 s extra. Mitigación: un solo globalSetup por suite; timeout de job holgado. No se cachea la DB entre PRs (datos temporales).
- **[axe vs Shadow DOM / glassmorphism]** → ruido `moderate`. Mitigación: umbral `critical`/`serious`; helper cubre el unhappy.
- **[Grep del YAML]** → un comentario `e2e:` falsearía. Mitigación: patrón ancla indentación de job; archivo corto.
- **[Listo cuando real es GitHub]** → harness cubre contrato YAML; Playwright local cubre el smoke; el PR de `T-09-09` es la prueba operativa del job.
- **[`openspec new change` rechaza `T-09-09`]** → scaffold en `t-09-09` y rename (igual que T-09-01 / T-09-05).
- **[MeiliTextSearch::new falla con URL inválida]** → la fixture siempre escribe una URL `http://127.0.0.1:<puerto>`.
- **[Alcance ~2 h]** → proyecto + fixture + un smoke + axe helper + job + grep. No se divide el backlog. T-09-10 escribe specs de negocio sobre esta fixture.

## Migration Plan

1. Red: `check-ci.sh` exige job `e2e` → el repo actual (seis jobs) sale ≠ 0.
2. Fixtures YAML sin el job / sin `playwright`: salen ≠ 0 y nombran el motivo.
3. Red Playwright: `e2e/tests/smoke.spec.js` existe; `npx playwright test` falla (no hay fixture/stack).
4. Red axe helper: documento con `<img>` sin alt → fallo.
5. Green: harness, job, deps; `check-ci.sh` sobre el repo sale 0; Playwright smoke + axe en verde con stack arriba.
6. Rollback: revertir `e2e/`, el job y las líneas nuevas de `check-ci.sh`. No hay datos de producción.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre del job (`e2e`), Chromium only, y el umbral axe `critical`/`serious` quedan fijados aquí.
