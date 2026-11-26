# Tasks

## 1. Tests (red) — árbol y hojas versionadas

- [x] 1.1 Extender `scripts/check-vectra-web.sh` para exigir `vectra-web/css/tokens.css` y `vectra-web/css/base.css`; el script sale distinto de 0 y el mensaje nombra `tokens.css` si falta — escenarios «el árbol incluye css, app, router y components», «faltan hojas o no están versionadas», «falta tokens.css»
- [x] 1.2 En el mismo harness, exigir en `index.html` los `link` `href="/static/{{VERSION}}/css/tokens.css"` y `/static/{{VERSION}}/css/base.css` (tokens antes que base), nombre `index_html_links_versioned_tokens_and_base`; con el HTML actual (sin `<link>`) el script sigue distinto de 0 — escenario «index.html enlaza tokens y base versionados»

## 2. Tests (red) — contrato `--vx-*`

- [x] 2.1 Agregar `scripts/fixtures/vectra-web/tokens-contrast.mjs` (`parseRootVars`, sin npm) y `tokens.test.mjs` (`node:test`) con `tokens_root_declares_vx_contract`: lee `vectra-web/css/tokens.css`, aserta todos los *tokens* de la tabla, `--vx-color-bg === "#0A0A0A"`, `--vx-touch-min === "44px"`, `--vx-color-surface` con alfa menor que 1 y `--vx-surface-blur` distinto de `0` — `node --test scripts/fixtures/vectra-web/tokens.test.mjs` no corre o falla (escenarios «tokens.css declara el contrato --vx-*», «falta un token o el fondo no es #0A0A0A», «la superficie por defecto no es translúcida»)

## 3. Tests (red) — contraste happy y unhappy

- [x] 3.1 Completar `tokens-contrast.mjs` con `compositeOver` y `contrastRatio` (WCAG 2.1) y el test `tokens_text_surface_pairs_meet_wcag_aa` sobre los nueve pares de la spec a partir de `vectra-web/css/tokens.css` — el test falla mientras no existan *tokens* válidos (escenario «los pares de tokens.css cumplen 4,5:1» / Listo cuando automatizado)
- [x] 3.2 Agregar fixture `scripts/fixtures/vectra-web/low-contrast.tokens.css` y `tokens_low_contrast_pair_fails`: el oráculo sobre ese archivo termina distinto de 0 y el mensaje nombra el par y el ratio — escenario «un par por debajo de 4,5:1 falla»

## 4. Tests (red) — transparencia reducida y base.css

- [x] 4.1 Agregar `tokens_reduced_transparency_disables_blur`: parsea `tokens.css` y `base.css`; exige `@media (prefers-reduced-transparency: reduce)` y que `.vx-surface` ahí no deje `backdrop-filter` con blur y use `--vx-color-surface-solid` — el test falla (escenarios «el glass tiene fallback sólido», «falta el media query o el glass sigue con blur»)
- [x] 4.2 Agregar `base_css_applies_bg_text_focus_and_touch_min`: `base.css` referencia `var(--vx-color-bg)`, `var(--vx-color-text)`, `.vx-surface`, `:focus-visible` + `var(--vx-color-focus-ring)`, y `var(--vx-touch-min)` en `a`/`button`/`input`/`select`/`textarea` — el test falla (escenarios «base.css pinta fondo, texto, foco y 44 px», «base.css no aplica el fondo o el mínimo táctil»)

## 5. Implementación (green) — hojas y documento

- [x] 5.1 Crear `vectra-web/css/tokens.css` con `:root` y los valores literales del design (ajustados hasta que el oráculo dé ≥ 4,5:1); borrar `css/.gitkeep` si ya no hace falta — `tokens_root_declares_vx_contract` y `tokens_text_surface_pairs_meet_wcag_aa` pasan; `tokens_low_contrast_pair_fails` sigue pasando sobre el fixture (escenarios 2.1 y 3.1–3.2)
- [x] 5.2 Crear `vectra-web/css/base.css` (`box-sizing`, fondo/texto de `body`, `.vx-surface` con `backdrop-filter`, `:focus-visible`, mínimo táctil, `@media (prefers-reduced-transparency: reduce)` sólido sin blur) — `tokens_reduced_transparency_disables_blur` y `base_css_applies_bg_text_focus_and_touch_min` pasan (escenarios 4.1–4.2)
- [x] 5.3 Enlazar las dos hojas versionadas en `vectra-web/index.html` (tokens luego base) y hacer que `check-vectra-web.sh` ejecute `node --test scripts/fixtures/vectra-web/*.test.mjs` (router + tokens) — `./scripts/check-vectra-web.sh` sale 0 (escenarios «index.html enlaza tokens y base versionados», «la suite acordada cubre happy y unhappy»; los tests del *router* de T-01-01 siguen verdes)

## 6. Cierre (regresión y § 5)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; sin tests Rust nuevos)
- [x] 6.2 Confirmar que no hay `package.json`, no se agregaron Web Components ni `vectra-web/dev/components.html`, y no se modificó `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --stat` / `ls vectra-web/js/components` (non-goals T-01-03+, CI, OpenAPI)
- [x] 6.3 `openspec validate T-01-02` (o `t-01-02` si el CLI exige minúsculas) sale 0
