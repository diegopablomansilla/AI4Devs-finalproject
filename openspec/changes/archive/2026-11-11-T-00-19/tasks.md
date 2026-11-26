# Tasks

## 1. Tests (red) — index.html y app shell

- [x] 1.1 Agregar `vortex-core/crates/api/tests/static_files.rs` con helper de fixture (`index.html` con `{{VERSION}}`, `sw.js`, `manifest.webmanifest`) y `index_html_replaces_version_and_sends_no_cache`: `GET /` → 200, `text/html`, cuerpo con la `version` y sin `{{VERSION}}`, `Cache-Control: no-cache` (no `no-store`); el mismo archivo por `GET /index.html` — `cargo test -p api --test static_files -- index_html` no compila o falla (escenarios «index.html sustituye VERSION y envía no-cache», «GET /index.html es el mismo documento» / Listo cuando)
- [x] 1.2 Agregar `index_html_missing_returns_404` y `post_index_is_not_200_html`: `GET /` sin `index.html` → 404 `application/problem+json` `not_found`; `POST /` con fixture presente no es 200 HTML — `cargo test -p api --test static_files -- index_html_missing post_index` no compila o falla (escenarios «index.html ausente es 404 problem+json», «POST / no es HTML 200»)
- [x] 1.3 Agregar `sw_js_sends_no_cache` y `manifest_sends_no_cache`: `GET /sw.js` y `GET /manifest.webmanifest` → 200, `Cache-Control: no-cache` (no `no-store`) — `cargo test -p api --test static_files -- sw_js manifest` no compila o falla (escenarios «sw.js envía no-cache», «manifest envía no-cache»)

## 2. Tests (red) — `/static` y `/media`

- [x] 2.1 Agregar `static_asset_sends_immutable_cache`: fixture `app.js` + `GET /static/<version>/app.js` → 200, cuerpo del archivo, `Cache-Control` con `max-age=31536000` e `immutable` — `cargo test -p api --test static_files -- static_asset_sends` no compila o falla (escenario «activo versionado envía caché inmutable» / Listo cuando)
- [x] 2.2 Agregar `static_asset_wrong_version_returns_404`: `GET /static/not-the-version/app.js` → 404 `not_found` y el cuerpo no es `app.js` — `cargo test -p api --test static_files -- static_asset_wrong` no compila o falla (escenario «activo con versión distinta es 404»)
- [x] 2.3 Agregar `static_path_traversal_returns_404`: archivo `secret.txt` fuera de `{web}`; `GET /static/<version>/../secret.txt` (y variante `%2e%2e` si el servidor decodifica) → 404 `not_found` y el cuerpo no es `secret.txt` — `cargo test -p api --test static_files -- static_path_traversal` no compila o falla (escenario «path traversal en /static es 404»)
- [x] 2.4 Agregar `media_file_sends_long_cache`, `media_missing_returns_404` y `media_path_traversal_returns_404`: `GET /media/productos/…/3f2a9c.jpg` → 200 + caché inmutable; missing → 404 `not_found`; `GET /media/../secret.txt` → 404 sin filtrar el secreto — `cargo test -p api --test static_files -- media_` no compila o falla (escenarios «/media envía caché larga», «/media inexistente es 404», «path traversal en /media es 404»)

## 3. Tests (red) — vortex serve

- [x] 3.1 Agregar `vortex_serve_serves_index_from_paths_web` en `vortex-core/bin/vortex/tests/serve.rs`: TOML con `paths.web` a un dir temporal que tiene `index.html` con `{{VERSION}}`; `vortex serve` + `GET /` → 200, `text/html`, `Cache-Control: no-cache`, cuerpo sin `{{VERSION}}` — `cargo test -p vortex -- vortex_serve_serves_index` no compila o falla (escenario «vortex serve GET / sirve index.html desde paths.web»)

## 4. Implementación (green) — AppState, join seguro y handlers

- [x] 4.1 Extender `AppState` con `web_root`, `media_root` y `version` (`CARGO_PKG_VERSION` por defecto); `with_health` / `new` siguen compilando; builder `with_static_roots` (o equivalente) para tests — `cargo test -p api --test router -- build_router_unknown_path` sale 0 (escenario «ruta inexistente responde 404» no se rompe)
- [x] 4.2 Implementar `safe_join` y los GET de `index.html`, `/sw.js`, `/manifest.webmanifest`, `/static/{version}/{*path}` y `/media/{*path}` en `router_shell`; 404 vía `ApiError::route_not_found`; MIME mínimo por extensión — `cargo test -p api --test static_files` sale 0 (escenarios 1.1–2.4 / Listo cuando)
- [x] 4.3 Confirmar que `/health`, `/metrics` y `GET /api/v1/no-such-route` (`no-store`) siguen iguales — `cargo test -p api --test health --test metrics --test middleware` sale 0

## 5. Implementación (green) — vortex serve

- [x] 5.1 En `serve.rs`, inyectar `PathBuf::from(config.paths.web)` y `PathBuf::from(config.paths.datos).join("media")` más la `version` del binario en `AppState` — `cargo test -p vortex -- vortex_serve_serves_index` sale 0 y los tests previos de `serve` siguen en verde (escenario 3.1)
- [x] 5.2 Confirmar que no se creó `vectra-web/`, no se cambió `docs/openapi.yaml` y no se agregaron crates a `[dependencies]` de `api` — `git diff --stat` / `rg vectra-web` (non-goals T-01-01, OpenAPI, allowlist)

## 6. Cierre (regresión y § 5)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; escenario «la suite acordada cubre happy y unhappy»)
- [x] 6.2 `./scripts/check-workspace.sh` y `./scripts/check-domain-deps.sh` salen 0 (`api` sin crates nuevas; `domain` sin I/O de estáticos)
- [x] 6.3 `openspec validate T-00-19` (o `t-00-19` si el CLI exige minúsculas) sale 0
