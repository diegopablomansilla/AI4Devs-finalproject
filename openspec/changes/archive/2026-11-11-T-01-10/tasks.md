# Tasks

## 1. Tests (red) — harness, listbox y stub

- [x] 1.1 Extender `scripts/check-vectra-web.sh` para exigir `vectra-web/js/keyboard-list.js` (`keyboard_list_tree_includes_module`; unhappy: falta el archivo y el mensaje nombra `keyboard-list.js`) y para ejecutar `scripts/fixtures/vectra-web/keyboard-list.test.mjs` — `./scripts/check-vectra-web.sh` falla en rojo
- [x] 1.2 Agregar `keyboard-list.test.mjs` con `keyboard_list_arrow_down_selects_first_and_updates_activedescendant`, `keyboard_list_enter_opens_active_item`, `keyboard_list_escape_restores_omnibox_without_clearing`, `keyboard_list_arrow_from_omnibox_selects_first`, `keyboard_list_plain_options_are_reusable`, `components_html_catalog_loads_keyboard_list` — `node --test scripts/fixtures/vectra-web/keyboard-list.test.mjs` falla en rojo (el módulo aún no existe)
- [x] 1.3 Agregar `keyboard_list_empty_does_not_throw`, `keyboard_list_enter_without_selection_does_not_open`, `keyboard_list_escape_without_omnibox_does_not_throw`, `keyboard_list_does_not_call_search` — los cuatro fallan en rojo; `keyboard_list_does_not_call_search` espía `globalThis.fetch` y exige 0 llamadas

## 2. Implementación (green) — módulo y catálogo

- [x] 2.1 Implementar `vectra-web/js/keyboard-list.js`: `bindKeyboardList({ list, getItems, controllers, restoreFocus, onOpen })` con listbox, `aria-activedescendant`, ciclo de flechas, Enter → `onOpen`, Escape solo en la lista → foco Omnibox, `unbind()`, sin `fetch` / sin `api.js` — pasan los tests de 1.2 y 1.3
- [x] 2.2 En `dev/components.html` agregar la sección de lista de ejemplo (≥2 `vx-product-card`), importar `keyboard-list.js`, cablear `controllers` con el input del Omnibox y `onOpen` al `vx-side-panel` de muestra — `components_html_catalog_loads_keyboard_list` verde; a mano se recorre, abre y cierra sin mouse (Listo cuando)

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (suite previa + keyboard-list happy/unhappy); `data-region="results"` del *router* sigue sin `vx-product-card`; actualizar el comentario del script
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 3.3 `openspec validate T-01-10` verde
