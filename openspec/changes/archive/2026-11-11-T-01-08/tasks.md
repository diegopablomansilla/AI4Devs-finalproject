# Tasks

## 1. Tests (red) — árbol, stub, layout y Omnibox

- [x] 1.1 Extender `scripts/check-vectra-web.sh` para exigir `vectra-web/js/components/vx-omnibox.js` y el tag `<vx-omnibox>` en `dev/components.html` (`components_tree_includes_omnibox`, `components_html_catalog_loads_omnibox`; unhappy: falta `vx-omnibox.js`) y para ejecutar `scripts/fixtures/vectra-web/omnibox.test.mjs` — `./scripts/check-vectra-web.sh` falla en rojo
- [x] 1.2 Extender `dom-stub.mjs`: `document` como EventTarget y burbujeo light DOM / `composed` hasta `document` — un test mínimo despacha un evento `bubbles`+`composed` desde un hijo y el listener en `document` lo recibe
- [x] 1.3 Agregar en `router.test.mjs` `router_operator_layout_has_omnibox_results_panel` y `router_admin_view_has_no_omnibox` — `node --test scripts/fixtures/vectra-web/router.test.mjs` falla en rojo (HTML actual sin regiones)
- [x] 1.4 Agregar `omnibox.test.mjs` con `omnibox_focuses_on_connect`, `omnibox_focuses_on_vx_closed`, `omnibox_side_panel_close_focuses_omnibox`, `omnibox_escape_clears_query`, `omnibox_escape_on_empty_query_does_not_throw`, `omnibox_vx_closed_when_disconnected_does_not_throw`, `omnibox_escape_with_open_panel_does_not_clear_query`, `omnibox_does_not_emit_vx_query` — `node --test scripts/fixtures/vectra-web/omnibox.test.mjs` falla en rojo

## 2. Implementación (green) — componente, layout y catálogo

- [x] 2.1 Implementar `vx-omnibox.js` (shadow, `adoptedStyleSheets`, `role="search"`, `aria-label="Buscar"`, input, `value`, foco al conectar, listener `vx-closed` en `document` + *microtask*, Escape limpia, sin `vx-query` / sin fetch) y registrarlo en `index.js` — pasan `omnibox_focuses_on_connect`, `omnibox_focuses_on_vx_closed`, `omnibox_escape_clears_query`, `omnibox_escape_on_empty_query_does_not_throw`, `omnibox_vx_closed_when_disconnected_does_not_throw`, `omnibox_does_not_emit_vx_query`
- [x] 2.2 Pintar el layout operador en `router.js` (regiones omnibox / results / side-panel con `vx-side-panel` cerrado; enlace admin; admin sin Omnibox) y estilos `.vx-operator` en `base.css` — pasan `router_operator_layout_has_omnibox_results_panel`, `router_admin_view_has_no_omnibox` y `omnibox_side_panel_close_focuses_omnibox` / `omnibox_escape_with_open_panel_does_not_clear_query`
- [x] 2.3 Agregar `<vx-omnibox>` a `vectra-web/dev/components.html` — harness de catálogo verde (`components_html_catalog_loads_omnibox`)

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (happy + unhappy de este ticket, incluida la suite previa de router/tokens/componentes/api/errors); el test aislado `vx_side_panel_close_restores_focus_to_opener` sigue verde
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 3.3 `openspec validate T-01-08` verde
