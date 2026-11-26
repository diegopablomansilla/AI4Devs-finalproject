# Spec Delta

Ticket padre: T-02-41. HU: HU-02-11.

## MODIFIED Requirements

### Requirement: Árbol seed/ con JSONL y README

El repositorio MUST contener el directorio `seed/` en la raíz con:

- `README.md` que documenta el esquema de cada archivo (campos, tipos, referencias por slug / clave / SKU y el ejemplo de error de precio)
- `categorias.jsonl`
- `marcas.jsonl`
- `atributos.jsonl`
- `productos.jsonl`
- `relaciones.jsonl`
- `sinonimos.jsonl`
- `notas.jsonl`
- `imagenes/<sku>/` con al menos un binario de ejemplo

Cada JSONL MUST ser UTF-8, un objeto JSON por línea; las líneas en blanco MUST ignorarse. MUST NOT usar IDs ULID ni nombres de API en inglés (`name`, `brandId`, `categoryId`, `price` como clave). Los campos MUST usar nombres del modelo en español ([4.4](../../../../../docs/4-modelo.md#44-catálogo)). `seed/` MUST ser el dataset de desarrollo de T-02-41 (ferretería y repuestos, ~200 SKUs), no el catálogo del comercio piloto. MUST NOT importar.

Un directorio semilla **arbitrario** (fixtures de test) MAY omitir `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/`; si existen, MUST validarse (esquema de registros JSONL). El validador MUST NOT exigir esos archivos para considerar válido un directorio mínimo.

#### Scenario: seed/ tiene README y JSONL maestros (happy)

- **WHEN** se inspecciona `seed/` en la raíz del repositorio
- **THEN** existen `README.md`, `categorias.jsonl`, `marcas.jsonl`, `atributos.jsonl` y `productos.jsonl`
- **AND** existen `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` y el directorio `imagenes/`
- **AND** `README.md` describe el esquema y las referencias por slug, clave y SKU
- **AND** `README.md` no afirma que el dataset de ~200 SKUs «llegará» en T-02-41

#### Scenario: el ejemplo mínimo es un perno válido (happy)

- **WHEN** se leen las líneas no vacías de los JSONL maestros en `seed/`
- **THEN** hay al menos una categoría, una marca, una definición de atributo y un producto con SKU `"PER-M8X40-ZN"`
- **AND** el producto referencia la categoría por slug, la marca por slug y los atributos por clave

## ADDED Requirements

### Requirement: Dataset de desarrollo de ferretería y repuestos (~200 SKUs)

El `productos.jsonl` del repositorio MUST contener entre **180 y 220** SKUs únicos (inclusive), rubro ferretería y repuestos. MUST incluir definiciones de atributo con claves `medida`, `largo`, `grado`, `material` y `rosca`, y MUST haber productos que usen cada una de esas claves. MUST existir al menos un código `tipo` `"oem"` cuyo `valor` aparezca en **dos** productos distintos. MUST haber productos cuyo `nombre` (case-insensitive) contenga `"tornillo"`, `"arandela"` y `"bujía"` (o `"bujia"`). MUST incluir los fixtures de T-03-14:

- un producto con `nombre` exacto `"Tornillo autoperforante 10 x 1"`
- un producto con `nombre` exacto `"Perno de Acero"`
- un producto con `nombre` exacto `"Arandela para Perno"`
- al menos un producto con `stock_inicial` `"40"` y otro con `stock_inicial` `"3"`

`relaciones.jsonl` MUST tener al menos una arista de cada `tipo` ∈ {`similar_a`, `sustituible_por`, `complementa`} con `origen` y `destino` presentes en `productos.jsonl`. `sinonimos.jsonl` MUST tener al menos un registro `mutuo` y uno `unidireccional` del rubro. `notas.jsonl` MUST tener 2 o 3 notas (`titulo` y `contenido` no vacíos). `imagenes/` MUST contener al menos un subdirectorio cuyo nombre es un SKU de `productos.jsonl` y al menos un archivo `image/jpeg`, `image/png` o `image/webp` liviano (MUST NOT exigir una imagen por cada SKU). MUST NOT presentarse como el catálogo del piloto (T-02-46).

#### Scenario: conteo de SKUs de desarrollo (happy)

- **WHEN** se cuentan los `sku` únicos de las líneas no vacías de `seed/productos.jsonl`
- **THEN** el conteo está entre 180 y 220 inclusive

#### Scenario: atributos del rubro presentes (happy)

- **WHEN** se leen `seed/atributos.jsonl` y los `atributos` de `seed/productos.jsonl`
- **THEN** existen definiciones con claves `medida`, `largo`, `grado`, `material` y `rosca`
- **AND** al menos un producto usa cada una de esas claves

#### Scenario: OEM compartido entre dos piezas (happy)

- **WHEN** se leen los `codigos` de `seed/productos.jsonl`
- **THEN** hay un `valor` con `tipo` `"oem"` que aparece en al menos dos SKUs distintos

#### Scenario: fixtures de typo y ranking (happy)

- **WHEN** se leen los `nombre` y `stock_inicial` de `seed/productos.jsonl`
- **THEN** existen productos cuyo nombre contiene `"tornillo"`, `"arandela"` y `"bujía"` o `"bujia"` (sin distinguir mayúsculas)
- **AND** existen los nombres exactos `"Tornillo autoperforante 10 x 1"`, `"Perno de Acero"` y `"Arandela para Perno"`
- **AND** hay un `stock_inicial` `"40"` y un `stock_inicial` `"3"`

#### Scenario: relaciones, sinónimos y notas del dataset (happy)

- **WHEN** se leen `seed/relaciones.jsonl`, `seed/sinonimos.jsonl` y `seed/notas.jsonl`
- **THEN** hay al menos una relación `similar_a`, una `sustituible_por` y una `complementa`
- **AND** hay al menos un sinónimo `mutuo` y uno `unidireccional`
- **AND** hay 2 o 3 notas técnicas con `titulo` y `contenido`

#### Scenario: imágenes livianas de ejemplo (happy)

- **WHEN** se inspecciona `seed/imagenes/`
- **THEN** hay al menos un subdirectorio cuyo nombre es un SKU de `productos.jsonl`
- **AND** ese subdirectorio contiene al menos un archivo con extensión `.jpg`, `.jpeg`, `.png` o `.webp`

### Requirement: Copia inválida del dataset se rechaza (unhappy)

Validar el `seed/` del repositorio MUST devolver lista vacía (**Listo cuando** de T-02-41). Si se copia el árbol a un directorio temporal y se agrega un producto con `precio` no numérico, o una relación con SKU desconocido, o un subdirectorio `imagenes/<sku>/` cuyo nombre no es un SKU del archivo, el validador MUST reportar al menos un issue con `archivo`/`linea`/`campo` coherentes con T-02-40. MUST NOT persistir ni modificar el `seed/` original.

#### Scenario: seed/ del repositorio no reporta errores (happy, Listo cuando)

- **WHEN** se valida el directorio `seed/` de la raíz del repositorio
- **THEN** la lista de issues está vacía

#### Scenario: producto con precio no numérico en una copia (unhappy)

- **WHEN** se copia `seed/` a un directorio temporal y se agrega a `productos.jsonl` un objeto cuyo `precio` no es decimal de dominio
- **THEN** el validador devuelve al menos un issue con `archivo` `productos.jsonl` y `campo` `precio`
- **AND** validar el `seed/` original sigue devolviendo lista vacía

#### Scenario: relación con SKU desconocido (unhappy)

- **WHEN** se copia `seed/` a un directorio temporal y se agrega a `relaciones.jsonl` una arista cuyo `destino` no está en `productos.jsonl`
- **THEN** hay un issue con `archivo` `relaciones.jsonl` y `campo` `destino`

#### Scenario: carpeta de imagen con SKU desconocido (unhappy)

- **WHEN** se copia `seed/` a un directorio temporal y se crea `imagenes/SKU-INEXISTENTE/`
- **THEN** hay un issue con `archivo` `imagenes` cuyo `campo` es `SKU-INEXISTENTE`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `seed_repo_seed_dir_has_no_issues` | `catalog` (`src/seed.rs`) | unitario happy (Listo cuando; ya existe, MUST seguir verde) |
| `seed_dev_dataset_has_expected_sku_count` | `catalog` | unitario happy |
| `seed_dev_dataset_has_rubro_attributes_oem_and_typo_fixtures` | `catalog` | unitario happy |
| `seed_dev_dataset_has_relations_synonyms_notes_and_example_images` | `catalog` | unitario happy |
| `seed_dev_copy_rejects_non_numeric_precio` | `catalog` | unitario unhappy |
| `seed_dev_copy_rejects_unknown_relacion_sku` | `catalog` | unitario unhappy |
| `seed_dev_copy_rejects_unknown_imagen_sku` | `catalog` | unitario unhappy |
| `seed_rejects_non_numeric_precio_with_file_line_field` (T-02-40) | `catalog` | unitario unhappy (regresión; MUST seguir verde) |
