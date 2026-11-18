# Tasks

## 1. Tests domain (red)

- [x] 1.1 Agregar test `attribute_usage_trait_exposes_valores` en `vortex-core/crates/domain/src/atributo_def.rs` — `cargo test -p domain attribute_usage_trait_exposes_valores` no compila (escenario «El puerto declara valores»)

## 2. Puerto domain (green)

- [x] 2.1 Añadir el trait `AttributeUsage` (`valores(id) -> Vec<ValorAtributo>`) y un dummy de test; reexportar desde `domain::lib` — `cargo test -p domain attribute_usage_trait_exposes_valores` sale 0; `./scripts/check-domain-deps.sh` sale 0
- [x] 2.2 Extraer `valor_compatible_con(tipo, opciones, valor) -> bool` (mismo match que `validate_valor_for_def`) y usarlo desde la validación de producto — `cargo test -p domain` sigue verde (escenario de compatibilidad / No hace: sin migración)

## 3. Tests storage (red)

- [x] 3.1 Agregar tests `attribute_usage_returns_medida_m8`, `attribute_usage_empty_when_unused`, `attribute_usage_includes_inactive_product`, `attribute_def_repo_update_does_not_apply_in_use` en `vortex-core/crates/storage/tests/` — `cargo test -p storage attribute_usage attribute_def_repo_update_does_not_apply` no compila o falla (Listo cuando: producto con "medida" = `M8`)

## 4. Storage AttributeUsage (green)

- [x] 4.1 Implementar `AttributeUsage` sobre `producto.atributos` (activos e inactivos; lista vacía si no hay coincidencias; sin outbox ni escritura) — `cargo test -p storage attribute_usage` sale 0
- [x] 4.2 Confirmar que `AtributoDefRepo::update` sigue cambiando `clave` aunque haya productos — `cargo test -p storage attribute_def_repo_update_does_not_apply_in_use` sale 0 (escenario «update no aplica occupancy»)

## 5. Tests catalog (red)

- [x] 5.1 Agregar tests `catalog_update_clave_in_use_is_attribute_in_use`, `catalog_update_tipo_incompatible_in_use_is_attribute_in_use`, `catalog_update_opciones_incompatible_in_use_is_attribute_in_use`, `catalog_update_etiqueta_in_use_succeeds`, `catalog_update_enum_to_texto_in_use_succeeds`, `catalog_deactivate_in_use_succeeds` en `vortex-core/crates/catalog/src/atributo_def.rs` — `cargo test -p catalog catalog_update_clave_in_use catalog_update_tipo_incompatible catalog_update_opciones_incompatible catalog_update_etiqueta_in_use catalog_update_enum_to_texto catalog_deactivate_in_use` no compila o falla (HU-02-03 / Listo cuando)
- [x] 5.2 Ajustar `catalog_update_clave_unused_succeeds` y el constructor del servicio a un `AttributeUsage` vacío — el test existente de clave no usada sigue en rojo hasta el green (regresión T-02-12)

## 6. Catalog occupancy (green)

- [x] 6.1 Hacer `AttributeDefService<R, U>` con `AttributeUsage`; en `update`, rechazar cambio de `clave` si hay valores y cambio de `tipo`/`opciones` si algún valor es incompatible; no consultar occupancy en `deactivate`/`reactivate` — los tests 5.x salen 0; `catalog_update_clave_unused_succeeds` y `catalog_deactivate_sets_inactive` siguen verdes
- [x] 6.2 Fake en memoria de `AttributeUsage` (mapa id → valores) y reexport; la fachada `AttributeDefGateway` no gana métodos de occupancy — `cargo test -p catalog attribute_def_gateway_is_object_safe` sale 0

## 7. Tests HTTP (red)

- [x] 7.1 En `vortex-core/crates/api/tests/attribute_definitions.rs` agregar `patch_attribute_def_key_in_use_returns_409`, `patch_attribute_def_type_incompatible_in_use_returns_409`, `patch_attribute_def_label_in_use_returns_200`, `patch_attribute_def_options_incompatible_in_use_returns_409`, `delete_attribute_def_in_use_returns_204` — `cargo test -p api --test attribute_definitions patch_attribute_def_key_in_use patch_attribute_def_type_incompatible patch_attribute_def_label_in_use patch_attribute_def_options_incompatible delete_attribute_def_in_use` no compila o falla (HU-02-03 / Listo cuando); los JSON de 409 llaman `assert_conforms`
- [x] 7.2 Extender el fake de `api/tests/support/fake_catalog.rs` para sembrar occupancy de "medida"/"grado" — `test_attribute_def_gateway()` sin productos sigue haciendo pasar `patch_attribute_def_changes_key_without_in_use`

## 8. Gateway HTTP y composition root (green)

- [x] 8.1 Cablear `AttributeDefService` con occupancy en el fake HTTP — los tests 7.1 salen 0; `cargo test -p api --test attribute_definitions` verde (incl. `patch_attribute_def_changes_key_without_in_use`)
- [x] 8.2 Inyectar el adaptador Surreal de `AttributeUsage` en `vortex-core/bin/vortex/src/serve.rs` — `cargo test -p vortex vortex_serve_get_attribute_definitions_returns_items` pasa (404/200 ≠ 503)

## 9. Cierre (regresión)

- [x] 9.1 En `vortex-core/`: `cargo fmt --all -- --check`, `cargo clippy --workspace -- -D warnings` y `cargo test --workspace` salen 0 ([docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5)
- [x] 9.2 `./scripts/check-domain-deps.sh` y `./scripts/check-workspace.sh` salen 0
- [x] 9.3 Confirmar non-goals: `rg` sin reescritura de `atributos[*].valor` ni `409` en el handler `DELETE` de attribute-definitions; `docs/openapi.yaml` sin path nuevo ni `409` en `DELETE`
- [x] 9.4 `openspec validate T-02-23` sale 0