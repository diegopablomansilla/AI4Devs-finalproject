# vortex-seed Specification

## Purpose

Importa categorías, marcas y definiciones de atributo desde un directorio semilla JSONL de forma idempotente, usando los casos de uso de catálogo y abriendo SurrealDB en modo directo con el servidor detenido.

Ticket padre: T-02-42. HU: HU-02-11.

## Requirements

### Requirement: Importación idempotente de maestros

El crate `catalog` MUST exponer una función de biblioteca que recibe la ruta de un directorio semilla y los casos de uso de categoría, marca y definición de atributo. MUST leer `categorias.jsonl`, `marcas.jsonl` y `atributos.jsonl` (mismo esquema que `seed-dataset`). MUST upsert: si ya existe una categoría o marca con el mismo `slug`, o una definición con la misma `clave`, MUST actualizar los campos mutables (`nombre`, `padre`, `etiqueta`, `tipo`, `unidad`, `opciones`, `facetable`, `buscable`, `orden`) en lugar de crear otra fila. Si el registro existente está inactivo, MUST reactivarlo. MUST NOT borrar registros que no estén en el JSONL.

Las categorías MUST persistirse en orden topológico (padres antes que hijas). `padre` en el JSONL MUST resolverse al `id` de la categoría cuyo `slug` coincide. Marcas MUST identificarse por `slug` global; atributos por `clave`. MUST delegar en los casos de uso de `catalog` (`create` / `rename` / `move_category` / `update` / `reactivate`); MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT persistir `producto`, `movimiento_stock`, imágenes, relaciones, sinónimos ni notas aunque esos archivos existan en el directorio.

Una segunda invocación sobre el mismo directorio y la misma base MUST dejar el mismo número de categorías, marcas y definiciones que la primera, con los mismos slugs y claves (**Listo cuando**). Si el JSONL cambió `nombre` u otros campos mutables, la segunda invocación MUST actualizar el registro existente.

#### Scenario: primera carga crea maestros (happy)

- **WHEN** la base no tiene categorías, marcas ni definiciones
- **AND** el directorio tiene `categorias.jsonl` con `fijaciones` (sin padre) y `pernos` (`padre` `fijaciones`), `marcas.jsonl` con `fixser`, y `atributos.jsonl` con `medida`
- **THEN** tras importar hay exactamente esas filas
- **AND** `pernos` tiene por padre a `fijaciones`
- **AND** no hay productos persistidos

#### Scenario: segunda ejecución no duplica (happy, Listo cuando)

- **WHEN** se importa el mismo directorio dos veces seguidas sobre la misma base
- **THEN** el conteo de categorías, marcas y definiciones de atributo es el de la primera ejecución
- **AND** cada `slug` de categoría y marca y cada `clave` de atributo aparece una sola vez

#### Scenario: segunda ejecución actualiza nombre (happy)

- **WHEN** ya existe la marca `fixser` con `nombre` `"Fixser"`
- **AND** `marcas.jsonl` trae el mismo `slug` con `nombre` `"Fixser Industrial"`
- **THEN** sigue habiendo una sola marca con `slug` `fixser`
- **AND** su `nombre` es `"Fixser Industrial"`

#### Scenario: productos del JSONL no se persisten (happy, No hace)

- **WHEN** el directorio incluye `productos.jsonl` válido con al menos un SKU
- **AND** se importan los maestros
- **THEN** no se crea ningún producto ni movimiento de stock

### Requirement: Issues de maestros abortan sin escrituras

Antes de persistir, la importación MUST parsear los tres JSONL de maestros y MUST acumular issues con `archivo`, `linea`, `campo` y `mensaje` (mismo contrato que el validador de `seed-dataset`). Si hay al menos un issue, MUST NOT llamar a crear ni actualizar, MUST devolver esos issues y MUST dejar la base igual que al inicio. MUST NOT exigir que existan `productos.jsonl`, `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` ni `imagenes/` para importar maestros. Un `productos.jsonl` inválido MUST NOT por sí solo abortar la importación de maestros (T-02-43). JSON malformado, `slug`/`clave` duplicados en el archivo, `nombre` vacío, `padre` desconocido o `tipo` de atributo inválido MUST producir issues con archivo, línea y campo.

#### Scenario: JSONL de categoría malformado no escribe (unhappy)

- **WHEN** una línea no vacía de `categorias.jsonl` no es un objeto JSON
- **AND** el resto del directorio de maestros sería válido
- **THEN** la importación devuelve al menos un issue con `archivo` `categorias.jsonl` y esa `linea`
- **AND** `campo` es la cadena vacía
- **AND** no se crea ni actualiza ninguna categoría, marca ni definición

#### Scenario: padre desconocido no escribe (unhappy)

- **WHEN** una categoría declara `padre` `"no-existe"` y ese slug no está en `categorias.jsonl`
- **THEN** hay un issue con `archivo` `categorias.jsonl` y `campo` `padre`
- **AND** no se persiste ninguna fila

#### Scenario: archivo maestro ausente no escribe (unhappy)

- **WHEN** falta `marcas.jsonl` (o `categorias.jsonl` o `atributos.jsonl`)
- **THEN** hay un issue con ese `archivo` y `linea` 0
- **AND** no se persiste ninguna fila

### Requirement: vortex seed abre la base en modo directo

`vortex seed` MUST dejar de responder `no implementado`. MUST aceptar un directorio opcional (`vortex seed [DIR]`); si se omite, MUST usar `seed` relativo al directorio de trabajo. MUST cargar `Config` con `--config` o `VORTEX_CONFIG` (mismo criterio que `vortex serve`). MUST abrir Storage en `<paths.datos>/db`, aplicar migraciones e importar solo maestros. MUST terminar con código 0 si la importación no tiene issues. MUST NOT arrancar el servidor HTTP ni sidecars. MUST NOT usar el canal Unix socket (T-00-22 es v2).

Si faltan `--config` y `VORTEX_CONFIG`, MUST terminar con código distinto de 0 y MUST NOT escribir `no implementado`. Si el archivo de configuración no existe o es inválido, MUST terminar ≠ 0 y MUST NOT importar. Si no se puede abrir la base (lock de RocksDB porque `vortex serve` está corriendo, o ruta inaccesible), MUST terminar ≠ 0, MUST informar en stderr que no se pudo abrir la base, y MUST NOT escribir `no implementado`.

#### Scenario: seed importa maestros y sale 0 (happy)

- **WHEN** existe un TOML válido, `paths.datos` escribible, el servidor no está corriendo
- **AND** `DIR` es un directorio con maestros JSONL válidos
- **AND** se ejecuta `vortex --config <toml> seed <DIR>`
- **THEN** el proceso termina con código 0
- **AND** stderr no contiene `no implementado`
- **AND** las categorías, marcas y definiciones del JSONL están en la base
- **AND** no hay productos persistidos

#### Scenario: seed dos veces no duplica (happy, Listo cuando)

- **WHEN** se ejecuta `vortex seed` dos veces seguidas con el mismo `--config` y el mismo `DIR` válido, con el servidor detenido
- **THEN** ambas invocaciones terminan con código 0
- **AND** el número de categorías, marcas y definiciones no aumenta en la segunda

#### Scenario: seed sin config (unhappy)

- **WHEN** se ejecuta `vortex seed` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: seed con config inexistente (unhappy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml seed`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: seed con base bloqueada (unhappy)

- **WHEN** SurrealDB en `<paths.datos>/db` ya está abierta por otro proceso (servidor en ejecución)
- **AND** se ejecuta `vortex --config <toml> seed <DIR>` con maestros válidos
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr indica que no se pudo abrir la base
- **AND** stderr no contiene `no implementado`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `seed_import_masters_creates_categoria_marca_atributo` | `catalog` (`src/seed.rs`) | unitario (memoria) happy |
| `seed_import_masters_second_run_does_not_duplicate` | `catalog` | unitario (memoria) happy (Listo cuando) |
| `seed_import_masters_second_run_updates_nombre` | `catalog` | unitario (memoria) happy |
| `seed_import_masters_does_not_persist_productos` | `catalog` | unitario (memoria) happy (No hace) |
| `seed_import_masters_rejects_malformed_categoria_jsonl_without_writes` | `catalog` | unitario (memoria) unhappy |
| `seed_import_masters_rejects_unknown_padre_without_writes` | `catalog` | unitario (memoria) unhappy |
| `seed_import_masters_rejects_missing_marcas_file_without_writes` | `catalog` | unitario (memoria) unhappy |
| `vortex_seed_imports_masters_exit_0` | `vortex` (`tests/cli.rs` o `tests/seed.rs`) | integración happy |
| `vortex_seed_twice_does_not_duplicate` | `vortex` | integración happy (Listo cuando) |
| `vortex_seed_without_config_is_error` | `vortex` | integración unhappy |
| `vortex_seed_missing_config_file_is_not_unimplemented` | `vortex` | integración unhappy |
| `vortex_seed_locked_db_is_error` | `vortex` | integración unhappy |
| `vortex_top_level_stubs_print_unimplemented` sin `seed` | `vortex` (`tests/cli.rs`) | regresión stub |
