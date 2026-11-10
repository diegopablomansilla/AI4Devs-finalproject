# Design

## Context

En `develop`, `api::build_router` es un `Router` vacío: el 404 es el de Axum (sin `problem+json`). `DomainError` ya vive en `crates/domain` con `code()` / `FieldError` y **sin** HTTP (T-00-03). `crates/api` solo declara `axum`; `check-workspace.sh` rechaza cualquier otra crate. `serde` / `serde_json` / `thiserror` ya están pineados en el workspace. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: MVP HTTP (no TLS); no `tower-http` (T-00-12); no rutas de negocio; ~2 h.

## Goals / Non-Goals

**Goals:**

- `ApiError` + `IntoResponse` RFC 9457 en el crate `api`; fallback 404; extractor JSON que unifica rechazos a `400 invalid_input`
- Allowlist de `api` ampliada a `domain` + `serde` / `serde_json` / `thiserror`
- Tests happy/unhappy que cierren el Listo cuando (status + `Content-Type` + `code`)

**Non-Goals (diseño):**

- `tower-http` / `RequestBodyLimitLayer` / timeout layer (T-00-12)
- Validar cuerpos contra `openapi.yaml` (T-09-04)
- Parser de `JsonRejection` campo-a-campo más allá de la ruta serde o el sentinel `body`
- `#[tokio::main]` ni cambios al apagado de `vortex serve`

## Decisions

### 1. `ApiError` en `api`, no en `domain`

**Elección:** struct (o enum) `api::ApiError` con `From<DomainError>` e `IntoResponse`. Campos: `status`, `code`, `title`, `detail`, `instance: Option<String>`, `errors: Option<Vec<{field, message}>>`. Títulos y `type` URI se resuelven en `api` con un match sobre `DomainError::code()`. `ApiError::internal(_cause: &dyn Error)` ignora la causa al serializar. `Retry-After: 5` en `Unavailable` (mismo orden de magnitud que el timeout por defecto de T-00-12).

**Por qué:** T-00-03 prohibió HTTP en `domain`. El ticket nombra `ApiError`. `instance` no existe en `DomainError`; se rellena en el fallback / extractor.

**Alternativa:** `impl IntoResponse for DomainError` en `api` vía newtype. Se descartó: no cubre `internal` ni rechazos JSON. **Alternativa:** `Retry-After` variable por código. Se descartó: 5.4 no da valores; un entero fijo cumple el contrato.

### 2. Fallback del router + extractor `AppJson<T>`

**Elección:** `build_router` instala `Router::fallback` que responde `ApiError` `not_found` con `instance = path`. Se exporta `AppJson<T>` (`FromRequest`, `Rejection = ApiError`) que envuelve `axum::Json<T>` y mapea `JsonRejection` → `400 invalid_input` con `errors[{ field, message }]`. Campo: ruta serde si el rechazo la trae; si no, `body`. Incluye `MissingJsonContentType` (no 415). Los handlers futuros usan `AppJson`, no `Json` crudo.

**Por qué:** el Listo cuando exige 404 `problem+json` en el router de producción (`vortex serve` reutiliza `build_router`). Sin extractor propio, Axum sigue respondiendo texto en rechazos JSON.

**Alternativa:** `HandleErrorLayer` global. Se descartó: no distingue JSON vs otros extractores y T-00-12 pondrá layers de tower-http encima. **Alternativa:** mapear `MissingJsonContentType` a 415. Se descartó: el ticket dice rechazos JSON → `400 invalid_input`.

### 3. Tests: `IntoResponse` + TCP; sonda JSON solo en tests

**Elección:** tabla `domain_error_variants_map_to_problem_json` construye cada `DomainError` → `ApiError` → `into_response()` y aserta status / `Content-Type` / `code` / `title` (sin servidor). `build_router_unknown_path_returns_404` pasa a parsear cabecera + JSON (`code`, `instance`). `json_rejection_returns_400_invalid_input` monta en el test `POST /_probe/json` con `AppJson<Value>` sobre el router de `build_router` (la sonda **no** va a producción). `internal_error_does_not_leak_cause` llama `ApiError::internal` con un `Error` cuyo `Display` contiene un marcador y aserta que el cuerpo no lo incluye. `vortex_serve_starts_and_returns_404` amplía el parseo HTTP (el binario ya tiene `serde_json` de test). Sin `reqwest` / `tower` extra: se sigue con `TcpStream` como T-00-10.

**Por qué:** no hay rutas de negocio que disparen `DomainError`. Meter `/_probe/*` en `build_router` contaminaría el Gateway. `IntoResponse` es el contrato observable del mapeo.

**Alternativa:** `cfg(test)` routes en `lib.rs`. Se descartó: el binario no las ejecuta y el 404 de `serve` quedaría sin `code` testeado en proceso. El test de binario cubre el fallback real.

### 4. Allowlist: `domain` + serde stack; fixture unhappy deja de usar `serde`

**Elección:** `api_deps_allowed` acepta en `[dependencies]` `axum`, `domain` (path), `serde`, `serde_json`, `thiserror` (`workspace = true` en crates.io). `[dev-dependencies]` sigue solo `tokio`. El fixture `api con serde` pasa a `clap` o `tower-http` (serde deja de ser un unhappy válido). El happy `api con allowlist axum` **sigue** pasando (MAY); se agrega un happy con la allowlist completa. `api` MUST NOT declarar `tower-http`.

**Por qué:** serializar `Problem` exige `serde`/`serde_json`; `From<DomainError>` exige path `domain`; `thiserror` es el default del stack si `ApiError` es enum. El fixture actual (`serde` prohibido) rompería el harness en verde.

**Alternativa:** armar el JSON a mano con `serde_json::json!` sin declarar `serde`. Sigue haciendo falta `serde_json` y `domain`; no ahorra el cambio de harness.

## Risks / Trade-offs

- **[Handlers futuros usan `Json<T>` y saltan el mapeo]** → Mitigación: exportar `AppJson` y mencionarlo en este design; T-02-06+ lo usan. Clippy no puede forzarlo.
- **[Fixture `api con serde` queda en verde y pierde el unhappy]** → Mitigación: tarea explícita de reescribir el fixture a `clap`/`tower-http`.
- **[Parseo HTTP/1.1 a mano falla con cuerpos fragmentados]** → Mitigación: respuestas de error son pequeñas; buffer 4 KiB como T-00-10; assert de `Content-Length` o lectura hasta EOF (`Connection: close`).
- **[Títulos en español duplicados en T-01-07]** → Aceptado: HU-01-05 quiere `title` en la API; T-01-07 es copia para toasts. Mismo wording en la tabla de la spec.

## Migration Plan

1. Red: tests de `problem+json` no compilan o fallan; `build_router_unknown_path_returns_404` deja de pasar si exige `Content-Type`.
2. Red: `check-workspace.sh` rechaza `domain`/`serde` en `api` hasta relajar allowlist.
3. Green: allowlist + `ApiError` + fallback + `AppJson` + fixtures.
4. Cierre: § 5 en `vortex-core/`, `openspec validate T-00-11`.
5. Rollback: revertir módulo de error, fallback, deps de `api`, allowlist y fixtures. Sin datos de usuario; el 404 vuelve al de Axum.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. `Retry-After: 5` es convención interna hasta que T-00-12 fije timeouts por ruta.
