# Spec Delta

Ticket padre: T-02-23. Change: T-02-23. HU: HU-02-03 (escenario «Atributo en uso»).

## ADDED Requirements

### Requirement: Puerto AttributeUsage

El crate `domain` MUST exponer el puerto `AttributeUsage` como *trait* con la operación `valores(id: Id<AtributoDef>)` que MUST devolver `Vec<ValorAtributo>` o `DomainError`. El resultado MUST incluir los valores de **todos** los productos que referencian ese `def` en `atributos[*].def`, activos e inactivos. Un `id` sin productos MUST devolver una lista vacía (no `not_found`). Los `id` MUST ser ULID sin prefijo de tabla. MUST NOT persistir, MUST NOT mapear a HTTP y MUST NOT aplicar la regla `attribute_in_use` (eso es `catalog`). MUST NOT depender de `surrealdb` ni de otros crates del workspace. `AttributeDefRepo` MUST permanecer sin occupancy.

#### Scenario: El puerto declara valores (happy)

- **WHEN** se inspecciona el puerto `AttributeUsage` en `domain`
- **THEN** expone `valores`
- **AND** el resultado de éxito es `Vec<ValorAtributo>`
- **AND** el resultado de error es `DomainError`

#### Scenario: domain no consulta SurrealDB (unhappy)

- **WHEN** se inspeccionan las dependencias `normal,build` del crate `domain`
- **THEN** no incluyen `surrealdb` ni el crate `storage`

## Tests previstos

| Test | Crate / archivo | Tipo |
|---|---|---|
| `attribute_usage_trait_exposes_valores` | `domain` (`src/atributo_def.rs`) | unitario happy (dummy) |
