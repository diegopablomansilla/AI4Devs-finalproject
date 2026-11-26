# Spec Delta

## Purpose

Define el formato JSONL del dataset semilla en `seed/` (claves naturales: slug, clave, SKU) y un validador de directorio que informa archivo, línea y campo, sin importar ni persistir. Ticket padre: T-02-40. HU: HU-02-11.

## ADDED Requirements

### Requirement: Árbol seed/ con JSONL y README

El repositorio MUST contener el directorio `seed/` en la raíz con:

- `README.md` que documenta el esquema de cada archivo (campos, tipos, referencias por slug / clave / SKU y el ejemplo de error de precio)
- `categorias.jsonl`
- `marcas.jsonl`
- `atributos.jsonl`
- `productos.jsonl`

MAY existir `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` y el directorio `imagenes/<sku>/` (formato de la importación futura; HU-02-11 MVP no exige filas ni binarios ahí). Cada JSONL MUST ser UTF-8, un objeto JSON por línea; las líneas en blanco MUST ignorarse. MUST NOT usar IDs ULID ni nombres de API en inglés (`name`, `brandId`, `categoryId`, `price` como clave). Los campos MUST usar nombres del modelo en español ([4.4](../../../../../docs/4-modelo.md#44-catálogo)). Este ticket MUST dejar un ejemplo **mínimo** válido (no el dataset de ~200 SKUs de T-02-41). MUST NOT importar.

#### Scenario: seed/ tiene README y JSONL maestros (happy)

- **WHEN** se inspecciona `seed/` en la raíz del repositorio
- **THEN** existen `README.md`, `categorias.jsonl`, `marcas.jsonl`, `atributos.jsonl` y `productos.jsonl`
- **AND** `README.md` describe el esquema y las referencias por slug, clave y SKU

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

MUST NOT abrir SurrealDB, MUST NOT escribir el ledger, MUST NOT llamar casos de uso de importación, MUST NOT arrancar HTTP ni cambiar el stub `vortex seed`. Un directorio válido MUST devolver lista vacía.

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

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `seed_valid_minimal_directory_has_no_issues` | `catalog` (`src/seed.rs` o `tests/seed.rs`) | unitario happy |
| `seed_repo_seed_dir_has_no_issues` | `catalog` | unitario happy |
| `seed_rejects_non_numeric_precio_with_file_line_field` | `catalog` | unitario unhappy (Listo cuando) |
| `seed_rejects_malformed_jsonl_line` | `catalog` | unitario unhappy |
| `seed_rejects_unknown_categoria_slug` | `catalog` | unitario unhappy |
| `seed_rejects_duplicate_sku` | `catalog` | unitario unhappy |
| `seed_rejects_unknown_atributo_clave` | `catalog` | unitario unhappy |
| `catalog con serde y serde_json` | `scripts/fixtures/workspace/run-script-tests.sh` | happy (allowlist) |
| `catalog con clap` / `catalog con surrealdb` | `scripts/fixtures/workspace/run-script-tests.sh` | unhappy (allowlist) |
