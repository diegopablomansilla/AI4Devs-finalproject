# Tasks

## 1. Tests (red) — Omnibox unavailable y layout

- [x] 1.1 Agregar en `scripts/fixtures/vectra-web/omnibox.test.mjs` los tests `omnibox_ai_unavailable_disables_ai_and_ignores_prefix`, `omnibox_voice_unavailable_disables_voice` y `omnibox_ai_unavailable_click_stays_text` — `node --test scripts/fixtures/vectra-web/omnibox.test.mjs` falla en rojo (`ai-unavailable` / `voice-unavailable` aún no existen); `omnibox_question_prefix_sets_ai_mode_and_emits_vx_query` sigue verde
- [x] 1.2 Agregar en `scripts/fixtures/vectra-web/router.test.mjs` los tests `router_operator_header_has_data_health` y `router_admin_header_has_data_health` — fallan en rojo (los encabezados no tienen `[data-health]`); `router_admin_does_not_call_api` sigue verde

## 2. Implementación (green) — atributos del Omnibox y placeholder

- [x] 2.1 En `vectra-web/js/components/vx-omnibox.js` observar `ai-unavailable` y `voice-unavailable`: botones IA/Voz `disabled` + `aria-disabled`, prefijo `?` no activa `ai`, clic ignorado, fallback a `text` — pasan los tests de 1.1; los de T-01-09 (sin esos atributos) siguen verdes
- [x] 2.2 En `vectra-web/js/router.js` agregar `[data-health aria-live="polite"]` en los encabezados operador y admin; en `base.css` estilo mínimo con *tokens* `--vx-*` existentes (sin *tokens* nuevos) — pasan los tests de 1.2; `router_admin_does_not_call_api` y `tokens_root_declares_vx_contract` siguen verdes

## 3. Tests (red) — poller `/health`

- [x] 3.1 Crear `scripts/fixtures/vectra-web/health.test.mjs` con `health_maps_llm_down_to_ai_false` y `health_llm_down_disables_ai_mode_text_stays` (Listo cuando); incluir el archivo en `node --test` de `scripts/check-vectra-web.sh` — fallan en rojo (`health.js` / `capabilitiesFromHealth` aún no existen)
- [x] 3.2 Agregar `health_polls_every_10s`, `health_refetches_on_focus`, `health_indicator_shows_degraded`, `health_speech_not_up_disables_voice`, `health_vision_absent_hides_image` — fallan en rojo
- [x] 3.3 Agregar `health_network_failure_disables_optional_keeps_text`, `health_503_starting_parses_snapshot`, `health_invalid_body_does_not_throw`, `health_does_not_show_reduced_search_banner` — fallan en rojo
- [x] 3.4 En `scripts/check-vectra-web.sh` exigir `vectra-web/js/health.js` (`health_tree_includes_health_js`) — el harness termina ≠ 0 y nombra `health.js`

## 4. Implementación (green) — health.js y arranque

- [x] 4.1 Crear `vectra-web/js/health.js` con `HEALTH_POLL_MS = 10000`, `capabilitiesFromHealth` y `startHealth({ fetch, window, document, intervalMs, setInterval })`: `GET /health` inmediato, cada 10 s, `focus` / `visibilitychange`; parsea 200 y 503; red/cuerpo inválido → `unknown` sin throw; evento `vx-health`; aplica `[data-health]` y atributos del Omnibox — pasan los tests de 3.1–3.3; `health_tree_includes_health_js` pasa
- [x] 4.2 En `vectra-web/js/app.js` importar y llamar `startHealth` **después** de `startRouter` — `app.js` referencia `health.js`; pintar admin sin `startHealth` no llama `fetch` (`router_admin_does_not_call_api` verde)

## 5. Refactor y cierre

- [x] 5.1 `./scripts/check-vectra-web.sh` verde (suite previa + health happy/unhappy + omnibox unavailable + `[data-health]`); actualizar el comentario del script; no hay texto "Búsqueda en modo reducido"
- [x] 5.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 5.3 `openspec validate T-01-12` verde
