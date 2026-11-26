# Tasks

## 1. Tests (red) — árbol y catálogo

- [x] 1.1 Extender `check-vectra-web.sh` para exigir `vx-button.js`, `vx-field.js`, `vx-chip.js`, `index.js`, `shared.js`, `dev/components.html` y los tres custom tags en el HTML — unhappy: falta `vx-button.js`
- [x] 1.2 Agregar `components.test.mjs` (stub DOM) con tests happy/unhappy de loading, error y chip disabled — fallan sin implementación

## 2. Implementación (green) — shared y componentes

- [x] 2.1 `shared.js` + `vx-button.js`, `vx-field.js`, `vx-chip.js`, `index.js`; `app.js` importa `index.js` — tests de componentes pasan
- [x] 2.2 `dev/components.html` con ejemplos navegables — harness de catálogo pasa

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde; `cargo fmt/clippy/test --workspace` en `vortex-core/` verde
- [x] 3.2 `openspec validate T-01-03` verde
