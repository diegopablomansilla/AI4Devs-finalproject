# seed-dataset Specification

## Purpose

Define el formato JSONL del dataset semilla en `seed/` (claves naturales: slug, clave, SKU), un validador de directorio que informa archivo, línea y campo sin importar, y el dataset de desarrollo de ferretería y repuestos (~200 SKUs). Tickets: T-02-40, T-02-41, T-02-42. HU: HU-02-11.

## Requirements

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

Cada JSONL MUST ser UTF-8, un objeto JSON por línea; las líneas en blanco MUST ignorarse. MUST NOT usar IDs ULID ni nombres de API en inglés (`name`, `brandId`, `categoryId`, `price` como clave). Los campos MUST usar nombres del modelo en español ([4.4](../../../docs/4-modelo.md#44-catálogo)). `seed/` MUST ser el dataset de desarrollo de T-02-41 (ferretería y repuestos, ~200 SKUs), no el catálogo del comercio piloto.

`README.md` MUST documentar que `vortex seed [DIR]` importa **solo maestros** (categorías, marcas, atributos) con `--config` o `VORTEX_CONFIG` y el servidor detenido, y que productos, imágenes, relaciones, sinónimos y notas quedan para tickets posteriores. El árbol `seed/` MUST NOT ejecutar importación por sí mismo.

Un directorio semilla **arbitrario** (fixtures de test) MAY omitir `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/`; si existen, MUST validarse (esquema de registros JSONL). El validador MUST NOT exigir esos archivos para considerar válido un directorio mínimo.

#### Scenario: seed/ tiene README y JSONL maestros (happy)

- **WHEN** se inspecciona `seed/` en la raíz del repositorio
- **THEN** existen `README.md`, `categorias.jsonl`, `marcas.jsonl`, `atributos.jsonl` y `productos.jsonl`
- **AND** existen `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` y el directorio `imagenes/`
- **AND** `README.md` describe el esquema y las referencias por slug, clave y SKU
- **AND** `README.md` no afirma que el dataset de ~200 SKUs «llegará» en T-02-41
- **AND** `README.md` menciona `vortex seed` y que la importación actual cubre categorías, marcas y atributos

#### Scenario: el ejemplo mínimo es un perno válido (happy)

- **WHEN** se leen las líneas no vacías de los JSONL maestros en `seed/`
- **THEN** hay al menos una categoría, una marca, una definición de atributo y un producto con SKU `"PER-M8X40-ZN"`
- **AND** el producto referencia la categoría por slug, la marca por slug y los atributos por clave

### Requirement: Esquema de registros JSONL

Cada objeto MUST cumplir:

- **categorias:** `slug` (patrón de dominio `Slug`), `nombre` no vacío; `padre` ausente, JSON `null` o slug de otra categoría del mismo archivo. `slug` MUST ser único en el archivo.
- **marcas:** `slug` único, `nombre` no vacío.
- **atributos:** `clave` única (`snake_case` de dominio), `etiqueta`, `tipo` ∈ {`texto`, `numero`, `booleano`, `enum`}; `unidad` opcional; `opciones` obligatorio (array de string no vacío) si `tipo` es `enum`; `facetable` y `buscable` booleanos opcionales; `orden` entero opcional.
- **productos:** `sku` único (1..=64), `nombre`, `tipo`, `categoria` (slug), `precio` (string decimal de dominio, ≥ 0). Opcionales: `descripcion`, `marca` (slug o `null`), `atributos` (array de `{ clave, valor }`), `codigos` (`tipo` ∈ {`fabricante`, `oem`, `barras`}, `valor`), `unidad_medida` ∈ {`unidad`, `par`, `caja`, `metro`, `kilogramo`, `litro`} (default `unidad`), `ubicacion` (`pasillo` / `estante` / `posicion` opcionales), `stock_inicial` (string decimal ≥ 0, default `"0"`).
- **relaciones** (si el archivo existe): `origen` y `destino` (SKU), `tipo` ∈ {`similar_a`, `sustituible_por`, `complementa`}; opcionales `prioridad` (entero ≥ 1), `nota`, `bidireccional` (bool).
- **sinonimos** (si existe): `tipo` ∈ {`mutuo`, `unidireccional`}, `equivalentes` array de string con al menos un elemento; `termino` obligatorio si `tipo` es `unidireccional`.
- **notas** (si existe): `titulo`, `contenido`; `productos` opcional (array de SKU).
- **imagenes:** cada subdirectorio MUST llamarse como un `sku` de `productos.jsonl`.

MUST NOT exigir que `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` o `imagenes/` existan para considerar válido un directorio mínimo (MVP: sin imágenes ni sinónimos en la importación). Si existen, MUST validarse.

#### Scenario: producto referencia slug y clave (happy)

- **WHEN** un producto tiene `categoria` `"pernos"`, `marca` `"fixser"` y `atributos` `[{ "clave": "medida", "valor": "M8" }]` y esos slug/clave existen en los JSONL maestros
- **THEN** el registro es válido respecto de este esquema
- **AND** no hay campo `categoryId` ni `brandId`

#### Scenario: sinonimos y notas ausentes no invalidan el directorio (happy, MVP)

- **WHEN** un directorio tiene los cuatro JSONL maestros válidos y no tiene `sinonimos.jsonl`, `notas.jsonl`, `relaciones.jsonl` ni `imagenes/`
- **THEN** el validador no reporta issues por archivos ausentes opcionales

### Requirement: Validador informa archivo, línea y campo

El crate `catalog` MUST exponer una función de biblioteca que recibe la ruta de un directorio semilla y MUST devolver una lista de issues. Cada issue MUST incluir:

- `archivo`: path relativo al directorio (`productos.jsonl`, `categorias.jsonl`, …)
- `linea`: número de línea 1-based del JSONL (0 si el issue es de archivo ausente)
- `campo`: nombre del campo del objeto (p. ej. `precio`, `categoria`, `sku`, `atributos[0].clave`); cadena vacía si la línea no es JSON válido
- `mensaje`: texto que describe el fallo

MUST NOT abrir SurrealDB, MUST NOT escribir el ledger, MUST NOT llamar casos de uso de importación y MUST NOT arrancar HTTP. Un directorio válido MUST devolver lista vacía. `vortex seed` MAY invocarse aparte (capability `vortex-seed`); este validador MUST seguir siendo una función pura de lectura.

`precio` MUST parsearse como `Decimal` de dominio (string en el JSON). Un valor no numérico MUST producir al menos un issue con `archivo` `productos.jsonl`, `linea` igual a la línea física del registro y `campo` `precio` (Listo cuando de T-02-40; escenario «Registro inválido» de HU-02-11).

JSON malformado en una línea MUST producir un issue de esa línea. Un `sku` duplicado MUST informar campo `sku`. Una `categoria` o `marca` cuyo slug no está en el maestro MUST informar el campo `categoria` o `marca`. Una `clave` de atributo desconocida MUST informar `atributos[n].clave`. MUST NOT persistir ningún registro.

#### Scenario: directorio mínimo válido sin issues (happy)

- **WHEN** se valida un directorio con README, JSONL maestros coherentes (categoría, marca, atributo y producto `PER-M8X40-ZN` con `precio` `"120.50"`) y sin archivos opcionales
- **THEN** la lista de issues está vacía

#### Scenario: seed/ del repositorio no reporta errores (happy)

- **WHEN** se valida el directorio `seed/` de la raíz del repositorio
- **THEN** la lista de issues está vacía

#### Scenario: precio no numérico informa línea y campo (unhappy, Listo cuando, HU-02-11)

- **WHEN** `productos.jsonl` contiene en la línea 1 un objeto cuyo `precio` es `"barato"` (u otro string que no es decimal de dominio) y el resto del directorio es válido
- **THEN** el validador devuelve al menos un issue
- **AND** ese issue tiene `archivo` `productos.jsonl`
- **AND** `linea` es 1
- **AND** `campo` es `precio`
- **AND** no se persiste nada

#### Scenario: línea JSON malformada (unhappy)

- **WHEN** una línea no vacía de un JSONL no es un objeto JSON
- **THEN** hay un issue con ese `archivo` y esa `linea`
- **AND** `campo` es la cadena vacía

#### Scenario: slug de categoría inexistente (unhappy)

- **WHEN** un producto referencia `categoria` `"no-existe"` y no hay esa slug en `categorias.jsonl`
- **THEN** hay un issue con `archivo` `productos.jsonl` y `campo` `categoria`

#### Scenario: SKU duplicado (unhappy)

- **WHEN** dos líneas de `productos.jsonl` tienen el mismo `sku`
- **THEN** hay un issue con `archivo` `productos.jsonl` y `campo` `sku` en la línea de la ocurrencia duplicada

#### Scenario: clave de atributo desconocida (unhappy)

- **WHEN** un producto declara `atributos[0].clave` que no está en `atributos.jsonl`
- **THEN** hay un issue con `archivo` `productos.jsonl` y `campo` `atributos[0].clave`

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
| `seed_valid_minimal_directory_has_no_issues` | `catalog` (`src/seed.rs` o `tests/seed.rs`) | unitario happy |
| `seed_repo_seed_dir_has_no_issues` | `catalog` | unitario happy (Listo cuando T-02-41; regresion T-02-40) |
| `seed_rejects_non_numeric_precio_with_file_line_field` | `catalog` | unitario unhappy (Listo cuando T-02-40) |
| `seed_rejects_malformed_jsonl_line` | `catalog` | unitario unhappy |
| `seed_rejects_unknown_categoria_slug` | `catalog` | unitario unhappy |
| `seed_rejects_duplicate_sku` | `catalog` | unitario unhappy |
| `seed_rejects_unknown_atributo_clave` | `catalog` | unitario unhappy |
| `seed_dev_dataset_has_expected_sku_count` | `catalog` | unitario happy |
| `seed_dev_dataset_has_rubro_attributes_oem_and_typo_fixtures` | `catalog` | unitario happy |
| `seed_dev_dataset_has_relations_synonyms_notes_and_example_images` | `catalog` | unitario happy |
| `seed_dev_copy_rejects_non_numeric_precio` | `catalog` | unitario unhappy |
| `seed_dev_copy_rejects_unknown_relacion_sku` | `catalog` | unitario unhappy |
| `seed_dev_copy_rejects_unknown_imagen_sku` | `catalog` | unitario unhappy |
| `catalog con serde y serde_json` | `scripts/fixtures/workspace/run-script-tests.sh` | happy (allowlist) |
| `catalog con clap` / `catalog con surrealdb` | `scripts/fixtures/workspace/run-script-tests.sh` | unhappy (allowlist) |
