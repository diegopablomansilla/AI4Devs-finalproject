# Proposal

## Ticket padre

- **Ticket:** [T-02-41](../../../docs/7-tickets.md) — Dataset de desarrollo (~200 SKUs)
- **Change:** `T-02-41`
- **HU:** [HU-02-11](../../../docs/6-historias.md#hu-02-11--dataset-semilla) (MVP parcial: dataset de desarrollo; catálogo piloto en v2)
- **Depende de:** T-02-40 (en `develop`: formato JSONL, validador `catalog::validate_dir`, ejemplo mínimo en `seed/`)
- **Hace:** dataset de ferretería y repuestos con ~200 productos, atributos (medida, largo, grado, material, rosca), códigos OEM compartidos, errores tipográficos útiles para pruebas ("tornillo", "arandela", "bujía"), relaciones, sinónimos del rubro y 2–3 notas técnicas; imágenes livianas de ejemplo.
- **No hace:** reemplazar al catálogo del piloto (T-02-46).
- **Listo cuando:** el validador de T-02-40 no reporta errores.

## Why

T-02-40 dejó el contrato JSONL y un perno de muestra. Sin ~200 SKUs realistas, T-03-14 no tiene datos fijos de ranking/typos, T-03-22 no puede probar OEM compartido, T-04-11/T-09-09 no tienen catálogo de demo, y HU-02-11 no cubre el dataset de desarrollo de [PA-05](../../../docs/1-descripcion-general-del-producto.md#113-decisiones-adoptadas). Ahora, porque el validador ya existe y este ticket es el siguiente de la ruta MVP §7.5.1.

## What Changes

- Expandir `seed/` del ejemplo mínimo a un dataset de desarrollo de ferretería y repuestos: **~200 productos** (rango 180–220) que pasa `catalog::validate_dir` sin issues (**Listo cuando**)
- Conservar el SKU `PER-M8X40-ZN` (ancla de T-02-40) y ampliar maestros: categorías, marcas, `atributo_def` (`medida`, `largo`, `grado`, **`material`**, **`rosca`**)
- Códigos OEM compartidos (al menos un `tipo: "oem"` en dos SKUs distintos) y nombres útiles para typos/ranking de T-03-14 (`tornillo` / `tornilo`, `arandela`, `bujía`; "Tornillo autoperforante 10 x 1"; "Perno de Acero" vs "Arandela para Perno"; stocks `40` y `3`)
- Llenar archivos opcionales del formato: `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` (2–3 notas) e `imagenes/<sku>/` con PNG/JPEG/WebP livianos de ejemplo (no una foto por SKU)
- Tests TDD en `catalog`: happy (conteo, contenido, validador vacío) + unhappy (copia del dataset + registro inválido / relación o carpeta de imagen con SKU desconocido)
- MUST NOT importar, MUST NOT cambiar el stub `vortex seed`, MUST NOT tocar el validador salvo tests de contenido

## Non-goals

- Importar maestros — **T-02-42**
- Importar productos y stock inicial — **T-02-43**
- Importar imágenes — **T-02-44** (v2)
- Importar relaciones — **T-04-11**
- Importar sinónimos — **T-03-29** (v2)
- Importar notas técnicas — **T-05-29** (v2)
- Catálogo del comercio piloto y conversor — **T-02-46** (v2) (**No hace** del ticket)
- Generador sintético 10 k SKUs — **T-09-06** (v2)
- Tests de ranking Meilisearch — **T-03-14** (consume este dataset; no se implementan aquí)
- Setup Playwright e2e — **T-09-09**
- HTTP, `openapi.yaml`, UI, CLI `vortex seed`, SurrealDB
- Cambiar el esquema JSONL o la API de `validate_dir` (contrato de T-02-40)

## Capabilities

### New Capabilities

- *(ninguna)* — el formato y el validador ya viven en `seed-dataset`

### Modified Capabilities

- `seed-dataset`: `seed/` deja de ser el ejemplo mínimo de T-02-40 y MUST ser el dataset de desarrollo (~200 SKUs, atributos del rubro, OEM compartido, fixtures de typo/ranking, relaciones, sinónimos, 2–3 notas, imágenes livianas). El validador MUST seguir reportando lista vacía sobre `seed/` del repo.

## Impact

- **Repo:** `seed/` (JSONL maestros + opcionales + `imagenes/<sku>/` + README actualizado), `vortex-core/crates/catalog/src/seed.rs` (tests de contenido; sin cambio de `validate_dir` salvo que un test lo requiera)
- **APIs / OpenAPI:** ninguna
- **CLI:** `vortex seed` permanece stub (T-00-07 / T-02-42)
- **Verificación:** `cargo test -p catalog`, [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`, `openspec validate T-02-41`
- **Ticket:** cierra T-02-41 (Listo cuando: validador sin errores sobre el dataset de desarrollo)
- **Ámbito:** ~2 h si el catálogo se arma por familias combinatorias (no 200 fichas a mano) y las imágenes son placeholders chicos. No se divide el ticket.
