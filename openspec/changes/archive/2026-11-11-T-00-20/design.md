# Design

## Context

En `develop`, `vortex serve` carga `Config`, bindea HTTP de inmediato, cablea `HealthRegistry::new` (seis componentes `disabled` → agregado `ok`) y sirve estáticos. **No** llama a `init_logging` (`#![allow(dead_code)]` en `logging.rs`) ni abre `storage::Db`. Los tests de binario en `tests/serve.rs` usan a menudo `paths.datos = "/tmp/datos"` compartido y timeout de 5 s. `Db::open(path)` + `Db::migrate()` ya existen; `storage::DB_DIR_NAME` es `"db"`. `check-workspace.sh` aún rechaza `storage` en `bin/vortex`. Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: HTTP `localhost` (MVP); sin supervisor/worker (T-00-27); ~2 h; sin crates nuevas en `[workspace.dependencies]`.

## Goals / Non-Goals

**Goals:**

- Secuenciador de boot en el composition root (`serve.rs`): config → logs → Storage+migraciones → salud → bind
- Tests de binario que cierran el Listo cuando (`/health` `ok` + base inaccesible) y las duraciones `boot.*`
- Allowlist `vortex` → path `storage`; fixtures de `check-workspace.sh`

**Non-Goals (diseño):**

- Extraer un crate `boot` o inyectar `Db` en `AppState`
- Listener HTTP durante Storage (el primer GET observable es `ok`, no 503 `starting`)
- Reloj inyectable para las duraciones (basta el elapsed del span de `tracing`)
- `kv-mem` en el proceso de `serve` (producción y tests del binario usan RocksDB)

## Decisions

### 1. No bindear HTTP hasta Storage OK; `starting` solo en el registro

**Elección:** crear `HealthRegistry` **después** de `init_logging` y **antes** de `Db::open`. Publicar `storage = starting`, abrir `<paths.datos>/db`, migrar, publicar `storage = up`, **entonces** `TcpListener::bind`. Si open/migrate fallan: `eprintln` + exit ≠ 0, sin listener. El primer `GET /health` del test es 200 `ok` con `storage` `up`.

**Por qué:** el Hace dice «si falla Storage o la configuración, no se expone la API» y el Listo cuando es «espera `ok`». Un bind previo dejaría 503 `starting` observable pero también una API arriba mientras RocksDB falla. T-00-13 ya cubre HTTP 503 `starting` inyectando snapshots en el crate `api`; este ticket no necesita un GET 503 de proceso.

**Alternativa:** bindear al inicio con `starting` y bajar el listener si Storage falla. Se descartó: contradice «no se expone la API» y alarga el ticket (shutdown a medias). **Alternativa:** crear el registro recién después de migrar (orden literal del ticket). Se descartó: no habría `storage = starting` ni siquiera internamente; crear el registro antes de `open` cumple «`/health` en `starting` hasta terminar» a nivel de publicación.

### 2. `Db` vive en `serve_config`; no entra a `AppState`

**Elección:** `let db = Db::open(datos.join(DB_DIR_NAME)).await?; db.migrate().await?;` y el valor permanece en el stack de `serve_config` hasta el shutdown (evita soltar el lock de RocksDB). `AppState` no cambia.

**Por qué:** no hay rutas de negocio. Inyectar `Arc<Db>` ahora obliga a tocar `api` sin contrato. T-02-06+ lo hará cuando exista un puerto de catálogo.

**Alternativa:** `AppState::with_storage`. Se pospone. **Alternativa:** dropear `Db` tras migrar. Se descartó: el motor quedaría cerrado y T-00-27 reabriría con lock/corrida inútil.

### 3. Base inaccesible = `<datos>/db` archivo; logs fallan si `datos` es archivo

**Elección:** unhappy Storage: directorio `paths.datos` escribible (así `init_logging` pasa) y `datos/db` es un archivo regular — `Db::open` ya rechaza ese caso (`OpenError`). Unhappy logs: `paths.datos` es un archivo (T-00-06). Config inválida sigue abortando antes de logs (tests existentes).

**Por qué:** el Listo cuando pide «base inaccesible». Un `chmod 000` sobre `datos` rompería logs primero y no aislaría Storage. El archivo-en-lugar-de-`db` es el mismo unhappy de `storage` y no depende de root.

**Alternativa:** checksum de migración alterado. Se descartó: T-00-09 ya lo cubre en el crate; montar una base pre-sembrada en el binario excede ~2 h.

### 4. Spans `boot.*` con elapsed; tests leen el JSON bajo `datos/logs`

**Elección:** `tracing::info_span!("boot.storage")` (y análogos) alrededor de cada fase; el JSON de T-00-06 ya incluye el nombre del span. Config se mide con `Instant` **antes** de `init_logging` y se cierra el span `boot.config` justo después de instalar el subscriber. Los tests de binario usan `paths.datos` **únicos** (dejan de compartir `/tmp/datos`) y `VORTEX_ENV=production` para no mezclar pretty en stderr.

**Por qué:** el Hace pide duración por fase. Parsear JSON de log es el mismo patrón que T-00-06; no hace falta un campo Prometheus (T-00-14 ya cubre HTTP).

**Alternativa:** métricas `boot_duration_seconds`. Se descartó: fuera del ticket. **Alternativa:** imprimir «listening on» en stderr. Insuficiente para las cinco fases.

### 5. Allowlist `storage` en `vortex`; fixtures happy

**Elección:** `vortex_deps_allowed` acepta `storage` con `path`. Fixture happy nuevo (o el existente de allowlist ya copia el `Cargo.toml` real, que pasará a declarar `storage`). Unhappy sigue siendo `metrics`. `api` / `domain` no cambian.

**Por qué:** sin esto `check-workspace.sh` bloquea el Hace. Hexagonal: solo el binario conoce `storage` concreto.

**Alternativa:** reexportar `Db` desde `supervisor`. Se descartó: el supervisor aún no supervisa nada; Storage no es un sidecar.

### 6. Timeouts de wait a 15 s en tests de binario que abren RocksDB

**Elección:** los `wait_for_*` de `tests/serve.rs` que hoy usan 5 s pasan a 15 s cuando el child abre RocksDB + migrate. Los unhappy de config (sin Storage) pueden seguir en 5 s porque el proceso sale de inmediato.

**Por qué:** T-00-08 ya abre RocksDB en CI; el boot del binario suma migrate `0000` y logging. 5 s puede flaquear en runners fríos.

## Risks / Trade-offs

- **[`/health` `starting` no es observable por HTTP en este ticket]** → Aceptado: el Listo cuando espera `ok`; 503 `starting` queda en tests de `api` (T-00-13). T-00-27 puede reevaluar si el wait de Meilisearch debe exponerse.
- **[Tests existentes con `datos = /tmp/datos` chocan al abrir la misma RocksDB]** → Mitigación: helper único de temp dir en todos los `write_config` de `serve.rs`.
- **[`init_logging` es global (`try_init`); no se puede llamar dos veces en el mismo proceso]** → Mitigación: los tests de `serve` spawn del binario; los de `logging.rs` siguen usando `with_logging` (subscriber scoped). No hay test in-process de `serve_config`.
- **[RocksDB deja `LOCK` si el test mata el child con SIGKILL]** → Mitigación: SIGTERM como hoy; dirs únicos por test. No reutilizar `datos`.
- **[macOS: `openspec new change` exige `t-00-20`]** → Mitigación: rename a `T-00-20` (igual que T-00-19).
- **[Default previo `ok` con `storage` `disabled` desaparece en el proceso real]** → Esperado: el boot ahora publica `up`. Los tests del crate `api` con snapshot inyectado no cambian.

## Migration Plan

1. Red: tests nuevos de boot en `tests/serve.rs` no compilan o fallan; `check-workspace.sh` rechaza `storage` en `vortex` hasta relajar allowlist.
2. Green: allowlist + `storage` en `Cargo.toml` + secuenciador en `serve.rs` + `init_logging` + `Db` retenido + helpers de temp `datos`.
3. Cierre: § 5 en `vortex-core/`; `openspec validate T-00-20`.
4. Rollback: revertir `serve.rs`, deps, allowlist y tests. Sin datos de usuario ni contrato OpenAPI. Un `datos/db` creado en tests temporales se borra con el temp dir.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre exacto del campo JSON de duración (`elapsed` vs `busy` de `tracing-subscriber`) se fija al escribir el test (se aserta presencia de span `boot.storage` y un número de duración, no un schema rígido).
