# Tasks

## 1. Tests (red) — modos, `vx-query` y stub

- [x] 1.1 En `omnibox.test.mjs`, reemplazar `omnibox_does_not_emit_vx_query` por `omnibox_question_prefix_sets_ai_mode_and_emits_vx_query`, `omnibox_text_mode_emits_vx_query_with_mode_text`, `omnibox_click_ai_sets_mode_and_indicator`, `omnibox_selector_shows_text_voice_ai` — `node --test scripts/fixtures/vectra-web/omnibox.test.mjs` falla en rojo (el componente aún no emite `vx-query` ni tiene selector)
- [x] 1.2 Agregar `omnibox_image_mode_visible_with_vision`, `omnibox_question_not_at_start_stays_text`, `omnibox_image_mode_hidden_without_vision`, `omnibox_mode_does_not_call_search`, `omnibox_escape_on_ai_prefix_resets_to_text`, `omnibox_unknown_mode_falls_back_to_text` — los seis fallan en rojo; `omnibox_mode_does_not_call_search` espía `globalThis.fetch` y exige 0 llamadas
- [x] 1.3 Si los tests usan `[data-mode="…"]`, extender `_matchSelector` en `dom-stub.mjs` (mismo patrón que `data-field`) — un test mínimo encuentra el botón IA por ese selector; si se usa `querySelectorAll("button")` + `getAttribute`, no hace falta cambiar el stub

## 2. Implementación (green) — selector, prefijo y evento

- [x] 2.1 En `vx-omnibox.js`: toolbar de botones nativos (Texto/Voz/IA/Imagen, `data-mode`, `aria-pressed`, `--vx-touch-min`), atributo `mode`, prefijo `?` → `ai`, `_pinned`, `vx-query` `{ mode, text }` en `input` y en clic; `vision` muestra Imagen; `mode` inválido → `text`; sin `fetch` / sin `api.js` — pasan los tests de 1.1 y 1.2 y los de foco/Escape de T-01-08
- [x] 2.2 En `dev/components.html` dejar el `<vx-omnibox>` de muestra (selector visible) y agregar un segundo `<vx-omnibox vision>` — `components_html_catalog_loads_omnibox` sigue verde; a mano se ve el control Imagen en el segundo host

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (suite previa + modos happy/unhappy); actualizar el comentario del script si aún dice que el Omnibox no emite `vx-query`
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 3.3 `openspec validate T-01-09` verde
