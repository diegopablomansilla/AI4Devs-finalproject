# Proposal

## Ticket padre

- **Ticket:** [T-01-12](../../../docs/7-tickets.md) — Estado del sistema desde `/health`
- **Change:** `T-01-12`
- **HU:** [HU-01-05](../../../docs/6-historias.md#hu-01-05--estado-del-sistema-y-errores-visibles)
- **Depende de:** T-01-06 (cerrado; `api.js` / `request`), T-01-09 (cerrado; modos del Omnibox), T-00-13 (cerrado; `GET /health`)
- **Hace:** `js/health.js` consulta `/health` cada 10 s y al recuperar el foco; publica capacidades (`textSearch`, `ai`, `voice`, `vision`); indicador de estado en el encabezado; deshabilita el modo IA si `llm` no está `up` y voz si `speech` no está `up`.
- **No hace:** mensajes de degradación de búsqueda (T-03-31).
- **Listo cuando:** con `llm: down` simulado, el modo IA aparece deshabilitado y el texto sigue disponible.

## Why

HU-01-05 pide que el operador vea cuándo una capacidad no está disponible (modo IA deshabilitado con indicador) y que la búsqueda por texto siga usable. T-00-13 ya expone `GET /health`; T-01-09 ya tiene selector Texto/Voz/IA. Falta el puente: sondear el snapshot, publicar capacidades y apagar IA/voz cuando el componente no está `up`. Sin eso, el mostrador ofrece modos que el Nodo Maestro no puede cumplir.

## What Changes

- `vectra-web/js/health.js`: ES Module que consulta `GET /health` al arrancar, cada 10 s y al recuperar el foco (`focus` / `visibilitychange`); mapea el esquema `Health` a capacidades `{ textSearch, ai, voice, vision }` y al `status` agregado; despacha `vx-health` y actualiza el indicador + el Omnibox.
- Indicador de estado en el encabezado de operador y de administración (`data-health`): texto en español, no solo color; `aria-live="polite"`.
- `vx-omnibox`: deshabilita **IA** si `ai` es falso (`llm` ≠ `up`) y **Voz** si `voice` es falso (`speech` ≠ `up`); el modo **Texto** permanece operable. El prefijo `?` no activa IA si está deshabilitada. El atributo `vision` se setea solo si la capacidad `vision` está `up` (en MVP el snapshot no trae `vision` → Imagen sigue oculta; gancho para T-07-03).
- TDD en `health.test.mjs` (y omnibox): happy = `llm: down` → IA deshabilitada, texto disponible, indicador degradado (Listo cuando); poll 10 s y refetch al foco; unhappy = red, 503 `starting`/`down`, cuerpo inválido, `speech` no `up`, sin aviso de "modo reducido".
- `app.js` arranca el *poller* (no el *router* al pintar, para no romper `router_admin_does_not_call_api`).
- MUST NOT cambiar `docs/openapi.yaml` ni el handler de `/health`.

## Non-goals

- Aviso "Búsqueda en modo reducido" / ocultar facetas y resaltado — **No hace** del ticket; **T-03-31** (v2, HU-03-10)
- Aviso "Sin conexión con el Nodo Maestro" (PA-04) — **T-01-14**
- Mensajes `problem+json` en español / `showError` — **T-01-07** (cerrado)
- Publicar `features.vision` / componente `vision` en OpenAPI — **T-07-03** (v2)
- Ejecutar búsqueda, RAG, transcripción o cámara — **T-03-15**, **T-05-20**, **T-06-09**, **T-07-11**
- UI límite 30 s y micrófono deshabilitado extra (además de este *hook*) — **T-06-10** (v2; consume este ticket)
- Mutar el registro de salud, sidecars o el esquema `Health` — **T-00-13** / **T-00-27**
- Playwright / e2e — **T-09-10**
- jsdom, `package.json`, job nuevo de CI, crates Rust

## Capabilities

### New Capabilities

- Ninguna. El *poller* y el indicador son parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `js/health.js` (poll `/health`, capacidades, indicador); deshabilitar modos IA/voz del Omnibox según el snapshot; ampliar la suite Node (happy + unhappy). MUST NOT pintar el aviso de búsqueda reducida (T-03-31).

## Impact

- **Repo:** `vectra-web/js/health.js`; `vectra-web/js/app.js`; `vectra-web/js/router.js` (placeholder `data-health` en encabezados); `vectra-web/js/components/vx-omnibox.js`; `vectra-web/css/base.css`; `scripts/fixtures/vectra-web/health.test.mjs` (nuevo) y `omnibox.test.mjs`; `scripts/check-vectra-web.sh`
- **APIs / OpenAPI:** sin paths nuevos; consume `GET /health` ya definido ([5.11](../../../docs/5-contract-api.md#511-degradación-vista-desde-la-api), esquema `Health` en `docs/openapi.yaml`). HTTP 200 (`ok`/`degraded`) y 503 (`starting`/`down`) devuelven el mismo JSON, no `problem+json`.
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-12`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-12 (Listo cuando: con `llm: down` simulado, el modo IA aparece deshabilitado y el texto sigue disponible)
- **Alcance:** cabe en ~2 h (*poller* + indicador + dos modos disabled + tests Node); no se divide
