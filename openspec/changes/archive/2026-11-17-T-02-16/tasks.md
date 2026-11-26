# Tasks

## 1. Tests (red) — value objects y entidad

- [x] 1.1 Agregar tests unitarios `codigo_constructs_fabricante` y `unidad_medida_parses_unidad` en `vortex-core/crates/domain` — `cargo test -p domain -- codigo_constructs_fabricante unidad_medida_parses_unidad` no compila o falla (escenarios «Codigo fabricante válido», «UnidadMedida unidad»)
- [x] 1.2 Agregar tests unhappy `codigo_rejects_unknown_tipo` y `unidad_medida_rejects_english_or_unknown` — `cargo test -p domain -- codigo_rejects_unknown_tipo unidad_medida_rejects_english_or_unknown` falla (escenarios «tipo de codigo inválido», «UnidadMedida inválida»)
- [x] 1.3 Agregar test `producto_constructs_perno_m8x40_zn` — `cargo test -p domain -- producto_constructs_perno_m8x40_zn` no compila o falla (escenario «Construcción válida perno M8 x 40»; HU-02-04)
- [x] 1.4 Agregar tests unhappy `producto_rejects_blank_sku` y `producto_rejects_negative_precio` — `cargo test -p domain -- producto_rejects_blank_sku producto_rejects_negative_precio` falla (escenarios «sku vacío», «precio negativo»)

## 2. Tests (red) — atributos y puerto

- [x] 2.1 Agregar test `producto_accepts_matching_atributos` — `cargo test -p domain -- producto_accepts_matching_atributos` no compila o falla (escenario «Atributos coinciden con texto, numero y enum»)
- [x] 2.2 Agregar test unhappy `producto_rejects_cuarenta_on_number_attribute` — `cargo test -p domain -- producto_rejects_cuarenta_on_number_attribute` falla (escenario «cuarenta en atributo number»; **Listo cuando** / HU-02-04)
- [x] 2.3 Agregar test unhappy `producto_rejects_duplicate_atributo_def` — `cargo test -p domain -- producto_rejects_duplicate_atributo_def` falla (escenario «def repetido»; **Listo cuando**)
- [x] 2.4 Agregar test unhappy `producto_rejects_enum_value_not_in_opciones` — `cargo test -p domain -- producto_rejects_enum_value_not_in_opciones` falla (escenario «valor de enum fuera de opciones»)
- [x] 2.5 Agregar test `product_repo_trait_exposes_create_get` (dummy que implementa el trait) — `cargo test -p domain -- product_repo_trait_exposes_create_get` falla (escenario «El puerto declara create y get»)

## 3. Implementación (green) — módulo `producto`

- [x] 3.1 Crear `vortex-core/crates/domain/src/producto.rs` con `Sku`, `Codigo`/`TipoCodigo`, `Ubicacion`, `UnidadMedida` (wire español 4.4.4; `"unit"` rechazado) y reexportar desde `lib.rs` — `cargo test -p domain -- codigo_ unidad_medida_` sale 0 (escenarios de VOs)
- [x] 3.2 Implementar `ValorAtributo`, `AtributoValor`, `ProductoDraft` y `Producto::try_new` (campos 4.4.4, `Id<Producto>`, nombre 1..=200, tipo 1..=60, `precio`/`stock` ≥ 0) — `cargo test -p domain -- producto_constructs_perno_m8x40_zn producto_rejects_blank_sku producto_rejects_negative_precio` sale 0
- [x] 3.3 Implementar validación pura de atributos contra `&[AtributoDef]`: match de `TipoAtributo`, `opciones` de `enum`, `def` único; `"cuarenta"` → `attributes[n].value`; duplicado → `attributes[n].definitionId` — `cargo test -p domain -- producto_accepts_matching_atributos producto_rejects_cuarenta_on_number_attribute producto_rejects_duplicate_atributo_def producto_rejects_enum_value_not_in_opciones` sale 0
- [x] 3.4 Declarar `NewProducto` (incl. `stock_inicial`) y el trait `ProductRepo` (`create`, `get`; `impl Future` sin `async-trait` ni `tokio`) más dummy de test — `cargo test -p domain -- product_repo_trait_exposes_create_get` sale 0
- [x] 3.5 Confirmar que no hay persistencia ni `update`/`set_activo` en el puerto, y que `domain` no declara crates nuevas — `rg -n "surrealdb|CREATE producto|fn update|fn set_activo" vortex-core/crates/domain/src/producto.rs` sin matches de producto; `./scripts/check-workspace.sh` sale 0

## 4. Cierre (regresión y non-goals)

- [x] 4.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 4.2 `./scripts/check-domain-deps.sh` sale 0 (escenario «domain no persiste productos»: sin `surrealdb` ni `storage` en el árbol)
- [x] 4.3 `git diff` vacío en `vortex-core/crates/storage`, `vortex-core/crates/api`, `vortex-core/crates/catalog`, `docs/openapi.yaml` (non-goals: T-02-18, HTTP)
- [x] 4.4 `openspec validate T-02-16` sale 0