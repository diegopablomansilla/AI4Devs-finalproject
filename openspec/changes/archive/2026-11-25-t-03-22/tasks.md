# Tasks

## 1. Tests (red) — domain y storage

- [x] 1.1 Extender dummy `ProductRepo` en `domain` con `find_by_code`; test `product_repo_trait_exposes_find_by_code` — compila/falla sin impl en storage
- [x] 1.2 Tests integración en `storage/tests/producto.rs`: `product_repo_find_by_code_shared_oem_returns_both`, `product_repo_find_by_code_matches_sku`, `product_repo_find_by_code_matches_fabricante`, `product_repo_find_by_code_does_not_prefix_match`, `product_repo_find_by_code_excludes_inactive`, `product_repo_find_by_code_unknown_returns_empty` — `cargo test -p storage --test producto find_by_code` falla

## 2. Implementación (green) — domain + storage

- [x] 2.1 `ProductRepo::find_by_code` en `domain` + `ProductoRepo` SurrealDB — tests §1.2 en verde
- [x] 2.2 `ProductService::find_by_code` + `ProductGateway::find_by_code` + test `product_service_find_by_code_delegates` — `cargo test -p catalog product_service_find_by_code` verde

## 3. Tests (red) — API

- [x] 3.1 Tests en `api/tests/search.rs` (happy + unhappy de la spec) con `FakeCatalog` — `cargo test -p api --test search by_code` falla

## 4. Implementación (green) — API y OpenAPI

- [x] 4.1 Ruta `GET /api/v1/search/by-code/{code}`, handler, validación `code`, mapper `ProductSummary`; `openapi.yaml` `400`/`503` — tests §3.1 verde; `assert_conforms`
- [x] 4.2 Actualizar `FakeCatalog` / fakes que implementan `ProductGateway`

## 5. Cierre

- [x] 5.1 `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings`, `cargo test --workspace` en verde
- [x] 5.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` en verde
- [x] 5.3 `openspec validate t-03-22` en verde
- [x] 5.4 Marcar tasks completadas; **sin git commit** (esperar archive)