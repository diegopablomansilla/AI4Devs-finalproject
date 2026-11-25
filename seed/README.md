# Dataset semilla — Vectra / Vortex Core

Formato compartido por `vortex seed` (importación) y por conversiones externas (p. ej. comercio piloto).
Los maestros se referencian por **slug** (categoría, marca) o **clave** (`atributo_def`); productos y relaciones por **SKU**.

## Archivos obligatorios (MVP)

| Archivo | Descripción |
|---------|-------------|
| `categorias.jsonl` | Árbol de categorías (`slug`, `nombre`, `padre` opcional) |
| `marcas.jsonl` | Marcas (`slug`, `nombre`) |
| `atributos.jsonl` | Definiciones (`clave` en `snake_case`, `etiqueta`, `tipo`, …) |
| `productos.jsonl` | Fichas de producto |

Cada línea es un objeto JSON UTF-8. Las líneas en blanco se ignoran. Los nombres de campo están en **español** alineados con [docs/4-modelo.md](../docs/4-modelo.md#44-catálogo) (no usar `name`, `brandId`, `categoryId`, etc.).

## Archivos opcionales (importación futura / v2)

| Archivo / ruta | Descripción |
|----------------|-------------|
| `relaciones.jsonl` | Aristas entre productos (`origen`, `destino` por SKU, `tipo`) |
| `sinonimos.jsonl` | Diccionario de sinónimos Meilisearch |
| `notas.jsonl` | Notas técnicas del rubro |
| `imagenes/<sku>/` | Binarios por SKU (nombre del directorio = `sku`) |

## Esquema resumido

### `categorias.jsonl`

- `slug` (requerido): patrón `^[a-z0-9]+(-[a-z0-9]+)*$`
- `nombre` (requerido)
- `padre` (opcional): slug de otra categoría en el mismo archivo, o `null`

### `marcas.jsonl`

- `slug`, `nombre` (requeridos)

### `atributos.jsonl`

- `clave` (requerido, `snake_case`)
- `etiqueta`, `tipo` (`texto` \| `numero` \| `booleano` \| `enum`)
- `opciones` (requerido si `tipo` = `enum`)
- `unidad`, `facetable`, `buscable`, `orden` (opcionales)

### `productos.jsonl`

- `sku`, `nombre`, `tipo`, `categoria` (slug), `precio` (requeridos)
- `precio` y `stock_inicial` son **strings decimales** (ej. `"120.50"`), no números JSON
- `marca` (slug o `null`), `descripcion`, `atributos` (`[{ "clave", "valor" }]`), `codigos`, `unidad_medida`, `ubicacion`, `stock_inicial` (opcionales)

### Importación (`vortex seed`)

Con el **servidor detenido** (no puede haber otro proceso con la base abierta):

```bash
vortex --config /ruta/vortex.toml seed [DIR]
```

- `DIR`: directorio del dataset; por defecto `seed` (relativo al directorio de trabajo).
- Requiere `--config` o la variable `VORTEX_CONFIG` (igual que `vortex serve`).
- En el MVP importa **solo maestros**: `categorias.jsonl`, `marcas.jsonl` y `atributos.jsonl` (idempotente por slug/clave).
- **No** importa `productos.jsonl`, imágenes, relaciones, sinónimos ni notas (tickets T-02-43 y posteriores).

Para comprobar el formato sin escribir en la base, use `catalog::validate_dir` en tests o herramientas de desarrollo.

### Validación

El validador (`catalog::validate_dir`) recorre un directorio y devuelve hallazgos con **archivo**, **línea** y **campo**, sin importar a SurrealDB.

Ejemplo de error (precio no numérico):

```json
{"sku":"X","nombre":"…","tipo":"Perno","categoria":"pernos","precio":"barato"}
```

→ `productos.jsonl`, línea 1, campo `precio`.

## Dataset de desarrollo en este repositorio

Este árbol es el **dataset de desarrollo** del MVP (~200 SKUs, ferretería y repuestos): maestros, productos, relaciones, sinónimos, notas técnicas e imágenes livianas de ejemplo. Conserva el perno ancla `PER-M8X40-ZN` de T-02-40. El catálogo del comercio piloto (2–5 k SKUs) se importará en v2 (T-02-46), mismo formato.