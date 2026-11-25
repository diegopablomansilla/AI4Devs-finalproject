# Spec Delta

Ticket padre: T-02-43. HU: HU-02-11.

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

`README.md` MUST documentar que `vortex seed [DIR]` importa **maestros y productos** (categorías, marcas, atributos y `productos.jsonl` con movimiento `inicial`) con `--config` o `VORTEX_CONFIG` y el servidor detenido, y que imágenes, relaciones, sinónimos y notas quedan para tickets posteriores. El árbol `seed/` MUST NOT ejecutar importación por sí mismo.

Un directorio semilla **arbitrario** (fixtures de test) MAY omitir `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/`; si existen, MUST validarse (esquema de registros JSONL). El validador MUST NOT exigir esos archivos para considerar válido un directorio mínimo.

#### Scenario: seed/ tiene README y JSONL maestros (happy)

- **WHEN** se inspecciona `seed/` en la raíz del repositorio
- **THEN** existen `README.md`, `categorias.jsonl`, `marcas.jsonl`, `atributos.jsonl` y `productos.jsonl`
- **AND** existen `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` y el directorio `imagenes/`
- **AND** `README.md` describe el esquema y las referencias por slug, clave y SKU
- **AND** `README.md` no afirma que el dataset de ~200 SKUs «llegará» en T-02-41
- **AND** `README.md` menciona `vortex seed` y que la importación actual cubre categorías, marcas, atributos y productos

#### Scenario: el ejemplo mínimo es un perno válido (happy)

- **WHEN** se leen las líneas no vacías de los JSONL maestros en `seed/`
- **THEN** hay al menos una categoría, una marca, una definición de atributo y un producto con SKU `"PER-M8X40-ZN"`
- **AND** el producto referencia la categoría por slug, la marca por slug y los atributos por clave
