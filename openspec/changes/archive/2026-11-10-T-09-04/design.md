# Design

## Context

En `develop`: `api::build_router` registra `GET /health` (T-00-13) y `GET /metrics` (T-00-14). Los tests de `crates/api/tests/health.rs` asertan campos a mano; no hay `tests/support/`. `api` `[dev-dependencies]` es solo `tokio` + `tracing-subscriber`; `scripts/check-workspace.sh` rechaza cualquier otra crate. `docs/openapi.yaml` (OpenAPI 3.1, lint T-09-03) define `GET /health` 200/503 `application/json` → `#/components/schemas/Health` (`required`: `status`, `version`, `uptimeSeconds`, `components`). Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: hexagonal (`api` no genera tipos de contrato); allowlist de `check-workspace.sh`; local-first (sin fetch HTTP de `$ref`); ~2 h; MVP; PAPI-04 sin codegen; comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) ya cubren `cargo test --workspace`.

## Goals / Non-Goals

**Goals:**

- Helper reutilizable en tests de integración de `api` (`assert_conforms` + `conforms`)
- Cerrar el Listo cuando sobre `GET /health` y un required omitido
- Allowlist + pins para `jsonschema` / `serde_yaml` solo en dev-deps

**Non-Goals (diseño):**

- Crate publicado o `[dependencies]` de runtime
- Resolver `$ref` `http(s):`
- Recablear `problem+json` / `GET /metrics`
- Mutar el DTO de `/health` ni `docs/openapi.yaml`
- `#[should_panic]` como único unhappy (se usa `conforms` → `Err`)

## Decisions

### 1. Módulo `tests/support/`, no un crate ni codegen

**Elección:** `vortex-core/crates/api/tests/support/mod.rs` + `tests/support/openapi.rs`. Cada binario de integración que lo use declara `mod support;`. Firma:

```text
conforms(method, path, status, content_type, body: &Value) -> Result<(), String>
assert_conforms(...)  // panic con el String si Err
```

**Por qué:** el **Hace** nombra exactamente `crates/api/tests/support/openapi.rs`. Un crate `openapi_test` inflaría el workspace y la allowlist de members. Codegen (utoipa, openapi-generator) es el **No hace**.

**Alternativa:** helper en `src/` detrás de `#[cfg(test)]`. Se descartó: no está disponible para integration tests (`tests/*.rs` son crates distintos). **Alternativa:** crate `dev-dependency` path. Se descartó: ~2 h y un member más.

### 2. `jsonschema` 0.61.1 draft 2020-12, sin features de red

**Elección:** pin `jsonschema = "=0.61.1"` y `serde_yaml = "=0.9.34"` en `[workspace.dependencies]`. En `api` `[dev-dependencies]`: `jsonschema = { workspace = true, default-features = false }` y `serde_yaml = { workspace = true }`. Compilar el schema con `jsonschema::draft202012::new` (o `options().with_draft(...)`). 0.61.1 incluye el fix de `$ref` por clave numérica (`#/responses/200`).

**Por qué:** el ticket nombra `jsonschema` y draft 2020-12. OpenAPI 3.1 usa ese dialecto (`type: [string, 'null']`, `allOf`). `default-features` de 0.61 tira `reqwest` + `aws-lc-rs` para `$ref` HTTP; el producto es local-first y el YAML vigente no tiene refs remotas.

**Alternativa:** `valico` / `boon`. Se descartó: el ticket nombra `jsonschema`. **Alternativa:** features default. Se descartó: arrastra TLS al grafo de tests. **Alternativa:** pin solo en el crate sin `[workspace.dependencies]`. Se descartó: el resto del workspace usa `workspace = true` y `check_pin`.

### 3. Inliner de `$ref` locales; el documento OpenAPI no es el schema raíz

**Elección:** parsear YAML → `serde_json::Value`. Lookup: `paths[pathItem][method].responses[status].content[mime].schema`, siguiendo `$ref` de respuesta (`#/components/responses/...`) y de schema (`#/components/schemas/...`). Antes de validar, **inline** recursivo de `$ref` JSON Pointer del mismo documento (incluye `allOf` de `sync`). No se pasa el documento OpenAPI entero como schema (tiene `openapi`/`paths`, no es JSON Schema).

**Por qué:** `Health` referencia `ComponentHealth` / `ComponentStatus`; `sync` usa `allOf` + `$ref`. Un inliner local cubre el YAML vigente y T-02-06 sin Retriever HTTP. Usar el OpenAPI como raíz confunde al validador (keywords desconocidos a nivel raíz).

**Alternativa:** `Registry` / retriever de `jsonschema` con URI `file://...`. Viable; más API surface y features `resolve-file`. Se deja como plan B si el inliner se queda corto. **Alternativa:** validar solo el objeto `Health` hardcodeando el pointer. Se descartó: el **Hace** pide lookup por método + ruta + estado + content type.

### 4. Path de carga: `CARGO_MANIFEST_DIR` → raíz del repo; cache `OnceLock`

**Elección:** `Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../docs/openapi.yaml")` (`crates/api` → repo). Cachear el documento parseado en `OnceLock`. Content type: tipo MIME en minúsculas, sin parámetros (`; charset=...`). Método en minúsculas. Status: buscar clave string `"200"` (el YAML cita `'200'`). Path: igualdad exacta con el path item; si no hay, un segmento por `{param}` (para T-02-06).

**Por qué:** CI corre `cargo test` con `working-directory: vortex-core`; `CARGO_MANIFEST_DIR` sigue siendo el crate, así que tres `..` llegan a `docs/`. Cache evita releer el YAML en cada `assert_conforms`. El matching de templates es barato y evita reescribir el helper en el próximo endpoint.

**Alternativa:** `include_str!` del YAML. Se descartó: el path relativo al crate es frágil en el source tree y no es el archivo de `docs/`. **Alternativa:** variable de entorno. Se descartó: no hace falta para el Listo cuando.

### 5. Allowlist: dev-deps sí, `[dependencies]` no

**Elección:** `api_deps_allowed` acepta `jsonschema` y `serde_yaml` **solo** en `[dev-dependencies]` (`workspace = true`; si `jsonschema` está, la línea MUST contener `default-features = false`). Las mismas crates en `[dependencies]` fallan. Fixture happy `api con allowlist openapi`. Fixture unhappy: `jsonschema` en `[dependencies]` (además de `api con clap`). `check_pin jsonschema 0.61.1` y `check_pin serde_yaml 0.9.34`. Fixtures prometheus / axum **siguen** MAY (no exigen las crates nuevas).

**Por qué:** sin esto `check-workspace.sh` falla al implementar el Hace. PAPI-04: el validador no entra al binario. Misma mecánica que T-00-12 / T-00-14.

**Alternativa:** crate `jsonschema` en un package `dev` del workspace. Se descartó: decisión 1.

### 6. Cablear `health.rs`; unhappy del helper en archivo propio

**Elección:** `get_health_ok_returns_200` y `get_health_starting_returns_503` llaman `assert_conforms` con el JSON ya parseado. `crates/api/tests/openapi_conformance.rs` construye un `Health` válido mínimo (copia del 200) y uno sin `version`; llama `conforms` (no HTTP). Un segundo test pide `POST /health` 200 `application/json` y espera `Err` que nombre la combinación.

**Por qué:** el Listo cuando es «el test de GET /health pasa» + «un required omitido falla». Reusar el HTTP de T-00-13 evita un segundo servidor. `conforms` deja asertar el mensaje sin `should_panic`.

**Alternativa:** solo tests del helper con JSON golden, sin pegarle a `/health`. Se descartó: el Listo cuando nombra el test de `GET /health`. **Alternativa:** recablear `problem.rs`. Se descartó: non-goal; T-02-06 adoptará el helper en 4xx reales.

## Risks / Trade-offs

- **[serde_yaml 0.9 está en mantenimiento / YAML 1.1 truthy]** → Mitigación: el contrato vigente cita `'200'` y `'null'`; no se usan `on`/`yes` como keys. Si un `$ref` se rompe, el unhappy de carga nombra el pointer.
- **[Inliner incompleto (`discriminator`, callbacks)]** → Aceptado en MVP: `/health` y los schemas de catálogo H1 son objetos + `$ref` + `allOf`. Plan B: Registry de jsonschema.
- **[Health sin `additionalProperties: false` deja pasar campos extra]** → Correcto según JSON Schema; no se endurece el YAML (non-goal).
- **[Caminos `../../../docs` rotos si se mueve el crate]** → Mitigación: el test unhappy de archivo faltante nombra `openapi.yaml`; el crate no se mueve en este ticket.
- **[jsonschema 0.61.1 + edition 2024 / MSRV 1.85]** → Compatible con `rust-toolchain.toml` estable actual; verificar en apply con `cargo test -p api`.
- **[Fixture prometheus sigue MAY]** → Aceptado: no exigir jsonschema en árboles de test que solo chequean la allowlist vieja.

## Migration Plan

1. Red: `mod support` / `assert_conforms` no compilan; `check-workspace.sh` rechaza `jsonschema` en `api` hasta relajar allowlist y pins.
2. Green: pins + allowlist + loader + inliner + `assert_conforms` + cableado `/health` + fixture.
3. Cierre: § 5 en `vortex-core/`, `scripts/check-workspace.sh`, `scripts/fixtures/workspace/run-script-tests.sh`, `openspec validate T-09-04` (o `t-09-04`).
4. Rollback: quitar `tests/support/`, el test de conformidad, dev-deps, pins y líneas de allowlist. Sin datos de usuario; `/health` sigue igual.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El pin `0.61.1` se confirma al resolver crates.io en apply; si Yank, subir al parche 0.61.x más nuevo con el mismo `default-features = false`.
