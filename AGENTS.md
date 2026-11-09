# Agent instructions — Vectra

Router for AI assistants (Cursor, OpenSpec, etc.). **Mandatory behavior** is enforced in `.cursor/rules/` (`alwaysApply: true`).

## Read first

1. **[docs/desarrollo-git-openspec.md](docs/desarrollo-git-openspec.md)** — Git (`main` / `develop` / `T-XX-YY-ZZ`), OpenSpec, SDD + TDD, [Conventional Commits](https://www.conventionalcommits.org/en/v1.0.0/) (§ 7), archive and commit gates.
2. **[docs/7-tickets.md](docs/7-tickets.md)** — Backlog tickets `T-XX-YY`; specs `T-XX-YY-ZZ`.
3. **[openspec/config.yaml](openspec/config.yaml)** — Context and rules when creating OpenSpec artifacts.

## Cursor rules (obligatory)

| Rule | Topic |
|------|--------|
| [git-workflow.mdc](.cursor/rules/git-workflow.mdc) | Branch model and naming |
| [git-agent-gates.mdc](.cursor/rules/git-agent-gates.mdc) | Propose commit & PR proactively; execute only with explicit user OK |
| [openspec-tdd-delivery.mdc](.cursor/rules/openspec-tdd-delivery.mdc) | One spec per branch, TDD, archive → propose commit |
| [ignore-prompts-md.mdc](.cursor/rules/ignore-prompts-md.mdc) | Do not use `prompts.md` |

## OpenSpec layout

- Active changes: `openspec/changes/<T-XX-YY-ZZ>/`
- Archived changes: `openspec/changes/archive/`
- Consolidated specs: `openspec/specs/`

Change name **must equal** the Git branch name (`T-XX-YY-ZZ`).

## Product and architecture docs

- [docs/1-descripcion-general-del-producto.md](docs/1-descripcion-general-del-producto.md) through [docs/7-tickets.md](docs/7-tickets.md)
- API: [docs/openapi.yaml](docs/openapi.yaml)

## Quick gates

- **Spec branches `T-XX-YY-ZZ`:** no `git commit` until **`openspec archive`** is done (TDD uncommitted until then).
- **Commit / PR:** agent **must propose** (message, files, PR title/body) when ready; user **authorizes** before any `git commit`, `push`, or `gh pr create`.
- **Spec branches:** commit proposal **only after** archive; then PR proposal → `develop`.
- **Archive:** only when tests are green (happy + unhappy path); then **propose** commit (Conventional Commits, scope `T-XX-YY-ZZ`, footer `Refs: T-XX-YY, HU-XX-YY`), never commit automatically.
- **Releases:** annotated SemVer tags only on `main` per delivery/hito (`v0.N.0`, H1 = `v1.0.0`); no HU branches or tags — HUs are GitHub issues in a hito milestone ([docs/desarrollo-git-openspec.md](docs/desarrollo-git-openspec.md) § 10–11).
