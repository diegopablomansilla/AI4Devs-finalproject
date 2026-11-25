# Proposal

## Ticket padre

- **Ticket:** [T-02-42](../../../docs/7-tickets.md) — `vortex seed`: categorías, marcas y atributos
- **Change:** `T-02-42`
- **HU:** [HU-02-11](../../../docs/6-historias.md#hu-02-11--dataset-semilla) (MVP parcial: solo maestros; productos en T-02-43; sin imágenes ni sinónimos)
- **Depende de:** T-02-40 (formato JSONL + `catalog::validate_dir`), T-02-05 / T-02-09 / T-02-12 (casos de uso de categoría, marca y `atributo_def`), en `develop`
- **Hace:** importa maestros usando los casos de uso de `catalog`, idempotente (actualiza por *slug* o clave), en modo directo con el servidor detenido (el canal de administración T-00-22 es v2).
- **No hace:** productos.
- **Listo cuando:** ejecutar dos veces seguidas no duplica registros.

## Why

El dataset de desarrollo (~200 SKUs) y el validador JSONL ya existen, pero `vortex seed` sigue siendo stub. Sin importar categorías, marcas y `atributo_def` por los casos de uso de `catalog`, T-02-43 no puede crear productos (referencias por slug/clave) y HU-02-11 no arranca el catálogo desde archivos. Ahora, porque las dependencias están en `develop` y el recableo MVP (§7.5.1) fija modo directo: abrir SurrealDB con el servidor detenido, sin socket T-00-22.

## What Changes

- Orquestador de importación de **maestros** en `catalog` (función de biblioteca): lee `categorias.jsonl`, `marcas.jsonl` y `atributos.jsonl`, upsert por slug (`categoria`, `marca`) o `clave` (`atributo_def`) vía `CategoryService` / `BrandService` / `AttributeDefService`
- Idempotencia: una segunda ejecución sobre la misma base **no duplica** filas (**Listo cuando**); si el nombre u otros campos mutables cambian, actualiza el registro existente
- `vortex seed [DIR]` deja de ser stub: carga `Config` (`--config` / `VORTEX_CONFIG`), abre Storage + migraciones en `<paths.datos>/db` con el servidor **detenido**, importa solo maestros y termina 0 si no hay issues
- **BREAKING** (CLI): `vortex seed` ya no imprime `no implementado` ni ignora `--config`; sin config o con el lock de RocksDB tomado, sale ≠ 0
- TDD: happy (primera carga + segunda sin duplicar; productos del JSONL no se persisten) + unhappy (JSONL inválido con archivo/línea/campo y cero escrituras; config ausente; base bloqueada)
- MUST NOT persistir `producto`, `movimiento_stock`, imágenes, relaciones, sinónimos ni notas

## Non-goals

- Importar productos y movimiento `inicial` — **T-02-43** (**No hace** del ticket)
- Importar imágenes — **T-02-44** (v2)
- Importar relaciones — **T-04-11**
- Importar sinónimos — **T-03-29** (v2)
- Importar notas técnicas — **T-05-29** (v2)
- Catálogo del comercio piloto — **T-02-46** (v2)
- Canal de administración Unix socket / seed con `vortex serve` vivo — **T-00-22** (v2)
- `vortex reindex`, Meilisearch, outbox worker, HTTP, `openapi.yaml`, UI
- Cambiar el esquema JSONL o el contrato de `validate_dir` (salvo levantar la prohibición de enganchar el stub)
- Crates nuevas en el workspace (members fijos)

## Capabilities

### New Capabilities

- `vortex-seed`: `vortex seed` importa maestros del directorio semilla (categorías, marcas, definiciones de atributo) de forma idempotente, usando casos de uso de `catalog` y abriendo SurrealDB en modo directo

### Modified Capabilities

- `vortex-cli`: `vortex seed` deja de ser stub «no implementado»; el resto de subcomandos no implementados no cambia
- `seed-dataset`: el validador sigue sin persistir; se retira la prohibición de cambiar el stub `vortex seed` (T-02-40), que este ticket reemplaza

## Impact

- **Repo:** `vortex-core/crates/catalog/` (orquestador + tests in-memory), `vortex-core/bin/vortex/` (`seed` en CLI, tests de stub y de comando), `seed/README.md` (cómo invocar `vortex seed`)
- **APIs / OpenAPI:** ninguna
- **CLI:** `vortex seed [DIR]` (default `seed/` relativo al CWD); exige `--config` o `VORTEX_CONFIG` como `serve`
- **Verificación:** `cargo test -p catalog`, `cargo test -p vortex --test cli`, [docs/desarrollo-git-openspec.md](../../../docs/desarrollo-git-openspec.md) § 5 en `vortex-core/`, `openspec validate T-02-42`
- **Ticket:** cierra T-02-42 (Listo cuando: dos ejecuciones seguidas no duplican)
- **Ámbito:** ~2 h; upsert sobre `list_all` + create/update existentes, sin puertos nuevos ni crate extra. No se divide el ticket.
