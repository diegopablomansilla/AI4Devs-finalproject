#!/usr/bin/env sh
# T-00-02-01: check-domain-deps.sh integration tests against fixtures.
set -eu

REPO_ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
SCRIPT="${REPO_ROOT}/scripts/check-domain-deps.sh"
FIXTURES="$(cd "$(dirname "$0")" && pwd)"
SCENARIOS="${FIXTURES}/scenarios"

assert_fails_with() {
  root=$1
  needle=$2
  out=$(mktemp "${TMPDIR:-/tmp}/cdd-out.XXXXXX")
  trap 'rm -f "${out}"' EXIT INT HUP TERM
  if CHECK_ROOT="${root}" "${SCRIPT}" 2>"${out}"; then
    printf 'expected failure for %s\n' "${root}" >&2
    exit 1
  fi
  if ! grep -qF "${needle}" "${out}"; then
    printf 'stderr missing %s for %s:\n' "${needle}" "${root}" >&2
    cat "${out}" >&2
    exit 1
  fi
}

assert_ok() {
  root=$1
  CHECK_ROOT="${root}" "${SCRIPT}"
}

assert_ok "${SCENARIOS}/surrealdb-dev-only"

assert_fails_with "${SCENARIOS}/surrealdb-direct" surrealdb
assert_fails_with "${SCENARIOS}/reqwest-direct" reqwest
assert_fails_with "${SCENARIOS}/reqwest-transitive" reqwest
assert_fails_with "${SCENARIOS}/storage-member" storage
assert_fails_with "${SCENARIOS}/no-domain" domain

CHECK_ROOT="${REPO_ROOT}" "${SCRIPT}"
"${REPO_ROOT}/scripts/check-workspace.sh"

printf 'check-domain-deps script tests OK\n'
