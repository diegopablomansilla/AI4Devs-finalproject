# Design

## Context

En `develop`, `api::build_router` no registra `/health` (la spec vigente lo prohíbe). `AppState` es un `Arc` vacío. `supervisor` es un lib placeholder sin `[dependencies]`. `vortex serve` hace `build_router(AppState::new())`. El contrato (`docs/openapi.yaml` `GET /health`, esquema `Health`) y HU-00-05 ya existen. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: hexagonal (`api` no depende de `supervisor`; `domain` no conoce HTTP); allowlist de `check-workspace.sh`; sin probes de sidecars; ~2 h; MVP del curso.

## Goals / Non-Goals

**Goals:**

- Puerto + agregación pura en `domain`; `HealthRegistry` in-memory en `supervisor`
- `GET /health` 200/503 con JSON `Health`; composition root en `vortex serve`
- Allowlist `supervisor` → `domain` y `vortex` → `supervisor`; tests que cierran el Listo cuando (cuatro estados + 503 `starting`)

**Non-Goals (diseño):**

- Reloj inyectable / fake clock para `uptimeSeconds` (basta `Instant` + assert ≥ 0)
- `outboxLag` / `failedEvents` en `sync` (T-03-09)
- Trait async / `async_trait` (el snapshot es sincrónico)
- Cambiar `docs/openapi.yaml`

## Decisions

### 1. Agregación pura en domain; registro en supervisor

**Elección:** `domain::health` expone enums (`ComponentId`, `ComponentStatus`, `AggregateStatus`), `ComponentHealth`, `HealthSnapshot` y `fn aggregate(components) -> AggregateStatus` con la prioridad de la spec. `HealthReport: Send + Sync { fn snapshot(&self) -> HealthSnapshot }`. `supervisor::HealthRegistry` guarda un mapa de los seis componentes, `version: String` y `started_at: Instant`; `set(id, status)`; implementa el puerto. Tests del agregador en `domain`; tests de publicación en `supervisor`.

**Por qué:** el Listo cuando pide tests del agregador; una función pura cubre los cuatro estados sin HTTP. El ticket nombra el puerto en `domain` y el registro en `supervisor`. Sin `tokio` ni crates extra: `std::sync::RwLock`.

**Alternativa:** agregar en `supervisor` solamente. Se descartó: duplicaría la regla y `api` no podría testear el agregado sin depender de `supervisor`. **Alternativa:** `HealthRegistry` en `domain`. Se descartó: el Hace lo pone en `supervisor`.

### 2. `api` no depende de `supervisor`; DTO JSON en el Gateway

**Elección:** `AppState` lleva `Arc<dyn HealthReport>`. El handler `GET /health` llama `snapshot()`, mapea `AggregateStatus` → 200 (`ok`/`degraded`) o 503 (`starting`/`down`) y serializa un DTO camelCase (`uptimeSeconds`, `textEmbeddings`) alineado a `Health`. `impl HealthReport for HealthSnapshot` permite `AppState::new()` con `HealthSnapshot::all_disabled(env!("CARGO_PKG_VERSION"))` para no romper tests de 404/middleware. `vortex serve` construye `HealthRegistry::new(env!("CARGO_PKG_VERSION"))` y `AppState::with_health(Arc::new(registry))`. `api` MUST NOT declarar `supervisor`.

**Por qué:** [3.2](../../../docs/3-componentes.md#32-diagrama-de-dependencias-entre-crates) — solo `bin/vortex` conoce implementaciones concretas. El 503 de `/health` es el cuerpo `Health`, no `problem+json` (OpenAPI); el DTO evita que `domain` hable camelCase HTTP.

**Alternativa:** `api` depende de `supervisor` y usa `HealthRegistry` directo. Se descartó: rompe hex y la allowlist de `api`. **Alternativa:** serde camelCase en `domain`. Viable (domain ya serializa ULID/decimal); se prefiere DTO para no filtrar nombres de contrato al puerto.

### 3. Default `disabled` → agregado `ok`; 503 también en `down`

**Elección:** registro y snapshot por defecto inicializan los seis en `disabled` (agregado `ok`, HTTP 200). `disabled` no entra en las reglas 1–3. HTTP 503 cubre `starting` **y** `down` (OpenAPI: «Arrancando o sin fuente de verdad»). El ticket solo nombra 503 en `starting`; el schema ya exige 503 para `down`. T-00-20 pasará a `starting` durante el boot.

**Por qué:** en este ticket no hay sidecars ni Storage cableados; si el default fuera `down` o `starting`, `vortex serve` devolvería 503 siempre y el happy path HU «sistema sano» no sería testeable con el stub. `disabled` es el estado de componente del contrato para capacidades no cargadas (Whisper, etc.).

**Alternativa:** default `starting` (503 hasta T-00-20). Se descartó: el Listo cuando pide cubrir `ok` y el 503 de `starting` por separado; un default 503 mezclaría ambos. **Alternativa:** 200 en `down`. Se descartó: contradice OpenAPI.

### 4. Allowlist: `supervisor` + `domain`; `vortex` + `supervisor`

**Elección:** `supervisor_deps_allowed` acepta solo path `domain`. `vortex_deps_allowed` suma path `supervisor`. Fixture happy nuevo; unhappy `supervisor` con `clap` y `vortex` con `metrics` siguen. `api` no cambia. `check-domain-deps.sh` no cambia (`domain` no depende de `supervisor`).

**Por qué:** sin esto `check-workspace.sh` falla al implementar el Hace. Misma mecánica que T-00-11/T-00-12 para `api`.

**Alternativa:** tests de `HealthRegistry` dentro de `domain`. Se descartó: el registro vive en `supervisor`.

### 5. Tests HTTP inyectando snapshots; binario con registro real

**Elección:** `crates/api/tests/health.rs` construye `AppState::with_health(Arc::new(snapshot))` para `ok` / `degraded` / `starting` / `down`. `POST /health` aserta no 200 Health. El test de `vortex serve` pega `GET /health` al proceso (default `disabled` → 200 `ok`). Agregación: `domain` unitarios con los nombres de la spec. Reloj: no se duerme 1 s; `uptimeSeconds` ≥ 0 basta.

**Por qué:** el Listo cuando es el agregador + 503 `starting`. Los 200/503 HTTP son el contrato OpenAPI y T-09-04 los reutilizará.

**Alternativa:** solo tests de `supervisor` sin HTTP. Se descartó: el Hace incluye `GET /health`. **Alternativa:** sonda de tiempo real para uptime. Se descartó: frágil en CI.

## Risks / Trade-offs

- **[Default `ok` con todo `disabled` parece “sano” sin Storage]** → Mitigación: T-00-20 pone `starting`/`storage` al boot; este ticket documenta el default. Test `get_health_down_returns_503` cubre Storage caído.
- **[503 `down` no está en el Listo cuando]** → Mitigación: OpenAPI lo exige; se testea. No se usa `problem+json`.
- **[`AppState::new()` cambia de “sin puertos” a snapshot default]** → Mitigación: firma `new()` se mantiene; `with_health` es el constructor de producción/tests de salud. Tests de 404 existentes siguen compilando.
- **[DTO en `api` vs schema OpenAPI]** → Mitigación: campos required copiados del schema; T-09-04 validará. No se implementa el validador aquí.
- **[Hex: `api` no ve `HealthRegistry`]** → Aceptado: el binario es el composition root; tests de `api` usan `HealthSnapshot`.

## Migration Plan

1. Red: tests de agregación / registro / `GET /health` no compilan; `check-workspace.sh` rechaza `domain` en `supervisor` hasta relajar allowlist.
2. Green: tipos + `aggregate` + `HealthRegistry` + handler + allowlist + `vortex serve`.
3. Cierre: § 5 en `vortex-core/`, `scripts/check-workspace.sh`, `scripts/check-domain-deps.sh`, `openspec validate T-00-13`.
4. Rollback: quitar ruta `/health`, puerto, registro y deps. Sin datos de usuario; `GET /health` vuelve a 404 `not_found`.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. Si `serde` en `domain` necesita `features = ["derive"]` para tests del snapshot, es detalle de `Cargo.toml` cubierto por la allowlist vigente de `domain`.
