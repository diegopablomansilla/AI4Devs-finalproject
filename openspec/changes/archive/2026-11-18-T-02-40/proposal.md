# Proposal

## Ticket padre

- **Ticket:** [T-02-40](../../../docs/7-tickets.md) — Formato del dataset semilla y validador
- **Change:** `T-02-40`
- **HU:** [HU-02-11](../../../docs/6-historias.md#hu-02-11--dataset-semilla) (MVP parcial: sin importar imágenes ni sinónimos; el dataset de ~200 SKUs es T-02-41)
- **Depende de:** T-02-16 (en `develop`: entidad `Producto`, `Decimal`, value objects y validación de atributos)
- **Hace:** formato en `seed/`: `categorias.jsonl`, `marcas.jsonl`, `atributos.jsonl`, `productos.jsonl` (referencias por *slug*, clave y SKU), `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/<sku>/…`; `seed/README.md` con el esquema; validador que informa archivo, línea y campo. Es el mismo formato que usará la importación futura.
- **No hace:** importar.
- **Listo cuando:** el validador detecta un precio no numérico e informa línea y campo.

## Why

HU-02-11 exige cargar el catálogo inicial desde archivos y, ante un registro inválido (precio no numérico), informar el registro y el campo sin dejar datos parciales. T-02-16 ya valida `precio` como `Decimal` en dominio, pero el repo no tiene `seed/` ni un validador de JSONL. Sin el formato y el informe archivo/línea/campo, T-02-41 no puede cerrar «el validador no reporta errores» y T-02-42 no tiene contrato de entrada.

## What Changes

- Árbol `seed/` en la raíz del repo con los JSONL del ticket, `imagenes/` y `seed/README.md` (esquema, claves naturales, ejemplo mínimo — no el dataset de ~200 SKUs)
- Validador de directorio en el crate `catalog` (función de biblioteca): parsea JSONL, resuelve referencias por slug / clave / SKU y acumula issues `{ archivo, línea, campo, mensaje }`
- `precio` en `productos.jsonl` es string decimal de dominio; un valor no numérico (p. ej. `"barato"`) produce un issue con archivo `productos.jsonl`, línea 1-based y campo `precio` (**Listo cuando**)
- Allowlist de `catalog`: MAY declarar `serde` y `serde_json` (`workspace = true`) además de `domain`, para DTO JSONL
- TDD: happy (directorio mínimo válido, incluido `seed/` del repo) + unhappy (precio no numérico, JSON malformado, referencia inexistente, SKU duplicado)
- MUST NOT importar a SurrealDB ni cambiar el stub `vortex seed` (sigue «no implementado»)

## Non-goals

- Importar maestros (`vortex seed` categorías/marcas/atributos) — **T-02-42** (**No hace** del ticket)
- Importar productos y movimiento `inicial` — **T-02-43**
- Dataset de desarrollo ~200 SKUs — **T-02-41**
- Importar imágenes — **T-02-44** (v2)
- Importar relaciones — **T-04-11**
- Importar sinónimos — **T-03-29** (v2)
- Importar notas técnicas — **T-05-29** (v2)
- Catálogo del piloto y conversor — **T-02-46** (v2)
- Generador sintético 10 k SKUs — **T-09-06** (v2)
- Canal de administración / `vortex seed` contra servicio vivo — **T-00-22** (v2)
- HTTP, `openapi.yaml`, UI, Meilisearch, reindex
- Crates nuevas en el workspace (members fijos de `cargo-workspace`)

## Capabilities

### New Capabilities

- `seed-dataset`: formato JSONL en `seed/` (claves naturales: slug, clave, SKU) y validador de directorio que informa archivo, línea y campo, sin persistir

### Modified Capabilities

- `cargo-workspace`: el crate `catalog` MAY declarar `serde` y `serde_json` en `[dependencies]` (`workspace = true`; `serde` MAY activar `derive`) además de `domain`. `scripts/check-workspace.sh` y sus fixtures se alinean. MUST NOT declarar `surrealdb` ni `storage`.

## Impact

- **Repo:** `seed/` (README + JSONL mínimos + `imagenes/`), `vortex-core/crates/catalog/` (módulo validador + tests), `vortex-core/crates/catalog/Cargo.toml`, `openspec/specs/cargo-workspace/spec.md` (tras archive), `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`
- **APIs / OpenAPI:** ninguna
- **CLI:** `vortex seed` permanece stub (T-00-07); T-02-42 enganchará el validador
- **Verificación:** `cargo test -p catalog`, `./scripts/check-workspace.sh`, [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`, `openspec validate T-02-40`
- **Ticket:** cierra T-02-40 (Listo cuando: precio no numérico → archivo, línea y campo)
- **Ámbito:** ~2 h; formato + validador de biblioteca + fixture mínimo. No se divide el ticket.
