# Proposal

## Ticket padre

- **Ticket:** [T-01-13](../../../docs/7-tickets.md) — *Manifest* PWA
- **Change:** `T-01-13`
- **HU:** [HU-01-01](../../../docs/6-historias.md#hu-01-01--aplicación-instalable-pwa)
- **Depende de:** T-01-01 (cerrado; esqueleto `vectra-web/` y `index.html`); T-00-19 (cerrado; `GET /manifest.webmanifest` con `no-cache`)
- **Hace:** `manifest.webmanifest` (nombre Vectra, `display: standalone`, colores `#0A0A0A`, íconos 192/512 y *maskable*) enlazado en `index.html`.
- **No hace:** Service Worker.
- **Listo cuando:** Lighthouse marca la app como instalable (junto con T-01-14).

## Why

HU-01-01 pide instalar Vectra como aplicación en la terminal (ícono, ventana propia). T-00-19 ya reserva `GET /manifest.webmanifest` y T-01-01 ya tiene `index.html`, pero no hay *manifest* ni íconos: Chrome/Lighthouse no pueden tratar la SPA como instalable. T-01-14 (Service Worker) no puede completar el criterio de instalable sin este documento.

## What Changes

- `vectra-web/manifest.webmanifest`: JSON con `name`/`short_name` **Vectra**, `display: "standalone"`, `background_color` y `theme_color` `#0A0A0A`, `start_url` `/`, `lang` `es`, íconos PNG 192 y 512 (`purpose` `any` y `maskable`).
- Íconos en `vectra-web/icons/` (PNG reales con dimensiones verificables). Los `src` apuntan a `/static/{{VERSION}}/icons/…` para que T-00-19 los sirva con caché inmutable.
- `index.html` enlaza el *manifest* en la ruta de *app shell* (`href="/manifest.webmanifest"`, no versionada) y declara `<meta name="theme-color" content="#0A0A0A">`.
- El Gateway sustituye `{{VERSION}}` en el cuerpo de `GET /manifest.webmanifest` (igual que en `index.html`); `Cache-Control: no-cache` no cambia.
- TDD en `scripts/check-vectra-web.sh` + `manifest.test.mjs`: happy = JSON válido, campos Lighthouse, íconos 192/512 *maskable*, *link* en `index.html`; unhappy = JSON inválido, sin *link*, sin *maskable*, PNG con tamaño distinto, registro de Service Worker (No hace).
- MUST NOT crear `sw.js` ni llamar a `navigator.serviceWorker.register`. MUST NOT cambiar `docs/openapi.yaml`.

El **Listo cuando** del ticket es compartido con T-01-14 (Lighthouse «instalable» exige *manifest* **y** Service Worker). Este change cierra la mitad *manifest*; la suite automatiza esos criterios (sin Lighthouse CLI ni npm). El SW queda para T-01-14.

## Non-goals

- Service Worker, precache, aviso «Sin conexión con el Nodo Maestro» (PA-04) — **No hace** del ticket; **T-01-14**
- Actualización del SW / «Nueva versión disponible» — **T-01-15**
- HTTPS, CA local, `vectra.local` — **T-00-16**, **T-00-17**, **T-00-18** (v2; en MVP `localhost` ya es contexto seguro)
- `apple-touch-icon` / metatags exclusivos de iOS — no están en el Hace; Chrome en `localhost` basta para el MVP
- Ruta nueva `/icons/` ni servir PNG desde la raíz del origen — se reutiliza `/static/<versión>/`
- Playwright / e2e / Lighthouse en CI — **T-09-10**; este ticket no agrega `package.json` ni lighthouse
- jsdom, `package.json`, job nuevo de CI
- Mutar OpenAPI, `/health`, el *router* *hash* o Web Components

## Capabilities

### New Capabilities

- Ninguna. El *manifest* y los íconos son parte de la SPA `vectra-web`.

### Modified Capabilities

- `vectra-web`: agregar `manifest.webmanifest`, PNG 192/512 *any* y *maskable*, *link* + `theme-color` en `index.html`; ampliar la suite Node (happy + unhappy). MUST NOT registrar Service Worker (T-01-14).
- `vortex-api`: `GET /manifest.webmanifest` MUST sustituir `{{VERSION}}` en el cuerpo (hoy solo lo hace `index.html`). Sin rutas nuevas; `Cache-Control: no-cache` y MIME `application/manifest+json` siguen.

## Impact

- **Repo:** `vectra-web/manifest.webmanifest`; `vectra-web/icons/*.png`; `vectra-web/index.html`; `scripts/fixtures/vectra-web/manifest.test.mjs` (nuevo); `scripts/check-vectra-web.sh`; `vortex-core/crates/api/src/static_files.rs` (`get_manifest`) y `vortex-core/crates/api/tests/static_files.rs`
- **APIs / OpenAPI:** sin paths nuevos; reutiliza `GET /manifest.webmanifest` y `/static/<versión>/…` de T-00-19
- **Verificación:** `./scripts/check-vectra-web.sh`; `openspec validate T-01-13`; en `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- **Ticket:** cierra T-01-13 (criterios *manifest* de instalable; Lighthouse completo espera T-01-14)
- **Alcance:** cabe en ~2 h (*manifest* + 4 PNG + *link* + sustitución mínima + tests Node/Rust); no se divide
