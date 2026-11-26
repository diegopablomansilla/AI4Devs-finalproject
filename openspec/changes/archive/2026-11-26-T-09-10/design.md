# Design

## Context

En `develop`: T-09-09 dejó `e2e/` (Playwright Chromium, `workers: 1`, `harness.mjs` que hace seed + Meili + `vortex serve` y espera `GET /health` 200). El smoke no tipea. T-03-15/T-03-17/T-01-10 ya pintan `vx-product-card` `full` con stock y `bindKeyboardList` (`aria-activedescendant`). T-03-14 cubre `"tornilo"` → `"Tornillo autoperforante 10 x 1"` contra Meilisearch, no contra la SPA. `seed/productos.jsonl` tiene `TOR-AUTO-10X1` con `stock_inicial` `"40"` y **cero** productos con stock 0. El Index Sync Worker indexa el outbox **después** de `serve`; `/health` 200 no garantiza hits. Ver `proposal.md` para motivación y `specs/e2e-playwright/spec.md` para el contrato.

Constraints: no cambiar el job `e2e` ni `check-ci.sh`; no tocar `vectra-web/` salvo bug que bloquee el Listo cuando; no alterar `seed/`; no abrir recomendaciones (T-09-11).

## Goals / Non-Goals

**Goals:**

- Un spec Playwright de negocio sobre la fixture existente
- Esperar el índice sin marcar skipped
- Cubrir typo + stock visible + teclado, y los unhappy de la spec
- Dejar el smoke y el YAML de CI intactos

**Non-Goals (diseño):**

- Extender `globalSetup` para drenar el outbox (salvo que el wait en el spec resulte inviable; entonces un poll acotado en el harness, no un job nuevo)
- Seed con stock 0 o SKUs extra
- Firefox / WebKit / `fullyParallel`
- Grep nuevo en `check-ci.sh` para nombres de tests
- Stub de `/search` (estos e2e hablan con el stack real)

## Decisions

### 1. Spec nuevo `search.spec.js`; smoke intocado

**Elección:** `e2e/tests/search.spec.js` con los nombres estables de la spec. `smoke.spec.js` sigue sin tipear.

**Por qué:** T-09-09 lo exige; T-09-11 colgará otro archivo (`recommendations.spec.js` o similar) sin mezclar.

**Alternativa:** un solo `h1.spec.js`. Se descartó: T-09-11 aún no corre y alargaría este ticket.

### 2. Wait de índice en helper de spec, no en `globalSetup`

**Elección:** `e2e/search-ready.js` (o función al tope del spec) hace `GET {baseURL}/api/v1/search?q=tornilo` en loop hasta que `hits` incluye `sku === "TOR-AUTO-10X1"` o timeout (~60 s) → fallo, no skip. Cada test de búsqueda que depende del catálogo lo llama (o un `test.beforeAll` del archivo).

**Por qué:** el contrato de T-09-09 (`/health` 200) permanece. El worker puede tardar decenas de segundos con ~200 SKUs. Playwright `expect.poll` / loop propio es el retry idiomático.

**Alternativa:** esperar `components.sync.outboxLag === 0` en `/health`. Se descartó como único criterio: lag 0 no prueba que Meili ya searchable `"tornilo"`. **Alternativa:** mover el wait al harness. Se deja como fallback si el spec se vuelve flaky (un solo sitio, todos los tests se benefician).

### 3. Interactuar el Omnibox perforando Shadow DOM

**Elección:** `page.locator("vx-omnibox").locator("input")` (igual que el smoke). `fill("tornilo")` o `pressSequentially` para disparar `input` → `vx-query`. Identificar la tarjeta con `page.locator('vx-product-card[sku="TOR-AUTO-10X1"]')`. Stock: `getAttribute("stock")` parseado como decimal + texto visible en el shadow (`locator("vx-product-card[sku=…]").locator("text=40")` o el nodo de stock del componente).

**Por qué:** el input vive en shadow abierto; Playwright lo perfora. El SKU es estable en el host (no depende de `<mark>`).

**Alternativa:** `page.evaluate` sobre `shadowRoot`. Se descartó: más frágil que locators.

### 4. Teclado: ArrowDown desde el input; Escape desde el listbox; sin Enter

**Elección:** con el input enfocado, `press("ArrowDown")` (el input es `controller` de `bindKeyboardList`). Aserción sobre el `role="listbox"` y `aria-selected` de la primera tarjeta. Para Escape: `locator('[data-region="results"] [role="listbox"]').focus()` y `press("Escape")` — el handler de Escape está en el listbox, no en el controller. MUST NOT `press("Enter")`.

**Por qué:** Enter abre recomendaciones (`onOpen` → T-04-08), alcance de T-09-11. Escape en el Omnibox (T-01-08) **vacía** la consulta; el escenario «sin vaciar» requiere foco en el listbox.

**Alternativa:** Tab hasta el listbox. MAY usarse si `focus()` no enfoca el `tabindex="0"`.

### 5. Unhappy de SKU y de «sin hits» son consultas distintas

**Elección:** `"TOR-AUTO-10XI"` (un carácter vs `TOR-AUTO-10X1`) para typo-off en códigos; `"zzzzz sinproducto"` (tiene espacio → no `looksLikeCode`, no llama `by-code`) para lista vacía.

**Por qué:** T-03-14 ya fijó que typo en `sku` no matchea. Una consulta tipo código sin espacio dispararía `by-code` en paralelo y ensuciaría el escenario de «sin hits».

**Alternativa:** `"xyzzyqqq"` sin espacio. Se descartó: `looksLikeCode` es verdadero y `by-code` 404 + `/search` vacío es otro contrato (T-03-23), no el de este ticket.

### 6. Facetas: aserción negativa, no UI nueva

**Elección:** `search_no_facet_filter_chips` comprueba que, con resultados, no hay `vx-chip` de filtro en operador (ni texto «Con stock» como chip de faceta). No se implementa T-03-19 para «tener algo que testear».

**Por qué:** el No hace y el recableo MVP son explícitos. Una aserción negativa documenta la frontera para T-09-11 / v2.

### 7. Sin cambios de CI YAML

**Elección:** el job `e2e` ya corre `npx playwright test` sobre `e2e/tests/`. Los specs nuevos entran solos. Timeout del job (30 min) alcanza.

**Por qué:** Listo cuando = «pasa en CI», no «nuevo job». `check-ci.sh` no lista nombres de tests.

## Risks / Trade-offs

- **[Flake por indexación lenta]** → helper de wait con timeout holgado; si falla en CI, mover el poll al harness (un cambio chico, mismo contrato de spec).
- **[Meilisearch typo umbral vs `"tornilo"`]** → T-03-14 ya lo cubre en el sidecar; si el e2e falla, el bug es de sync/UI, no de umbral, salvo regresión de settings.
- **[`fill()` no dispara `input`]** → caer a `pressSequentially` o `locator.fill` + `dispatchEvent('input')`.
- **[Stock serializado `"40"` vs `"40.00"`]** → comparar numéricamente, no string exacto.
- **[Enter accidental abre el panel y rompe aserciones]** → no pulsar Enter; asertar `vx-side-panel` no visible / cerrado.
- **[Alcance ~2 h]** → un archivo de spec + helper. No se divide.

## Migration Plan

1. Red: agregar `search.spec.js` con los nombres de la tabla; `npx playwright test search.spec.js` falla (no hay aserciones o timeout de índice / UI).
2. Green: helper `search-ready` + locators Omnibox / tarjeta / teclado / unhappy.
3. Regresión: `npx playwright test` (smoke + axe + search); § 5 en `vortex-core/`; `./scripts/check-vectra-web.sh`; `./scripts/check-ci.sh`.
4. Rollback: borrar `e2e/tests/search.spec.js` y el helper; el job `e2e` vuelve al smoke.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El wait vive en el spec; el fallback al harness no cambia requisitos.
