# Proposal

## Ticket padre

- **Ticket:** [T-09-03](../../../docs/7-tickets.md) — Fuente única OpenAPI y *lint* Redocly
- **Change:** `T-09-03`
- **HU:** [HU-09-02](../../../docs/6-historias.md#hu-09-02--validación-del-contrato-de-api)
- **Depende de:** T-09-01 (cerrado en `develop`; T-00-02 ya enganchó `domain-deps`)
- **Hace:** PAPI-01 ya aplicado en docs (sin copia embebida en 5.13); agrega `redocly.yaml` y *job* de CI `npx @redocly/cli lint docs/openapi.yaml`
- **No hace:** validación de respuestas (T-09-04)
- **Listo cuando:** la CI falla si `openapi.yaml` tiene un error de *lint*

## Why

`docs/openapi.yaml` ya es la fuente única del contrato (PAPI-01, § 5.13), pero la CI de T-09-01 **prohíbe** Redocly a propósito para dejar este ticket. Un PR puede romper la especificación y seguir verde. HU-09-02 pide que la integración continua falle si el YAML no pasa el lint; T-09-04 necesita esa puerta antes de validar respuestas.

## What Changes

- Agregar `redocly.yaml` en la raíz que apunta a `docs/openapi.yaml` y usa el conjunto `recommended` (warnings no fallan; errores sí)
- Agregar un job de GitHub Actions en paralelo (`openapi-lint`) que ejecuta `npx --yes @redocly/cli@1 lint docs/openapi.yaml`
- Extender `scripts/check-ci.sh`: exigir ese job y `redocly.yaml`; dejar de tratar `redocly` / `docs/openapi.yaml` como extra prohibido de T-09-01
- TDD: happy = `docs/openapi.yaml` actual sale 0; unhappy = YAML inválido sale ≠ 0 y nombra el error; fixture de workflow sin el job hace fallar `check-ci.sh`
- **BREAKING** (harness interno): `check-ci.sh` deja de fallar si el workflow menciona Redocly / `docs/openapi.yaml`; pasa a exigir el job `openapi-lint`

## Non-goals

Este change **no** cubre:

- Validación de respuestas HTTP contra esquemas — **T-09-04** (**No hace** del ticket)
- Codegen desde OpenAPI — PAPI-04
- Quitar la copia embebida de 5.13 — PAPI-01 **ya aplicado** en docs
- Corregir los 13 *warnings* actuales de Redocly (`info-license`, `operation-4xx-response`, `no-unused-components`) ni cambiar `docs/openapi.yaml` para silenciarlos
- Endpoints v2 / contratos posteriores — T-03-25, T-05-26, T-08-04
- Meilisearch / Ollama / Whisper reales en CI — **T-09-05**, **T-09-13**, **T-09-18**
- Un job que ejecute `scripts/check-ci.sh` en GitHub (sigue siendo harness local)

## Capabilities

### New Capabilities

- `openapi-lint`: `redocly.yaml` + lint Redocly 1.x de `docs/openapi.yaml`; el job de CI corre el mismo comando; un error de lint falla; los warnings del archivo actual no fallan

### Modified Capabilities

- `ci-base`: `scripts/check-ci.sh` MUST exigir el job `openapi-lint` y `redocly.yaml`; MUST NOT tratar `redocly` / `docs/openapi.yaml` como extra fuera de alcance; MUST seguir rechazando Meilisearch, Ollama, Whisper y un job que ejecute `check-ci.sh` en GitHub

## Impact

- **Repo:** `redocly.yaml` (nuevo), `.github/workflows/ci.yml` (job nuevo), `scripts/check-ci.sh` (contrato ampliado), fixtures de CI y un oráculo de lint inválido. No toca crates de `vortex-core/` ni el contenido de `docs/openapi.yaml` / `docs/5-contract-api.md`
- **APIs / OpenAPI:** ninguna mutación del contrato; solo se lint-ea el YAML existente
- **CI / verificación:** quinto check en cada PR; comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sin cambio; `openspec validate T-09-03` (o `t-09-03` si el CLI exige minúsculas)
- **Ticket:** cierra T-09-03 (Listo cuando: la CI falla si `openapi.yaml` tiene un error de lint). El escenario operativo en GitHub se confirma al abrir el PR de esta rama
- **BREAKING** (harness interno): la fixture `redocly-extra` deja de ser el unhappy de «extra fuera de alcance»; ese rol pasa a Meilisearch/Ollama/Whisper
