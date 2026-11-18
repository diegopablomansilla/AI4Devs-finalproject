# Spec Delta

## ADDED Requirements

### Requirement: CategoriaRepo inyectable en AppState

El adaptador `CategoriaRepo` MUST poder vivir tanto como el proceso (`'static`) y MUST ser `Send + Sync` para inyectarse, vía `CategoryService`, en un `AppState` clonable. MUST poseer un handle clonable de `Db` (p. ej. `Arc<Db>`). MUST NOT exigir un préstamo de `Db` limitado al stack de `vortex serve`. El comportamiento de `create`/`get`/`list_all`/`update`/`set_activo` MUST NOT cambiar.

#### Scenario: el adaptador sobrevive al stack de serve (happy)

- **WHEN** `vortex serve` abre `Db`, construye `CategoriaRepo` y lo inyecta en `AppState`
- **THEN** el router atiende `GET /api/v1/categories` sin que el `Db` quede solo en una variable local no compartida

#### Scenario: préstamo corto no basta (unhappy)

- **WHEN** el adaptador solo guardara `&Db` del stack de `serve_config`
- **THEN** no se podría construir un `AppState` `'static` con el servicio de categorías
