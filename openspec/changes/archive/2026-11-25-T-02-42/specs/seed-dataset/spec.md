# Spec Delta

Ticket padre: T-02-42. HU: HU-02-11.

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

Cada JSONL MUST ser UTF-8, un objeto JSON por línea; las líneas en blanco MUST ignorarse. MUST NOT usar IDs ULID ni nombres de API en inglés (`name`, `brandId`, `categoryId`, `price` como clave). Los campos MUST usar nombres del modelo en español ([4.4](../../../../../docs/4-modelo.md#44-catálogo)). `seed/` MUST ser el dataset de desarrollo de T-02-41 (ferretería y repuestos, ~200 SKUs), no el catálogo del comercio piloto.

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
