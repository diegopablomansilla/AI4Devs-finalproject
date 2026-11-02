# Desarrollo: Git, OpenSpec, TDD y SDD — Vectra

> **Estado:** Convención del proyecto  
> **Relacionado:** [7. Tickets](7-tickets.md) · [6. Historias (DoD)](6-historias.md) · [OpenSpec `config.yaml`](../openspec/config.yaml)

Este documento es la **fuente canónica** del flujo de trabajo humano. Las reglas del agente en Cursor (`.cursor/rules/`) y la configuración de OpenSpec deben estar alineadas con lo aquí descrito.

---

## 1. Modelo de ramas

```mermaid
flowchart TB
  main[main]
  develop[develop]
  specBranch["rama T-XX-YY-ZZ"]
  main --> develop
  develop --> specBranch
  specBranch -->|"PR con revisión"| develop
  develop -->|"PR cuando corresponda"| main
```

| Rama | Propósito |
|------|-----------|
| **`main`** | Línea estable; refleja entregas consolidadas. |
| **`develop`** | Integración continua; **todas** las ramas de trabajo salen de aquí. |
| **`T-XX-YY-ZZ`** | Una **spec OpenSpec** en curso (ver § 2). No usar prefijos `feature/`. |

**Reglas:**

- Crear la rama desde `develop` actualizado: `git checkout develop && git pull && git checkout -b T-02-14-01`.
- Integrar en **`develop`** mediante Pull Request (no merge directo a `main` desde ramas de spec).
- **`main`** se actualiza desde `develop` cuando corresponda a un hito o entrega (criterio del equipo).

Ramas con convención antigua (p. ej. `feature/entrega-1-DPM`) no usan `T-XX-YY-ZZ`. Integrarlas en `develop` vía PR cuando corresponda; **desde entonces** abrir specs y ramas con el formato nuevo.

---

## 2. Tickets, specs e identificadores

| Concepto | ID | Dónde vive |
|----------|-----|------------|
| **Ticket** (backlog) | `T-XX-YY` | [7-tickets.md](7-tickets.md) |
| **Spec de implementación** | `T-XX-YY-ZZ` | OpenSpec change + rama Git |

- Un ticket puede requerir **varias specs** (`ZZ = 01`, `02`, …). Cada spec acota un slice implementable (~2 h, alineado al ticket).
- **`ZZ`** es secuencial por ticket (dos dígitos: `01`, `02`, …).
- **1 change OpenSpec activo = 1 rama Git = 1 spec** (`T-XX-YY-ZZ`).
- El **nombre del change** en OpenSpec coincide con el nombre de la rama.

Cada change debe declarar el **ticket padre** `T-XX-YY` (frontmatter o sección «Ticket padre») y enlazar **Hace** / **Listo cuando** del ticket.

**Ejemplo:** ticket `T-00-02` (verificación de dependencias de `domain`) podría dividirse en:

- `T-00-02-01` — script `check-domain-deps.sh`
- `T-00-02-02` — job de CI que ejecuta el script

---

## 3. SDD + TDD

**SDD (Spec-Driven Development):** la spec OpenSpec se deriva del ticket; define alcance, criterios de aceptación y comportamiento esperado antes de cerrar la implementación.

**TDD (Test-Driven Development):** los tests se escriben en función de la spec; el ciclo es **Red → Green → Refactor**.

| Fase | Qué ocurre |
|------|------------|
| **Spec** | Change OpenSpec `T-XX-YY-ZZ` con escenarios verificables (happy path y **camino no feliz**: errores de dominio, `problem+json`, validación, conflictos, etc.). |
| **Red** | Tests que fallan porque aún no existe la implementación. |
| **Green** | Implementación mínima hasta que la suite acordada pase. |
| **Refactor** | Mejora sin cambiar comportamiento; tests siguen en verde. |

La DoD común de historias ([6.1](6-historias.md)) aplica: conformidad con `openapi.yaml`, trazas, accesibilidad en UI cuando corresponda.

---

## 4. Flujo paso a paso

1. Elegir ticket `T-XX-YY` en [7-tickets.md](7-tickets.md) (dependencias satisfechas).
2. Definir o crear la spec **`T-XX-YY-ZZ`** en OpenSpec (change bajo `openspec/changes/`).
3. Documentar en el change los **tests** previstos (unitarios, integración, API unhappy path según alcance).
4. Crear rama **`T-XX-YY-ZZ`** desde `develop`.
5. TDD: red → green → refactor.
6. Ejecutar la **suite completa** relevante (local; cuando exista CI, los mismos comandos que GitHub Actions — ver ticket T-09-01).
7. `openspec validate` sobre el change (si aplica).
8. **`openspec archive <change-name>`** solo con tests en verde (happy + unhappy). Quien archiva: el maintainer o el agente **solo con autorización explícita**.
9. **Solo después del paso 8:** el agente puede **proponer** un commit con mensaje [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (§ 7); **no ejecuta** `git commit` sin tu OK.
10. Abrir PR hacia **`develop`** cuando autorices; el agente **no** crea PRs sin tu OK. El título del PR sigue la misma convención que el subject del commit.

### Regla de commit en ramas `T-XX-YY-ZZ`

En una rama de spec **no hay commits de Git hasta que el change OpenSpec esté archivado**.

| Fase | Git |
|------|-----|
| Change activo + TDD (red / green / refactor) | Cambios en el **working tree** (y staging solo para revisión). **Sin** `git commit`. |
| Tras `openspec archive` | **Primer** commit de la rama (convención: uno por spec archivada). |
| Integración | PR → `develop` con ese commit (o commits acordados **solo post-archive**). |

Esto aplica al **agente de Cursor** siempre. Si pedís commit antes de archivar, debe rechazarlo, recordar esta regla y ofrecer archivar primero (con tests green y tu OK).

Excepciones: ramas que **no** son specs (`develop`, `main`, ramas legacy) o acuerdo explícito documentado fuera de este flujo.

---

## 5. Comandos de verificación (evolutivos)

Hasta que el workspace Rust y la CI (T-09-01) existan, la suite concreta se acuerda por spec en el change. Cuando estén disponibles, la referencia mínima es:

```bash
cargo fmt --all -- --check
cargo clippy --workspace -- -D warnings
cargo test --workspace
```

Tests de API deben incluir respuestas de error alineadas con [5-contract-api.md](5-contract-api.md) y `openapi.yaml`.

OpenSpec:

```bash
openspec validate <change-name>
openspec archive <change-name>
```

---

## 6. OpenSpec en el repositorio

| Ruta | Uso |
|------|-----|
| `openspec/config.yaml` | Contexto del proyecto y reglas al generar artefactos |
| `openspec/changes/<T-XX-YY-ZZ>/` | Change activo |
| `openspec/changes/archive/` | Changes archivados |
| `openspec/specs/` | Specs consolidadas tras archivar |

Tras `openspec archive`, las specs maestras se actualizan en `openspec/specs/` según el schema `spec-driven`.

---

## 7. Mensajes de commit (Conventional Commits)

Seguimos [Conventional Commits 1.0.0](https://www.conventionalcommits.org/en/v1.0.0/) en la medida que aplique al repo.

### Formato

```
<type>(<scope>): <description>

[cuerpo opcional]

[footer opcional]
```

- **Subject:** imperativo, minúsculas en la descripción (salvo nombres propios), sin punto final, ~72 caracteres.
- **Scope (recomendado):** ID de la spec en curso, p. ej. `(T-02-14-01)`. Alternativa: crate o área (`api`, `catalog`) si el commit es transversal dentro de la misma rama.
- **Footer (recomendado):** `Refs: T-02-14` para el ticket padre cuando el scope es la spec.

### Tipos habituales en Vectra

| Tipo | Uso |
|------|-----|
| `feat` | Comportamiento nuevo (endpoint, UI, comando CLI, regla de dominio). |
| `fix` | Corrección de bug. |
| `test` | Cambios centrados en tests (suelen ir en el commit de cierre post-archive). |
| `docs` | Solo documentación (`docs/`, OpenSpec archivado, README). |
| `refactor` | Cambio interno sin alterar comportamiento observable. |
| `chore` | Mantenimiento (deps, scripts auxiliares no funcionales). |
| `ci` | Pipelines y checks (p. ej. GitHub Actions). |
| `build` | Toolchain, `Cargo.toml`, empaquetado. |

`BREAKING CHANGE:` o `!` tras el tipo/scope solo si el cambio rompe contrato (`openapi.yaml`) o compatibilidad; en el MVP es raro y debe mencionarse en la spec.

### Ejemplos

```
feat(T-00-02-01): add check-domain-deps script

Refs: T-00-02
```

```
test(T-03-13-01): cover search 400 invalid query

Refs: T-03-13
```

```
docs(T-09-01-01): document CI workflow in readme

Refs: T-09-01
```

**Un commit de cierre por spec archivada** en la rama `T-XX-YY-ZZ` (implementación + tests + artefactos OpenSpec archivados). Partir en varios commits **solo post-archive** si el maintainer lo pide explícitamente; mismo **scope** (`T-XX-YY-ZZ`).

---

## 8. Agente Cursor (resumen)

- En ramas **`T-XX-YY-ZZ`**: **ningún** `git commit` hasta **`openspec archive`** completado (TDD sin commits intermedios).
- Sin **commit**, **amend**, ni **PR** sin instrucción explícita del maintainer (y commit solo **después** de archive en ramas de spec).
- Tras **archive** + tests green: **proponer** commit, no aplicarlo.
- Detalle ejecutable: `.cursor/rules/git-workflow.mdc`, `git-agent-gates.mdc`, `openspec-tdd-delivery.mdc` y [AGENTS.md](../AGENTS.md).

---

## 9. Plantilla mínima de Pull Request

Usar al integrar `T-XX-YY-ZZ` → `develop`:

- **Rama:** `T-XX-YY-ZZ`
- **Ticket padre:** `T-XX-YY` (enlace o sección en 7-tickets)
- **Change OpenSpec:** archivado (`openspec archive` completado)
- **Tests:** comandos ejecutados y resultado (green, incl. unhappy path)
- **Título del PR:** Conventional Commits, p. ej. `feat(T-02-14-01): …` (alinear con el commit principal)
- **Notas:** cambios en `openapi.yaml` / modelo si aplica
