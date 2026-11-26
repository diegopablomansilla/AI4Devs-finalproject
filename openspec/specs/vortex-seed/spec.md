# vortex-seed Specification

## Purpose

Importa categorías, marcas, definiciones de atributo, productos (con movimiento `inicial`) y relaciones de producto desde un directorio semilla JSONL de forma idempotente, usando los casos de uso de catálogo y de relaciones, y abriendo SurrealDB en modo directo con el servidor detenido.

Tickets: T-02-42, T-02-43, T-04-11. HU: HU-02-11.

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

### Requirement: Importación de productos con movimiento inicial

El crate `catalog` MUST exponer una función de biblioteca que, dado un directorio semilla y los casos de uso de catálogo (categoría, marca, definición de atributo y producto), importa `productos.jsonl` **después** de que los maestros estén disponibles. MUST resolver `categoria` y `marca` por slug y `atributos[].clave` a los `id` de las filas ya persistidas. MUST delegar el alta en el caso de uso de producto (`ProductGateway` / `create`) con `stock_inicial` del JSONL (default `"0"` si el campo está ausente). MUST NOT abrir SurrealDB ni mapear a HTTP. MUST NOT llamar al caso de uso de stock (ingreso/ajuste): el movimiento `inicial` lo crea el puerto de producto en la misma transacción que la ficha (T-02-18). MUST NOT persistir imágenes, relaciones, sinónimos ni notas aunque esos archivos existan.

Cada producto MUST persistirse de forma aislada (una transacción por SKU). Si `stock_inicial > 0`, esa transacción MUST dejar un `movimiento_stock` con `tipo` `inicial`, `cantidad` y `stock_resultante` iguales a `stock_inicial`, y `producto.stock` igual a ese valor. Si `stock_inicial` es 0, MUST crear el producto con `stock` 0 y MUST NOT insertar movimiento (`cantidad` de `inicial` no puede ser 0). MUST NOT agrupar varios SKUs en una sola transacción.

Si el SKU ya existe en la base, MUST NOT crear otro producto ni otro movimiento `inicial`; MUST contarlo como importado (idempotencia). MUST NOT actualizar ficha ni reactivar inactivos en este ticket.

El resultado MUST incluir un resumen con `importados` (SKUs persistidos en esta corrida o ya presentes) y `rechazados` (líneas con issue), más la lista de issues con `archivo`, `linea`, `campo` y `mensaje` (mismo contrato que `seed-dataset`).

#### Scenario: primera carga crea producto y movimiento inicial (happy)

- **WHEN** la base ya tiene maestros `pernos`, `fixser` y `medida`
- **AND** `productos.jsonl` tiene un SKU `"PER-M8X40-ZN"` con `categoria` `"pernos"`, `marca` `"fixser"`, `precio` `"120.50"` y `stock_inicial` `"50"`
- **THEN** tras importar hay exactamente ese producto
- **AND** `stock` es `"50"`
- **AND** existe un movimiento `tipo` `inicial` con `cantidad` `"50"` y `stock_resultante` `"50"`
- **AND** el resumen tiene `importados` 1 y `rechazados` 0

#### Scenario: stock_inicial cero no inserta movimiento (happy)

- **WHEN** un producto válido declara `stock_inicial` `"0"` (o omite el campo)
- **AND** se importa
- **THEN** el producto existe con `stock` `"0"`
- **AND** no hay movimiento `inicial` de ese SKU
- **AND** el resumen lo cuenta en `importados`

#### Scenario: segunda ejecución no duplica SKU ni ledger (happy)

- **WHEN** se importan los mismos productos dos veces seguidas sobre la misma base
- **THEN** el conteo de productos es el de la primera ejecución
- **AND** cada `sku` aparece una sola vez
- **AND** el número de movimientos `inicial` no aumenta
- **AND** ambas corridas tienen `rechazados` 0

#### Scenario: dataset de desarrollo se importa completo (happy, Listo cuando)

- **WHEN** el directorio es el `seed/` del repositorio (dataset T-02-41, 180–220 SKUs, validador sin issues)
- **AND** se importan maestros y productos
- **THEN** hay tantos productos como SKUs únicos en `productos.jsonl`
- **AND** existe el SKU `"PER-M8X40-ZN"`
- **AND** `rechazados` es 0

#### Scenario: relaciones e imágenes no se persisten (happy, No hace)

- **WHEN** el directorio incluye `relaciones.jsonl`, `sinonimos.jsonl`, `notas.jsonl` e `imagenes/`
- **AND** se importan productos
- **THEN** no se crean filas de relación, sinónimo, nota ni imagen
- **AND** los productos válidos sí se persisten

### Requirement: Registro inválido se rechaza solo

Un issue de un producto (JSON malformado, `precio` no numérico, `sku` duplicado en el archivo, `categoria`/`marca` desconocida, `atributos[n].clave` desconocida, `stock_inicial` negativo, o error de dominio al crear) MUST informarse con `archivo` `productos.jsonl` (salvo archivo ausente: `linea` 0), `linea` y `campo`, MUST incrementar `rechazados`, y MUST NOT dejar ficha ni movimiento de ese SKU. MUST continuar con las líneas siguientes: los productos válidos ya persistidos MUST permanecer. MUST NOT revertir maestros. MUST NOT abortar el resto del archivo por un solo issue.

Si falta `productos.jsonl`, MUST haber un issue con ese `archivo` y `linea` 0, `importados` 0 de productos, y MUST NOT crear productos.

Un `productos.jsonl` inválido MUST NOT por sí solo abortar ni revertir la importación de maestros (T-02-42).

#### Scenario: precio no numérico se rechaza solo (unhappy, Listo cuando, HU-02-11)

- **WHEN** `productos.jsonl` tiene al menos un producto válido y otro cuyo `precio` no es decimal de dominio
- **AND** se importa
- **THEN** hay un issue con `archivo` `productos.jsonl` y `campo` `precio` en la línea del inválido
- **AND** el producto válido está persistido
- **AND** el SKU inválido no tiene ficha ni movimiento
- **AND** el resumen tiene `importados` ≥ 1 y `rechazados` ≥ 1

#### Scenario: JSONL de producto malformado no deja parciales (unhappy)

- **WHEN** una línea no vacía de `productos.jsonl` no es un objeto JSON y hay otra línea válida
- **THEN** hay un issue con `archivo` `productos.jsonl`, esa `linea` y `campo` cadena vacía
- **AND** el producto de la línea válida se persiste
- **AND** no hay producto asociado a la línea malformada

#### Scenario: categoria desconocida se rechaza solo (unhappy)

- **WHEN** un producto declara `categoria` `"no-existe"` y ese slug no está entre las categorías importadas
- **AND** hay otro producto válido en el mismo archivo
- **THEN** hay un issue con `campo` `categoria`
- **AND** el producto válido se persiste
- **AND** el inválido no

#### Scenario: archivo productos.jsonl ausente (unhappy)

- **WHEN** faltan `productos.jsonl` y los tres JSONL de maestros son válidos
- **THEN** hay un issue con `archivo` `productos.jsonl` y `linea` 0
- **AND** no se crea ningún producto
- **AND** las categorías, marcas y definiciones sí quedan persistidas

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
| `seed_import_masters_creates_categoria_marca_atributo` | `catalog` (`src/seed.rs`) | unitario (memoria) happy |
| `seed_import_masters_second_run_does_not_duplicate` | `catalog` | unitario (memoria) happy (Listo cuando) |
| `seed_import_masters_second_run_updates_nombre` | `catalog` | unitario (memoria) happy |
| `seed_import_masters_does_not_persist_productos` | `catalog` | unitario (memoria) happy (No hace) |
| `seed_import_masters_rejects_malformed_categoria_jsonl_without_writes` | `catalog` | unitario (memoria) unhappy |
| `seed_import_masters_rejects_unknown_padre_without_writes` | `catalog` | unitario (memoria) unhappy |
| `seed_import_masters_rejects_missing_marcas_file_without_writes` | `catalog` | unitario (memoria) unhappy |
| `seed_import_products_creates_producto_and_stock_inicial` | `catalog` (`src/seed.rs`) | unitario (memoria) happy |
| `seed_import_products_zero_stock_has_no_inicial_movement` | `catalog` | unitario (memoria) happy |
| `seed_import_products_second_run_does_not_duplicate` | `catalog` | unitario (memoria) happy |
| `seed_import_dev_dataset_imports_all_skus` | `catalog` | unitario (memoria) happy (Listo cuando) |
| `seed_import_products_does_not_persist_relaciones_imagenes` | `catalog` | unitario (memoria) happy (No hace) |
| `seed_import_products_rejects_non_numeric_precio_only` | `catalog` | unitario (memoria) unhappy (Listo cuando) |
| `seed_import_products_rejects_malformed_line_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_products_rejects_unknown_categoria_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_products_rejects_missing_productos_file` | `catalog` | unitario (memoria) unhappy |
| `seed_import_relations_creates_complement_edge` | `catalog` (`src/seed.rs`) | unitario (memoria) happy |
| `seed_import_relations_similar_creates_bidirectional_pair` | `catalog` | unitario (memoria) happy |
| `seed_import_relations_second_run_omits_existing` | `catalog` | unitario (memoria) happy (Hace) |
| `seed_import_relations_dev_dataset_seeds_panel_edges` | `catalog` | unitario (memoria) happy (Listo cuando) |
| `seed_import_relations_missing_file_is_ok` | `catalog` | unitario (memoria) happy |
| `seed_import_relations_does_not_persist_synonyms_notes_images` | `catalog` | unitario (memoria) happy (No hace) |
| `seed_import_relations_rejects_unknown_sku_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_relations_rejects_malformed_line_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_relations_rejects_self_relation_only` | `catalog` | unitario (memoria) unhappy |
| `vortex_seed_imports_masters_and_products_exit_0` | `vortex` (`tests/seed.rs`) | integración happy |
| `vortex_seed_twice_does_not_duplicate_products` | `vortex` | integración happy |
| `vortex_seed_dev_dataset_exit_0` | `vortex` | integración happy (Listo cuando) |
| `vortex_seed_invalid_precio_rejected_alone` | `vortex` | integración unhappy (Listo cuando) |
| `vortex_seed_imports_relations_exit_0` | `vortex` (`tests/seed.rs`) | integración happy |
| `vortex_seed_twice_does_not_duplicate_relations` | `vortex` | integración happy |
| `vortex_seed_dev_dataset_persists_panel_relations` | `vortex` | integración happy (Listo cuando) |
| `vortex_seed_invalid_relation_sku_rejected_alone` | `vortex` | integración unhappy |
| `vortex_seed_without_config_is_error` | `vortex` (`tests/cli.rs`) | integración unhappy (regresión T-02-42) |
| `vortex_seed_missing_config_file_is_not_unimplemented` | `vortex` | integración unhappy |
| `vortex_seed_locked_db_is_error` | `vortex` | integración unhappy (regresión T-02-42) |
| `vortex_top_level_stubs_print_unimplemented` sin `seed` | `vortex` (`tests/cli.rs`) | regresión stub |
