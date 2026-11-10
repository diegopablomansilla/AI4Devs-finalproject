# vortex-cli Specification

## Purpose

Define el esqueleto de la CLI de Vortex Core: subcomandos operativos, flag global `--config` y respuesta uniforme «no implementado» (exit 1) para los que aún no tienen lógica (salvo `serve`, implementado en T-00-10), de modo que `vortex --help` liste el contrato completo.

Ticket padre: T-00-07. Changes: T-00-07, T-00-10. HU: HU-00-06.

## Requirements

### Requirement: vortex --help lista todos los subcomandos

El binario `vortex` MUST aceptar `--help` y `-h` en el nivel raíz. Esa invocación MUST terminar con código 0 y MUST escribir en stdout (o en el canal de ayuda del proceso) los ocho subcomandos de primer nivel, cada uno identificable por su nombre:

- `serve`
- `reindex`
- `backup`
- `cert`
- `models`
- `seed`
- `verify`
- `service`

MUST NOT exigirse archivo de configuración ni variable `VORTEX_CONFIG` para mostrar esa ayuda. `vortex service --help` MUST terminar con código 0 y MUST listar los subcomandos anidados `install` y `uninstall`.

#### Scenario: vortex --help lista los ocho subcomandos (happy)

- **WHEN** se ejecuta `vortex --help` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código 0
- **AND** la salida de ayuda contiene `serve`, `reindex`, `backup`, `cert`, `models`, `seed`, `verify` y `service`

#### Scenario: vortex -h también lista los subcomandos (happy)

- **WHEN** se ejecuta `vortex -h` sin archivo de configuración
- **THEN** el proceso termina con código 0
- **AND** la salida de ayuda contiene los mismos ocho nombres

#### Scenario: vortex service --help lista install y uninstall (happy)

- **WHEN** se ejecuta `vortex service --help`
- **THEN** el proceso termina con código 0
- **AND** la salida de ayuda contiene `install` y `uninstall`

### Requirement: --config es una opción global

El binario MUST exponer `--config <path>` y `--config=<path>` como opción **global** (válida antes o después del subcomando). `vortex --help` MUST mencionar `--config`. Los subcomandos stub MUST NOT abrir ni validar el archivo. `vortex serve` MUST cargar `Config` al despacharse (capability `vortex-api` / `vortex-config`).

#### Scenario: --config aparece en la ayuda raíz (happy)

- **WHEN** se ejecuta `vortex --help`
- **THEN** la salida de ayuda menciona `--config`

#### Scenario: stub ignora un --config inexistente (happy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml reindex`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`
- **AND** stderr no trata la ausencia del archivo como el error principal (no exige que el TOML exista)

### Requirement: Subcomandos no implementados responden no implementado con código 1

Cada una de estas invocaciones MUST escribir `no implementado` en stderr y MUST terminar con código **1** (ni 0 ni otro código de uso):

- `vortex reindex`
- `vortex backup`
- `vortex cert`
- `vortex models`
- `vortex seed`
- `vortex verify`
- `vortex service install`
- `vortex service uninstall`

MUST NOT arrancar el servidor HTTP, Storage, sidecars, `launchd` ni ninguna lógica de producto. MUST NOT exigir un archivo de configuración para producir esa respuesta. `vortex serve` MUST NOT formar parte de esta lista (capability `vortex-api`).

#### Scenario: vortex serve no implementado (happy / stub)

- **WHEN** se ejecuta `vortex serve` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`
- **AND** el proceso no queda en ejecución

#### Scenario: el resto de subcomandos de primer nivel son stub (happy / stub)

- **WHEN** se ejecuta `vortex` con subcomando `reindex`, `backup`, `cert`, `models`, `seed` o `verify`
- **THEN** cada invocación termina con código 1
- **AND** stderr contiene `no implementado`

#### Scenario: service install y uninstall son stub (happy / stub)

- **WHEN** se ejecuta `vortex service install` o `vortex service uninstall`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`

### Requirement: Argv inválido no se confunde con un stub

Si falta el subcomando, el subcomando no está en la lista del ticket, o `service` se invoca sin `install` ni `uninstall`, el proceso MUST terminar con código distinto de 0 y MUST NOT escribir `no implementado` (ese mensaje queda reservado a subcomandos reconocidos). La ayuda de uso MAY emitirse en stderr.

#### Scenario: sin subcomando (unhappy)

- **WHEN** se ejecuta `vortex` sin subcomando (con o sin `--config`)
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: subcomando desconocido (unhappy)

- **WHEN** se ejecuta `vortex definitely-not-a-command`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: service sin acción (unhappy)

- **WHEN** se ejecuta `vortex service` sin `install` ni `uninstall`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`
