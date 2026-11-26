# Design

## Context

En `develop`, T-02-12 dejó `AttributeDefService::update` → `AttributeDefRepo::update` sin consultar productos, y el test HTTP `patch_attribute_def_changes_key_without_in_use` espera 200. T-02-18/T-02-19 persistieron `producto.atributos[*].def` / `valor`. `ConflictCode::AttributeInUse` y el mapeo HTTP `409` + título «El atributo está en uso» ya existen (T-00-03 / T-00-11). OpenAPI ya describe los `409` de PATCH. `CategoryOccupancy` vive en `catalog` con stub `ProductionOccupancy`; storage no depende de `catalog`. See proposal.md — Why.

Constraints: MVP del curso; ~2 h; `domain` no gana deps; `api` no depende de `storage`; `catalog` no depende de `storage`; sin migración de valores; sin `409` en `DELETE`.

## Goals / Non-Goals

**Goals:**

- Regla de occupancy en el caso de uso de edición (clave inmutable en uso; tipo/`opciones` solo si todos los valores siguen válidos)
- Lectura de valores reales en SurrealDB para el Listo cuando («medida» = `M8`)
- `409 attribute_in_use` en PATCH sin cambiar el contrato YAML

**Non-Goals (diseño):**

- Convertir `Texto("M8")` a número, ni reescribir `atributos[*].valor`
- Occupancy en baja de la definición o de categorías
- Extraer un puerto genérico de occupancy compartido con `CategoryOccupancy`
- Cambiar `AttributeDefRepo::update` para que falle solo

## Decisions

### 1. Puerto `AttributeUsage` en `domain`, no en `catalog`

**Elección:** `domain::AttributeUsage` con `valores(id) -> Vec<ValorAtributo>`. `storage` lo implementa. `AttributeDefService<R, U>` lo inyecta. Lista vacía = no usado.

**Por qué:** `storage` no puede implementar un trait de `catalog` (dependencia invertida). Meter occupancy en `AttributeDefRepo::update` mezclaría persistencia de la definición con lectura de `producto` y rompería el test «el adaptador no aplica la regla». Un método en `ProductRepo` inflaría el puerto de ficha.

**Alternativa:** trait solo en `catalog` y un wrapper en `bin/vortex`. Se descartó: el SQL de occupancy quedaría fuera de `storage`. **Alternativa:** `CategoryOccupancy`-style stub que siempre devuelve vacío. Se descartó: el Listo cuando exige un producto real.

### 2. La regla vive en `catalog::update`, no en storage

**Elección:** `update` carga la fila, pide `valores`, aplica clave/compatibilidad, y solo entonces llama al repo. `AtributoDefRepo::update` sigue aceptando cambio de `clave` (test de storage lo documenta).

**Por qué:** mismo corte que `CategoryService::deactivate` + occupancy. Permite tests de `catalog` con fake. El Gateway HTTP no implementa la regla (sigue el MUST NOT de T-02-12).

### 3. Clave vs tipo: umbrales distintos

**Elección:**

- Cualquier valor persistido + cambio de `clave` → `attribute_in_use` (sin mirar el valor).
- Cambio de `tipo`/`opciones` → `attribute_in_use` solo si algún valor falla la validación de `Producto` (`texto`/`numero`/`booleano`/`enum` por variante, sin coercionar).
- `enum` → `texto` permitido; ampliar opciones permitido; sacar una opción usada no.

**Por qué:** el Hace del ticket separa «cambio de key» (inmutable en uso, modelo 4.4.3 / Meilisearch `attr_<clave>`) de «cambio de type si incompatible». OpenAPI añade `options` incompatibles; es el mismo predicado que `enum`. No hace: migración de valores → no hay `parse` de `"M8"` a número.

**Alternativa:** rechazar todo cambio de `tipo` si hay productos. Se descartó: el ticket pide incompatibilidad, no inmutabilidad de tipo. **Alternativa:** permitir clave si se actualizan todos los productos. Es migración; No hace.

### 4. Productos inactivos cuentan

**Elección:** `valores` no filtra `producto.activo`.

**Por qué:** «si algún producto la usa» incluye fichas discontinuadas: los valores siguen ahí y `attr_<clave>` seguiría existiendo al reactivar. Categorías ignoran inactivos (H-10) porque la baja de categoría no rompe un campo de índice.

**Alternativa:** solo activos. Se descartó para no dejar claves huérfanas al reactivar.

### 5. Compatibilidad reutiliza la validación de producto

**Elección:** extraer (o copiar en `catalog` si el extract es ruidoso) el match de `validate_valor_for_def` a una función pública de `domain` (`valor_compatible_con(tipo, opciones, valor) -> bool`). Sin cambiar el comportamiento de alta/edición de producto.

**Por qué:** un segundo predicado divergiría (p. ej. aceptar `"40"` como número). El No hace queda explícito: incompatibilidad = el valor **ya almacenado** no pasaría `Producto::try_new`.

### 6. Fakes HTTP vs storage

**Elección:** tests de `api` configuran un `AttributeUsage` en memoria (p. ej. mapa `def_id → Vec<ValorAtributo>`). El Listo cuando con producto real vive en `storage/tests` (`create` perno con `medida` = `M8`, luego `valores`). `vortex serve` inyecta el adaptador Surreal.

**Por qué:** mismo corte que T-02-12 / T-02-22. El fake HTTP no abre Surreal; `assert_conforms` cubre el `409`.

### 7. `AttributeDefService::new(repo, usage)`

**Elección:** segundo genérico obligatorio. Tests existentes pasan un usage vacío. `serve.rs` cablea `ProductoRepo`/`Db` como `AttributeUsage`.

**Por qué:** un default «siempre vacío» en producción silenciaría la regla. El cambio de firma es interno al workspace.

### 8. Scaffold kebab-case

`openspec new change` exige kebab-case: scaffold `t-02-23` y rename a `T-02-23`.

## Risks / Trade-offs

- **[Query Surreal sobre array `atributos`]** → dialecto `WHERE`/`CONTAINS` puede devolver de más. Mitigación: filtrar en Rust por `def` tras `SELECT atributos FROM producto`; el catálogo MVP es chico.
- **[`enum` → `texto` permitido]** → deja de ser facetable-enum en Meilisearch hasta T-03-08. Aceptado: el ticket pide incompatibilidad de **valores**, no de facetas.
- **[Baja de definición en uso permitida]** → productos conservan el `record<atributo_def>`. Alineado con marcas y con T-02-12; la UI de T-02-13 ya no bloquea.
- **[Alcance ~2 h]** → un puerto de lectura + `if` en `update` + tests en tres crates. No se divide el ticket.

## Migration Plan

Sin migración de esquema: `producto.atributos` ya existe. Rollback = revertir el PR; las definiciones no se reescriben en este ticket.

## Open Questions

Ninguna que altere specs o tareas. El predicado de compatibilidad queda fijado a la validación de producto existente.
