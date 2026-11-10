# Proposal

## Ticket padre

- **Ticket:** [T-00-02](../../../docs/7-tickets.md) — Verificación automática de la regla de dependencias de `domain`
- **Change:** `T-00-02`
- **HU:** [HU-00-01](../../../docs/6-historias.md#hu-00-01--workspace-rust-y-regla-de-dependencias)
- **Hace:** `scripts/check-domain-deps.sh`, que analiza `cargo tree -p domain -e normal,build` y falla si aparece `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort`, `whisper-rs` o cualquier crate del workspace; *job* de CI que lo ejecuta
- **No hace:** reglas para otros crates
- **Listo cuando:** agregar `surrealdb` a `domain` en una rama hace fallar la CI

## Why

El script `scripts/check-domain-deps.sh` ya está en `develop` (change archivado `T-00-02-01`) y certifica el árbol `normal,build` de `domain` en local. T-09-01 dejó `.github/workflows/ci.yml` con `fmt` / `clippy` / `test`, pero **no** corre ese script: un PR que acople `domain` a `surrealdb` pasa la CI. HU-00-01 pide que la integración continua falle si el dominio depende de un motor; este ticket cierra ese **Listo cuando**.

## What Changes

- Agregar un job de GitHub Actions (cuarto job, en paralelo) que ejecute `scripts/check-domain-deps.sh` en el mismo workflow y runner que T-09-01
- Extender `scripts/check-ci.sh` para exigir ese job y dejar de tratar `check-domain-deps` como extra prohibido de T-09-01
- TDD: happy path = repo limpio + YAML con el job (check-ci y check-domain-deps en 0); unhappy path = YAML sin el job, y las fixtures ya existentes de `surrealdb` / SDK / workspace (el script sigue fallando)
- **No** reescribir el script ni su denylist: el contrato de `cargo tree` ya está en `openspec/specs/domain-deps/`

## Non-goals

Este change **no** cubre:

- Reglas equivalentes para otros crates (`storage`, `api`, …) — **No hace** del ticket
- Ampliar la denylist (Ollama/`tokenizers`/aristas `dev`) — fuera del **Hace**; el recorte de T-00-02-01 se mantiene
- Reescribir o relajar `scripts/check-workspace.sh` — **T-00-03** / **T-00-04**
- *Lint* Redocly de `docs/openapi.yaml` — **T-09-03**
- Meilisearch / Ollama / Whisper reales en CI — **T-09-05**, **T-09-13**, **T-09-18**
- Taxonomía de errores o tipos de dominio — **T-00-03**, **T-00-04**
- Un job que ejecute `check-ci.sh` en GitHub (sigue siendo harness local)

## Capabilities

### New Capabilities

- Ninguna. El script y la regla hexagonal ya viven en `domain-deps`.

### Modified Capabilities

- `domain-deps`: el árbol limpio de `domain` MUST verificarse también en CI; un job del workflow MUST ejecutar `scripts/check-domain-deps.sh`
- `ci-base`: el harness `scripts/check-ci.sh` MUST exigir ese job y MUST NOT tratar `check-domain-deps` como extra fuera de alcance; `fmt` / `clippy` / `test` y la prohibición de motores/OpenAPI no cambian

## Impact

- **Repo:** `.github/workflows/ci.yml` (job nuevo), `scripts/check-ci.sh` (contrato ampliado), fixtures de CI. No toca `vortex-core/crates/domain` ni reescribe `scripts/check-domain-deps.sh` salvo comentario de ticket
- **APIs / OpenAPI:** ninguna
- **CI:** cuarto check en cada PR; el **Listo cuando** («agregar `surrealdb` a `domain` hace fallar la CI») queda cubierto porque el job corre el mismo script que ya falla en esa violación
- **Ticket:** cierra T-00-02 (script ya entregado + job + Listo cuando)
- **BREAKING:** no. `check-ci.sh` pasa a exigir un job que este change agrega en el mismo árbol
