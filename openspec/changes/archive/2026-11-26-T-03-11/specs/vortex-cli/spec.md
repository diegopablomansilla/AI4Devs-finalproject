# Spec Delta

Ticket padre: T-03-11. HU: HU-03-02.

## MODIFIED Requirements

### Requirement: --config es una opción global

El binario MUST exponer `--config <path>` y `--config=<path>` como opción **global** (válida antes o después del subcomando). `vortex --help` MUST mencionar `--config`. Los subcomandos stub MUST NOT abrir ni validar el archivo. `vortex serve`, `vortex seed` y `vortex reindex` MUST cargar `Config` al despacharse (`vortex-api` / `vortex-config` / `vortex-seed` / `vortex-reindex`).

#### Scenario: --config aparece en la ayuda raíz (happy)

- **WHEN** se ejecuta `vortex --help`
- **THEN** la salida de ayuda menciona `--config`

#### Scenario: stub ignora un --config inexistente (happy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml backup`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`
- **AND** stderr no trata la ausencia del archivo como el error principal (no exige que el TOML exista)

#### Scenario: seed no ignora un --config inexistente (unhappy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml seed`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: reindex no ignora un --config inexistente (unhappy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml reindex`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

### Requirement: Subcomandos no implementados responden no implementado con código 1

Cada una de estas invocaciones MUST escribir `no implementado` en stderr y MUST terminar con código **1** (ni 0 ni otro código de uso):

- `vortex backup`
- `vortex cert`
- `vortex models`
- `vortex verify`
- `vortex service install`
- `vortex service uninstall`

MUST NOT arrancar el servidor HTTP, Storage, sidecars, `launchd` ni ninguna lógica de producto. MUST NOT exigir un archivo de configuración para producir esa respuesta. `vortex serve` MUST NOT formar parte de esta lista (capability `vortex-api`). `vortex seed` MUST NOT formar parte de esta lista (capability `vortex-seed`). `vortex reindex` MUST NOT formar parte de esta lista (capability `vortex-reindex`).

#### Scenario: vortex serve no implementado (happy / stub)

- **WHEN** se ejecuta `vortex serve` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`
- **AND** el proceso no queda en ejecución

#### Scenario: el resto de subcomandos de primer nivel son stub (happy / stub)

- **WHEN** se ejecuta `vortex` con subcomando `backup`, `cert`, `models` o `verify`
- **THEN** cada invocación termina con código 1
- **AND** stderr contiene `no implementado`

#### Scenario: service install y uninstall son stub (happy / stub)

- **WHEN** se ejecuta `vortex service install` o `vortex service uninstall`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`

#### Scenario: vortex seed no es stub (happy)

- **WHEN** se ejecuta `vortex seed` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`

#### Scenario: vortex reindex no es stub (happy)

- **WHEN** se ejecuta `vortex reindex` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** el proceso termina con código distinto de 0
- **AND** stderr no contiene `no implementado`
