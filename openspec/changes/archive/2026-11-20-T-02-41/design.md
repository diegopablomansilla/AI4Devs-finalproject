# Design

## Context

En `develop`, T-02-40 ya dejó `seed/` con un perno de muestra (`PER-M8X40-ZN`) y `catalog::validate_dir` (issues `{ archivo, linea, campo, mensaje }`). Los archivos `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/` son opcionales para el validador; este ticket los llena en el árbol del repo. `vortex seed` sigue stub. Ver `proposal.md` para motivación y `specs/seed-dataset/spec.md` para el contrato.

Constraints: No hace = catálogo piloto (T-02-46); ~2 h; MUST NOT importar ni cambiar la API del validador; `openspec new change` exige kebab-case (`t-02-41` → rename `T-02-41`).

## Goals / Non-Goals

**Goals:**

- `seed/` del repo = dataset de desarrollo (180–220 SKUs) que `validate_dir` acepta sin issues
- Contenido útil para T-03-14, T-03-22 y demos H1: typos, ranking, OEM compartido, relaciones, sinónimos, 2–3 notas, pocas imágenes livianas
- Tests de contenido (happy) y copias mutadas (unhappy) en `catalog`, reusando `validate_dir`

**Non-Goals (diseño):**

- Binario `bench/seedgen` (T-09-06) o crate nueva
- Enganchar `vortex seed` / persistir en SurrealDB
- Foto por cada SKU, o fotos reales del piloto
- Cambiar allowlist de `catalog` o el parseo JSONL

## Decisions

### 1. JSONL committed; generación combinatoria solo en apply

**Elección:** la fuente de verdad es `seed/*.jsonl` versionado. En apply se arma el catálogo por **familias combinatorias** (tipo × medida × largo × material × marca) más un bloque de fixtures nominales (typos/ranking/OEM/`PER-M8X40-ZN`). No se agrega un generador al workspace.

**Por qué:** T-09-06 es el generador 10 k con semilla fija. Commitear JSONL hace que T-03-14 y e2e (T-09-09) dependan de datos estables, no de un script. Las familias cubren ~200 SKUs dentro de ~2 h.

**Alternativa:** script permanente en `seed/gen.py`. Se descartó: duplica T-09-06 y no es el contrato de importación. **Alternativa:** 200 fichas a mano. Se descartó: excede ~2 h y es frágil.

### 2. Conservar `PER-M8X40-ZN` y expandir maestros

**Elección:** no borrar el perno de T-02-40. Ampliar categorías (p. ej. `fijaciones` → `pernos` / `tornillos` / `tuercas` / `arandelas`; rama automotor `bujias`, `filtros`) y marcas (`fixser` + 2–3 del rubro). En `atributos.jsonl` agregar `material` y `rosca` (además de `medida`, `largo`, `grado`).

**Por qué:** el test `seed_repo_seed_dir_has_no_issues` y el escenario del perno M8 ya anclan el SKU. Los atributos del Hace del ticket tienen que existir como `atributo_def` y usarse.

**Alternativa:** reemplazar el ejemplo mínimo por otro SKU. Se descartó: rompe la ancla de T-02-40 sin ganancia.

### 3. Fixtures de T-03-14 y OEM con nombres exactos

**Elección:** productos con nombres **exactos** `"Tornillo autoperforante 10 x 1"`, `"Perno de Acero"`, `"Arandela para Perno"`; `stock_inicial` `"40"` y `"3"` en SKUs distintos; al menos un `codigos[].tipo = "oem"` repetido en dos SKUs. Nombres que contienen tornillo / arandela / bujía para typos.

**Por qué:** T-03-14 y T-03-22 ya describen esos casos. Si los strings no están fijados aquí, el dataset «válido» no sirve para ranking.

**Alternativa:** dejar nombres libres y que T-03-14 cree sus propios fixtures. Se descartó: el ticket Hace pide esos errores tipográficos en **este** dataset.

### 4. Archivos opcionales poblados; validador intacto

**Elección:** crear `relaciones.jsonl` (los tres `tipo`), `sinonimos.jsonl` (`mutuo` + `unidireccional`, p. ej. bulón/perno y tarugo→taco), `notas.jsonl` (2 o 3), `imagenes/<sku>/principal.png` en **pocos** SKUs (el perno de muestra + 1–2 fixtures). No cambiar `validate_dir`. Tests unhappy copian el árbol a `temp_dir` y mutan la copia.

**Por qué:** T-02-40 ya valida esos archivos **si existen**. Importar sigue en T-02-42/43/44, T-04-11, T-03-29, T-05-29. Una imagen por SKU inflaría git y el ticket dice «livianas de ejemplo».

**Alternativa:** exigir imagen en los ~200 SKUs. Se descartó: peso, tiempo, T-02-44. **Alternativa:** SVG. Se descartó: el modelo solo admite jpeg/png/webp ([4.4.5](../../docs/4-modelo.md#445-imagen_producto)).

### 5. PNG mínimo válido, no assets de diseño

**Elección:** PNG de pocas decenas de bytes (o JPEG/WebP chico) generado en apply; nombre `principal.png` para alinearse con T-02-44 (`principal.*` = imagen principal).

**Por qué:** el validador solo chequea que el directorio se llame como un SKU; T-02-44 importa el archivo. Un PNG real liviano cumple mime del modelo sin LFS.

**Alternativa:** `.gitkeep` vacío. Se descartó: no es una imagen y T-02-44 no tendría qué importar.

### 6. Tests de contenido en `catalog`, no un crate de fixtures

**Elección:** tests nuevos junto a `seed_repo_seed_dir_has_no_issues` que leen `CARGO_MANIFEST_DIR/../../../seed` y parsean JSONL (conteo, claves, OEM, nombres, archivos opcionales). Unhappy: copiar con `std::fs` a temp, mutar, `validate_dir`, borrar temp.

**Por qué:** el Listo cuando **es** el validador sobre `seed/`. Un segundo validador de contenido en `scripts/` duplicaría el camino y no corre en `cargo test -p catalog`.

**Alternativa:** snapshot golden del JSONL. Se descartó: ruido en review cada vez que se ajusta un precio.

## Risks / Trade-offs

- **[~200 líneas JSONL en git]** → Mitigación: familias compactas (una línea por SKU, sin pretty-print); README corto.
- **[Nombres combinatorios poco «reales»]** → Mitigación: fixtures nominales con copy de ferretería (tornillo autoperforante, bujía, arandela) más combinaciones tipo-medida-largo.
- **[T-03-14 depende de strings exactos]** → Mitigación: la spec los fija; no renombrar esos tres productos en tickets posteriores sin actualizar T-03-14.
- **[Binarios en git]** → Mitigación: 1–3 PNG minúsculos; no fotos de catálogo.
- **[Copia temp del seed en tests unhappy]** → Mitigación: nombre único + `remove_dir_all`; no mutar el árbol del repo.

## Migration Plan

- Expandir archivos en `seed/`; actualizar el párrafo «Ejemplo mínimo» de `seed/README.md`.
- Tests de T-02-40 (`seed_rejects_*`, directorio mínimo en temp) MUST seguir verdes: el directorio mínimo **no** exige archivos opcionales.
- Rollback: revertir el change; el validador no cambia.
- Downstream: T-02-42/43 importan este árbol; T-03-14/T-09-09 lo consumen. Nada que migrar en SurrealDB (aún no hay seed importado).

## Open Questions

Ninguna que altere spec, enfoque o tasks. El recorte 180–220 (en lugar de «exactamente 200») queda fijado en la spec para no forzar un conteo mágico.
