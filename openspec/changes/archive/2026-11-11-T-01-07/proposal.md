# Proposal

## Ticket padre

- **Ticket:** [T-01-07](../../../docs/7-tickets.md) — Mensajes de error por `code`
- **Change:** `T-01-07`
- **HU:** [HU-01-05](../../../docs/6-historias.md#hu-01-05--estado-del-sistema-y-errores-visibles)
- **Depende de:** T-01-06 (`js/api.js` / `ApiError`), T-01-04 (`vx-toast`, `vx-field`) — ambos en `develop`
- **Hace:** `js/errors.js` con un mensaje en español por cada valor del esquema `ErrorCode` de `openapi.yaml`, más `network`; `showError(err)` usa `vx-toast`; los errores de validación se asignan a los `vx-field` por `field`.
- **No hace:** textos de pantallas de negocio.
- **Listo cuando:** ningún `code` del esquema queda sin mensaje (verificación con un script que compara ambas listas).

## Why

T-01-06 ya entrega `ApiError { status, code, title, errors }` y `code: "network"` en fallos de `fetch`, pero la SPA aún no traduce esos `code` a texto de operador ni los muestra. HU-01-05 exige un mensaje en español asociado al `code`, sin textos técnicos. Sin un mapa único y un `showError`, cada vista futura (ABM, búsqueda, stock) reinventaría toasts y el cableado a `vx-field`.

## What Changes

- `vectra-web/js/errors.js`: mapa `ErrorCode` (enum de `docs/openapi.yaml`) + `network` → mensaje en español; `messageFor(code)` y `showError(err)`
- `showError` pinta `vx-toast` con `variant="error"` usando el mensaje del mapa (no el `code` ni stacks)
- Errores de validación (`errors[]` con `field` + `message`) se asignan al `vx-field` cuyo `name` coincide con `field`
- TDD: happy = cobertura completa del enum + `network`, toast por `code`, asignación a campos; unhappy = `code` ausente en el mapa (script rojo), `code` desconocido en runtime, abort sin toast, campo sin `vx-field` matching, toast que mostraría un `code` técnico
- Script de cobertura (Listo cuando): compara el enum `ErrorCode` de `docs/openapi.yaml` con las claves del mapa; se ejecuta desde `scripts/check-vectra-web.sh`
- Extender el árbol exigido (`js/errors.js`) y la suite `node --test` (sin `package.json`)

## Non-goals

- Textos de pantallas de negocio (formularios de producto, categorías, marcas, búsqueda, stock) — ticket **No hace**; tickets T-02-* / T-03-*
- Cablear `showError` en vistas concretas ni importarlo desde `app.js` / `router.js` (aún no hay ABM; los callers llegan con T-02-07, T-02-35, T-03-15, etc.)
- Cola, apilado o auto-dismiss de toasts — T-01-04 dejó un toast a la vez; este change solo decide *qué* texto mostrar
- Polling de `/health` y capacidades degradadas — **T-01-12**
- Cambiar títulos `problem+json` del Gateway (`crates/api`) o el enum `ErrorCode` de OpenAPI
- SSE (T-05-20), *multipart* (T-02-30…T-02-39), Playwright, jsdom, `package.json`, job nuevo de CI, crates Rust
- Sanitización HTML de mensajes (T-03-16); los textos del mapa y de `errors[].message` se asignan como texto plano

## Capabilities

### New Capabilities

- Ninguna. El mapa de errores es parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `js/errors.js` (`messageFor`, `showError`, mapa `ErrorCode` + `network`); ampliar el árbol exigido y la suite con cobertura OpenAPI vs mapa, toast y asignación a `vx-field` (happy + unhappy)

## Impact

- **Repo:** `vectra-web/js/errors.js`, `scripts/check-vectra-web.sh`, `scripts/fixtures/vectra-web/` (p. ej. `errors.test.mjs`)
- **APIs / OpenAPI:** sin paths nuevos; el mapa cubre el enum `ErrorCode` ya definido ([5.4](../../../docs/5-contract-api.md#54-errores), `docs/openapi.yaml`) más el `network` de T-01-06
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-07`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-07 (Listo cuando: script que compara enum `ErrorCode` vs claves del mapa; ningún `code` del esquema sin mensaje)
