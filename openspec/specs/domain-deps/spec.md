# domain-deps Specification

## Purpose
Verifica de forma automática que el árbol de dependencias `normal` y `build` del crate `domain` no incluye SDKs de motores ni otros crates del workspace, para preservar la regla hexagonal de HU-00-01.

## Requirements

### Requirement: Script de verificación del árbol de domain

El repositorio MUST contener `scripts/check-domain-deps.sh`, ejecutable, que analiza el árbol de dependencias del package `domain` con aristas `normal` y `build` (`cargo tree -p domain -e normal,build`) usando el manifiesto `vortex-core/Cargo.toml` bajo la raíz de verificación. La raíz MUST ser `CHECK_ROOT` si está definida, o la raíz del repositorio en caso contrario. El script MUST terminar con código 0 cuando el árbol no viola las reglas de esta spec.

#### Scenario: Script presente

- **WHEN** se inspecciona `scripts/check-domain-deps.sh`
- **THEN** el archivo existe y es ejecutable

#### Scenario: domain limpio

- **WHEN** se ejecuta el script sobre el repositorio (crate `domain` sin SDKs de motores ni crates del workspace en su árbol `normal,build`)
- **THEN** termina con código 0

#### Scenario: falta el package domain

- **WHEN** el manifiesto de workspace no contiene un package `domain`
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje indica que falta `domain`

### Requirement: SDKs de motores prohibidos en el árbol de domain

El árbol `normal,build` de `domain` MUST NOT incluir ninguno de estos packages: `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort`, `whisper-rs`. Si alguno aparece (dependencia directa o transitiva), el script MUST terminar con código distinto de 0 y el mensaje MUST nombrar el package ofensor.

#### Scenario: surrealdb aparece en el árbol

- **WHEN** el árbol `normal,build` de `domain` incluye el package `surrealdb`
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra `surrealdb`

#### Scenario: otro SDK prohibido aparece en el árbol

- **WHEN** el árbol `normal,build` de `domain` incluye `meilisearch-sdk`, `reqwest`, `ort` o `whisper-rs`
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra el package ofensor

#### Scenario: SDK prohibido solo como dependencia transitiva

- **WHEN** `domain` no declara el package prohibido de forma directa
- **AND** el árbol `normal,build` lo incluye a través de un crate intermedio
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra el package prohibido

### Requirement: Crates del workspace prohibidos en el árbol de domain

El árbol `normal,build` de `domain` MUST NOT incluir ningún package del mismo workspace distinto de `domain`. El package `domain` como raíz del árbol MUST NOT considerarse una violación. Si aparece otro package del workspace, el script MUST terminar con código distinto de 0 y el mensaje MUST nombrar ese package.

#### Scenario: domain depende de un crate del workspace

- **WHEN** el árbol `normal,build` de `domain` incluye un package del workspace distinto de `domain` (por ejemplo `storage`)
- **THEN** el script termina con código distinto de 0
- **AND** el mensaje nombra ese package

#### Scenario: domain es la raíz del árbol

- **WHEN** el único package del workspace en el árbol es `domain`
- **THEN** el script no falla por esa aparición de `domain`

### Requirement: Aristas de desarrollo fuera del chequeo

El análisis MUST usar las aristas `normal` y `build`, no `dev`. Un package prohibido que solo figure como `[dev-dependencies]` de `domain` MUST NOT hacer fallar el script.

#### Scenario: SDK prohibido solo en dev-dependencies

- **WHEN** `domain` declara `surrealdb` únicamente en `[dev-dependencies]` y no aparece en el árbol `normal,build`
- **THEN** el script termina con código 0
