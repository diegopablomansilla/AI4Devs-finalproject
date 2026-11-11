# domain-categoria (delta)

## ADDED Requirements

### Requirement: UpdateCategoria y update en el puerto

El crate `domain` MUST exponer `UpdateCategoria { id, nombre, padre }` y `CategoryRepo::update`. MUST NOT implementar adaptador.

#### Scenario: El puerto declara update (happy)

- **WHEN** se inspecciona `CategoryRepo` en compilación
- **THEN** el trait incluye `update` con `UpdateCategoria`

#### Scenario: Tests del puerto incluyen update (happy)

- **WHEN** se ejecuta `cargo test -p domain -- category_repo_trait_exposes_update`
- **THEN** el test pasa
