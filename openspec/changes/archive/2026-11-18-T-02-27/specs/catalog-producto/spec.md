# Spec Delta

Ticket padre: T-02-27. Change: T-02-27. HU: HU-02-10.

## ADDED Requirements

### Requirement: Listar productos admin

El crate `catalog` MUST exponer un caso de uso `list` que delega en `ProductRepo::list` y MUST NOT abrir SurrealDB ni mapear a HTTP. MUST aceptar `FiltroProducto`, `after` y `take` y devolver `Vec<ResumenProducto>`. MUST NOT validar `cursor`/`limit` HTTP (Gateway / helper T-00-21). MUST NOT exigir que categoría o marca existan o estén activas (un filtro que no matchea MUST ser lista vacía). MUST NOT buscar por texto. MUST NOT reindexar (T-03-07). MUST NOT hidratar imágenes. Los errores del puerto MUST propagarse sin remapeo.

#### Scenario: Filtro por rama incluye descendientes (happy, HU-02-10, Listo cuando)

- **WHEN** existe la categoría `"Fijaciones"` con hija `"Pernos"`
- **AND** hay un producto en `"Fijaciones"` y otro en `"Pernos"`, ambos activos, y un tercero en otra rama
- **AND** el caso de uso lista con `categoria` = id de `"Fijaciones"`, `activo` verdadero y `marca` ausente
- **THEN** el resultado incluye los productos de `"Fijaciones"` y de `"Pernos"`
- **AND** no incluye el producto de la otra rama

#### Scenario: Filtro por inactivos (happy, HU-02-10, Listo cuando)

- **WHEN** existe un producto activo y otro dado de baja
- **AND** el caso de uso lista con `activo` falso
- **THEN** el resultado incluye solo el producto inactivo

#### Scenario: Filtro por marca (happy)

- **WHEN** existen productos de marca `"Fixser"` y de otra marca, todos activos
- **AND** el caso de uso lista con `marca` = id de `"Fixser"` y `activo` verdadero
- **THEN** el resultado incluye solo los de `"Fixser"`

#### Scenario: Orden por id y after (happy)

- **WHEN** existen tres productos activos A, B, C ordenados por `id` ascendente
- **AND** el caso de uso lista con `take` 2 y `after` ausente
- **THEN** el resultado es A, B en ese orden
- **WHEN** lista de nuevo con `after` = id de B y `take` 2
- **THEN** el resultado es C
- **AND** no incluye A ni B

#### Scenario: Categoría inexistente es lista vacía (happy)

- **WHEN** el caso de uso lista con `categoria` igual a un `Id` que no existe
- **THEN** el resultado es una lista vacía
- **AND** no es `not_found`

#### Scenario: list no busca por texto (unhappy)

- **WHEN** se inspecciona el caso de uso `list`
- **THEN** no acepta `q` ni otro texto de búsqueda
- **AND** no llama a un índice de búsqueda

## MODIFIED Requirements

### Requirement: Tests en memoria de cada caso

Los tests del crate `catalog` MUST usar repositorios en memoria (sin SurrealDB) para producto, categoría, marca y definiciones de atributo. MUST cubrir crear, obtener, editar, dar de baja, reactivar y listar, incluido el camino no feliz (Listo cuando). El crate `catalog` MUST NOT depender de `storage` ni `surrealdb`.

| Test | Crate | Tipo |
|---|---|---|
| `catalog_create_perno_m8x40_sets_version_1` | `catalog` | unitario (memoria), happy, HU-02-04 / Listo cuando |
| `catalog_get_returns_ficha_with_marca_ruta_and_attributes` | `catalog` | unitario (memoria), happy |
| `catalog_update_price_increments_version` | `catalog` | unitario (memoria), happy, HU-02-05 |
| `catalog_update_partial_price_keeps_other_fields` | `catalog` | unitario (memoria), happy |
| `catalog_create_duplicate_sku_is_duplicate_sku` | `catalog` | unitario (memoria), unhappy, HU-02-04 |
| `catalog_create_cuarenta_on_number_attribute_is_invalid_input` | `catalog` | unitario (memoria), unhappy, HU-02-04 |
| `catalog_create_inactive_categoria_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_create_inactive_marca_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_create_unknown_categoria_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_get_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_update_stale_version_is_version_conflict` | `catalog` | unitario (memoria), unhappy, HU-02-05 / Listo cuando |
| `catalog_update_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_update_inactive_categoria_is_invalid_input` | `catalog` | unitario (memoria), unhappy |
| `catalog_deactivate_sets_inactive` | `catalog` | unitario (memoria), happy, HU-02-06 |
| `catalog_reactivate_sets_active` | `catalog` | unitario (memoria), happy, HU-02-06 / Listo cuando |
| `catalog_deactivate_idempotent_when_already_inactive` | `catalog` | unitario (memoria), happy |
| `catalog_get_inactive_product_returns_ficha` | `catalog` | unitario (memoria), happy |
| `catalog_deactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_reactivate_unknown_id_is_not_found` | `catalog` | unitario (memoria), unhappy |
| `catalog_list_branch_includes_descendants` | `catalog` | unitario (memoria), happy, HU-02-10 / Listo cuando |
| `catalog_list_inactive_only` | `catalog` | unitario (memoria), happy, HU-02-10 / Listo cuando |
| `catalog_list_filters_by_brand` | `catalog` | unitario (memoria), happy |
| `catalog_list_orders_by_id_and_after` | `catalog` | unitario (memoria), happy |
| `catalog_list_unknown_category_is_empty` | `catalog` | unitario (memoria), happy |

#### Scenario: Suite en memoria (happy)

- **WHEN** se ejecuta `cargo test -p catalog` filtrando los tests de la tabla
- **THEN** todos pasan
- **AND** ninguno abre SurrealDB

### Requirement: Fachada object-safe para el Gateway

El crate `catalog` MUST exponer una fachada object-safe (`Send + Sync + 'static`) de los casos de uso de producto (crear, obtener, editar, dar de baja, reactivar, listar) para que el Gateway la inyecte como `Arc<dyn …>`. El servicio de producto MUST implementar esa fachada. MUST NOT mapear a HTTP ni abrir SurrealDB.

#### Scenario: el servicio es inyectable en AppState (happy)

- **WHEN** se construye el servicio de producto con repositorios en memoria
- **THEN** se puede almacenar detrás de `Arc<dyn …>` compartido entre peticiones
- **AND** crear un producto válido vía la fachada termina sin error

#### Scenario: la fachada expone list (happy)

- **WHEN** se inspecciona la fachada object-safe de producto
- **THEN** expone `list`
- **AND** listar con un filtro vacío de categoría/marca y `activo` verdadero termina sin error
