# Proposal

## Ticket padre

- **Ticket:** [T-09-05](../../../docs/7-tickets.md) — CI: Meilisearch real para integración
- **Change:** `T-09-05`
- **HU:** [HU-09-01](../../../docs/6-historias.md#hu-09-01--tests-de-integración-contra-motores-reales)
- **Depende de:** T-09-01 (cerrado en `develop`; T-09-03 ya enganchó `openapi-lint`)
- **Hace:** *job* de CI que levanta Meilisearch con una versión fijada (la misma que se usa en desarrollo; el manifiesto de `vortex models` es v2) y ejecuta los tests marcados `#[ignore = "meilisearch"]` con `--include-ignored`
- **No hace:** otros motores
- **Listo cuando:** la CI corre un test de integración contra Meilisearch real

## Why

La CI de T-09-01 **prohíbe** Meilisearch a propósito para dejar este ticket. HU-09-01 pide tests de integración contra el motor real; T-03-04 (adaptador) depende de esa puerta. Hoy `cargo test --workspace` no puede hablar con Meilisearch en GitHub, y no hay ningún `#[ignore = "meilisearch"]`. Sin este job, el Listo cuando («la CI corre un test de integración contra Meilisearch real») no se cumple.

## What Changes

- Fijar la versión de Meilisearch de desarrollo/CI en `scripts/meilisearch-version` (`v1.58.0`, binario `meilisearch-macos-apple-silicon`); no hay manifiesto de `vortex models` (T-00-24, v2)
- Agregar un job de GitHub Actions en paralelo (`meilisearch`) que descarga esa versión, espera `/health` en `127.0.0.1:7700` y corre `cargo test --workspace -- --include-ignored meilisearch`
- Probe de integración en `search` (`#[ignore = "meilisearch"]`) que pega a `/health` del motor real, para que el job no quede verde en vacío
- Extender `scripts/check-ci.sh`: exigir el job, el pin de versión y `--include-ignored`; dejar de tratar `Meilisearch`/`meilisearch` como extra prohibido
- TDD: happy = contrato CI + probe con motor arriba; unhappy = falta job/pin/`--include-ignored`, extra Ollama/Whisper, y probe con Meilisearch caído
- **BREAKING** (harness interno): `check-ci.sh` deja de fallar si el workflow menciona Meilisearch; pasa a exigir el job `meilisearch`

## Non-goals

Este change **no** cubre:

- Adaptador `meilisearch-sdk`, espera de *tasks*, `health()` de dominio ni traducción de errores → **T-03-04** (**No hace** relativo al motor de búsqueda)
- Puerto `TextSearch` → **T-03-03**
- Ollama / embeddings en CI → **T-09-13** (**No hace:** otros motores)
- Whisper en CI → **T-09-18**
- Manifiesto `models/manifest.toml` / `vortex models` → **T-00-24** (v2; recableo § 7.5.1)
- Supervisor que levanta Meilisearch como hijo → **T-00-25** (v2)
- Índice `productos`, worker, `GET /search` → T-03-05 en adelante
- Un job que ejecute `scripts/check-ci.sh` en GitHub (sigue siendo harness local)

## Capabilities

### New Capabilities

- `ci-meilisearch`: versión fijada de Meilisearch, job de CI que la levanta en `macos-14` y corre los tests `#[ignore = "meilisearch"]` con `--include-ignored`, y un probe de `/health` para que el Listo cuando no sea vacío

### Modified Capabilities

- `ci-base`: `scripts/check-ci.sh` MUST exigir el job `meilisearch`, el pin de versión y `--include-ignored`; MUST NOT tratar `Meilisearch`/`meilisearch` como extra fuera de alcance; MUST seguir rechazando Ollama, Whisper y un job que ejecute `check-ci.sh` en GitHub

## Impact

- **Repo:** `.github/workflows/ci.yml` (job nuevo), `scripts/meilisearch-version` (nuevo), `scripts/check-ci.sh` (contrato ampliado), fixtures de CI, `vortex-core/crates/search/tests/` (probe). No toca `meilisearch-sdk` ni `docs/openapi.yaml`
- **APIs / OpenAPI:** ninguna
- **CI / verificación:** sexto check en cada PR; comandos [§ 5](../../../docs/desarrollo-git-openspec.md#5-comandos-de-verificación-evolutivos) sin cambio en los jobs `fmt`/`clippy`/`test`; el job nuevo añade `--include-ignored`; `openspec validate T-09-05` (o `t-09-05` si el CLI exige minúsculas)
- **Ticket:** cierra T-09-05; desbloquea T-03-04. El Listo cuando en GitHub se confirma en el PR de esta rama
- **BREAKING** (harness interno): la fixture `meilisearch-extra` deja de ser el unhappy de «extra fuera de alcance»; ese rol pasa a Ollama/Whisper
