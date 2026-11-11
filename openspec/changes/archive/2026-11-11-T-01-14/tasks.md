# Tasks

## 1. Tests (red) — harness y Service Worker Node

- [x] 1.1 En `scripts/check-vectra-web.sh` exigir `vectra-web/sw.js` (`sw_tree_includes_sw_js`) — el harness termina ≠ 0 y nombra `sw.js`
- [x] 1.2 Crear `scripts/fixtures/vectra-web/sw.test.mjs` con `sw_precache_is_versioned_app_shell` y `app_registers_service_worker` — `node --test scripts/fixtures/vectra-web/sw.test.mjs` falla en rojo (no hay `sw.js` / no hay `serviceWorker.register("/sw.js")` en `app.js`); incluir el archivo en `node --test` de `check-vectra-web.sh`; invertir/eliminar `manifest_does_not_register_service_worker`
- [x] 1.3 Agregar VM `sw_navigation_online_uses_network` y `sw_navigation_offline_serves_cached_shell` (Listo cuando) — fallan en rojo (no hay `install`/`fetch` evaluables)
- [x] 1.4 Agregar unhappy: `sw_health_and_api_are_network_only` (No hace), `sw_precache_with_api_or_media_fails` (fixture in-memory), `sw_register_without_service_worker_does_not_throw` — fallan en rojo hasta existir el oráculo que rechaza precache de catálogo y el helper de registro

## 2. Implementación (green) — sw.js y registro

- [x] 2.1 Crear `vectra-web/sw.js` clásico: caché `vectra-shell-{{VERSION}}`, precache del *app shell* (spec), `install` + `fetch` red-primero en `navigate` con respaldo `/` o `/index.html`; `/health`, `/api/*` y `/media/*` solo red; sin `skipWaiting` ni `clients.claim` — pasan 1.2 y 1.3; `sw_health_and_api_are_network_only` y `sw_precache_with_api_or_media_fails` verdes
- [x] 2.2 En `vectra-web/js/app.js` registrar `navigator.serviceWorker.register("/sw.js")` si `serviceWorker` existe; si no, no tirar — pasan `app_registers_service_worker` y `sw_register_without_service_worker_does_not_throw`; pasa 1.1

## 3. Tests (red) — banner PA-04

- [x] 3.1 En `router.test.mjs` agregar `router_operator_header_has_offline_banner` y `router_admin_header_has_offline_banner` — fallan en rojo (no hay `[data-offline-banner]` / `aria-live="assertive"` / texto PA-04)
- [x] 3.2 En `health.test.mjs` agregar `health_network_failure_shows_offline_banner` (Listo cuando), `health_reachable_hides_offline_banner` y `health_degraded_does_not_show_offline_banner`; extender `health_network_failure_disables_optional_keeps_text` y `health_does_not_show_reduced_search_banner` — fallan en rojo (el *poller* no muestra/oculta el banner)

## 4. Implementación (green) — banner en router y health.js

- [x] 4.1 En `router.js` (operador y admin) agregar `[data-offline-banner]` `hidden` con `aria-live="assertive"` y texto exacto `Sin conexión con el Nodo Maestro`; estilo en `base.css` con tokens existentes (sin tokens nuevos) — pasan 3.1
- [x] 4.2 En `health.js`, tras cada snapshot y en `hashchange`, mostrar el banner solo si `GET /health` falló por red; ocultarlo si 200 o 503 parseable; no mostrarlo en `degraded`; no cambiar copy de `[data-health]` ni `errors.js` — pasan 3.2; `health_503_starting_parses_snapshot` sigue verde sin banner

## 5. Tests (red) — Gateway sustituye VERSION en sw.js

- [x] 5.1 En `vortex-core/crates/api/tests/static_files.rs` agregar `sw_js_replaces_version`: fixture con `{{VERSION}}` en el JS, `GET /sw.js` → 200, `Content-Type` javascript, cuerpo con `version` y sin `{{VERSION}}`, `Cache-Control: no-cache` — `cargo test -p api --test static_files -- sw_js_replaces_version` falla (hoy `get_sw_js` sirve bytes crudos)

## 6. Implementación (green) — get_sw_js

- [x] 6.1 En `vortex-core/crates/api/src/static_files.rs`, `get_sw_js` lee el archivo como texto y sustituye `{{VERSION}}` por `state.version()` (mismo placeholder que `get_index` / `get_manifest`); MIME y `no-cache` no cambian — pasa 5.1; `sw_js_sends_no_cache` sigue verde

## 7. Refactor y cierre

- [x] 7.1 `./scripts/check-vectra-web.sh` verde (suite previa + sw happy/unhappy + banner + harness); actualizar el comentario del script; `app.js` registra `/sw.js`; no hay `skipWaiting` ni `clients.claim`
- [x] 7.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 7.3 `openspec validate T-01-14` verde
