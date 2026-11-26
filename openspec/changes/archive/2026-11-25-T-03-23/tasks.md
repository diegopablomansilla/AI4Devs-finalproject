# Tasks

## 1. Tests (red) — search.test.mjs

- [x] 1.1 Extender el fake de `request` en `scripts/fixtures/vectra-web/search.test.mjs` para ramificar por path (`/search/by-code/` → `{ items }`, `/search?` → `SearchResponse`); actualizar `search_cancels_previous_and_paints_last_only`, `search_non_text_mode_does_not_call_search`, `search_empty_query_clears_without_fetch` y `search_admin_view_does_not_call_search` para asertar que no queda `by-code` colgado cuando no corresponde — los tests previos siguen en rojo o verde según el fake, sin implementación nueva
- [x] 1.2 Agregar tests happy `search_code_like_query_calls_by_code_in_parallel`, `search_shared_oem_pins_both_exact_matches_first` (Listo cuando: `OEM-PER-8840` → tarjetas `PER-ACERO-01` y `PER-ACERO-02` primero + texto «Código exacto», sin duplicar) y `search_arrow_down_selects_first_exact_match` — `node --test scripts/fixtures/vectra-web/search.test.mjs` falla en rojo
- [x] 1.3 Agregar tests unhappy `search_phrase_with_spaces_does_not_call_by_code`, `search_code_longer_than_64_does_not_call_by_code`, `search_by_code_empty_items_does_not_pin`, `search_by_code_error_still_paints_search_hits`, `search_search_error_still_paints_exact_matches` — los cinco fallan en rojo (heurística, pin vacío, fallos independientes)

## 2. Implementación (green) — search.js

- [x] 2.1 En `vectra-web/js/search.js`: `looksLikeCode` (`/^[A-Za-z0-9-]{1,64}$/`); si aplica, `GET /api/v1/search/by-code/${encodeURIComponent(q)}` en paralelo con `/search` con el mismo `AbortController` + `Promise.allSettled` — pasan `search_code_like_query_calls_by_code_in_parallel`, `search_phrase_with_spaces_does_not_call_by_code`, `search_code_longer_than_64_does_not_call_by_code`; abort de ambas patas en `search_cancels_previous_and_paints_last_only` / `search_aborted_response_does_not_paint`
- [x] 2.2 Pintar grupo «Código exacto» (nodo estático, no `vx-chip`) + tarjetas `ProductSummary` arriba, dedup de `hits` por `id`, `getItems` solo tarjetas; fallos independientes según spec — pasan 1.2 y 1.3; MAY `base.css` solo con tokens `--vx-*` existentes

## 3. Refactor y cierre

- [x] 3.1 Actualizar el comentario de `scripts/check-vectra-web.sh` (UI búsqueda por código T-03-23); `./scripts/check-vectra-web.sh` verde (suite previa + by-code happy/unhappy); `vx-omnibox.js` sigue sin importar `api.js`; sin lector de barras; sin cambios a `docs/openapi.yaml` ni crates Rust
- [x] 3.2 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` verdes ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) §5)
- [x] 3.3 `openspec validate T-03-23` verde