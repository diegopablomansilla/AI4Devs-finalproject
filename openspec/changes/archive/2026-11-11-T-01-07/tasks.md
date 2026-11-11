# Tasks

## 1. Tests (red) — árbol y errores por code

- [x] 1.1 Extender `scripts/check-vectra-web.sh` para exigir `vectra-web/js/errors.js` (`errors_tree_includes_errors_js`; unhappy: falta `errors.js`) y para ejecutar `scripts/fixtures/vectra-web/errors.test.mjs` — `./scripts/check-vectra-web.sh` falla en rojo porque falta el módulo o los tests
- [x] 1.2 Crear `scripts/fixtures/vectra-web/errors.test.mjs` con los tests happy `errors_map_covers_openapi_error_codes` (lee `docs/openapi.yaml` vs mapa; Listo cuando), `errors_map_includes_network`, `errors_message_for_not_found_is_spanish`, `errors_show_error_sets_toast_error_variant`, `errors_show_error_assigns_field_errors_to_vx_field` — `node --test scripts/fixtures/vectra-web/errors.test.mjs` falla en rojo
- [x] 1.3 Agregar en el mismo archivo los tests unhappy `errors_map_missing_openapi_code_fails` (mapa sin `timeout` nombra el `code`; Listo cuando), `errors_unknown_code_uses_fallback_not_raw_code`, `errors_abort_does_not_show_toast`, `errors_unmatched_field_does_not_throw`, `errors_toast_text_is_not_technical_code` — `node --test scripts/fixtures/vectra-web/errors.test.mjs` falla en rojo cubriendo cobertura incompleta, fallback, abort, campo sin match y `title` técnico

## 2. Implementación (green) — errors.js

- [x] 2.1 Implementar `vectra-web/js/errors.js` (`MESSAGES`, `messageFor`, `showError`, `createErrorUi`, `errorCodesFromOpenApi`, `missingErrorCodes`) según spec/design: tabla de mensajes, fallback `internal`, toast `variant="error"`, `vx-field` por `name` exacto, abort no-op, cobertura OpenAPI + `network` — pasan los tests de 1.2 y 1.3
- [x] 2.2 Verificar el Listo cuando: el test `errors_map_covers_openapi_error_codes` compara el enum `ErrorCode` de `docs/openapi.yaml` con las claves de `MESSAGES` y queda verde; `errors_map_missing_openapi_code_fails` sigue cubriendo el camino no feliz

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde (happy + unhappy de este ticket, incluida la suite previa de router/tokens/componentes/api)
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 3.3 `openspec validate T-01-07` verde
