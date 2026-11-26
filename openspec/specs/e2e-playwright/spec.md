# e2e-playwright Specification

## Purpose
Define el *setup* de tests de extremo a extremo de la SPA Vectra: un proyecto de desarrollo con navegador real, una fixture que arranca Vortex contra datos temporales y el dataset de desarrollo, un smoke de carga con foco en el Omnibox, un chequeo de accesibilidad, y el job de CI que lo ejecuta, para que T-09-10 y siguientes cuelguen escenarios de negocio de un stack reproducible.

## Requirements

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

### Requirement: e2e H1 — búsqueda con error tipográfico

La suite MUST incluir un spec Playwright cuyo nombre estable identifique la búsqueda con typo (`search_typo_tornilo_finds_tornillo_autoperforante` o equivalente). MUST reutilizar la fixture de T-09-09 (`vortex serve` + dataset `seed/` + Meilisearch). MUST abrir la vista operador y escribir `"tornilo"` en el campo de búsqueda del Omnibox (modo texto). MUST esperar a que el índice de productos esté consultable: `GET /health` 200 **no** alcanza si el outbox de Meilisearch aún no indexó el seed; el spec MUST reintentar hasta ver el hit o fallar por timeout. MUST NOT marcar el spec skipped.

Tras la espera, `data-region="results"` MUST contener un `vx-product-card` del producto semilla cuyo `nombre` es `"Tornillo autoperforante 10 x 1"` (SKU `TOR-AUTO-10X1`). MUST NOT exigir coincidencia exacta del string en el atributo `name` si el nombre va resaltado con `<mark>`: basta con que la tarjeta sea identificable por `sku="TOR-AUTO-10X1"` y el texto visible incluya «Tornillo autoperforante». MUST NOT abrir el panel de recomendaciones ni navegar a administración (T-09-11). MUST NOT enviar `facets`, `categoryId`, `brandId`, `inStock` ni `attr` desde la UI (recableo: sin T-03-19).

#### Scenario: tornilo encuentra Tornillo autoperforante (happy, HU-03-03, Listo cuando)

- **WHEN** la fixture dejó `vortex serve` escuchando con el dataset de desarrollo
- **AND** el navegador abre la vista del operador
- **AND** el operador escribe `"tornilo"` en el Omnibox
- **THEN** aparece un `vx-product-card` con `sku="TOR-AUTO-10X1"`
- **AND** el texto visible de esa tarjeta incluye «Tornillo autoperforante»
- **AND** no se abre `vx-side-panel` de recomendaciones
- **AND** el spec no queda skipped

#### Scenario: el índice aún no listo no se saltea (unhappy)

- **WHEN** corre el spec de typo y el índice no devuelve el tornillo a tiempo
- **THEN** el spec termina en fallo (timeout o aserción), no skipped

### Requirement: e2e H1 — tarjetas de resultado con stock

El spec de búsqueda (el de typo u otro en el mismo archivo) MUST asertar que el `vx-product-card` de `TOR-AUTO-10X1` es variante `full` y expone stock observable: atributo `stock` con valor decimal equivalente a `"40"` (`"40"`, `"40.0"` o `"40.00"`) **y** el valor visible en la tarjeta. MUST NOT exigir el estado «Sin stock» (`data-out-of-stock`): el dataset de desarrollo no incluye `stock_inicial` `"0"` y este ticket MUST NOT alterar `seed/`. MUST NOT exigir foto principal (`imageUrl` suele ser `null` en el MVP).

#### Scenario: la tarjeta del tornillo muestra stock 40 (happy, HU-03-05)

- **WHEN** la búsqueda `"tornilo"` ya pintó el `vx-product-card` `TOR-AUTO-10X1`
- **THEN** esa tarjeta tiene `variant="full"` o el default full
- **AND** el atributo `stock` es un decimal equivalente a 40
- **AND** el stock es visible en la tarjeta (no solo un dato interno)

#### Scenario: stock ausente o no visible hace fallar el spec (unhappy)

- **WHEN** la tarjeta de `TOR-AUTO-10X1` no expone `stock` equivalente a 40 o el stock no es visible
- **AND** corre el spec de tarjetas con stock
- **THEN** el spec termina en fallo (no skipped)

### Requirement: e2e H1 — navegación por teclado de resultados

Tras resultados visibles, con el foco en el input del Omnibox, el spec MUST enviar `ArrowDown`. MUST asertar que el `role="listbox"` de `data-region="results"` tiene `aria-activedescendant` igual al `id` de la primera `vx-product-card` (opción) y que esa tarjeta tiene `aria-selected="true"`. MUST NOT pulsar Enter ni clic en la tarjeta (eso abre recomendaciones, T-09-11). Un segundo escenario MUST enfocar el listbox y pulsar Escape: el foco MUST volver al Omnibox y las tarjetas MUST permanecer (no se vacía la consulta).

#### Scenario: ArrowDown desde el Omnibox activa la primera tarjeta (happy, T-01-10)

- **WHEN** hay al menos un `vx-product-card` en resultados
- **AND** el input del Omnibox tiene el foco
- **AND** el operador pulsa ArrowDown
- **THEN** el listbox tiene `aria-activedescendant` igual al `id` de la primera tarjeta
- **AND** esa tarjeta tiene `aria-selected="true"`
- **AND** no se abre el panel de recomendaciones

#### Scenario: Escape desde el listbox restaura el Omnibox sin vaciar (happy)

- **WHEN** hay resultados y el listbox tiene el foco
- **AND** el operador pulsa Escape
- **THEN** el input del Omnibox recupera el foco
- **AND** las `vx-product-card` siguen en `data-region="results"`

#### Scenario: sin selección de teclado el spec de flecha falla (unhappy)

- **WHEN** hay resultados y ArrowDown no actualiza `aria-activedescendant` ni `aria-selected`
- **AND** corre el spec de teclado
- **THEN** el spec termina en fallo (no skipped)

### Requirement: e2e H1 — caminos no felices de búsqueda (sin facetas)

La suite MUST cubrir estos unhappy paths en el mismo spec de búsqueda (nombres estables). MUST NOT implementar UI de filtros para hacerlos pasar.

1. Consulta de texto sin coincidencias (con espacio, para no disparar búsqueda por código): p. ej. `"zzzzz sinproducto"`. MUST dejar `data-region="results"` sin `vx-product-card`. MAY mostrar un mensaje en español «Sin resultados». MUST NOT dejar tarjetas de una consulta anterior si el spec busca primero y luego esta consulta.
2. Typo en SKU: escribir `"TOR-AUTO-10XI"` (un carácter distinto de `TOR-AUTO-10X1`). MUST NOT pintar un `vx-product-card` con `sku="TOR-AUTO-10X1"` (typo desactivado en `sku` / `codigos`).
3. Vaciar el Omnibox después de una búsqueda con hits: MUST quitar todas las `vx-product-card` de la región.
4. Ausencia de filtros facetados (No hace / recableo): con resultados visibles, MUST NOT haber una barra de `vx-chip` de filtros por atributo, marca, categoría o «con stock» en la vista operador.

#### Scenario: consulta sin hits deja la lista vacía (unhappy)

- **WHEN** el operador escribe `"zzzzz sinproducto"` en el Omnibox (índice ya consultable)
- **THEN** `data-region="results"` no contiene `vx-product-card`
- **AND** el spec no queda skipped

#### Scenario: typo en SKU no encuentra el tornillo (unhappy)

- **WHEN** el operador escribe `"TOR-AUTO-10XI"` en el Omnibox
- **THEN** no hay `vx-product-card` con `sku="TOR-AUTO-10X1"`

#### Scenario: vaciar el Omnibox limpia las tarjetas (unhappy)

- **WHEN** una búsqueda previa pintó al menos una `vx-product-card`
- **AND** el operador deja el Omnibox vacío
- **THEN** `data-region="results"` no contiene `vx-product-card`

#### Scenario: no hay filtros facetados en la vista operador (unhappy / non-goal)

- **WHEN** hay resultados de búsqueda de texto en la vista operador
- **THEN** no hay barra de filtros con `vx-chip` de categoría, marca, atributo o «con stock»

### Requirement: Tests previstos de e2e búsqueda H1

Antes de archive, la suite MUST incluir estos tests (nombres estables). Happy y unhappy path son obligatorios. El smoke de T-09-09 MUST seguir pasando y MUST NOT tipear consultas.

| Test | Ubicación | Camino |
|---|---|---|
| `search_typo_tornilo_finds_tornillo_autoperforante` | `e2e/tests/search.spec.js` | happy (Listo cuando, HU-03-03) |
| `search_card_shows_stock_40` | mismo spec | happy (HU-03-05) |
| `search_arrow_down_selects_first_card` | mismo spec | happy (T-01-10) |
| `search_escape_from_listbox_restores_omnibox` | mismo spec | happy |
| `search_unknown_query_shows_no_cards` | mismo spec | unhappy |
| `search_sku_typo_does_not_match_tornillo` | mismo spec | unhappy |
| `search_empty_query_clears_cards` | mismo spec | unhappy |
| `search_no_facet_filter_chips` | mismo spec | unhappy (No hace T-03-19) |
| `smoke_app_loads_omnibox_focused` | `e2e/tests/smoke.spec.js` | happy (regresión T-09-09) |

`npx playwright test` en `e2e/` MUST ejecutar el Listo cuando (typo + stock + teclado) y los unhappy de esta tabla. El job CI `e2e` existente MUST seguir invocando esa suite; este change MUST NOT exigir un job nuevo ni cambios a `scripts/check-ci.sh`. `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` (manifiesto `vortex-core/Cargo.toml`) MUST seguir en verde ([docs/desarrollo-git-openspec.md](../../../../../docs/desarrollo-git-openspec.md) § 5). `./scripts/check-vectra-web.sh` MUST seguir en verde.

#### Scenario: suite de búsqueda H1 cubre happy y unhappy (happy)

- **WHEN** se ejecuta `npx playwright test` en `e2e/`
- **THEN** pasan `search_typo_tornilo_finds_tornillo_autoperforante`, `search_card_shows_stock_40` y `search_arrow_down_selects_first_card`
- **AND** pasan los unhappy de consulta vacía de hits, typo de SKU, Omnibox vacío y ausencia de facetas
- **AND** el smoke de Omnibox de T-09-09 sigue pasando
