# Tasks

## 1. Tests (red) — árbol y cliente HTTP

- [x] 1.1 Extender `scripts/check-vectra-web.sh` para exigir `vectra-web/js/api.js` (`api_tree_includes_api_js`; unhappy: falta `api.js`) y para ejecutar `scripts/fixtures/vectra-web/api.test.mjs` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo o los tests
- [x] 1.2 Crear `scripts/fixtures/vectra-web/api.test.mjs` con `createApi({ fetch, storage })` y los tests `api_request_parses_json_ok`, `api_request_posts_json_body`, `api_request_sends_terminal_header`, `api_request_forwards_abort_signal` (happy: JSON 2xx, POST JSON, `X-Vectra-Terminal` desde `vectra.terminal`, `signal`) — `node --test scripts/fixtures/vectra-web/api.test.mjs` falla en rojo
- [x] 1.3 Agregar en el mismo archivo los tests unhappy `api_request_problem_json_not_found`, `api_request_invalid_input_exposes_errors`, `api_request_network_failure_is_network_code`, `api_request_abort_is_not_network`, `api_request_non_json_error_is_internal`, `api_request_omits_terminal_header_when_empty` — `node --test scripts/fixtures/vectra-web/api.test.mjs` falla en rojo cubriendo `not_found`, `errors[]`, `network`, abort, `internal` y cabecera omitida

## 2. Implementación (green) — api.js

- [x] 2.1 Implementar `vectra-web/js/api.js` (`ApiError`, `createApi`, `request`) según spec/design: JSON 2xx, POST con `Content-Type`, cabecera `X-Vectra-Terminal`, `signal`, `problem+json` → `ApiError`, red → `network`, abort no envuelto, no JSON → `internal` — pasan los tests de 1.2 y 1.3
- [x] 2.2 Verificar el Listo cuando contra el Gateway: con `vortex serve` (o el binario ya levantado), `request("GET", "/no-such-route")` rechaza `ApiError` con `code: "not_found"` — observable en consola del navegador o un `node` que apunte al bind; no se agrega página `dev/`

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (happy + unhappy de este ticket, incluida la suite previa de router/tokens/componentes)
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 3.3 `openspec validate T-01-06` verde
