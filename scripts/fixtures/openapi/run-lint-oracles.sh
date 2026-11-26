#!/usr/bin/env sh
# T-09-03: Redocly lint oracles (happy repo spec, unhappy fixture).
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
FIXTURES="$(cd "$(dirname "$0")" && pwd)"
INVALID="${FIXTURES}/scenarios/invalid-spec/openapi.yaml"
LINT="npx --yes @redocly/cli@1 lint"

cd "${REPO_ROOT}"
if ! ${LINT} docs/openapi.yaml >/dev/null 2>&1; then
  printf 'expected happy lint on docs/openapi.yaml\n' >&2
  ${LINT} docs/openapi.yaml >&2
  exit 1
fi

out=$(mktemp "${TMPDIR:-/tmp}/lint-out.XXXXXX")
trap 'rm -f "${out}"' EXIT INT HUP TERM
if ${LINT} "${INVALID}" >"${out}" 2>&1; then
  printf 'expected failure for invalid-spec openapi.yaml\n' >&2
  cat "${out}" >&2
  exit 1
fi
if ! grep -qE 'openapi\.yaml|version|error|Error' "${out}"; then
  printf 'lint output missing file or rule for invalid-spec:\n' >&2
  cat "${out}" >&2
  exit 1
fi

printf 'openapi lint oracles OK\n'
