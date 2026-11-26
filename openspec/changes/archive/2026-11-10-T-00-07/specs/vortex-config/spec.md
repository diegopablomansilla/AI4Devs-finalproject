# Spec Delta

Ticket padre: T-00-07. Change: T-00-07. HU: HU-00-06.

## MODIFIED Requirements

### Requirement: Carga desde --config o VORTEX_CONFIG

Cuando se carga `Config`, el path del archivo MUST resolverse así: si argv incluye `--config <path>` o `--config=<path>`, usa ese path; si no, usa el valor de la variable de entorno `VORTEX_CONFIG`. El flag MUST ganar si ambos están presentes. El binario MUST exponer `--config` como opción global de la CLI (capability `vortex-cli`). `--help`, `-h` y los subcomandos stub MUST NOT exigir que el archivo exista ni que sea válido. Si un comando implementado (T-00-10+) carga `Config` y no hay flag ni variable, o el archivo no existe / no es legible, MUST ser un error de ese comando. MUST NOT recargar el archivo en caliente. `Config::from_path` MUST seguir fallando si el path no existe o no es legible.

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

- **WHEN** se ejecuta `vortex --config /nonexistent/vortex-config.toml serve`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`

### Requirement: Configuración inválida aborta el proceso con clave y motivo

Si se invoca la carga de `Config` y el archivo es ilegible, TOML inválido o no cumple el esquema, MUST devolverse un error cuyo mensaje incluye la **clave** (path de sección/campo, o `toml` / `--config` / `VORTEX_CONFIG` cuando aplique) y el **motivo**. MUST NOT arrancar Storage, el servidor HTTP ni otros subsistemas. En este change el proceso MUST NOT cargar `Config` al despachar un subcomando stub: un TOML inválido pasado a `vortex --config … serve` MUST NOT sustituir el mensaje `no implementado`. Un archivo válido ya no implica que el proceso termine con código 0 (hace falta un subcomando; los stubs salen 1).

#### Scenario: Configuración inválida aborta con clave y motivo (unhappy)

- **WHEN** se carga `Config` desde un TOML con un valor ilegal (p. ej. `negocio.moneda = "peso"`)
- **THEN** la operación falla
- **AND** el mensaje contiene la clave (`negocio.moneda` o equivalente)
- **AND** el mensaje contiene un motivo legible

#### Scenario: Configuración válida termina 0 (happy)

- **WHEN** se carga `Config` desde `vortex-core/config.example.toml`
- **THEN** la operación termina sin error

#### Scenario: Configuración válida no arranca un stub como éxito (happy)

- **WHEN** se ejecuta `vortex --config` apuntando a `vortex-core/config.example.toml` con subcomando `serve`
- **THEN** el proceso termina con código 1
- **AND** stderr contiene `no implementado`
- **AND** el proceso no queda en ejecución
