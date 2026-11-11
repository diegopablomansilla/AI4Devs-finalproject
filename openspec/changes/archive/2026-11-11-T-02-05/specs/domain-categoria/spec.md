# Spec Delta

## ADDED Requirements

### Requirement: set_activo en el puerto CategoryRepo

El crate `domain` MUST exponer `CategoryRepo::set_activo(id, activo)` que devuelve `Categoria` o `DomainError`. MUST NOT implementar adaptador ni consultar productos. MUST NOT mapear a HTTP.

#### Scenario: El puerto declara set_activo (happy)

- **WHEN** se inspecciona `CategoryRepo` en compilación
- **THEN** el trait incluye `set_activo` con `Id<Categoria>` y `bool`

#### Scenario: Tests del puerto incluyen set_activo (happy)

- **WHEN** se ejecuta `cargo test -p domain -- category_repo_trait_exposes_set_activo`
- **THEN** el test pasa
