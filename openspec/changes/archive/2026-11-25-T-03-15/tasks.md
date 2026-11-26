# Tasks

## 1. Tests (red) — harness y search.test.mjs

- [x] 1.1 Extender `scripts/check-vectra-web.sh` para exigir `vectra-web/js/search.js` (`search_missing_module_fails`; el mensaje nombra `search.js`) y para ejecutar `scripts/fixtures/vectra-web/search.test.mjs`; actualizar el comentario del script — `./scripts/check-vectra-web.sh` falla en rojo (falta el módulo y/o el fixture)
- [x] 1.2 Agregar `search.test.mjs` con `search_text_query_calls_get_search_with_q`, `search_cancels_previous_and_paints_last_only`, `search_renders_name_and_sku_as_text`, `search_arrow_down_from_omnibox_selects_first_hit` — `node --test scripts/fixtures/vectra-web/search.test.mjs` falla en rojo (escenarios happy / Listo cuando)
- [x] 1.3 Agregar `search_aborted_response_does_not_paint`, `search_abort_error_does_not_call_showError`, `search_non_text_mode_does_not_call_search`, `search_empty_query_clears_without_fetch`, `search_problem_json_calls_showError`, `search_empty_hits_shows_empty_list`, `search_highlight_html_is_plain_text`, `search_admin_view_does_not_call_search` — los ocho fallan en rojo (unhappy path)

## 2. Implementación (green) — módulo y mount

- [x] 2.1 Implementar `vectra-web/js/search.js` (`mountSearch`): `vx-query` modo texto → abort + `GET /api/v1/search?q=`; lista nombre/SKU con `textContent` + `bindKeyboardList`; consulta vacía limpia sin fetch; modo no-texto no llama; AbortError no pinta ni `showError`; `problem+json`/`network` → `showError` — pasan 1.2 y 1.3 salvo el de admin/router
- [x] 2.2 Cablear `startRouter` + `app.js`: en vista operador llamar `mountSearch` con `request`/`showError`; `unmount` al `paint`; admin y 404 no llaman a `/search` — `search_admin_view_does_not_call_search` verde; `search_missing_module_fails` cubierto por el harness

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (suite previa + search happy/unhappy); sin `vx-product-card` en resultados; sin `facets` en el query; `vx-omnibox.js` sigue sin importar `api.js`
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 3.3 `openspec validate T-03-15` verde