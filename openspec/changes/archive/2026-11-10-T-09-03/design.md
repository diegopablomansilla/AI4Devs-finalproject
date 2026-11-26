# Design

## Context

En `develop`: `.github/workflows/ci.yml` tiene `fmt`, `clippy`, `test` y `domain-deps` en `macos-14`. `scripts/check-ci.sh` **prohíbe** las cadenas `Redocly`, `redocly` y `docs/openapi.yaml` a propósito, para que este ticket las enganche. La fixture `scripts/fixtures/ci/scenarios/redocly-extra` es el unhappy de T-09-01. No existe `redocly.yaml`. PAPI-01 ya está aplicado: [5.13](../../docs/5-contract-api.md#513-especificación-openapi-31) no embebe el spec. `npx --yes @redocly/cli@1 lint docs/openapi.yaml` sobre el YAML vigente sale **0** con 13 warnings (`info-license`, `operation-4xx-response` en GETs de listado/`health`/`metrics`, `no-unused-components` de eventos SSE). Ver proposal.md para motivación y `specs/` para el contrato.

Constraints: runner `macos-14` ([7.2](../../docs/7-tickets.md)); el lint corre sobre un path de la raíz (`docs/openapi.yaml`); Node 22 ya está en el runner GitHub `macos-14` y en el entorno local; no hay `package.json`.

## Goals / Non-Goals

**Goals:**

- Un job de CI independiente que corre exactamente el comando de [5.13](../../docs/5-contract-api.md#513-especificación-openapi-31)
- Un solo harness local (`check-ci.sh`) que falle si falta ese job o `redocly.yaml`
- TDD del lint **sin** `act` y **sin** romper el YAML git: oráculo inválido en fixture

**Non-Goals (diseño):**

- `package.json` / lockfile npm en la raíz
- `actions/setup-node` (el runner ya trae Node)
- Caché de Cargo en el job de lint
- Parser YAML propio; se sigue usando grep POSIX como T-09-01 / T-00-02
- Job que ejecute `check-ci.sh` en GitHub
- Silenciar o corregir los 13 warnings (no son el Listo cuando)

## Decisions

### 1. Job `openapi-lint` en paralelo, no un step de `test`

**Elección:** `jobs.openapi-lint` con `runs-on: macos-14`, en el mismo workflow (`pull_request` + `push` a `develop`/`main`).

**Por qué:** T-09-01 diseñó el YAML para que T-09-03 agregue **jobs**. El Listo cuando («la CI falla si `openapi.yaml` tiene un error de lint») queda como check independiente. El nombre alinea con la capability.

**Alternativa:** un step `npx …` al final de `test`. Se descartó: mezcla señales (un fallo de tests Rust vs. un contrato OpenAPI roto) y contradice el wording del ticket (*job*).

### 2. El job corre desde la raíz; sin caché Cargo ni working-directory de vortex-core

**Elección:** **no** usar `defaults.run.working-directory: vortex-core`. Steps: `actions/checkout@v4`; `run: npx --yes @redocly/cli@1 lint docs/openapi.yaml`. Sin `rustup`, sin `actions/cache` de Cargo.

**Por qué:** el path del **Hace** es `docs/openapi.yaml` relativo a la raíz. El job no compila Rust. Exigir una cuarta caché Cargo rompería el orden de error de las fixtures T-09-01 si se subiera el mínimo antes de nombrar el job.

**Alternativa:** `actions/setup-node@v4` + caché de npm. Se descartó: el runner `macos-14` ya trae Node/npx; el pin `@1` evita un lockfile solo para este comando.

### 3. Pin `@redocly/cli@1`, no `latest` ni v2

**Elección:** el comando del job (y de los oráculos locales) es `npx --yes @redocly/cli@1 lint docs/openapi.yaml`. Línea 1.x actual: 1.34.24. `--yes` evita el prompt de npm.

**Por qué:** el ticket escribe `npx @redocly/cli lint` sin versión, pero `npx` sin pin puede resolver Redocly 2.x (hoy 2.77.0) y romper CI. `@1` cubre el YAML vigente (exit 0). No se agrega `package.json`.

**Alternativa:** `package.json` con versión exacta. Se descartó: introduce un ecosistema Node en un repo Rust solo para un binario de lint. **Alternativa:** `@redocly/cli@2`. Se descartó: no está validado contra este YAML y no lo pide el ticket.

### 4. `redocly.yaml` mínimo con `extends: recommended`

**Elección:** archivo en la raíz:

```yaml
extends:
  - recommended
apis:
  vortex@v1:
    root: docs/openapi.yaml
```

No se cambia la severidad de `info-license`, `operation-4xx-response` ni `no-unused-components`. Warnings siguen siendo warnings (exit 0).

**Por qué:** el **Hace** pide `redocly.yaml`. `recommended` es el default que ya valida el archivo. Elevar esas reglas a error exigiría mutar `docs/openapi.yaml` (licencia, 4xx en listados/`health`/`metrics`, componentes SSE) — fuera de alcance y de T-09-04.

**Alternativa:** no agregar `redocly.yaml` y depender del default. Se descartó: el ticket lo nombra. **Alternativa:** `extends: minimal`. Se descartó: más débil que el default ya verde.

### 5. Extender `check-ci.sh`, no un script nuevo

**Elección:** quitar `Redocly`, `redocly` y `docs/openapi.yaml` de la denylist de extras; exigir job `openapi-lint` (`^[[:space:]]{2}openapi-lint:`) y las cadenas `@redocly/cli` + `lint docs/openapi.yaml`; exigir archivo `redocly.yaml`. Mantener `>= 3` caches y `macos-14` para no romper el orden de error de las fixtures T-09-01; el job nuevo se comprueba **después** de `domain-deps` y de los extras que siguen prohibidos (Meilisearch, Ollama, Whisper, `check-ci.sh`).

**Por qué:** un solo comando local cubre el contrato CI, igual que T-00-02. Si se exigiera `runs-on`/`cache` `>= 5` *antes* del nombre del job, la fixture de cuatro jobs fallaría con «macos-14» en vez de nombrar `openapi-lint`.

**Alternativa:** `scripts/check-openapi.sh`. Se descartó: el job de CI debe ser el comando del ticket, no un wrapper; duplicaría el grep de workflow.

Orden de chequeos (relevante para TDD):

1. Archivo, `pull_request`, jobs `fmt`/`clippy`/`test`, no ubuntu, `macos-14` `>= 3`, cache `>= 3`, `vortex-core`, comandos § 5
2. Extras prohibidos **sin** `redocly` / `docs/openapi.yaml` (sigue Meilisearch, Ollama, Whisper, `check-ci.sh`)
3. Job `domain-deps` + `scripts/check-domain-deps.sh`
4. Job `openapi-lint` + `@redocly/cli` + `lint docs/openapi.yaml`
5. Existe `redocly.yaml`

Así `missing-domain-deps-job` sigue fallando por `domain-deps`, no por el job nuevo.

### 6. Unhappy de lint = fixture, no el YAML git; `redocly-extra` cambia de rol

**Elección:** no modificar `docs/openapi.yaml`. El escenario «un error de lint falla» se prueba con `scripts/fixtures/openapi/scenarios/invalid-spec` (OpenAPI 3.1 sin `info.version` o YAML roto). La fixture `redocly-extra` **deja de usarse** como unhappy de extra: se reemplaza por `scripts/fixtures/ci/scenarios/meilisearch-extra` (mismo esqueleto, step que invoca Meilisearch). Se agrega `missing-openapi-job` (cuatro jobs válidos, sin `openapi-lint`). Se agrega `missing-redocly-yaml` (workflow con el job, sin el archivo de config) si el chequeo 5 corre después del nombre del job.

**Por qué:** el Listo cuando en GitHub es «el job corre el mismo comando»; ensuciar el contrato real rompería T-09-04 y el PR. El extra fuera de alcance de `ci-base` pasa a sidecars, coherente con el requirement removido.

**Alternativa:** un commit temporal que rompa `openapi.yaml` y revert. Se descartó: no entra en el árbol de `T-09-03`.

## Risks / Trade-offs

- **[Grep del YAML]** → un comentario `openapi-lint:` podría falsear. Mitigación: el archivo real es corto; el patrón ancla indentación de job.
- **[Redocly 1.x vs 2.x]** → pin `@1` evita un salto mayor; si 1.x desaparece de npm, el job falla de forma visible.
- **[13 warnings]** → no fallan hoy; si Redocly 1.x los eleva a error, el job se pone rojo. Mitigación: el pin y no tocar severidad en `redocly.yaml`.
- **[Listo cuando real es GitHub]** → el harness cubre contrato + el mismo comando sobre fixture inválida; el PR de `T-09-03` es la prueba operativa.
- **[`openspec new change` rechaza `T-09-03`]** → scaffold en `t-09-03` y rename (igual que T-09-01). Nota operativa, no bloquea.
- **[Node en el runner macos-14]** → GitHub lo incluye; si un runner lo pierde, el job falla en `npx`. No se agrega `setup-node` en este ticket.

## Migration Plan

1. Red: `check-ci.sh` exige el job `openapi-lint` → el repo actual (cuatro jobs) sale ≠ 0.
2. Fixture YAML sin el job: sale ≠ 0 y nombra `openapi-lint`.
3. Oráculo inválido: `npx … lint` sobre la fixture sale ≠ 0.
4. Green: agregar `redocly.yaml` y el job; `check-ci.sh` sobre el repo sale 0; lint del YAML vigente sale 0.
5. Reemplazar `redocly-extra` por `meilisearch-extra` en `run-script-tests.sh`.
6. Rollback: revertir el job, `redocly.yaml` y la denylist de `check-ci.sh`. No hay datos.

## Open Questions

Ninguna que cambie specs, enfoque o desglose. El nombre del job (`openapi-lint`) y el pin `@redocly/cli@1` quedan fijados aquí.
