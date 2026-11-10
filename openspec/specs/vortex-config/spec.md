# vortex-config Specification

## Purpose

Define la carga y validación del archivo TOML de Vortex Core (siete secciones, `--config` / `VORTEX_CONFIG`, ejemplo canónico) para que un arranque con configuración inválida falle con clave y motivo antes de Storage, logs o la CLI.

Ticket padre: T-00-05. Changes: T-00-05, T-00-07, T-00-10. HU: HU-00-06.

## Requirements

### Requirement: Config cubre las siete secciones del ticket

El binario `vortex` MUST cargar un documento TOML con exactamente estas secciones y claves. Las rutas son strings no vacíos; no MUST exigirse que existan en disco.

| Sección | Claves | Restricción |
|---|---|---|
| `paths` | `datos`, `modelos`, `backups`, `web` | string no vacío |
| `server` | `bind`, `hostname` | `bind` es `host:port` (dirección de socket); `hostname` string no vacío |
| `negocio` | `moneda` | código ISO 4217 de tres letras mayúsculas (`ARS`, `USD`, …) |
| `meilisearch` | `url`, `api_key` (opcional) | `url` empieza por `http://` o `https://`; `api_key` MAY omitirse o ser vacío |
| `ollama` | `url` | empieza por `http://` o `https://` |
| `backup` | `hora`, `retencion` | `hora` es `HH:MM` 24 h (`00:00`–`23:59`); `retencion` entero ≥ 1 |
| `logs` | `nivel`, `retencion` | `nivel` es `trace`, `debug`, `info`, `warn` o `error`; `retencion` entero ≥ 1 |

Una clave o sección de esa tabla que falte (salvo `meilisearch.api_key`), un valor que no cumpla la restricción, o una clave/sección no listada, MUST considerarse configuración inválida. El repositorio MUST incluir `vortex-core/config.example.toml` que cumple este esquema (valores de ejemplo para MVP en `localhost`: `server.bind = "127.0.0.1:8080"`, `server.hostname = "localhost"`, `negocio.moneda = "ARS"`, Meilisearch `http://127.0.0.1:7700`, Ollama `http://127.0.0.1:11434`).

#### Scenario: Parse de TOML válido (happy)

- **WHEN** se carga un archivo con las siete secciones y valores que cumplen la tabla (p. ej. el `config.example.toml`)
- **THEN** la operación termina sin error
- **AND** el resultado expone `paths.datos`, `paths.modelos`, `paths.backups`, `paths.web`, `server.bind`, `server.hostname`, `negocio.moneda`, `meilisearch.url`, `ollama.url`, `backup.hora`, `backup.retencion`, `logs.nivel` y `logs.retencion`

#### Scenario: El example del repo es válido (happy)

- **WHEN** se carga `vortex-core/config.example.toml` con el mismo validador que el proceso
- **THEN** la operación termina sin error

#### Scenario: api_key de Meilisearch omitida (happy)

- **WHEN** se carga un TOML válido sin la clave `meilisearch.api_key`
- **THEN** la operación termina sin error
- **AND** la clave se trata como ausente o vacía

#### Scenario: Clave o sección faltante (unhappy)

- **WHEN** se carga un TOML al que le falta `paths.datos`, `negocio`, u otra clave/sección obligatoria
- **THEN** la operación falla
- **AND** el mensaje nombra la clave o sección faltante
- **AND** indica el motivo (ausencia / requerida)

#### Scenario: Valor inválido (unhappy)

- **WHEN** se carga un TOML con `server.bind` sin puerto, `negocio.moneda = "peso"`, `backup.hora = "25:00"`, `backup.retencion = 0`, `logs.nivel = "verbose"` o una `url` que no es `http`/`https`
- **THEN** la operación falla
- **AND** el mensaje nombra la clave inválida
- **AND** indica el motivo

#### Scenario: Clave desconocida (unhappy)

- **WHEN** se carga un TOML con una clave o sección no listada (p. ej. `server.port` o `[foo]`)
- **THEN** la operación falla
- **AND** el mensaje nombra esa clave o sección

#### Scenario: TOML mal formado (unhappy)

- **WHEN** se carga un archivo que no es TOML válido (p. ej. `[paths` sin cerrar)
- **THEN** la operación falla
- **AND** el mensaje indica que el documento no se pudo parsear (clave `toml` o equivalente de sintaxis)

### Requirement: Carga desde --config o VORTEX_CONFIG

Cuando se carga `Config`, el path del archivo MUST resolverse así: si argv incluye `--config <path>` o `--config=<path>`, usa ese path; si no, usa el valor de la variable de entorno `VORTEX_CONFIG`. El flag MUST ganar si ambos están presentes. El binario MUST exponer `--config` como opción global de la CLI (capability `vortex-cli`). `--help`, `-h` y los subcomandos stub MUST NOT exigir que el archivo exista ni que sea válido. `vortex serve` MUST cargar `Config` con esa resolución: si no hay flag ni variable, o el archivo no existe / no es legible, MUST ser un error de ese comando (capability `vortex-api`). MUST NOT recargar el archivo en caliente. `Config::from_path` MUST seguir fallando si el path no existe o no es legible.

#### Scenario: --config con archivo válido (happy)

- **WHEN** se carga `Config` desde un archivo válido vía `--config <archivo-válido.toml>`
- **THEN** la operación termina sin error

#### Scenario: VORTEX_CONFIG con archivo válido (happy)

- **WHEN** `VORTEX_CONFIG` apunta a un archivo válido y argv no incluye `--config`
- **AND** se carga `Config` con esa resolución
- **THEN** la operación termina sin error

#### Scenario: --config gana sobre VORTEX_CONFIG (happy)

- **WHEN** `VORTEX_CONFIG` apunta a un archivo inválido y `--config` apunta a uno válido
- **AND** se carga `Config` con esa resolución
- **THEN** la operación termina sin error
- **AND** el documento usado es el del flag

#### Scenario: Sin path de configuración (unhappy)

- **WHEN** se intenta cargar `Config` sin `--config` y sin `VORTEX_CONFIG`
- **THEN** la operación falla
- **AND** el mensaje indica que falta el path (`--config` o `VORTEX_CONFIG`)

#### Scenario: Archivo inexistente (unhappy)

- **WHEN** se carga `Config` con `--config` apuntando a un path que no existe
- **THEN** la operación falla
- **AND** el mensaje nombra el path
- **AND** indica que el archivo no existe o no se pudo leer

#### Scenario: CLI stub no exige el archivo (happy)

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml reindex`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`

### Requirement: Configuración inválida aborta el proceso con clave y motivo

Si se invoca la carga de `Config` y el archivo es ilegible, TOML inválido o no cumple el esquema, MUST devolverse un error cuyo mensaje incluye la **clave** (path de sección/campo, o `toml` / `--config` / `VORTEX_CONFIG` cuando aplique) y el **motivo**. MUST NOT arrancar Storage, el servidor HTTP ni otros subsistemas. `vortex serve` MUST cargar `Config` y, si el TOML es inválido, MUST abortar con clave y motivo (MUST NOT escribir `no implementado`). Los subcomandos stub MUST NOT cargar `Config`: un TOML inválido pasado a `vortex --config … reindex` MUST NOT sustituir el mensaje `no implementado`. Un archivo válido ya no implica que el proceso termine con código 0 (hace falta un subcomando; los stubs salen 1; `serve` queda en ejecución hasta la señal de apagado).

#### Scenario: Configuración inválida aborta con clave y motivo (unhappy)

- **WHEN** se carga `Config` desde un TOML con un valor ilegal (p. ej. `negocio.moneda = "peso"`)
- **THEN** la operación falla
- **AND** el mensaje contiene la clave (`negocio.moneda` o equivalente)
- **AND** el mensaje contiene un motivo legible

#### Scenario: Configuración válida termina 0 (happy)

- **WHEN** se carga `Config` desde `vortex-core/config.example.toml`
- **THEN** la operación termina sin error

#### Scenario: Configuración válida no arranca un stub como éxito (happy)

- **WHEN** se ejecuta `vortex --config` apuntando a `vortex-core/config.example.toml` con subcomando `reindex`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`
- **AND** el proceso no queda en ejecución
