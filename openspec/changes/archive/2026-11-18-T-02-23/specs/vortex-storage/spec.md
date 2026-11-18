# Spec Delta

Ticket padre: T-02-23. Change: T-02-23. HU: HU-02-03.

## ADDED Requirements

### Requirement: Adaptador AttributeUsage

El crate `storage` MUST implementar el puerto `AttributeUsage` (`valores`) contra la tabla `producto` ya migrada. MUST devolver cada `atributos[*].valor` cuyo `def` coincida con el `id` pedido, en productos **activos e inactivos**. MUST NOT filtrar por `producto.activo`. MUST NOT escribir filas, MUST NOT emitir outbox y MUST NOT aplicar `attribute_in_use`. MUST NOT mapear a HTTP. `id` sin coincidencias MUST devolver lista vacía.

`AtributoDefRepo::update` MUST seguir persistiendo el cambio de `clave`/`tipo` que reciba: la regla de occupancy MUST vivir en `catalog`, no en este adaptador.

#### Scenario: valores de medida en un producto (happy, Listo cuando)

- **WHEN** existe la definición `"medida"` y un producto con `atributos` `{ def: medida, valor: "M8" }`
- **AND** se llama `valores` con el `id` de `"medida"`
- **THEN** la operación termina sin error
- **AND** el resultado contiene un `ValorAtributo::Texto` `"M8"`

#### Scenario: definición sin productos (happy)

- **WHEN** existe `"medida"` y ningún producto la referencia
- **AND** se llama `valores` con su `id`
- **THEN** la operación termina sin error
- **AND** el resultado es una lista vacía

#### Scenario: producto inactivo sigue contando (happy)

- **WHEN** un producto inactivo tiene el atributo `"medida"` con valor `"M8"`
- **AND** se llama `valores` con el `id` de `"medida"`
- **THEN** el resultado contiene `"M8"`

#### Scenario: AttributeDefRepo update no aplica occupancy (happy)

- **WHEN** un producto usa `"medida"` y se llama `AtributoDefRepo::update` cambiando `clave` a `"medida_mm"`
- **THEN** la operación termina sin error
- **AND** la fila persistida tiene `clave` `"medida_mm"`
- **AND** el error no es `attribute_in_use`

## Tests previstos

| Test | Crate | Tipo |
|---|---|---|
| `attribute_usage_returns_medida_m8` | `storage` | integración, happy, Listo cuando |
| `attribute_usage_empty_when_unused` | `storage` | integración, happy |
| `attribute_usage_includes_inactive_product` | `storage` | integración, happy |
| `attribute_def_repo_update_does_not_apply_in_use` | `storage` | integración, happy (non-goal del adaptador) |
