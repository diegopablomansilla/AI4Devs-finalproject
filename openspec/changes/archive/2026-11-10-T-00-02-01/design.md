# Design

## Context

En `develop`, `vortex-core/crates/domain` es un lib vacío sin `[dependencies]`. `scripts/check-workspace.sh` (T-00-01) rechaza cualquier `[dependencies]` en los libs; no invoca `cargo tree`. No hay CI (T-09-01 pendiente). Ver proposal.md para el recorte; la regla hexagonal está en [3.2](../../../docs/3-componentes.md#32-mapa-de-crates-y-regla-de-dependencias).

## Goals / Non-Goals

**Goals:**

- Un script POSIX alineado a `check-toolchain.sh` / `check-workspace.sh` (`CHECK_ROOT`, `fail`, stderr con el crate ofensor)
- Detectar violaciones en el *grafo* (directas y transitivas), no solo en `domain/Cargo.toml`
- Fixtures offline: no hace falta bajar el crate real `surrealdb` para el unhappy path

**Non-Goals (diseño):**

- Job de CI (T-00-02-02)
- Relajar o reescribir `check-workspace.sh`
- Parser TOML / crate extra; el oráculo es `cargo tree`

## Decisions

### 1. Script nuevo, no extender `check-workspace.sh`

**Elección:** `scripts/check-domain-deps.sh` independiente.

**Por qué:** el chequeo de T-00-01 es estructural (members, pins, libs vacíos) y dejará de prohibir *todas* las deps de `domain` en T-00-03/T-00-04. Mezclar `cargo tree` ahí acoplaría dos contratos. T-00-02-02 enganchará este script, no el de workspace.

**Alternativa:** un flag `--domain-tree` en `check-workspace.sh`. Se descartó: cambia el harness de T-00-01 y complica fixtures.

### 2. `cargo tree -p domain -e normal,build`, no grep del manifiesto

**Elección:** `--manifest-path "${ROOT}/vortex-core/Cargo.toml"`, aristas `normal,build` (como el ticket). Flags extra de formato para parsear nombres (`--prefix none --format '{p}'` o equivalente) son detalle de implementación.

**Por qué:** un grep de `domain/Cargo.toml` no ve transitivas (un wrapper `sneaky` → `reqwest` pasaría). `-e normal,build` excluye `dev` a propósito: el ticket lo fija; T-00-03 podrá testear `domain` sin disparar el chequeo.

**Alternativa:** `cargo metadata --filter-platform` + walk JSON. Se descartó: más código, y el ticket nombra `cargo tree`.

### 3. Lista cerrada de SDKs; workspace dinámico

**Elección:** denylist fija `surrealdb`, `meilisearch-sdk`, `reqwest`, `ort`, `whisper-rs`. Los packages del workspace se leen del mismo manifiesto (p. ej. `cargo metadata --no-deps` o `cargo tree -p domain` cruzado con la lista de packages), excluyendo `domain`. Match por **nombre de package** exacto (primer campo de `{p}`), no substring.

**Por qué:** HU-00-01 menciona Ollama; el ticket lo cubre con `reqwest` (HTTP), no hay crate `ollama` en la denylist. `tokenizers` tampoco está. Hardcodear `api`/`storage`/… se desactualiza al agregar members. `domain` es la raíz del árbol: no es violación.

**Alternativa:** denylist de Ollama/`tokenizers`. Se descartó: fuera del **Hace**. T-00-02-02 no cambia la lista.

### 4. Fixtures con path crates del nombre prohibido

**Elección:** `CHECK_ROOT` apunta a un árbol temporal mínimo (`vortex-core/Cargo.toml` + `crates/domain` + path crates locales). Un path crate `name = "surrealdb"` (fuera de `members` si hace falta) basta para el unhappy path. El happy path corre contra el repo real.

**Por qué:** agregar `surrealdb.workspace = true` en una copia del workspace real dispara fetch de crates.io y ensucia lockfiles. El script compara nombres de package; un crate local con ese nombre es el mismo oráculo. La fixture transitiva: `domain` → `wrapper` → path crate `reqwest`.

**Alternativa:** copiar `vortex-core/` y depender del pin real. Se descartó: red, tiempo, y no aporta al contrato de nombres.

### 5. Un fixture representativo por clase de SDK, más `surrealdb` canónico

**Elección:** TDD cubre `surrealdb` (el ejemplo del **Listo cuando**), al menos un segundo SDK de la lista (`reqwest` o `ort`), la transitiva, un member (`storage`) y el caso `dev-dependencies`. La denylist completa vive en el script; no hace falta cinco copias idénticas del mismo mecanismo.

**Por qué:** ~1 h. Cinco fixtures de SDK idénticos no cambian el diseño.

**Alternativa:** un fixture por cada SDK. Válido si en apply resulta barato; no es requisito de la spec.

## Risks / Trade-offs

- **[Match por substring]** → `surrealdb-core` o un path con la palabra `reqwest` podrían disparar falsos positivos. Mitigación: nombre de package exacto.
- **[dev-dependencies permitidos]** → alguien podría acoplar tests de `domain` a SurrealDB sin que el script falle. Mitigación: es el recorte del ticket; si duele, otro ZZ amplia aristas a `dev`.
- **[`cargo tree` ausente o workspace roto]** → el script falla (≠ 0). Aceptable: no hay árbol que certificar.
- **[check-workspace.sh vs este script]** → hasta T-00-03 ambos rechazan deps en `domain` por razones distintas. No hay conflicto: el repo real sigue vacío.

## Migration Plan

1. Escribir el script que exige árbol limpio; el repo actual debe pasar (happy, a la inversa de T-00-01: `domain` ya está limpio).
2. Red: fixtures unhappy (SDK, transitiva, workspace) contra un script que aún no detecta, o contra ausencia del script.
3. Green: denylist + packages del workspace.
4. Confirmar `./scripts/check-workspace.sh` sigue en 0.
5. Rollback: borrar `scripts/check-domain-deps.sh`. No hay datos ni CI.

## Open Questions

Ninguna que cambie el desglose. La denylist queda cerrada al **Hace** del ticket.
