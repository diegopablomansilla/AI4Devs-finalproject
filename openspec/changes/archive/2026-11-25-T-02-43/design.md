# Design

## Context

En `develop`, `catalog::import_masters` upserta categorías/marcas/`atributo_def` y `vortex seed` abre SurrealDB en modo directo (T-02-42). `ProductGateway::create` valida refs y delega en `ProductRepo::create`, que ya hace una `Tx` por alta: ficha + movimiento `inicial` si `stock_inicial > 0` (T-02-18 / T-02-20). `catalog-stock` rechaza `tipo` `inicial`. `ProductoRow` en `seed.rs` valida el JSONL pero aún no conserva `nombre`/`tipo`/`codigos`/`ubicacion` para el alta. `catalog` MUST NOT depender de `storage`. Ver `proposal.md` para motivación y `specs/` para el contrato.

Constraints: No hace = imágenes/relaciones/sinónimos/notas; ~2 h; members fijos; CLI OpenSpec kebab-case (`t-02-43` → carpeta `T-02-43`).

## Goals / Non-Goals

**Goals:**

- `import_products` (nombre MAY variar) en `catalog` sobre `ProductGateway` + mapas de slug/clave de maestros
- Aislar rechazos por línea; resumen `importados`/`rechazados`
- `vortex seed` llama maestros y luego productos; exit 0 solo si `rechazados == 0`
- Idempotencia: SKU existente → no segundo `create` ni segundo `inicial`

**Non-Goals (diseño):**

- Puerto `get_by_sku` en `domain`
- Upsert de ficha (precio/nombre) ni `reactivate` en re-seed
- `vortex seed --check`; el validador sigue siendo `validate_dir` (no se invoca en el comando: un `relaciones.jsonl` inválido no debe bloquear productos)
- Extraer crate `boot` compartida con `serve`

## Decisions

### 1. `import_products` separado; CLI encadena `import_masters` + productos

**Elección:** `catalog::import_products(dir, categories, brands, attrs, products) -> SeedProductSummary`. El binario, tras `import_masters` Ok, construye `ProductService`/`ProductoRepo` (igual que `serve`) y llama a `import_products`. Si maestros fallan, no se tocan productos.

**Por qué:** T-02-42 dejó `import_masters` sin persistir productos a propósito. Reusar esa función evita mezclar fail-closed de maestros con fail-open de SKUs.

**Alternativa:** un solo `import_seed`. Se descartó: rompería el contrato y los tests de maestros. **Alternativa:** que el binario parsee JSONL. Se descartó: el Listo cuando de aislamiento vive en `catalog` con repos in-memory.

### 2. No llamar `validate_dir` en el comando

**Elección:** parsear solo `productos.jsonl` (ampliar `ProductoRow` con los campos de `NewProducto`). Issues de relaciones/imágenes/sinónimos/notas no abortan ni se reportan aquí.

**Por qué:** `validate_dir` exige coherencia de archivos v2 (T-02-44 / T-03-29 / T-04-11 / T-05-29). El No hace es no importarlos, no validar el árbol entero en seed.

**Alternativa:** `validate_dir` primero. Se descartó: un SKU de imagen desconocido bloquearía productos.

### 3. Una Tx por SKU = un `ProductGateway::create`

**Elección:** un `create` por línea válida. La `Tx` (producto + `inicial`) ya está en `storage::ProductoRepo`. `catalog` no abre `Tx`. MUST NOT usar `StockGateway`.

**Por qué:** el Hace pide transacción por producto con movimiento `inicial`; reimplementar SQL duplicaría T-02-18. `stock_inicial == 0` no inserta movimiento (regla de dominio `cantidad ≠ 0`).

**Alternativa:** `StockLedger::append` con `tipo` `inicial` tras un create con stock 0. Se descartó: el caso de uso de stock rechaza `inicial`; dos escrituras no son una Tx desde `catalog`.

### 4. Rechazo aislado: continuar; `duplicate_sku` de archivo vs de re-seed

**Elección:** (1) parsear todas las líneas; SKU duplicado **en el archivo** → issue en la ocurrencia posterior, no se intenta `create`. (2) Antes del loop, paginar `ProductGateway::list` (`activo` true y false) a un mapa SKU→id. Si el SKU ya está en la base, incrementa `importados` y no llama `create`. (3) Cualquier otro fallo de parseo o de `create` → issue, `rechazados++`, siguiente línea.

**Por qué:** tratar todo `duplicate_sku` como skip haría pasar un JSONL con SKU repetido. El mapa evita un puerto nuevo. ~200 SKUs caben en dos pasadas de `list`.

**Alternativa:** fail-closed de productos (como maestros). Se descartó: contradice «se rechaza solo». **Alternativa:** `get_by_sku`. Se descartó: cambia `domain` fuera del ticket.

### 5. Resumen y código de salida

**Elección:** `SeedProductSummary { importados, rechazados, issues }`. El CLI imprime issues (mismo formato T-02-42) y una línea de resumen (`importados` / `rechazados`). Exit 0 iff maestros Ok y `rechazados == 0`. Con rechazos, los SKUs válidos **quedan** (no hay rollback global).

**Por qué:** el Listo cuando pide dataset completo (exit 0) y registro inválido aislado (exit ≠ 0 + resto persistido).

**Alternativa:** exit 0 con rechazos. Se descartó: un operador no distinguiría seed sucio. **Alternativa:** transacción global. Se descartó: `catalog` no abre `Tx` y el ticket pide Tx **por** producto.

### 6. Tests: in-memory = stock; CLI = ledger

**Elección:** tests de `catalog` usan `InMemoryProductRepo` (el `stock` queda en `stock_inicial`; no hay tabla `movimiento_stock`). Tests de `vortex` (`tests/seed.rs`) reabren RocksDB, listan productos y `MovimientoStockRepo::list_by_producto` para afirmar `tipo` `inicial`. El Listo cuando del dataset corre en ambos: in-memory sobre `seed/` del repo, y `vortex seed` contra una copia o el árbol real con `datos/` temp.

**Por qué:** allowlist de `catalog`. El movimiento real solo existe en storage.

**Alternativa:** test de integración en `storage`. Se descartó: el ticket es el comando y el orquestador de `catalog`.

### 7. Fixtures CLI de T-02-42

**Elección:** `write_master_seed` pasa a incluir un `productos.jsonl` mínimo (p. ej. `PER-M8X40-ZN` con `stock_inicial` `"50"`). Sin ese archivo, seed sale ≠ 0 (issue `productos.jsonl` línea 0).

**Por qué:** el comando ahora exige productos para considerarse completo. Los tests de lock/config no necesitan JSONL válido si fallan antes de importar.

## Risks / Trade-offs

- **[In-memory no prueba el ledger]** → Mitigación: escenario de movimiento `inicial` obligatorio en `tests/seed.rs` con `MovimientoStockRepo`.
- **[`list` paginado omite inactivos si se filtra mal]** → Mitigación: dos filtros `activo: true` y `activo: false`; paginar hasta página vacía.
- **[Dataset ~200 SKUs en CI]** → Mitigación: un test CLI del `seed/` del repo; el resto usa fixture de 2–3 líneas. Tiempo esperado segundos, no minutos.
- **[Create a mitad de 200 deja maestros + N productos si luego hay issues]** → Aceptado: es el Hace (rechazo aislado), no un rollback.
- **[SKU inactivo no se reactiva]** → Aceptado: Non-goal; documentado en proposal.

## Migration Plan

No hay datos de producción. Quien ya corrió T-02-42 tiene maestros sin productos: la primera corrida de T-02-43 completa el catálogo. Rollback: revertir el PR; los productos sembrados permanecen. T-04-11 reutilizará el mismo comando para relaciones.

## Open Questions

Ninguna que altere specs o tareas. Idempotencia skip-if-exists (sin update de ficha) y exit ≠ 0 si `rechazados ≥ 1` quedan fijados arriba.
