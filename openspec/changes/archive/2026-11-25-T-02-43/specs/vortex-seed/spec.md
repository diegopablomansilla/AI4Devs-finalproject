# Spec Delta

Ticket padre: T-02-43. HU: HU-02-11.

## ADDED Requirements

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

## MODIFIED Requirements

### Requirement: vortex seed abre la base en modo directo

`vortex seed` MUST dejar de responder `no implementado`. MUST aceptar un directorio opcional (`vortex seed [DIR]`); si se omite, MUST usar `seed` relativo al directorio de trabajo. MUST cargar `Config` con `--config` o `VORTEX_CONFIG` (mismo criterio que `vortex serve`). MUST abrir Storage en `<paths.datos>/db`, aplicar migraciones, importar maestros y a continuación productos. MUST escribir un resumen con importados y rechazados de productos. MUST terminar con código 0 si la importación de maestros no tiene issues y `rechazados` de productos es 0. MUST terminar con código distinto de 0 si hay issues de maestros o `rechazados` ≥ 1; en el segundo caso MUST NOT revertir los productos válidos ya persistidos. MUST NOT arrancar el servidor HTTP ni sidecars. MUST NOT usar el canal Unix socket (T-00-22 es v2).

Si faltan `--config` y `VORTEX_CONFIG`, MUST terminar con código distinto de 0 y MUST NOT escribir `no implementado`. Si el archivo de configuración no existe o es inválido, MUST terminar ≠ 0 y MUST NOT importar. Si no se puede abrir la base (lock de RocksDB porque `vortex serve` está corriendo, o ruta inaccesible), MUST terminar ≠ 0, MUST informar en stderr que no se pudo abrir la base, y MUST NOT escribir `no implementado`.

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

- **WHEN** se ejecuta `vortex seed` dos veces seguidas con el mismo `--config` y el mismo `DIR` válido (maestros + productos), con el servidor detenido
- **THEN** ambas invocaciones terminan con código 0
- **AND** el número de categorías, marcas, definiciones y productos no aumenta en la segunda
- **AND** el número de movimientos `inicial` no aumenta en la segunda

#### Scenario: seed dataset de desarrollo exit 0 (happy, Listo cuando)

- **WHEN** se ejecuta `vortex --config <toml> seed` apuntando al `seed/` del repositorio, servidor detenido
- **THEN** el proceso termina con código 0
- **AND** hay entre 180 y 220 productos persistidos
- **AND** el resumen tiene `rechazados` 0

#### Scenario: seed con un producto inválido sale distinto de 0 y aísla el rechazo (unhappy, Listo cuando)

- **WHEN** se copia el dataset de desarrollo y se agrega a `productos.jsonl` un objeto cuyo `precio` no es decimal de dominio
- **AND** se ejecuta `vortex --config <toml> seed <copia>`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr informa `productos.jsonl`, línea y campo `precio`
- **AND** los productos válidos de la copia están persistidos
- **AND** el SKU del registro inválido no está persistido

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
| `seed_import_products_creates_producto_and_stock_inicial` | `catalog` (`src/seed.rs`) | unitario (memoria) happy |
| `seed_import_products_zero_stock_has_no_inicial_movement` | `catalog` | unitario (memoria) happy |
| `seed_import_products_second_run_does_not_duplicate` | `catalog` | unitario (memoria) happy |
| `seed_import_dev_dataset_imports_all_skus` | `catalog` | unitario (memoria) happy (Listo cuando) |
| `seed_import_products_does_not_persist_relaciones_imagenes` | `catalog` | unitario (memoria) happy (No hace) |
| `seed_import_products_rejects_non_numeric_precio_only` | `catalog` | unitario (memoria) unhappy (Listo cuando) |
| `seed_import_products_rejects_malformed_line_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_products_rejects_unknown_categoria_only` | `catalog` | unitario (memoria) unhappy |
| `seed_import_products_rejects_missing_productos_file` | `catalog` | unitario (memoria) unhappy |
| `vortex_seed_imports_masters_and_products_exit_0` | `vortex` (`tests/seed.rs`) | integración happy |
| `vortex_seed_twice_does_not_duplicate_products` | `vortex` | integración happy |
| `vortex_seed_dev_dataset_exit_0` | `vortex` | integración happy (Listo cuando) |
| `vortex_seed_invalid_precio_rejected_alone` | `vortex` | integración unhappy (Listo cuando) |
| `vortex_seed_without_config_is_error` | `vortex` (`tests/cli.rs`) | integración unhappy (regresión T-02-42) |
| `vortex_seed_locked_db_is_error` | `vortex` | integración unhappy (regresión T-02-42) |
