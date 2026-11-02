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
| [git-agent-gates.mdc](.cursor/rules/git-agent-gates.mdc) | No commit / PR without explicit user OK |
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
- **Commit / PR:** only when the user explicitly asks — and on spec branches, commit **only after** archive.
- **Archive:** only when tests are green (happy + unhappy path); then **propose** commit (Conventional Commits, scope `T-XX-YY-ZZ`), do not commit automatically.
