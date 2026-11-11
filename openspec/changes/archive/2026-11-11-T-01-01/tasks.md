# Tasks

## 1. Tests (red) — árbol y documento

- [x] 1.1 Agregar `scripts/check-vectra-web.sh` que falla si faltan `vectra-web/index.html`, `css/`, `js/app.js`, `js/router.js` o `js/components/`; nombres `vectra_web_tree_includes_css_app_router_components` y (cuando exista el HTML) `index_html_declares_lang_es_and_versioned_module` — `./scripts/check-vectra-web.sh` sale distinto de 0 (escenarios «el árbol incluye css, app, router y components», «falta index.html o lang=es», «falta el router o el documento»)
- [x] 1.2 Completar el mismo script para exigir `lang="es"` en `html` y un `<script type="module" src="/static/{{VERSION}}/js/app.js">` — con un HTML incompleto (o ausente) el script sigue distinto de 0 (escenarios «el documento está en español y carga el módulo versionado», «el script no es módulo versionado»)

## 2. Tests (red) — router happy path

- [x] 2.1 Agregar `scripts/fixtures/vectra-web/router.test.mjs` (`node:test`) con stub de `window`/`document`/`outlet` y `router_operator_to_admin_productos_does_not_reload`: hash `#/` → `#/admin/productos` pinta `data-view="admin"` `data-section="productos"`, oculta operador y `reloadCount === 0` — `node --test scripts/fixtures/vectra-web/router.test.mjs` no corre o falla (escenario «navegar de operador a admin/productos cambia la vista sin recargar» / Listo cuando)
- [x] 2.2 Agregar `router_admin_productos_to_operator_does_not_reload` y `router_empty_hash_is_operator`: `#/admin/productos` → `#/` pinta operador sin recarga; hash vacío o `#` pinta operador — el mismo `node --test` falla (escenarios «volver de admin/productos al operador no recarga», «hash vacío se trata como operador»)

## 3. Tests (red) — router unhappy path

- [x] 3.1 Agregar `router_unknown_hash_is_not_found`: `#/no-existe` pinta `data-view="not-found"`, no operador ni admin, sin recarga — `node --test scripts/fixtures/vectra-web/router.test.mjs --test-name-pattern unknown` falla (escenario «ruta desconocida es not-found sin recargar»)
- [x] 3.2 Agregar `router_admin_without_section_is_not_found` y `router_invalid_admin_slug_is_not_found`: `#/admin`, `#/admin/`, `#/admin/productos/extra`, `#/admin/Productos` → `not-found` y sin `data-section="productos"` — `node --test` de esos nombres falla (escenarios «admin sin sección es not-found», «admin con segmento extra o slug inválido es not-found»)

## 4. Implementación (green) — esqueleto

- [x] 4.1 Crear `vectra-web/css/.gitkeep`, `vectra-web/js/components/.gitkeep` y `vectra-web/index.html` (`lang="es"`, *outlet* `id="app"`, script módulo `/static/{{VERSION}}/js/app.js`, sin hojas de estilo ni `package.json`) — la parte de árbol de `./scripts/check-vectra-web.sh` ya no falla por HTML/directorios (escenarios 1.1–1.2)
- [x] 4.2 Crear `vectra-web/js/app.js` que importa `startRouter` desde `./router.js` y lo arranca contra `#app` — el archivo existe y el import es ES Module (escenario «el árbol incluye css, app, router y components»; el *router* aún puede fallar)

## 5. Implementación (green) — router

- [x] 5.1 Implementar `parseHash` y `startRouter` en `vectra-web/js/router.js` (hashchange, vistas placeholder con `data-view` / `data-section`, enlaces `#/` y `#/admin/productos`, sin login) — `node --test scripts/fixtures/vectra-web/router.test.mjs` sale 0 (escenarios 2.1–3.2, «administración no pide credenciales» / Listo cuando)
- [x] 5.2 Hacer que `scripts/check-vectra-web.sh` invoque `node --test scripts/fixtures/vectra-web/router.test.mjs` al final — `./scripts/check-vectra-web.sh` sale 0 (escenario «la suite acordada cubre happy y unhappy»)

## 6. Cierre (regresión y § 5)

- [x] 6.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5; sin tests Rust nuevos)
- [x] 6.2 Confirmar que no hay `package.json`, no se agregaron Web Components ni `css/tokens.css`, y no se modificó `.github/workflows/ci.yml` ni `docs/openapi.yaml` — `git diff --stat` / `ls vectra-web/js/components` (non-goals T-01-02, T-01-03, CI, OpenAPI)
- [x] 6.3 `openspec validate T-01-01` (o `t-01-01` si el CLI exige minúsculas) sale 0
