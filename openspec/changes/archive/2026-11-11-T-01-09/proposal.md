# Proposal

## Ticket padre

- **Ticket:** [T-01-09](../../../docs/7-tickets.md) — Modos del Omnibox
- **Change:** `T-01-09`
- **HU:** [HU-01-03](../../../docs/6-historias.md#hu-01-03--componente-omnibox)
- **Depende de:** T-01-08 (cerrado en `develop`; `vx-omnibox` con foco y Escape)
- **Hace:** selector de modos texto, voz e IA (imagen oculto hasta que la capacidad `vision` esté activa); prefijo `?` activa IA; evento `vx-query { mode, text }`; indicador visual del modo activo.
- **No hace:** ejecución de cada modo.
- **Listo cuando:** escribir `?hola` cambia el indicador a IA y emite el evento con `mode: "ai"`.

## Why

HU-01-03 pide que el Omnibox ofrezca los modos (texto, voz, IA; imagen si EP-07 está habilitada) y que el modo activo se vea. Hoy `vx-omnibox` es solo un campo: no hay selector, el prefijo `?` no cambia nada y T-01-08 prohibió `vx-query`. T-03-15, T-01-12, T-05-20 y T-06-09 necesitan el contrato de modos y el evento **antes** de ejecutar búsqueda, degradación o voz/IA.

## What Changes

- **BREAKING** (respecto de T-01-08): `vx-omnibox` **sí** emite `vx-query` y **sí** muestra selector de modos. Se retira el veto `MUST NOT emitir vx-query` / `MUST NOT implementar selector`.
- Selector en el shadow: controles **Texto**, **Voz** e **IA** (español, `min` táctil `--vx-touch-min`); modo **Imagen** en el DOM pero **oculto** salvo atributo booleano `vision` en el host (gancho para T-07-03 / T-01-12; en MVP el *router* no lo setea).
- Prefijo `?` al inicio de la consulta activa el modo `ai` y actualiza el indicador.
- Evento `vx-query` (`bubbles`, `composed`) con `detail: { mode, text }` en cada `input` y al cambiar de modo. `mode` ∈ `"text" | "voice" | "ai" | "image"`. `text` es el valor actual del input (incluye `?` si está).
- Indicador visual + `aria-pressed` / atributo `mode` del host.
- MUST NOT llamar a `/search`, `/ai`, `/speech`, cámara ni importar `api.js` (sin ejecución).
- TDD en `omnibox.test.mjs`: happy = `?hola` → indicador IA + `vx-query` `mode: "ai"` (Listo cuando); unhappy = `?` no inicial, Escape, `vision` ausente, modo sin fetch.
- `dev/components.html`: el Omnibox de muestra muestra el selector; un segundo host MAY llevar `vision` para ver Imagen.

## Non-goals

- Ejecutar búsqueda, RAG, transcripción o cámara — **T-03-15**, **T-05-20**, **T-06-09**, **T-07-11**
- Deshabilitar IA/voz según `/health` — **T-01-12**
- Publicar `features.vision` en config/`/health` — **T-07-03** (v2)
- Navegación por flechas/Enter de listas — **T-01-10**
- Atajo «mantener Espacio» con Omnibox vacío — **T-06-09** (v2)
- Intent router / prefijo `?` en el servidor — **T-05-24** (v2)
- Playwright, jsdom, `package.json`, job nuevo de CI, OpenAPI, crates Rust

El recorte MVP de HU-01-03 («solo modo texto; voz, IA e imagen en v2») aplica a la **ejecución**. Este ticket deja el selector y el evento listos; voz/IA/imagen no hacen nada más que cambiar modo y emitir `vx-query`.

## Capabilities

### New Capabilities

- Ninguna. Los modos son parte de `vx-omnibox` en la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: ampliar `vx-omnibox` con selector, prefijo `?`, indicador y `vx-query`; reemplazar el escenario que prohibía el evento.

## Impact

- **Repo:** `vectra-web/js/components/vx-omnibox.js` (y estilos locales del shadow); `vectra-web/dev/components.html`; `scripts/fixtures/vectra-web/omnibox.test.mjs`; `scripts/check-vectra-web.sh` (comentario / suite, sin archivos nuevos de árbol)
- **APIs / OpenAPI:** sin paths nuevos; MUST NOT llamar endpoints
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-09`; regresión `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en `vortex-core/` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-09 (Listo cuando: escribir `?hola` cambia el indicador a IA y emite `vx-query` con `mode: "ai"`)
- **Alcance:** cabe en ~2 h (un componente + tests Node); no se divide
