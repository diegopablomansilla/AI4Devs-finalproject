# vortex-logging Specification

## Purpose

Define el registro estructurado de Vortex Core: líneas JSON bajo `<datos>/logs/vortex.log*`, rotación diaria, retención configurable, consola legible en desarrollo y nivel tomado de `Config`, de modo que un test en directorio temporal pueda verificar formato y retención antes de Storage, HTTP o métricas.

Ticket padre: T-00-06. HU: HU-00-09.

## Requirements

### Requirement: Eventos se escriben como JSON bajo el directorio de datos

Tras una inicialización correcta del logging con un `Config` válido, cada evento emitido cuyo nivel sea mayor o igual que `logs.nivel` MUST persistirse como **una línea JSON** (un objeto por línea) en un archivo bajo `<paths.datos>/logs/` cuyo nombre empieza por `vortex.log`. El directorio `logs/` MUST crearse si no existe. El JSON MUST incluir al menos el nivel y el mensaje del evento. Si el evento se emite dentro de un span, el JSON MUST incluir el nombre de ese span. Eventos por debajo de `logs.nivel` MUST NOT aparecer en el archivo. El nivel MUST tomarse de `logs.nivel` (`trace`, `debug`, `info`, `warn` o `error`); MUST NOT usarse `RUST_LOG` como fuente de verdad.

#### Scenario: Evento info se persiste como JSON (happy)

- **WHEN** se inicializa el logging con `paths.datos` apuntando a un directorio temporal, `logs.nivel = "info"` y `logs.retencion = 14`
- **AND** se emite un evento de nivel `info` con un mensaje conocido
- **THEN** existe al menos un archivo bajo `<datos>/logs/` cuyo nombre empieza por `vortex.log`
- **AND** alguna línea de ese archivo es JSON parseable
- **AND** el objeto incluye el nivel `INFO` (o equivalente) y el mensaje conocido

#### Scenario: Evento debug se filtra con nivel info (unhappy)

- **WHEN** se inicializa el logging con `logs.nivel = "info"` en un directorio temporal
- **AND** se emite un evento de nivel `debug` con un mensaje conocido
- **THEN** ninguna línea JSON del archivo de log contiene ese mensaje

#### Scenario: Evento dentro de un span incluye el span (happy)

- **WHEN** se inicializa el logging en un directorio temporal
- **AND** se emite un evento `info` dentro de un span con nombre conocido
- **THEN** la línea JSON de ese evento incluye el nombre del span

### Requirement: Rotación diaria y retención de N archivos

Los archivos bajo `<paths.datos>/logs/` cuyo nombre empieza por `vortex.log` MUST rotar al menos una vez por día. MUST conservarse como máximo `logs.retencion` de esos archivos (el `config.example.toml` usa 14). Archivos que excedan ese tope MUST eliminarse, preferentemente los más antiguos. La retención MUST aplicarse también al inicializar (si ya hay más archivos de los permitidos, se recortan sin esperar a la próxima rotación).

#### Scenario: Retención recorta archivos de más (happy)

- **WHEN** el directorio `<datos>/logs/` contiene más de `N` archivos cuyo nombre empieza por `vortex.log`, con `N = logs.retencion` (p. ej. `N = 2` y 4 archivos)
- **AND** se inicializa el logging con esa retención
- **THEN** quedan como máximo `N` archivos cuyo nombre empieza por `vortex.log`

### Requirement: Consola legible en modo desarrollo

En **modo desarrollo** MUST escribirse además una línea de texto **legible** (no un objeto JSON) a stderr por cada evento que pase el filtro de nivel. El archivo JSON de la primera requirement MUST seguir escribiéndose. Modo desarrollo: la variable `VORTEX_ENV` vale `development` o `dev`, **o** está ausente y stderr es un terminal. Si `VORTEX_ENV` vale `production` o `prod`, MUST NOT instalarse esa salida pretty a stderr.

#### Scenario: Consola pretty en desarrollo (happy)

- **WHEN** `VORTEX_ENV=development` y se inicializa el logging con captura de stderr
- **AND** se emite un evento `info` con un mensaje conocido
- **THEN** stderr contiene ese mensaje en texto que no es un objeto JSON de una sola línea
- **AND** el archivo bajo `<datos>/logs/` sigue conteniendo la línea JSON del mismo evento

#### Scenario: Sin pretty si VORTEX_ENV es production (unhappy)

- **WHEN** `VORTEX_ENV=production` y se inicializa el logging con captura de stderr
- **AND** se emite un evento `info` con un mensaje conocido
- **THEN** stderr no contiene una línea pretty de ese evento
- **AND** el archivo JSON bajo `<datos>/logs/` sí contiene el evento

### Requirement: Fallo al crear o escribir el directorio de logs

Si no se puede crear `<paths.datos>/logs/` o no se puede abrir el archivo de log (p. ej. `paths.datos` es un archivo regular, o el directorio no es escribible), la inicialización MUST fallar. El error MUST nombrar una **clave** (`paths.datos` o `logs`) y un **motivo** legible. MUST NOT instalarse un subscriber a medias ni quedar el proceso colgado. MUST NOT arrancar Storage ni el servidor HTTP (esta inicialización es previa; en este change `main` aún no la invoca).

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
