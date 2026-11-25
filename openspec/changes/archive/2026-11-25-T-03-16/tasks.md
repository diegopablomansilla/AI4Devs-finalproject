# Tasks

## 1. Tests (red)

- [x] 1.1 Extender `check-vectra-web.sh` (`highlight.js`, `highlight.test.mjs`) — falla sin módulo
- [x] 1.2 Agregar `highlight.test.mjs` (happy + unhappy) — rojo
- [x] 1.3 Ampliar `search.test.mjs` (mark, description, null fallback, img) — rojo; ajustar test legacy highlight

## 2. Implementación (green)

- [x] 2.1 Implementar `vectra-web/js/highlight.js`
- [x] 2.2 Integrar en `search.js` (nombre + descripción)
- [x] 2.3 Extender `dom-stub.mjs` mínimo para tests (`createTextNode`, selectors)

## 3. Refactor y cierre

- [x] 3.1 `./scripts/check-vectra-web.sh` verde
- [x] 3.2 `cargo fmt/clippy/test --workspace` verde; `openspec validate T-03-16`