# Tasks

## 1. Tests (red) — harness y manifest Node

- [x] 1.1 En `scripts/check-vectra-web.sh` exigir `vectra-web/manifest.webmanifest`, los cuatro PNG bajo `vectra-web/icons/`, `link rel="manifest" href="/manifest.webmanifest"` y `meta name="theme-color" content="#0A0A0A"` en `index.html` (`manifest_tree_includes_webmanifest`, `manifest_tree_includes_png_icons`, `index_html_links_unversioned_manifest`, `index_html_declares_theme_color`) — el harness termina ≠ 0 y nombra el archivo o el `href` que falta
- [x] 1.2 Crear `scripts/fixtures/vectra-web/manifest.test.mjs` con `manifest_json_is_valid`, `manifest_declares_vectra_standalone_and_colors` y `manifest_declares_192_512_and_maskable_icons` (Listo cuando) — `node --test scripts/fixtures/vectra-web/manifest.test.mjs` falla en rojo (no hay JSON / campos); incluir el archivo en `node --test` de `check-vectra-web.sh`
- [x] 1.3 Agregar `manifest_icon_files_match_declared_sizes` (IHDR 192/512) — falla en rojo (no hay PNG o las dimensiones no coinciden)
- [x] 1.4 Agregar unhappy in-memory: `manifest_invalid_json_fails`, `manifest_without_standalone_fails`, `manifest_without_maskable_icon_fails`, `manifest_icon_src_file_missing_fails`, `manifest_wrong_icon_dimensions_fail` — fallan en rojo hasta existir el oráculo que rechaza esos fixtures
- [x] 1.5 Agregar `manifest_does_not_register_service_worker` (No hace) — falla en rojo si el test aún no aserta la ausencia de `serviceWorker.register` y de `vectra-web/sw.js`

## 2. Implementación (green) — manifest, íconos y index.html

- [x] 2.1 Crear `vectra-web/manifest.webmanifest` con `name`/`short_name` Vectra, `display: standalone`, `background_color` y `theme_color` `#0A0A0A`, `start_url` `/`, `scope` `/`, `lang` `es`, `icons` con `src` `/static/{{VERSION}}/icons/…` — pasan 1.2; `manifest_json_is_valid` y `manifest_declares_vectra_standalone_and_colors` verdes
- [x] 2.2 Crear los cuatro PNG (`icon-192.png`, `icon-512.png`, `icon-192-maskable.png`, `icon-512-maskable.png`) fondo `#0A0A0A`, *maskable* con marca en el 80 % central — pasan 1.3 y `manifest_declares_192_512_and_maskable_icons`; los unhappy de 1.4 verdes contra fixtures in-memory
- [x] 2.3 En `vectra-web/index.html` agregar `<link rel="manifest" href="/manifest.webmanifest">` y `<meta name="theme-color" content="#0A0A0A">` — pasan 1.1; `manifest_does_not_register_service_worker` verde; no existe `sw.js` ni `serviceWorker.register`

## 3. Tests (red) — Gateway sustituye VERSION

- [x] 3.1 En `vortex-core/crates/api/tests/static_files.rs` agregar `manifest_replaces_version`: fixture con `{{VERSION}}` en el JSON, `GET /manifest.webmanifest` → 200, `application/manifest+json`, cuerpo con `version` y sin `{{VERSION}}`, `Cache-Control: no-cache` — `cargo test -p api --test static_files -- manifest_replaces_version` falla (hoy `get_manifest` sirve bytes crudos)

## 4. Implementación (green) — get_manifest

- [x] 4.1 En `vortex-core/crates/api/src/static_files.rs`, `get_manifest` lee el archivo como texto y sustituye `{{VERSION}}` por `state.version()` (mismo placeholder que `get_index`); MIME y `no-cache` no cambian — pasa 3.1; `manifest_sends_no_cache` sigue verde

## 5. Refactor y cierre

- [x] 5.1 `./scripts/check-vectra-web.sh` verde (suite previa + manifest happy/unhappy + harness); actualizar el comentario del script; no hay `serviceWorker.register` ni `vectra-web/sw.js`
- [x] 5.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes (§5)
- [x] 5.3 `openspec validate T-01-13` verde
