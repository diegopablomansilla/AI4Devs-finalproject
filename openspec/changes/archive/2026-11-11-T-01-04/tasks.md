# Tasks

## 1. Tests (red) — árbol, catálogo y overlay

- [x] 1.1 Extender `check-vectra-web.sh` para exigir `vx-modal.js`, `vx-side-panel.js`, `vx-toast.js` y esos tags en `dev/components.html` (`components_tree_includes_modal_panel_toast`, `components_html_catalog_loads_overlay_tags`; unhappy: falta `vx-modal.js`) — `./scripts/check-vectra-web.sh` falla en rojo
- [x] 1.2 Extender `dom-stub.mjs` con `focus()`, `document.activeElement` y `KeyboardEvent` — un test mínimo de stub mueve `activeElement` al llamar `focus()`
- [x] 1.3 Agregar en `components.test.mjs` los tests `vx_modal_escape_emits_vx_closed`, `vx_modal_tab_stays_inside`, `vx_side_panel_close_restores_focus_to_opener`, `vx_toast_info_aria_live_polite`, `overlay_already_closed_escape_does_not_emit`, `vx_toast_without_aria_live_fails`, `vx_toast_unknown_variant_falls_back_to_info` — `node --test scripts/fixtures/vectra-web/components.test.mjs` falla en rojo

## 2. Implementación (green) — overlay, componentes y muestra

- [x] 2.1 `overlay.js` + `vx-modal.js` + `vx-side-panel.js`; `index.js` los registra — pasan `vx_modal_escape_emits_vx_closed`, `vx_modal_tab_stays_inside`, `vx_side_panel_close_restores_focus_to_opener`, `overlay_already_closed_escape_does_not_emit`
- [x] 2.2 `vx-toast.js` (variantes `info`/`error`, `aria-live`, sin `vx-closed`) — pasan `vx_toast_info_aria_live_polite`, `vx_toast_unknown_variant_falls_back_to_info`; el caso sin `aria-live` queda cubierto por el componente real
- [x] 2.3 Actualizar `dev/components.html` con modal, panel, toast y un control que abre el panel — harness de catálogo verde; al cerrar el panel el foco vuelve al abridor (Listo cuando)

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (happy + unhappy de este ticket)
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 3.3 `openspec validate T-01-04` verde
