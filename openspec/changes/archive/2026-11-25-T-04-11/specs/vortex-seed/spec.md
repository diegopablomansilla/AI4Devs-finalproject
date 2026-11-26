# Spec Delta

Ticket padre: T-04-11. HU: HU-02-11.

## ADDED Requirements

### Requirement: Importación de relaciones omitiendo existentes

El crate `catalog` MUST exponer una función de biblioteca que, dado un directorio semilla, los productos ya persistidos y un puerto de alta de relaciones, importa `relaciones.jsonl` **después** de maestros y productos. MUST resolver `origen` y `destino` por SKU a los `id` de producto ya importados. MUST delegar el alta en el caso de uso de relaciones (`create` con `NewRelacion`: `tipo`, `prioridad`, `nota`, `bidireccional`). MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT implementar el ABM de relaciones (sigue en `recommend`, DC-05). MUST NOT persistir imágenes, sinónimos ni notas aunque esos archivos existan.

Si `relaciones.jsonl` no existe, MUST devolver resumen con `importados` 0, `omitidas` 0 y `rechazados` 0 y MUST NOT tratar la ausencia como issue. Cada línea no vacía MUST procesarse de forma aislada (un `create` por registro válido). `prioridad` ausente MUST ser 1. `bidireccional` ausente MUST usar el default de dominio por `tipo` (`similar_a` sí; `sustituible_por` y `complementa` no). `nota` ausente MUST enviarse como `None`.

Si `create` responde `duplicate_relation` (la arista `origen`→`destino` de ese `tipo` ya existe), MUST contar la línea en `omitidas` y MUST NOT incrementarla en `rechazados` ni volver a crearla (**Hace:** omitir las existentes). MUST NOT actualizar `prioridad` ni `nota` de una arista existente. Una segunda invocación sobre el mismo directorio y la misma base MUST dejar el mismo número de aristas que la primera.

El resultado MUST incluir `importados` (líneas que crearon al menos una arista en esta corrida), `omitidas`, `rechazados` y la lista de issues con `archivo`, `linea`, `campo` y `mensaje` (mismo contrato que `seed-dataset`).

#### Scenario: primera carga crea las aristas del JSONL (happy)

- **WHEN** la base ya tiene los productos `PER-M8X40-ZN` y `ARA-PERNO-01`
- **AND** `relaciones.jsonl` tiene `origen` `"PER-M8X40-ZN"`, `destino` `"ARA-PERNO-01"`, `tipo` `"complementa"`, `nota` `"Kit fijación"`
- **THEN** tras importar existe una arista `complementa` de `PER-M8X40-ZN` hacia `ARA-PERNO-01`
- **AND** no hay arista inversa
- **AND** el resumen tiene `importados` 1, `omitidas` 0 y `rechazados` 0

#### Scenario: similar_a omitido bidireccional crea el par (happy)

- **WHEN** `relaciones.jsonl` tiene `tipo` `"similar_a"` entre dos SKUs existentes y no declara `bidireccional`
- **THEN** se crean las dos aristas del grupo (A→B y B→A)
- **AND** el resumen cuenta `importados` 1 (una línea)

#### Scenario: segunda ejecución omite existentes (happy, Hace)

- **WHEN** se importan las mismas relaciones dos veces seguidas sobre la misma base
- **THEN** el conteo de aristas es el de la primera ejecución
- **AND** la segunda corrida tiene `importados` 0, `omitidas` igual al número de líneas válidas y `rechazados` 0

#### Scenario: dataset de desarrollo siembra el panel (happy, Listo cuando)

- **WHEN** el directorio es el `seed/` del repositorio y ya se importaron maestros y productos
- **AND** se importan relaciones
- **THEN** `rechazados` es 0
- **AND** el producto `PER-M8X40-ZN` tiene un complementario con SKU `"ARA-PERNO-01"`
- **AND** existen aristas `similar_a` y `sustituible_por` del dataset (`TOR-AUTO-10X1`→`TOR-4-16-AC-001` y `PER-ACERO-01`→`PER-ACERO-02`)

#### Scenario: archivo relaciones.jsonl ausente no es error (happy)

- **WHEN** el directorio no tiene `relaciones.jsonl` y los productos ya están importados
- **THEN** el resumen tiene `importados` 0, `omitidas` 0, `rechazados` 0 y lista de issues vacía
- **AND** no se crea ninguna arista

#### Scenario: imágenes y sinónimos no se persisten (happy, No hace)

- **WHEN** el directorio incluye `sinonimos.jsonl`, `notas.jsonl` e `imagenes/`
- **AND** se importan relaciones
- **THEN** no se crean filas de sinónimo, nota ni imagen
- **AND** las relaciones válidas sí se persisten

### Requirement: Registro inválido de relación se rechaza solo

Un issue de una relación (JSON malformado, `tipo` inválido, SKU de `origen` o `destino` ausente en los productos persistidos, `origen` igual a `destino`, `prioridad` menor que 1, `nota` de más de 500 caracteres, o error de dominio al crear distinto de `duplicate_relation`) MUST informarse con `archivo` `relaciones.jsonl`, `linea` y `campo`, MUST incrementar `rechazados`, y MUST NOT dejar aristas de esa línea. MUST continuar con las líneas siguientes: las relaciones válidas ya persistidas MUST permanecer. MUST NOT revertir productos ni maestros. MUST NOT abortar el resto del archivo por un solo issue.

#### Scenario: SKU desconocido se rechaza solo (unhappy)

- **WHEN** `relaciones.jsonl` tiene una arista válida y otra cuyo `destino` no coincide con ningún SKU persistido
- **AND** se importa
- **THEN** hay un issue con `archivo` `relaciones.jsonl` y `campo` `destino` en la línea del inválido
- **AND** la arista válida está persistida
- **AND** la línea inválida no creó aristas
- **AND** el resumen tiene `importados` ≥ 1 y `rechazados` ≥ 1

#### Scenario: JSONL de relación malformado no deja parciales (unhappy)

- **WHEN** una línea no vacía de `relaciones.jsonl` no es un objeto JSON y hay otra línea válida
- **THEN** hay un issue con `archivo` `relaciones.jsonl`, esa `linea` y `campo` cadena vacía
- **AND** la arista de la línea válida se persiste
- **AND** no hay arista asociada a la línea malformada

#### Scenario: self_relation se rechaza solo (unhappy)

- **WHEN** una línea declara el mismo SKU en `origen` y `destino` y hay otra línea válida
- **THEN** hay un issue con `campo` `destino`
- **AND** la arista válida se persiste
- **AND** no se crea arista de la línea de auto-relación

## MODIFIED Requirements

### Requirement: vortex seed abre la base en modo directo

`vortex seed` MUST dejar de responder `no implementado`. MUST aceptar un directorio opcional (`vortex seed [DIR]`); si se omite, MUST usar `seed` relativo al directorio de trabajo. MUST cargar `Config` con `--config` o `VORTEX_CONFIG` (mismo criterio que `vortex serve`). MUST abrir Storage en `<paths.datos>/db`, aplicar migraciones, importar maestros, a continuación productos y a continuación relaciones. MUST escribir un resumen con importados y rechazados de productos y un resumen con importados, omitidas y rechazados de relaciones. MUST terminar con código 0 si la importación de maestros no tiene issues, `rechazados` de productos es 0 y `rechazados` de relaciones es 0. MUST terminar con código distinto de 0 si hay issues de maestros o `rechazados` ≥ 1 en productos o en relaciones; en esos casos MUST NOT revertir los productos ni las relaciones válidas ya persistidas. MUST NOT arrancar el servidor HTTP ni sidecars. MUST NOT usar el canal Unix socket (T-00-22 es v2). MUST NOT persistir imágenes, sinónimos ni notas.

Si faltan `--config` y `VORTEX_CONFIG`, MUST terminar con código distinto de 0 y MUST NOT escribir `no implementado`. Si el archivo de configuración no existe o es inválido, MUST terminar ≠ 0 y MUST NOT importar. Si no se puede abrir la base (lock de RocksDB porque `vortex serve` está corriendo, o ruta inaccesible), MUST terminar ≠ 0, MUST informar en stderr que no se pudo abrir la base, y MUST NOT escribir `no implementado`.

Si la importación de maestros falla, MUST NOT importar productos ni relaciones. Si hay rechazos de productos, MUST igual importar relaciones (los SKUs válidos ya persistidos) y MUST terminar ≠ 0.

#### Scenario: seed importa maestros y sale 0 (happy)

- **WHEN** existe un TOML válido, `paths.datos` escribible, el servidor no está corriendo
- **AND** `DIR` es un directorio con maestros y `productos.jsonl` válidos (al menos un SKU con `stock_inicial` > 0)
- **AND** se ejecuta `vortex --config <toml> seed <DIR>`
- **THEN** el proceso termina con código 0
- **AND** stderr no contiene `no implementado`
- **AND** las categorías, marcas y definiciones del JSONL están en la base
- **AND** los productos del JSONL están en la base
- **AND** hay un movimiento `inicial` por cada producto con `stock_inicial` > 0
- **AND** la salida incluye un resumen con importados y rechazados

#### Scenario: seed dos veces no duplica (happy, Listo cuando)

- **WHEN** se ejecuta `vortex seed` dos veces seguidas con el mismo `--config` y el mismo `DIR` válido (maestros + productos + relaciones), con el servidor detenido
- **THEN** ambas invocaciones terminan con código 0
- **AND** el número de categorías, marcas, definiciones y productos no aumenta en la segunda
- **AND** el número de movimientos `inicial` no aumenta en la segunda
- **AND** el número de aristas de relación no aumenta en la segunda

#### Scenario: seed dataset de desarrollo exit 0 (happy, Listo cuando)

- **WHEN** se ejecuta `vortex --config <toml> seed` apuntando al `seed/` del repositorio, servidor detenido
- **THEN** el proceso termina con código 0
- **AND** hay entre 180 y 220 productos persistidos
- **AND** el resumen de productos y el de relaciones tienen `rechazados` 0
- **AND** el producto `PER-M8X40-ZN` tiene un complementario `ARA-PERNO-01` consultable para el panel de recomendaciones

#### Scenario: seed con un producto inválido sale distinto de 0 y aísla el rechazo (unhappy, Listo cuando)

- **WHEN** se copia el dataset de desarrollo y se agrega a `productos.jsonl` un objeto cuyo `precio` no es decimal de dominio
- **AND** se ejecuta `vortex --config <toml> seed <copia>`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr informa `productos.jsonl`, línea y campo `precio`
- **AND** los productos válidos de la copia están persistidos
- **AND** el SKU del registro inválido no está persistido

#### Scenario: seed con una relación inválida sale distinto de 0 y aísla el rechazo (unhappy)

- **WHEN** se copia el dataset de desarrollo y se agrega a `relaciones.jsonl` una arista cuyo `destino` no es un SKU del archivo
- **AND** se ejecuta `vortex --config <toml> seed <copia>`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr informa `relaciones.jsonl`, línea y campo `destino`
- **AND** las relaciones válidas de la copia están persistidas
- **AND** la línea inválida no creó aristas

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
- **AND** se ejecuta `vortex --config <toml> seed <DIR>` con maestros y productos válidos
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr indica que no se pudo abrir la base
- **AND** stderr no contiene `no implementado`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `seed_import_relations_creates_complement_edge` | `catalog` (`src/seed.rs`) | unitario (memoria) happy |
| `seed_import_relations_similar_creates_bidirectional_pair` | `catalog` | unitario (memoria) happy |
| `seed_import_relations_second_run_omits_existing` | `catalog` | unitario (memoria) happy (Hace) |
| `seed_import_relations_dev_dataset_seeds_panel_edges` | `catalog` | unitario (memoria) happy (Listo cuando) |
| `seed_import_relations_missing_file_is_ok` | `catalog` | unitario (memoria) happy |
| `seed_import_relations_does_not_persist_synonyms_notes_images` | `catalog` | unitario (memoria) happy (No hace) |
| `seed_import_relations_rejects_unknown_sku_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_relations_rejects_malformed_line_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_relations_rejects_self_relation_only` | `catalog` | unitario (memoria) unhappy |
| `vortex_seed_imports_relations_exit_0` | `vortex` (`tests/seed.rs`) | integración happy |
| `vortex_seed_twice_does_not_duplicate_relations` | `vortex` | integración happy |
| `vortex_seed_dev_dataset_persists_panel_relations` | `vortex` | integración happy (Listo cuando) |
| `vortex_seed_invalid_relation_sku_rejected_alone` | `vortex` | integración unhappy |
