# rust-toolchain Specification

## Purpose

Fija la toolchain Rust del repositorio, el formato con rustfmt y las exclusiones Git de artefactos de build, modelos descargados y datos locales, para que el workspace de Vortex Core sea reproducible.

Ticket padre: T-00-01. Change: T-00-01-01. HU: HU-00-01.

## Requirements

### Requirement: Toolchain Rust estable fijada

El repositorio MUST declarar en su raíz un archivo `rust-toolchain.toml` que fije el canal estable a la versión `1.99.0` e incluya los componentes `rustfmt` y `clippy`. Quien clone el repositorio y use rustup MUST obtener esa versión sin elegir canal a mano.

#### Scenario: Canal fijado en la raíz

- **WHEN** se inspecciona `rust-toolchain.toml` en la raíz del repositorio
- **THEN** el archivo existe
- **AND** declara canal `1.99.0`
- **AND** declara los componentes `rustfmt` y `clippy`

#### Scenario: Falta el canal fijado

- **WHEN** `rust-toolchain.toml` no existe o no declara un canal `1.99.0`
- **THEN** la verificación de toolchain falla con código de salida distinto de 0
- **AND** el mensaje indica que falta el canal fijado

### Requirement: Formato rustfmt de proyecto

El repositorio MUST declarar en su raíz un archivo `rustfmt.toml` con edición `2024`, de modo que el formateo no dependa de la configuración local del desarrollador.

#### Scenario: rustfmt con edición 2024

- **WHEN** se inspecciona `rustfmt.toml` en la raíz del repositorio
- **THEN** el archivo existe
- **AND** declara `edition = "2024"`

#### Scenario: Falta la edición de rustfmt

- **WHEN** `rustfmt.toml` no existe o no declara edición `2024`
- **THEN** la verificación de toolchain falla con código de salida distinto de 0
- **AND** el mensaje indica que falta la edición de rustfmt

### Requirement: Git ignora modelos, target y datos locales

El `.gitignore` del repositorio MUST excluir:

- el directorio `models/` (modelos locales descargados, ver [2.8](../../../docs/2-arquitectura.md#28-estructura-de-repositorio))
- directorios `target/` de Cargo a cualquier profundidad (no solo `/target/` en la raíz)
- el directorio de datos locales de desarrollo `data/`

Los archivos de documentación, specs y código fuente MUST seguir versionándose.

#### Scenario: Artefactos locales no se versionan

- **WHEN** existen rutas `models/ejemplo.bin`, `vortex-core/target/debug/libdomain.rlib` y `data/db/LOCK`
- **THEN** Git las ignora
- **AND** `.gitignore` no ignora `docs/7-tickets.md` ni `openspec/config.yaml`

#### Scenario: Ignore incompleto

- **WHEN** `.gitignore` no excluye `models/`, `target/` a cualquier profundidad, o `data/`
- **THEN** la verificación de toolchain falla con código de salida distinto de 0
- **AND** el mensaje nombra el patrón que falta

### Requirement: Verificación ejecutable sin workspace Cargo

El repositorio MUST ofrecer un comando de verificación que compruebe las tres reglas anteriores sin requerir `Cargo.toml` ni `cargo`. El comando MUST salir 0 si el árbol es válido.

#### Scenario: Árbol válido

- **WHEN** existen `rust-toolchain.toml` y `rustfmt.toml` correctos y `.gitignore` con los patrones exigidos
- **THEN** el comando de verificación termina con código 0

#### Scenario: Árbol inválido

- **WHEN** falta cualquiera de canal, edición rustfmt o un patrón de ignore exigido
- **THEN** el comando de verificación termina con código distinto de 0
