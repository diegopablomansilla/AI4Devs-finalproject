# Tasks

## 1. Tests (red) — harness, precache y activate

- [x] 1.1 En `scripts/check-vectra-web.sh` exigir `vectra-web/js/sw-update.js` (`sw_update_tree_includes_module` / `sw_update_tree_missing_module_fails`) — el harness termina ≠ 0 y nombra `sw-update.js`
- [x] 1.2 En `scripts/fixtures/vectra-web/sw.mjs` extender `createMemoryCacheMock` con `keys()` y `delete(name)`; en `sw.test.mjs` agregar `sw_precache_includes_sw_update`, `sw_activate_deletes_old_shell_caches` (Listo cuando), `sw_activate_keeps_current_cache` y `sw_does_not_skip_waiting_or_claim` (No hace, cubre `sw.js` y `sw-update.js`) — `node --test scripts/fixtures/vectra-web/sw.test.mjs` falla en rojo (no hay `activate` evaluable / precache sin `sw-update.js`)

## 2. Implementación (green) — activate y precache

- [x] 2.1 Crear `vectra-web/js/sw-update.js` como ES Module vacío exportable (stub) — pasa 1.1
- [x] 2.2 En `vectra-web/sw.js` agregar `activate` que borre cachés `vectra-shell-*` distintas de `CACHE`; incluir `/static/{{VERSION}}/js/sw-update.js` en `PRECACHE`; MUST NOT `skipWaiting` ni `clients.claim` — pasan 1.2; `sw_navigation_offline_serves_cached_shell` y `sw_health_and_api_are_network_only` siguen verdes

## 3. Tests (red) — markup del aviso

- [x] 3.1 En `router.test.mjs` agregar `router_operator_header_has_update_banner` y `router_admin_header_has_update_banner` — fallan en rojo (no hay `[data-update-banner]` / `aria-live="polite"` / texto `Nueva versión disponible` / `[data-update-reload]`)

## 4. Implementación (green) — banner en router

- [x] 4.1 En `router.js` (operador y admin) agregar `[data-update-banner]` `hidden` con `aria-live="polite"`, texto `Nueva versión disponible` y `button` `[data-update-reload]` «Recargar» (`var(--vx-touch-min)`); estilo en `base.css` con tokens existentes (sin tokens nuevos) — pasan 3.1; `router_operator_header_has_offline_banner` sigue verde

## 5. Tests (red) — watcher de waiting y Recargar

- [x] 5.1 En `sw.test.mjs` agregar `sw_waiting_shows_update_banner`, `sw_reload_control_reloads_without_skip_waiting` (Listo cuando), `sw_without_waiting_hides_update_banner`, `sw_first_install_does_not_show_update_banner` y `sw_update_check_failure_does_not_throw` — fallan en rojo (el stub no detecta *waiting* ni llama `location.reload`)

## 6. Implementación (green) — sw-update.js y app.js

- [x] 6.1 En `sw-update.js` implementar `startSwUpdateWatcher` / `applyUpdateBanner`: aviso solo si `waiting` + `controller`; `updatefound` → `installed` con controller; `registration.update()` al foco; Recargar = `location.reload()` sin `skipWaiting`; si no hay `serviceWorker` o `update()` rechaza, no tirar — pasan 5.1
- [x] 6.2 En `vectra-web/js/app.js` arrancar el *watcher* tras `registerAppServiceWorker` cuando existe `#app` — `app_registers_service_worker` y `sw_register_without_service_worker_does_not_throw` siguen verdes

## 7. Refactor y cierre

- [x] 7.1 `./scripts/check-vectra-web.sh` verde (suite previa + activate + aviso + Recargar + harness `sw-update.js`); actualizar el comentario del script; no hay `skipWaiting` ni `clients.claim` ni `reload` automático al detectar *waiting*
- [x] 7.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 7.3 `openspec validate T-01-15` verde
