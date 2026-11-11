# Spec Delta

Ticket padre: T-00-20. Change: T-00-20. HU: HU-00-06 (enganche del subscriber; el formato JSON sigue siendo HU-00-09).

## ADDED Requirements

### Requirement: vortex serve instala el subscriber al arrancar

Tras cargar un `Config` válido, `vortex serve` MUST instalar el subscriber de logging **antes** de abrir Storage y **antes** de enlazar HTTP. Los stubs de los demás subcomandos MUST NOT instalar el subscriber ni crear `<datos>/logs/` solo por invocarse. Un arranque exitoso de `serve` MUST dejar al menos un archivo `vortex.log*` bajo `<paths.datos>/logs/` cuando se emiten los eventos de boot.

#### Scenario: serve escribe JSON de boot bajo datos/logs (happy)

- **WHEN** se ejecuta `vortex serve --config <toml-válido>` con `paths.datos` en un directorio temporal y el proceso llega a escuchar HTTP
- **THEN** existe al menos un archivo bajo `<datos>/logs/` cuyo nombre empieza por `vortex.log`
- **AND** alguna línea es JSON parseable

#### Scenario: un stub no instala logging (unhappy)

- **WHEN** se ejecuta `vortex reindex` (u otro stub reconocido) con `--config` apuntando a un TOML válido cuyo `paths.datos` es un directorio temporal vacío
- **THEN** el proceso imprime `no implementado` y termina con código distinto de 0
- **AND** no se exige que exista `<datos>/logs/` como resultado de esa invocación

## MODIFIED Requirements

### Requirement: Fallo al crear o escribir el directorio de logs

Si no se puede crear `<paths.datos>/logs/` o no se puede abrir el archivo de log (p. ej. `paths.datos` es un archivo regular, o el directorio no es escribible), la inicialización MUST fallar. El error MUST nombrar una **clave** (`paths.datos` o `logs`) y un **motivo** legible. MUST NOT instalarse un subscriber a medias ni quedar el proceso colgado. MUST NOT arrancar Storage ni el servidor HTTP. `vortex serve` MUST invocar esta inicialización tras cargar `Config`; si falla, MUST terminar con código distinto de 0 y MUST NOT quedar escuchando HTTP.

#### Scenario: datos no es un directorio (unhappy)

- **WHEN** `paths.datos` apunta a un archivo regular
- **AND** se intenta inicializar el logging
- **THEN** la operación falla
- **AND** el mensaje incluye la clave `paths.datos` o `logs`
- **AND** el mensaje incluye un motivo (no es directorio / no se pudo crear / no se pudo escribir)

#### Scenario: directorio de logs no escribible (unhappy)

- **WHEN** `<paths.datos>/logs` no puede crearse o no admite escritura
- **AND** se intenta inicializar el logging
- **THEN** la operación falla
- **AND** el mensaje incluye clave y motivo

#### Scenario: serve con datos archivo no escucha HTTP (unhappy)

- **WHEN** se ejecuta `vortex serve --config` con `paths.datos` apuntando a un archivo regular y `server.bind` libre
- **THEN** el proceso termina con código distinto de 0
- **AND** el proceso no queda escuchando HTTP
